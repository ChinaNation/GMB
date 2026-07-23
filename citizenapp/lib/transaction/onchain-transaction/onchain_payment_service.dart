import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/transaction/onchain-transaction/onchain_payment_models.dart';
import 'package:citizenapp/rpc/transfer_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

class OnchainPaymentService {
  OnchainPaymentService({
    WalletManager? walletManager,
    TransferRpc? onchainRpc,
  })  : _walletManager = walletManager ?? WalletManager(),
        _onchainRpc = onchainRpc ?? TransferRpc();

  final WalletManager _walletManager;
  final TransferRpc _onchainRpc;

  Future<WalletProfile?> getCurrentWallet() {
    return _walletManager.getWallet();
  }

  /// 提交转账交易，返回交易哈希和 nonce。
  ///
  /// [sign] 回调由调用方根据钱包模式提供：
  /// - 热钱包：从 seed 派生密钥对，本机签名
  /// - 冷钱包：构造 QR 签名请求，由外部设备签名后回传
  Future<({String txHash, int usedNonce})> submitTransfer(
    OnchainPaymentDraft draft, {
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final toSs58Address = draft.toSs58Address.trim();
    final symbol = draft.symbol.trim().toUpperCase();
    final remarkBytes = utf8.encode(draft.remark).length;
    if (toSs58Address.isEmpty || symbol.isEmpty || draft.amount <= 0) {
      throw const OnchainPaymentException(
        OnchainPaymentErrorCode.invalidDraft,
        '交易草稿不合法，请检查收款地址、数量和币种',
      );
    }
    if (remarkBytes > TransferRpc.maxTransferRemarkBytes) {
      throw const OnchainPaymentException(
        OnchainPaymentErrorCode.invalidDraft,
        '转账备注不能超过 ${TransferRpc.maxTransferRemarkBytes} 字节',
      );
    }

    final wallet = await _walletManager.getWallet();
    if (wallet == null) {
      throw const OnchainPaymentException(
        OnchainPaymentErrorCode.walletMissing,
        '请先创建或导入钱包，再进行链上交易',
      );
    }

    final publicKeyBytes = _hexToBytes(wallet.accountId);

    ({String txHash, int usedNonce}) result;
    try {
      result = await _onchainRpc.transferWithRemark(
        fromSs58Address: wallet.ss58Address,
        signerPublicKey: Uint8List.fromList(publicKeyBytes),
        toSs58Address: toSs58Address,
        amountYuan: draft.amount,
        remark: draft.remark,
        sign: sign,
        onWatchEvent: onWatchEvent,
      );
    } catch (e) {
      if (e is OnchainPaymentException) rethrow;
      throw OnchainPaymentException(
        OnchainPaymentErrorCode.broadcastFailed,
        '交易提交失败: $e',
      );
    }

    return result;
  }

  List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
  }
}
