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

class ApiClient {
  ApiClient({String? baseUrl}) : _baseUrl = baseUrl ?? _defaultBaseUrl;

  final String _baseUrl;

  static String get _defaultBaseUrl {
    if (Platform.isAndroid) {
      return 'http://10.0.2.2:8787';
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
}
