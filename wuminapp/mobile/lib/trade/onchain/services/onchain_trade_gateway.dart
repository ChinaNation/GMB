import 'dart:math';

import 'package:wuminapp_mobile/services/api_client.dart';
import 'package:wuminapp_mobile/trade/onchain/models/onchain_trade_models.dart';

abstract class OnchainTradeGateway {
  Future<OnchainSubmitResult> submitTransfer(OnchainSignedTransfer transfer);

  Future<OnchainSubmitResult> queryStatus(String txHash);
}

class HttpOnchainTradeGateway implements OnchainTradeGateway {
  HttpOnchainTradeGateway({ApiClient? apiClient})
      : _apiClient = apiClient ?? ApiClient();

  final ApiClient _apiClient;

  @override
  Future<OnchainSubmitResult> submitTransfer(
      OnchainSignedTransfer transfer) async {
    final response = await _apiClient.submitTx({
      'from_address': transfer.fromAddress,
      'pubkey_hex': transfer.pubkeyHex,
      'to_address': transfer.toAddress,
      'amount': transfer.amount,
      'symbol': transfer.symbol,
      'nonce': transfer.nonce,
      'signed_at': transfer.signedAt,
      'sign_message': transfer.signMessage,
      'signature_hex': transfer.signatureHex,
    });

    return OnchainSubmitResult(
      txHash: response.txHash,
      status: _toStatus(response.status),
      failureReason: response.failureReason,
    );
  }

  @override
  Future<OnchainSubmitResult> queryStatus(String txHash) async {
    final response = await _apiClient.fetchTxStatus(txHash);
    return OnchainSubmitResult(
      txHash: response.txHash,
      status: _toStatus(response.status),
      failureReason: response.failureReason,
    );
  }

  OnchainTxStatus _toStatus(String status) {
    switch (status.toLowerCase()) {
      case 'confirmed':
        return OnchainTxStatus.confirmed;
      case 'failed':
        return OnchainTxStatus.failed;
      case 'pending':
      default:
        return OnchainTxStatus.pending;
    }
  }
}

class MockOnchainTradeGateway implements OnchainTradeGateway {
  MockOnchainTradeGateway({double failureRate = 0})
      : _failureRate = failureRate.clamp(0, 1).toDouble();

  final double _failureRate;
  final Random _random = Random();

  @override
  Future<OnchainSubmitResult> submitTransfer(
      OnchainSignedTransfer transfer) async {
    final now = DateTime.now();
    final txHash = '0x${now.microsecondsSinceEpoch.toRadixString(16)}';
    final failed = _random.nextDouble() < _failureRate;
    if (failed) {
      return OnchainSubmitResult(
        txHash: txHash,
        status: OnchainTxStatus.failed,
        failureReason: 'mock broadcast failed',
      );
    }
    return OnchainSubmitResult(
      txHash: txHash,
      status: OnchainTxStatus.pending,
    );
  }

  @override
  Future<OnchainSubmitResult> queryStatus(String txHash) async {
    final roll = _random.nextInt(100);
    if (roll < 75) {
      return OnchainSubmitResult(
        txHash: txHash,
        status: OnchainTxStatus.pending,
      );
    }
    return OnchainSubmitResult(
      txHash: txHash,
      status: OnchainTxStatus.confirmed,
    );
  }
}
