import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show ByteOutput;

import '../rpc/chain_rpc.dart';
import '../rpc/nonce_manager.dart';

/// 投票引擎统一投票入口服务。
///
/// Phase 3(2026-04-22)「投票引擎统一入口整改」在客户端的落地:
///
/// - 所有业务 pallet(admins_change / resolution_destro /
///   grandpakey_change / duoqian_manage / duoqian_transfer)的
///   `vote_X` call 在 Phase 2 已从链端物理删除,管理员一人一票一律走
///   `VotingEngine::internal_vote(proposal_id, approve)` 一条路径。
/// - 业务 service(TransferProposalService / DuoqianManageService 等)
///   只负责发起提案(propose_X)与提案执行重试(execute_X),投票动作统一
///   委托本服务,避免多处构造相同的 call。
///
/// Runtime 位置: `pallet_index=9, call_index=0`。
/// Call 编码: `[0x09][0x00][proposal_id:u64_le][approve:bool]` 共 11 字节。
class InternalVoteService {
  InternalVoteService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// VotingEngine pallet index（runtime pallet_index=9）。
  static const int votingEnginePallet = 9;

  /// internal_vote call_index=0（Phase 2 重排后在投票引擎第 0 位）。
  static const int internalVoteCallIndex = 0;

  /// Mortal era 周期（与其他业务 service 保持一致）。
  static const int _eraPeriod = 64;

  // ──── 公开 API ────

  /// 提交 `VotingEngine::internal_vote(proposal_id, approve)` extrinsic。
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
  }) async {
    final callData = buildCallData(proposalId: proposalId, approve: approve);
    return _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 构造 internal_vote call data（对外公开，便于冷钱包/热钱包复用）。
  ///
  /// 格式: `[0x09][0x00][proposal_id:u64_le][approve:bool]`。
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
  }) async {
    debugPrint('[InternalVote] 步骤1: 获取 metadata...');
    final metadata = await _rpc.fetchMetadata();
    debugPrint('[InternalVote] 步骤2: 获取 genesisHash...');
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

    debugPrint('[InternalVote] 步骤3: 并行获取 runtimeVersion/nonce/latestBlock...');
    final results = await Future.wait([
      _rpc.fetchRuntimeVersion(),
      NonceManager.instance.getNextNonce(
        address: fromAddress,
        fetchChainNonce: _rpc.fetchNonce,
      ),
      _rpc.fetchLatestBlock(),
    ]);
    final runtimeVersion = results[0] as dynamic;
    final nonce = results[1] as int;
    final latestBlock = results[2] as ({Uint8List blockHash, int blockNumber});
    debugPrint('[InternalVote] nonce=$nonce, block=${latestBlock.blockNumber}');

    debugPrint('[InternalVote] 步骤4: 构造签名载荷...');
    final signingPayload = SigningPayload(
      method: callData,
      specVersion: runtimeVersion.specVersion,
      transactionVersion: runtimeVersion.transactionVersion,
      genesisHash: '0x${_hexEncode(genesisHash)}',
      blockHash: '0x${_hexEncode(latestBlock.blockHash)}',
      blockNumber: latestBlock.blockNumber,
      eraPeriod: _eraPeriod,
      nonce: nonce,
      tip: 0,
    );
    final payloadBytes = signingPayload.encode(registry);

    debugPrint('[InternalVote] 步骤5: 签名 (${payloadBytes.length} bytes)...');
    final signature = await sign(payloadBytes);
    debugPrint('[InternalVote] 签名完成 (${signature.length} bytes)');

    debugPrint('[InternalVote] 步骤6: 编码 extrinsic...');
    final extrinsicPayload = ExtrinsicPayload(
      signer: signerPubkey,
      method: callData,
      signature: signature,
      eraPeriod: _eraPeriod,
      blockNumber: latestBlock.blockNumber,
      nonce: nonce,
      tip: 0,
    );
    final encoded = extrinsicPayload.encode(registry, SignatureType.sr25519);
    debugPrint('[InternalVote] extrinsic 编码完成 (${encoded.length} bytes)');

    debugPrint('[InternalVote] 步骤7: 提交到链...');
    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      debugPrint('[InternalVote] 提交成功: 0x${_hexEncode(txHash)}');
      return (txHash: '0x${_hexEncode(txHash)}', usedNonce: nonce);
    } catch (e) {
      NonceManager.instance.rollback(fromAddress);
      debugPrint('[InternalVote] 提交失败: $e');
      rethrow;
    }
  }

  // ──── 内部：编码工具 ────

  static Uint8List _u64ToLeBytes(int value) {
    final bytes = Uint8List(8);
    final bd = ByteData.sublistView(bytes);
    bd.setUint64(0, value, Endian.little);
    return bytes;
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
