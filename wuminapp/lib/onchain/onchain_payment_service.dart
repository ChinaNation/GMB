import 'dart:typed_data';

import 'package:wuminapp_mobile/onchain/onchain_payment_models.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

class OnchainPaymentService {
  OnchainPaymentService({
    WalletManager? walletManager,
    OnchainRpc? onchainRpc,
  })  : _walletManager = walletManager ?? WalletManager(),
        _onchainRpc = onchainRpc ?? OnchainRpc();

  final WalletManager _walletManager;
  final OnchainRpc _onchainRpc;

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
  }) async {
    final toAddress = draft.toAddress.trim();
    final symbol = draft.symbol.trim().toUpperCase();
    if (toAddress.isEmpty || symbol.isEmpty || draft.amount <= 0) {
      throw const OnchainPaymentException(
        OnchainPaymentErrorCode.invalidDraft,
        '交易草稿不合法，请检查收款地址、数量和币种',
      );
    }

    final wallet = await _walletManager.getWallet();
    if (wallet == null) {
      throw const OnchainPaymentException(
        OnchainPaymentErrorCode.walletMissing,
        '请先创建或导入钱包，再进行链上交易',
      );
    }

    final pubkeyBytes = _hexToBytes(wallet.pubkeyHex);

    ({String txHash, int usedNonce}) result;
    try {
      result = await _onchainRpc.transferKeepAlive(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        toAddress: toAddress,
        amountYuan: draft.amount,
        sign: sign,
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
