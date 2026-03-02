import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;

class HealthStatus {
  const HealthStatus({
    required this.service,
    required this.version,
    required this.status,
  });

  final String service;
  final String version;
  final String status;
}

class TxSubmitResponse {
  const TxSubmitResponse({
    required this.txHash,
    required this.status,
    this.failureReason,
  });

  final String txHash;
  final String status;
  final String? failureReason;
}

class TxPrepareResponse {
  const TxPrepareResponse({
    required this.preparedId,
    required this.signerPayloadHex,
    required this.expiresAt,
  });

  final String preparedId;
  final String signerPayloadHex;
  final int expiresAt;
}

class TxStatusResponse {
  const TxStatusResponse({
    required this.txHash,
    required this.status,
    required this.updatedAt,
    this.failureReason,
  });

  final String txHash;
  final String status;
  final int updatedAt;
  final String? failureReason;
}

class WalletBalanceResponse {
  const WalletBalanceResponse({
    required this.account,
    required this.balance,
    required this.symbol,
    required this.updatedAt,
  });

  final String account;
  final double balance;
  final String symbol;
  final int updatedAt;
}

class ApiClient {
  ApiClient({String? baseUrl}) : _baseUrl = baseUrl ?? _defaultBaseUrl;

  final String _baseUrl;

  static String get _defaultBaseUrl {
    if (Platform.isAndroid) {
      const fromDefine = String.fromEnvironment('WUMINAPP_API_BASE_URL');
      if (fromDefine.isEmpty) {
        throw StateError(
          'Android真机调试请传 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
      }
      return fromDefine;
    }
    return 'http://127.0.0.1:8787';
  }

  Future<HealthStatus> fetchHealth() async {
    final uri = Uri.parse('$_baseUrl/api/v1/health');
    final response = await http.get(uri);
    if (response.statusCode != 200) {
      throw Exception('health check failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final data = payload['data'] as Map<String, dynamic>;
    return HealthStatus(
      service: data['service'] as String? ?? '-',
      version: data['version'] as String? ?? '-',
      status: data['status'] as String? ?? '-',
    );
  }

  Future<TxSubmitResponse> submitTx(Map<String, dynamic> body) async {
    final uri = Uri.parse('$_baseUrl/api/v1/tx/submit');
    final response = await http.post(
      uri,
      headers: const {'Content-Type': 'application/json'},
      body: jsonEncode(body),
    );
    if (response.statusCode != 200) {
      throw Exception('tx submit failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('tx submit rejected: code=$code message=$message');
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('tx submit invalid response: missing data');
    }

    final txHash = data['tx_hash']?.toString();
    if (txHash == null || txHash.isEmpty) {
      throw Exception('tx submit invalid response: missing tx_hash');
    }

    return TxSubmitResponse(
      txHash: txHash,
      status: data['status']?.toString() ?? 'pending',
      failureReason: data['failure_reason']?.toString(),
    );
  }

  Future<TxPrepareResponse> prepareTx(Map<String, dynamic> body) async {
    final uri = Uri.parse('$_baseUrl/api/v1/tx/prepare');
    final response = await http.post(
      uri,
      headers: const {'Content-Type': 'application/json'},
      body: jsonEncode(body),
    );
    if (response.statusCode != 200) {
      throw Exception('tx prepare failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('tx prepare rejected: code=$code message=$message');
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('tx prepare invalid response: missing data');
    }

    final preparedId = data['prepared_id']?.toString();
    final signerPayloadHex = data['signer_payload_hex']?.toString();
    if (preparedId == null ||
        preparedId.isEmpty ||
        signerPayloadHex == null ||
        signerPayloadHex.isEmpty) {
      throw Exception('tx prepare invalid response: missing payload fields');
    }

    return TxPrepareResponse(
      preparedId: preparedId,
      signerPayloadHex: signerPayloadHex,
      expiresAt: data['expires_at'] as int? ?? 0,
    );
  }

  Future<TxStatusResponse> fetchTxStatus(String txHash) async {
    final encodedHash = Uri.encodeComponent(txHash);
    final uri = Uri.parse('$_baseUrl/api/v1/tx/status/$encodedHash');
    final response = await http.get(uri);
    if (response.statusCode != 200) {
      throw Exception('tx status failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('tx status invalid response: missing data');
    }

    return TxStatusResponse(
      txHash: data['tx_hash']?.toString() ?? txHash,
      status: data['status']?.toString() ?? (code == 0 ? 'pending' : 'failed'),
      updatedAt: data['updated_at'] as int? ?? 0,
      failureReason:
          data['failure_reason']?.toString() ?? (code == 0 ? null : message),
    );
  }

  Future<WalletBalanceResponse> fetchWalletBalance(
    String account, {
    String? pubkeyHex,
  }) async {
    final encoded = Uri.encodeQueryComponent(account);
    final pubkeyParam = (pubkeyHex != null && pubkeyHex.trim().isNotEmpty)
        ? '&pubkey_hex=${Uri.encodeQueryComponent(pubkeyHex)}'
        : '';
    final uri = Uri.parse(
      '$_baseUrl/api/v1/wallet/balance?account=$encoded$pubkeyParam',
    );
    final response = await http.get(uri);
    if (response.statusCode != 200) {
      throw Exception('wallet balance failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('wallet balance rejected: code=$code message=$message');
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('wallet balance invalid response: missing data');
    }
    final rawBalance = data['balance'];
    final balance = switch (rawBalance) {
      num v => v.toDouble(),
      String v => double.tryParse(v) ?? 0.0,
      _ => 0.0,
    };

    return WalletBalanceResponse(
      account: data['account']?.toString() ?? account,
      balance: balance,
      symbol: data['symbol']?.toString() ?? 'CIT',
      updatedAt: data['updated_at'] as int? ?? 0,
    );
  }
}
