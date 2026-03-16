import 'dart:typed_data';

import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_models.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_repository.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

class OnchainTradeService {
  OnchainTradeService({
    WalletManager? walletManager,
    OnchainTradeRepository? repository,
    OnchainRpc? onchainRpc,
  })  : _walletManager = walletManager ?? WalletManager(),
        _repository = repository ?? LocalOnchainTradeRepository(),
        _onchainRpc = onchainRpc ?? OnchainRpc();

  final WalletManager _walletManager;
  final OnchainTradeRepository _repository;
  final OnchainRpc _onchainRpc;

  Future<WalletProfile?> getCurrentWallet() {
    return _walletManager.getWallet();
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

    final walletSecret = await _walletManager.getLatestWalletSecret();
    if (walletSecret == null) {
      throw const OnchainTradeException(
        OnchainTradeErrorCode.walletMissing,
        '请先创建或导入钱包，再进行链上交易',
      );
    }

    final wallet = walletSecret.profile;
    final pubkeyBytes = _hexToBytes(wallet.pubkeyHex);

    ({String txHash, int usedNonce}) result;
    try {
      result = await _onchainRpc.transferKeepAlive(
        fromAddress: wallet.address,
        signerPubkey: Uint8List.fromList(pubkeyBytes),
        toAddress: toAddress,
        amountYuan: draft.amount,
        sign: (payload) async {
          final pair =
              await Keyring.sr25519.fromMnemonic(walletSecret.mnemonic);
          return Uint8List.fromList(pair.sign(payload));
        },
      );
    } catch (e) {
      if (e is OnchainTradeException) rethrow;
      throw OnchainTradeException(
        OnchainTradeErrorCode.broadcastFailed,
        '交易提交失败: $e',
      );
    }

    final now = DateTime.now();
    final record = OnchainTxRecord(
      txHash: result.txHash,
      fromAddress: wallet.address,
      toAddress: toAddress,
      amount: draft.amount,
      symbol: symbol,
      createdAt: now,
      status: OnchainTxStatus.pending,
      usedNonce: result.usedNonce,
    );
    await _repository.save(record);
    return record;
  }

  Future<List<OnchainTxRecord>> listRecentRecords() async {
    final all = await _repository.listRecent();
    final wallet = await _walletManager.getWallet();
    if (wallet == null) {
      return const <OnchainTxRecord>[];
    }
    return all.where((it) => it.fromAddress == wallet.address).toList();
  }

  Future<List<OnchainTxRecord>> refreshPendingRecords() async {
    final records = await _repository.listRecent();
    for (final record in records) {
      if (onchainTxStatusIsFinal(record.status)) continue;
      if (record.usedNonce == null) continue;
      try {
        final confirmed = await _onchainRpc.isTxConfirmed(
          address: record.fromAddress,
          usedNonce: record.usedNonce!,
        );
        if (confirmed) {
          final updated = record.copyWith(status: OnchainTxStatus.confirmed);
          await _repository.upsert(updated);
        }
      } catch (_) {
        // 节点不可达时跳过，下次轮询重试
      }
    }
    return listRecentRecords();
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
