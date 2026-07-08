import 'dart:convert';

import 'package:http/http.dart' as http;

import 'chain_bootstrap_api.dart';

class SignedExtrinsicRelayApiException implements Exception {
  const SignedExtrinsicRelayApiException(
    this.message, {
    this.statusCode,
    this.errorCode,
  });

  final String message;
  final int? statusCode;
  final String? errorCode;

  @override
  String toString() => message;
}

class SignedExtrinsicRelayApi {
  SignedExtrinsicRelayApi({
    String? baseUrl,
    http.Client? httpClient,
    this.timeout = const Duration(seconds: 10),
  })  : baseUrl = ChainBootstrapApiConfig.normalizeBaseUrl(
          baseUrl ?? ChainBootstrapApiConfig.defaultBaseUrl,
        ),
        _http = httpClient ?? http.Client();

  static const relayPath = '/v1/chain/extrinsics/relay';
  static const maxExtrinsicBytes = 64 * 1024;

  final String baseUrl;
  final http.Client _http;
  final Duration timeout;

  Future<SignedExtrinsicRelayResult> relaySignedExtrinsic({
    required String signedExtrinsicHex,
  }) async {
    final normalized = normalizeSignedExtrinsicHex(signedExtrinsicHex);
    final response = await _http
        .post(
          Uri.parse('$baseUrl$relayPath'),
          headers: const {
            'content-type': 'application/json; charset=utf-8',
            'accept': 'application/json',
          },
          body: jsonEncode({
            'signed_extrinsic_hex': normalized,
          }),
        )
        .timeout(timeout);

    final decoded = _decodeObject(response);
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw SignedExtrinsicRelayApiException(
        decoded['message']?.toString() ?? '签名交易广播兜底失败',
        statusCode: response.statusCode,
        errorCode: decoded['error_code']?.toString(),
      );
    }
    return SignedExtrinsicRelayResult.fromJson(decoded);
  }

  void close() => _http.close();

  static String normalizeSignedExtrinsicHex(String value) {
    final normalized = value.trim().toLowerCase();
    if (!RegExp(r'^0x[0-9a-f]+$').hasMatch(normalized) ||
        normalized.length.isOdd) {
      throw const SignedExtrinsicRelayApiException('签名交易 hex 格式不合法');
    }
    final byteSize = (normalized.length - 2) ~/ 2;
    if (byteSize <= 0 || byteSize > maxExtrinsicBytes) {
      throw const SignedExtrinsicRelayApiException('签名交易大小超出限制');
    }
    return normalized;
  }
}

class SignedExtrinsicRelayResult {
  const SignedExtrinsicRelayResult({
    required this.relayId,
    required this.relayStatus,
    required this.deduplicated,
    required this.txHash,
    required this.acceptedAt,
    required this.chainSuccessSource,
  });

  final String relayId;
  final String relayStatus;
  final bool deduplicated;
  final String txHash;
  final int acceptedAt;
  final String chainSuccessSource;

  factory SignedExtrinsicRelayResult.fromJson(Map<String, dynamic> json) {
    if (json['ok'] != true ||
        json['schema'] != 'citizenapp.chain.extrinsic_relay.v1') {
      throw const SignedExtrinsicRelayApiException('签名交易广播响应 schema 不匹配');
    }
    final result = SignedExtrinsicRelayResult(
      relayId: _string(json, 'relay_id'),
      relayStatus: _string(json, 'relay_status'),
      deduplicated: _bool(json, 'deduplicated'),
      txHash: _txHash(json, 'tx_hash'),
      acceptedAt: _int(json, 'accepted_at'),
      chainSuccessSource: _string(json, 'chain_success_source'),
    );
    if (result.relayStatus != 'broadcast' ||
        result.chainSuccessSource != 'finalized_runtime_storage_or_events') {
      throw const SignedExtrinsicRelayApiException('签名交易广播响应违反最终性边界');
    }
    return result;
  }
}

Map<String, dynamic> _decodeObject(http.Response response) {
  try {
    final decoded = jsonDecode(response.body);
    if (decoded is Map<String, dynamic>) {
      return decoded;
    }
  } catch (_) {
    // 统一落到下面的结构错误，避免把 HTML/网关错误当成链响应。
  }
  throw SignedExtrinsicRelayApiException(
    '签名交易广播响应不是 JSON 对象',
    statusCode: response.statusCode,
  );
}

String _string(Map<String, dynamic> json, String key) {
  final value = json[key];
  if (value is String && value.trim().isNotEmpty) {
    return value;
  }
  throw SignedExtrinsicRelayApiException('签名交易广播响应缺少 $key');
}

int _int(Map<String, dynamic> json, String key) {
  final value = json[key];
  if (value is int && value >= 0) {
    return value;
  }
  throw SignedExtrinsicRelayApiException('签名交易广播响应字段 $key 无效');
}

bool _bool(Map<String, dynamic> json, String key) {
  final value = json[key];
  if (value is bool) {
    return value;
  }
  throw SignedExtrinsicRelayApiException('签名交易广播响应字段 $key 无效');
}

String _txHash(Map<String, dynamic> json, String key) {
  final value = _string(json, key).toLowerCase();
  if (RegExp(r'^0x[0-9a-f]{64}$').hasMatch(value)) {
    return value;
  }
  throw SignedExtrinsicRelayApiException('签名交易广播响应字段 $key 不是交易哈希');
}
