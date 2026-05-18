import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_rpc.dart';
import 'signed_extrinsic_builder.dart';

/// onchain 模块所有 RPC 功能：extrinsic 构造与普通转账提交。
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

  // 2026-04-23 整改:`findTxInRecentBlocks` 已删除。
  // 原实现逐块调 `fetchBlockExtrinsicHashes` → `getBlockExtrinsics`,
  // 触发 substrate block-request 反滥用机制(MAX_NUMBER_OF_SAME_REQUESTS_PER_PEER=2)
  // 把轻节点 peer ban 掉。钱包交易流水改由 finalized 事件监听写入本地记录。

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
