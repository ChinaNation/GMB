import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_rpc.dart';
import 'signed_extrinsic_builder.dart';

/// onchain 模块所有 RPC 功能：extrinsic 构造与普通转账提交。
class TransferRpc {
  TransferRpc({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// 普通转账备注最大 UTF-8 字节数，与 runtime `MaxTransferRemarkLen` 保持一致。
  static const int maxTransferRemarkBytes = 99;

  /// OnchainTransaction pallet index（citizenchain runtime 定义）。
  static const _onchainTransactionPalletIndex = 4;

  /// transfer_with_remark call index。
  static const _transferWithRemarkCallIndex = 0;

  // ──── 公开方法 ────

  /// 执行 OnchainTransaction::transfer_with_remark 转账。
  ///
  /// [fromSs58Address] 发送方 SS58 地址
  /// [signerPublicKey] 发送方公钥 32 字节
  /// [toSs58Address] 接收方 SS58 地址
  /// [amountYuan] 转账金额（元），内部转为分
  /// [remark] 转账备注，按 UTF-8 字节编码并随交易事件上链
  /// [sign] 签名回调：接收签名载荷字节，返回 64 字节 sr25519 签名
  ///
  /// 返回交易哈希 hex（含 0x 前缀）和提交时使用的 nonce。
  Future<({String txHash, int usedNonce})> transferWithRemark({
    required String fromSs58Address,
    required Uint8List signerPublicKey,
    required String toSs58Address,
    required double amountYuan,
    required String remark,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final destAccountId = Keyring().decodeAddress(toSs58Address);
    final amountFen = BigInt.from((amountYuan * 100).round());
    final remarkBytes = Uint8List.fromList(utf8.encode(remark));
    final callData =
        _buildTransferWithRemarkCall(destAccountId, amountFen, remarkBytes);
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'TransferRpc',
    ).signAndSubmit(
      callData: callData,
      fromSs58Address: fromSs58Address,
      signerPublicKey: signerPublicKey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
  }

  // 钱包交易流水由区块事件监听写入本地记录,不逐块拉 extrinsic 搜索
  // (逐块拉 body 会触发 substrate block-request 反滥用机制
  // MAX_NUMBER_OF_SAME_REQUESTS_PER_PEER=2 把轻节点 peer ban 掉)。

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

  /// 构造 OnchainTransaction::transfer_with_remark 的 SCALE 编码 call data。
  ///
  /// 格式：[pallet_index=4] [call_index=0] [beneficiary:AccountId32] [amount:u128_le] [remark:BoundedVec<u8>]
  Uint8List _buildTransferWithRemarkCall(
    Uint8List destAccountId,
    BigInt amountFen,
    Uint8List remarkBytes,
  ) {
    if (remarkBytes.length > maxTransferRemarkBytes) {
      throw ArgumentError(
        '转账备注不能超过 $maxTransferRemarkBytes 字节，当前 ${remarkBytes.length} 字节',
      );
    }
    final output = ByteOutput();
    output.pushByte(_onchainTransactionPalletIndex);
    output.pushByte(_transferWithRemarkCallIndex);
    output.write(destAccountId);
    output.write(_u128LittleEndian(amountFen));
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(remarkBytes.length)));
    output.write(remarkBytes);
    return output.toBytes();
  }

  Uint8List _u128LittleEndian(BigInt value) {
    if (value < BigInt.zero) {
      throw ArgumentError('u128 不能为负数');
    }
    final out = Uint8List(16);
    var remaining = value;
    for (var i = 0; i < out.length; i++) {
      out[i] = (remaining & BigInt.from(0xff)).toInt();
      remaining = remaining >> 8;
    }
    if (remaining != BigInt.zero) {
      throw ArgumentError('金额超出 u128 范围');
    }
    return out;
  }
}
