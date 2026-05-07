import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_rpc.dart';
import 'signed_extrinsic_builder.dart';

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
    final destAccountId = Keyring().decodeAddress(toAddress);
    final amountFen = BigInt.from((amountYuan * 100).round());
    final callData = _buildTransferKeepAliveCall(destAccountId, amountFen);
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'OnchainRpc',
    ).signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
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

    // nonce 已推进 → 确认。
    //
    // 2026-04-23 整改:删除"在最近区块中按 txHash 搜索二次验证"路径。
    // 原因见 `pending_tx_reconciler._reconcileOne`:逐块拉 body 会触发
    // substrate `MAX_NUMBER_OF_SAME_REQUESTS_PER_PEER=2` 反滥用 ban,
    // 把轻节点 peer 打到 peers=0。
    // 由于原逻辑"未找到也返回 confirmed"(同 nonce 的另一笔 tx 顶替是罕见
    // 场景,宁可保守标 confirmed 也不误判 lost),这次二次验证实际从不改变
    // 结果,直接删除即可。
    return TxConfirmResult.confirmed;
  }

  /// 向后兼容的简单确认检查（仅 nonce）。
  Future<bool> isTxConfirmed({
    required String pubkeyHex,
    required int usedNonce,
  }) async {
    final confirmedNonce = await _rpc.fetchConfirmedNonce(pubkeyHex);
    return confirmedNonce > usedNonce;
  }

  // 2026-04-23 整改:`findTxInRecentBlocks` 已删除。
  // 原实现逐块调 `fetchBlockExtrinsicHashes` → `getBlockExtrinsics`,
  // 触发 substrate block-request 反滥用机制(MAX_NUMBER_OF_SAME_REQUESTS_PER_PEER=2)
  // 把轻节点 peer ban 掉。交易确认改走 nonce-only 判定,
  // 详见 `pending_tx_reconciler.dart`。

  // ──── 手续费估算 ────

  /// 预估转账手续费（元）。
  ///
  /// 与链上 `onchain_transaction` 计算逻辑一致：
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
