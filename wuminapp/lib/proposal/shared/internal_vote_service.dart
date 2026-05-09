import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show ByteOutput;

import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/signed_extrinsic_builder.dart';

/// 投票引擎统一投票入口服务。
///
/// Phase 3(2026-04-22)「投票引擎统一入口整改」在客户端的落地:
///
/// - 所有业务 pallet(admins_change / resolution_destro /
///   grandpakey_change / duoqian_manage / transaction 业务)的
///   业务 pallet 不再提供独立投票入口,管理员一人一票一律走
///   `InternalVote::cast(proposal_id, approve)` 一条路径。
/// - 业务 service(DuoqianManageService 等)
///   只负责发起提案(propose_X)；执行重试统一走 VotingEngine.retry_passed_proposal,
///   投票动作统一
///   委托本服务,避免多处构造相同的 call。
///
/// Runtime 位置: `pallet_index=22, call_index=0`(InternalVote sub-pallet)。
/// Call 编码: `[0x16][0x00][proposal_id:u64_le][approve:bool]` 共 11 字节。
class InternalVoteService {
  InternalVoteService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// InternalVote sub-pallet。runtime pallet_index=22。
  static const int votingEnginePallet = 22;

  /// InternalVote::cast call_index=0。
  static const int internalVoteCallIndex = 0;

  // ──── 公开 API ────

  /// 提交 `InternalVote::cast(proposal_id, approve)` extrinsic(pallet 22.0)。
  ///
  /// 返回交易哈希 hex（含 0x 前缀）和使用的 nonce。业务模块无需感知
  /// 提案所属 pallet/MODULE_TAG,投票引擎会按 ProposalData 前缀自动
  /// 分派到对应 `InternalVoteExecutor`。
  Future<({String txHash, int usedNonce})> submit({
    required int proposalId,
    required bool approve,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final callData = buildCallData(proposalId: proposalId, approve: approve);
    return _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  /// 构造 InternalVote::cast call data(对外公开,便于冷钱包/热钱包复用)。
  ///
  /// 格式: `[0x16][0x00][proposal_id:u64_le][approve:bool]`(pallet=22, call=0)。
  static Uint8List buildCallData({
    required int proposalId,
    required bool approve,
  }) {
    final output = ByteOutput();
    output.pushByte(votingEnginePallet);
    output.pushByte(internalVoteCallIndex);
    output.write(_u64ToLeBytes(proposalId));
    output.pushByte(approve ? 1 : 0);
    return output.toBytes();
  }

  // ──── 内部：签名提交 ────

  Future<({String txHash, int usedNonce})> _signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'InternalVote',
    ).signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  // ──── 内部：编码工具 ────

  static Uint8List _u64ToLeBytes(int value) {
    final bytes = Uint8List(8);
    final bd = ByteData.sublistView(bytes);
    bd.setUint64(0, value, Endian.little);
    return bytes;
  }
}
