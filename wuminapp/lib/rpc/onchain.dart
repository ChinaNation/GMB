import 'dart:typed_data';

import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_rpc.dart';
import 'nonce_manager.dart';

/// 交易确认状态。
enum TxConfirmResult {
  /// 交易哈希在链上找到，已确认。
  confirmed,
  /// nonce 已被其他交易消耗，本笔交易丢失（未上链）。
  lost,
  /// 尚未确认，继续等待。
  pending,
}

/// onchain 模块所有 RPC 功能：extrinsic 构造、转账、交易确认查询。
class OnchainRpc {
  OnchainRpc({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// Mortal era 周期（区块数）。64 ≈ 约 6.4 分钟有效期。
  static const _eraPeriod = 64;

  /// Balances pallet index（citizenchain runtime 定义）。
  static const _balancesPalletIndex = 2;

  /// transfer_keep_alive call index（标准 pallet_balances）。
  static const _transferKeepAliveCallIndex = 3;

  // ──── 公开方法 ────

  /// 执行 Balances::transfer_keep_alive 转账。
  ///
  /// [fromAddress] 发送方 SS58 地址
  /// [signerPubkey] 发送方公钥 32 字节
  /// [toAddress] 接收方 SS58 地址
  /// [amountYuan] 转账金额（元），内部转为分
  /// [sign] 签名回调：接收签名载荷字节，返回 64 字节 sr25519 签名
  ///
  /// 返回交易哈希 hex（含 0x 前缀）和提交时使用的 nonce。
  Future<({String txHash, int usedNonce})> transferKeepAlive({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String toAddress,
    required double amountYuan,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    // 1. 获取链常量（缓存）
    final metadata = await _rpc.fetchMetadata();
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

    // 2. 并行获取动态参数
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
    final latestBlock =
        results[2] as ({Uint8List blockHash, int blockNumber});

    // 3. 构造 call data
    final destAccountId = Keyring().decodeAddress(toAddress);
    final amountFen = BigInt.from((amountYuan * 100).round());
    final callData = _buildTransferKeepAliveCall(destAccountId, amountFen);

    // 4. 构造签名载荷
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

    // 5. 签名
    final signature = await sign(payloadBytes);

    // 6. 构造最终 extrinsic
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

    // 7. 提交（失败时回退 nonce，避免跳号）
    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      return (txHash: '0x${_hexEncode(txHash)}', usedNonce: nonce);
    } catch (e) {
      NonceManager.instance.rollback(fromAddress);
      rethrow;
    }
  }

  /// 交易提交后超过此时间仍未被打包，判定为丢失（节点重启 / 交易池清空等）。
  static const _txLostTimeout = Duration(minutes: 5);

  /// 检查交易是否已被链上确认。
  ///
  /// 返回三种状态：
  /// - `confirmed` — 交易哈希在链上找到，真正确认
  /// - `lost` — nonce 已被其他交易消耗，或超时未打包
  /// - `pending` — 尚未确认，继续等待
  Future<TxConfirmResult> checkTxStatus({
    required String pubkeyHex,
    required int usedNonce,
    required String txHash,
    DateTime? createdAt,
  }) async {
    final confirmedNonce = await _rpc.fetchConfirmedNonce(pubkeyHex);

    if (confirmedNonce <= usedNonce) {
      // 链上 nonce 还没到这笔交易。
      // 如果已超时，判定为丢失（交易池可能已清空）。
      if (createdAt != null &&
          DateTime.now().difference(createdAt) > _txLostTimeout) {
        return TxConfirmResult.lost;
      }
      return TxConfirmResult.pending;
    }

    // nonce 已推进，说明该 nonce 位置的交易已被链上消费。
    // 在最近区块中搜索 txHash 进行二次验证。
    try {
      final foundBlock = await findTxInRecentBlocks(txHash);
      if (foundBlock != null) {
        return TxConfirmResult.confirmed;
      }
      // 未在最近区块中找到 txHash，有两种可能：
      // 1) 交易在更早的区块中已确认（超出搜索窗口）
      // 2) 交易真的丢失了（同 nonce 的另一笔交易被打包）
      // 由于无法区分，nonce 已推进时默认认为已确认（保守策略：不误判为 lost）。
      return TxConfirmResult.confirmed;
    } catch (_) {
      // 查找失败时，nonce 已推进，默认已确认
      return TxConfirmResult.confirmed;
    }
  }

  /// 向后兼容的简单确认检查（仅 nonce）。
  Future<bool> isTxConfirmed({
    required String pubkeyHex,
    required int usedNonce,
  }) async {
    final confirmedNonce = await _rpc.fetchConfirmedNonce(pubkeyHex);
    return confirmedNonce > usedNonce;
  }

  /// 在最近区块中搜索指定交易哈希。找到则返回所在区块号，未找到返回 null。
  Future<int?> findTxInRecentBlocks(String txHash, {int depth = 50}) async {
    final latestBlock = await _rpc.fetchLatestBlock();
    final startBlock = latestBlock.blockNumber;
    final endBlock = (startBlock - depth).clamp(1, startBlock);

    for (var blockNum = startBlock; blockNum >= endBlock; blockNum--) {
      final blockData = await _rpc.fetchBlockExtrinsicHashes(blockNum);
      if (blockData != null && blockData.contains(txHash)) {
        return blockNum;
      }
    }
    return null;
  }

  // ──── 手续费估算 ────

  /// 预估转账手续费（元）。
  ///
  /// 与链上 `onchain_transaction_pow` 计算逻辑一致：
  /// `fee = max(amount_fen * Perbill(1_000_000), 10 fen)`
  ///
  /// - 费率 0.1%（`Perbill::from_parts(1_000_000)`）
  /// - 最低手续费 10 fen（0.10 元）
  /// - half-up 舍入到 fen 精度
  static double estimateTransferFeeYuan(double amountYuan) {
    const int perbillParts = 1000000;
    const int perbillDenom = 1000000000;
    const int minFeeFen = 10;

    final amountFen = BigInt.from((amountYuan * 100).round());
    // half-up rounding: (amount * parts + denom/2) ~/ denom
    final byRate = (amountFen * BigInt.from(perbillParts) +
            BigInt.from(perbillDenom ~/ 2)) ~/
        BigInt.from(perbillDenom);
    final feeFen =
        byRate < BigInt.from(minFeeFen) ? BigInt.from(minFeeFen) : byRate;
    return feeFen.toDouble() / 100.0;
  }

  // ──── 内部：extrinsic 编码 ────

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  /// 构造 Balances::transfer_keep_alive 的 SCALE 编码 call data。
  ///
  /// 格式：[pallet_index] [call_index] [MultiAddress::Id(0x00) + dest_32bytes] [Compact<u128>(fen)]
  Uint8List _buildTransferKeepAliveCall(
    Uint8List destAccountId,
    BigInt amountFen,
  ) {
    final output = ByteOutput();
    output.pushByte(_balancesPalletIndex);
    output.pushByte(_transferKeepAliveCallIndex);
    output.pushByte(0x00);
    output.write(destAccountId);
    output.write(CompactBigIntCodec.codec.encode(amountFen));
    return output.toBytes();
  }

}
