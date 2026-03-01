import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wuminapp_mobile/services/wallet_service.dart';
import 'package:wuminapp_mobile/trade/onchain/models/onchain_trade_models.dart';
import 'package:wuminapp_mobile/trade/onchain/repositories/onchain_trade_repository.dart';
import 'package:wuminapp_mobile/trade/onchain/services/onchain_trade_gateway.dart';

class OnchainTradeService {
  OnchainTradeService({
    WalletService? walletService,
    OnchainTradeRepository? repository,
    OnchainTradeGateway? gateway,
  })  : _walletService = walletService ?? WalletService(),
        _repository = repository ?? LocalOnchainTradeRepository(),
        _gateway = gateway ?? HttpOnchainTradeGateway();

  final WalletService _walletService;
  final OnchainTradeRepository _repository;
  final OnchainTradeGateway _gateway;

  Future<WalletProfile?> getCurrentWallet() {
    return _walletService.getWallet();
  }

  Future<OnchainTxRecord> submitTransfer(OnchainTransferDraft draft) async {
    final toAddress = draft.toAddress.trim();
    final symbol = draft.symbol.trim().toUpperCase();
    if (toAddress.isEmpty || symbol.isEmpty || draft.amount <= 0) {
      throw const OnchainTradeException(
        OnchainTradeErrorCode.invalidDraft,
        '交易草稿不合法，请检查收款地址、数量和币种',
      );
    }

    final walletSecret = await _walletService.getLatestWalletSecret();
    if (walletSecret == null) {
      throw const OnchainTradeException(
        OnchainTradeErrorCode.walletMissing,
        '请先创建或导入钱包，再进行链上交易',
      );
    }

    final wallet = walletSecret.profile;
    final pair = await Keyring.sr25519.fromMnemonic(walletSecret.mnemonic);
    pair.ss58Format = wallet.ss58;
    final localPubkeyHex = _toHex(pair.bytes().toList(growable: false));
    if (localPubkeyHex.toLowerCase() != wallet.pubkeyHex.toLowerCase()) {
      throw const OnchainTradeException(
        OnchainTradeErrorCode.walletMismatch,
        '钱包密钥与地址不匹配，请重新导入钱包',
      );
    }

    final signedAt = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final nonce = DateTime.now().microsecondsSinceEpoch.toString();
    final amountText = draft.amount.toStringAsFixed(8);
    final signMessage = [
      'WUMINAPP_TX_V1',
      wallet.address,
      toAddress,
      amountText,
      symbol,
      nonce,
      signedAt.toString(),
    ].join('|');
    final signature = pair.sign(Uint8List.fromList(utf8.encode(signMessage)));
    final signedTransfer = OnchainSignedTransfer(
      fromAddress: wallet.address,
      pubkeyHex: '0x${wallet.pubkeyHex}',
      toAddress: toAddress,
      amount: draft.amount,
      symbol: symbol,
      nonce: nonce,
      signedAt: signedAt,
      signMessage: signMessage,
      signatureHex: '0x${_toHex(signature.toList(growable: false))}',
    );

    OnchainSubmitResult submitResult;
    try {
      submitResult = await _gateway.submitTransfer(signedTransfer);
    } catch (_) {
      throw const OnchainTradeException(
        OnchainTradeErrorCode.broadcastFailed,
        '交易广播失败，请稍后重试',
      );
    }

    final now = DateTime.now();
    final record = OnchainTxRecord(
      txHash: submitResult.txHash,
      fromAddress: wallet.address,
      toAddress: toAddress,
      amount: draft.amount,
      symbol: symbol,
      createdAt: now,
      status: submitResult.status,
      failureReason: submitResult.failureReason,
    );
    await _repository.save(record);

    if (record.status == OnchainTxStatus.failed) {
      throw OnchainTradeException(
        OnchainTradeErrorCode.broadcastFailed,
        submitResult.failureReason ?? '交易广播失败，请稍后重试',
      );
    }
    return record;
  }

  Future<List<OnchainTxRecord>> listRecentRecords() async {
    final all = await _repository.listRecent();
    final wallet = await _walletService.getWallet();
    if (wallet == null) {
      return const <OnchainTxRecord>[];
    }
    return all.where((it) => it.fromAddress == wallet.address).toList();
  }

  Future<List<OnchainTxRecord>> refreshPendingRecords() async {
    final records = await _repository.listRecent();
    for (final record in records) {
      if (onchainTxStatusIsFinal(record.status)) {
        continue;
      }
      try {
        final status = await _gateway.queryStatus(record.txHash);
        final updated = record.copyWith(
          status: status.status,
          failureReason: status.failureReason,
          clearFailureReason: status.failureReason == null,
        );
        await _repository.upsert(updated);
      } catch (_) {
        // Keep the previous status when polling fails.
      }
    }
    return listRecentRecords();
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buf = StringBuffer();
    for (final b in bytes) {
      buf
        ..write(chars[(b >> 4) & 0x0f])
        ..write(chars[b & 0x0f]);
    }
    return buf.toString();
  }
}
