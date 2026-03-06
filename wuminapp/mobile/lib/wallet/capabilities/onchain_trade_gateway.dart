import 'dart:math';

import 'package:wuminapp_mobile/wallet/capabilities/api_client.dart';
import 'package:wuminapp_mobile/wallet/capabilities/onchain_trade_models.dart';

abstract class OnchainTradeGateway {
  Future<OnchainPrepareResult> prepareTransfer(OnchainPrepareRequest request);

  Future<OnchainSubmitResult> submitTransfer(
    OnchainSignedPreparedTransfer transfer,
  );

  Future<OnchainSubmitResult> queryStatus(String txHash);
}

class HttpOnchainTradeGateway implements OnchainTradeGateway {
  HttpOnchainTradeGateway({ApiClient? apiClient})
      : _apiClient = apiClient ?? ApiClient();

  final ApiClient _apiClient;

  @override
  Future<OnchainPrepareResult> prepareTransfer(
    OnchainPrepareRequest request,
  ) async {
    final response = await _apiClient.prepareTx({
      'from_address': request.fromAddress,
      'pubkey_hex': request.pubkeyHex,
      'to_address': request.toAddress,
      'amount': request.amount,
      'symbol': request.symbol,
    });

    return OnchainPrepareResult(
      preparedId: response.preparedId,
      signerPayloadHex: response.signerPayloadHex,
      expiresAt: response.expiresAt,
    );
  }

  @override
  Future<OnchainSubmitResult> submitTransfer(
    OnchainSignedPreparedTransfer transfer,
  ) async {
    final response = await _apiClient.submitTx({
      'prepared_id': transfer.preparedId,
      'pubkey_hex': transfer.pubkeyHex,
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
  Future<OnchainPrepareResult> prepareTransfer(
    OnchainPrepareRequest request,
  ) async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    return OnchainPrepareResult(
      preparedId: now.toString(),
      signerPayloadHex: '0xdeadbeef',
      expiresAt: now + 120,
    );
  }

  @override
  Future<OnchainSubmitResult> submitTransfer(
      OnchainSignedPreparedTransfer transfer) async {
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
