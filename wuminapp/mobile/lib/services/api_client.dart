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
}
