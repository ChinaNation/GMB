import 'dart:typed_data';

import 'package:wuminapp_mobile/rpc/nonce_manager.dart';
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

  /// 提交转账交易。
  ///
  /// [sign] 回调由调用方根据钱包模式提供：
  /// - 热钱包：从 seed 派生密钥对，本机签名
  /// - 冷钱包：构造 QR 签名请求，由外部设备签名后回传
  Future<OnchainTxRecord> submitTransfer(
    OnchainTransferDraft draft, {
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final toAddress = draft.toAddress.trim();
    final symbol = draft.symbol.trim().toUpperCase();
    if (toAddress.isEmpty || symbol.isEmpty || draft.amount <= 0) {
      throw const OnchainTradeException(
        OnchainTradeErrorCode.invalidDraft,
        '交易草稿不合法，请检查收款地址、数量和币种',
      );
    }

    final wallet = await _walletManager.getWallet();
    if (wallet == null) {
      throw const OnchainTradeException(
        OnchainTradeErrorCode.walletMissing,
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
      if (e is OnchainTradeException) rethrow;
      throw OnchainTradeException(
        OnchainTradeErrorCode.broadcastFailed,
        '交易提交失败: $e',
      );
    }

    final estimatedFee = OnchainRpc.estimateTransferFeeYuan(draft.amount);
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
      estimatedFee: estimatedFee,
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
    final wallet = await _walletManager.getWallet();
    if (wallet == null) return const <OnchainTxRecord>[];

    final records = await _repository.listRecent();
    for (final record in records) {
      if (onchainTxStatusIsFinal(record.status)) continue;
      if (record.usedNonce == null) {
        // 旧记录无 nonce，无法精确判断，直接标记为已确认
        final updated = record.copyWith(status: OnchainTxStatus.confirmed);
        await _repository.upsert(updated);
        continue;
      }
      try {
        // 使用交易哈希 + nonce 双重检查，避免误判丢失的交易为已确认
        final result = await _onchainRpc.checkTxStatus(
          pubkeyHex: wallet.pubkeyHex,
          usedNonce: record.usedNonce!,
          txHash: record.txHash,
        );
        switch (result) {
          case TxConfirmResult.confirmed:
            final updated =
                record.copyWith(status: OnchainTxStatus.confirmed);
            await _repository.upsert(updated);
            // 交易上链确认，清除本地 nonce 缓存，下次从链上重新获取。
            NonceManager.instance.reset(record.fromAddress);
          case TxConfirmResult.lost:
            // 交易丢失：nonce 被其他交易消耗，本笔从未上链
            final updated = record.copyWith(status: OnchainTxStatus.failed);
            await _repository.upsert(updated);
            // 交易丢失，同样清除缓存以重新同步链上 nonce。
            NonceManager.instance.reset(record.fromAddress);
          case TxConfirmResult.pending:
            break; // 继续等待
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
