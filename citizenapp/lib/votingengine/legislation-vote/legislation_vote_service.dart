import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show ByteOutput;

import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';
import 'package:citizenapp/votingengine/legislation-vote/legislation_vote_query_service.dart';

/// 立法投票/签署提交服务(LegislationVote sub-pallet,pallet_index=26)。
///
/// 代表机构表决/行政签署/三人会签/护宪终审四个动作都是**纯 extrinsic**
/// (signer=origin=动作人本人,零 op_tag),统一走 [SignedExtrinsicBuilder] 标准
/// 交易签名,范式照搬 internal-vote。提交后必须回读 legislation-vote storage 确认
/// runtime 已记账,txHash 不代表已执行。特别案公投(referendum/snapshot)带 CID
/// 凭证,另见 legislation_referendum_service。
///
/// Call 格式统一 `[26][call_index][proposal_id:u64_le][approve:bool]`(11 字节)。
class LegislationVoteService {
  LegislationVoteService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc(),
        _query = LegislationVoteQueryService(chainRpc: chainRpc);

  final ChainRpc _rpc;
  final LegislationVoteQueryService _query;

  /// LegislationVote runtime pallet_index。
  static const int legislationVotePallet = 26;

  static const int callCastRepresentativeVote = 1;
  static const int callExecutiveSign = 3;
  static const int callOverrideSign = 4;
  static const int callGuardVote = 5;

  // ──── 公开 API ────

  /// 当前代表机构表决；同一钱包在不同机构的席位分别记票。
  Future<({String txHash, int usedNonce, String blockHashHex})>
      castRepresentativeVote({
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final meta = await _query.fetchRepresentativeMeta(proposalId);
    if (meta == null) throw StateError('代表机构表决元数据不存在');
    final bodyIndex = meta.currentBody;
    final result = await _signAndSubmit(
      callIndex: callCastRepresentativeVote,
      proposalId: proposalId,
      approve: approve,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
    await _confirmRepresentativeVote(
      proposalId,
      bodyIndex,
      approve,
      signerPubkey,
      result.blockHashHex,
    );
    return result;
  }

  /// 行政首长(机构法定代表人)签署或否决。
  Future<({String txHash, int usedNonce, String blockHashHex})> executiveSign({
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final result = await _signAndSubmit(
      callIndex: callExecutiveSign,
      proposalId: proposalId,
      approve: approve,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
    // 行政签署无 per-signer 账本:确认提案已离开签署阶段(进会签/已生效/已否决)。
    await _confirmStageAdvanced(proposalId, LegStage.sign, result.blockHashHex);
    return result;
  }

  /// 三人会签(立法院院长 + 参议长 + 众议长)签署或否决。
  Future<({String txHash, int usedNonce, String blockHashHex})> overrideSign({
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final result = await _signAndSubmit(
      callIndex: callOverrideSign,
      proposalId: proposalId,
      approve: approve,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
    await _confirmSignRecorded(
      proposalId,
      signerPubkey,
      result.blockHashHex,
      _query.fetchOverrideSigns,
    );
    return result;
  }

  /// 护宪大法官终审表决(修宪)。
  Future<({String txHash, int usedNonce, String blockHashHex})> guardVote({
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final result = await _signAndSubmit(
      callIndex: callGuardVote,
      proposalId: proposalId,
      approve: approve,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
    await _confirmSignRecorded(
      proposalId,
      signerPubkey,
      result.blockHashHex,
      _query.fetchGuardSigns,
    );
    return result;
  }

  /// 构造 `[28][call][proposal_id:u64_le][approve:bool]` call data(对外公开供冷钱包复用)。
  static Uint8List buildCallData({
    required int callIndex,
    required int proposalId,
    required bool approve,
  }) {
    final output = ByteOutput();
    output.pushByte(legislationVotePallet);
    output.pushByte(callIndex);
    output.write(_u64ToLeBytes(proposalId));
    output.pushByte(approve ? 1 : 0);
    return output.toBytes();
  }

  // ──── 内部:签名提交 ────

  Future<({String txHash, int usedNonce, String blockHashHex})> _signAndSubmit({
    required int callIndex,
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) {
    final callData = buildCallData(
      callIndex: callIndex,
      proposalId: proposalId,
      approve: approve,
    );
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'LegislationVote',
    ).signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  // ──── 内部:入块后确认 ────

  Future<void> _confirmRepresentativeVote(
    int proposalId,
    int bodyIndex,
    bool approve,
    Uint8List signerPubkey,
    String blockHashHex,
  ) async {
    final pubkeyHex = _hexEncode(signerPubkey);
    for (var attempt = 0; attempt < 6; attempt++) {
      final vote = await _query.fetchRepresentativeVote(
        proposalId,
        bodyIndex,
        pubkeyHex,
      );
      if (vote == approve) return;
      if (vote != null && vote != approve) {
        throw StateError('runtime 投票记录与本次投票方向不一致');
      }
      if (attempt < 5) {
        await Future<void>.delayed(const Duration(milliseconds: 500));
      }
    }
    await _throwWithEventFailure(blockHashHex, '交易已入块，但 runtime 未记录该议员投票');
  }

  Future<void> _confirmSignRecorded(
    int proposalId,
    Uint8List signerPubkey,
    String blockHashHex,
    Future<List<({String pubkeyHex, bool approve})>> Function(int) fetchSigns,
  ) async {
    final pubkeyHex = _hexEncode(signerPubkey);
    for (var attempt = 0; attempt < 6; attempt++) {
      final signs = await fetchSigns(proposalId);
      if (signs.any((s) => s.pubkeyHex == pubkeyHex)) return;
      // 终审/会签可能一签即终态清账;若提案已离开该阶段也算成功。
      final state = await _query.fetchProposalState(proposalId);
      if (state != null && state.status != LegProposalStatus.voting) return;
      if (attempt < 5) {
        await Future<void>.delayed(const Duration(milliseconds: 500));
      }
    }
    await _throwWithEventFailure(blockHashHex, '交易已入块，但 runtime 未记录该签署');
  }

  Future<void> _confirmStageAdvanced(
    int proposalId,
    int fromStage,
    String blockHashHex,
  ) async {
    for (var attempt = 0; attempt < 6; attempt++) {
      final state = await _query.fetchProposalState(proposalId);
      // 签署被处理后:或已终态(passed/rejected),或推进到下一阶段(会签)。
      if (state != null &&
          (state.status != LegProposalStatus.voting ||
              state.stage != fromStage)) {
        return;
      }
      if (attempt < 5) {
        await Future<void>.delayed(const Duration(milliseconds: 500));
      }
    }
    await _throwWithEventFailure(blockHashHex, '交易已入块，但 runtime 未推进签署阶段');
  }

  Future<void> _throwWithEventFailure(
      String blockHashHex, String fallback) async {
    final events = await _rpc.fetchSystemEventsAtBlock(blockHashHex);
    final failure =
        events == null ? null : _rpc.findExtrinsicFailureInEvents(events);
    if (failure != null) {
      throw StateError('runtime 拒绝该操作：${failure.description}');
    }
    throw StateError(fallback);
  }

  // ──── 内部:编码工具 ────

  static Uint8List _u64ToLeBytes(int value) {
    final bytes = Uint8List(8);
    ByteData.sublistView(bytes).setUint64(0, value, Endian.little);
    return bytes;
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
