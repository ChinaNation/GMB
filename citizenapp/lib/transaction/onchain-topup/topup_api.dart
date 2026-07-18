import 'dart:convert';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/services/square_api_client.dart' show SquareApiConfig;
import 'topup_models.dart';

/// 稳定币充值 Worker 客户端(/v1/square/topup/*)。
///
/// topup 接口不挂广场会话(钱包功能独立于广场登录),正确性来自链上真实到账;
/// 复用同一个 Worker 基址(SquareApiConfig)。仅做无 session 的 GET/POST。
class TopupApiException implements Exception {
  const TopupApiException(this.message, {this.statusCode, this.errorCode});

  final String message;
  final int? statusCode;
  final String? errorCode;

  @override
  String toString() => message;
}

class TopupApi {
  TopupApi({String? baseUrl, http.Client? httpClient})
      : baseUrl = SquareApiConfig.normalizeBaseUrl(
          baseUrl ?? SquareApiConfig.defaultBaseUrl,
        ),
        _http = httpClient ?? http.Client();

  final String baseUrl;
  final http.Client _http;

  Future<TopupConfig> fetchConfig() async {
    final data = await _getJson('/v1/square/topup/config');
    return TopupConfig.fromJson(data);
  }

  /// 上报付款交易:confirmed→pending(待支付);未确认→confirming;非法→抛错。
  Future<TopupSubmitResult> submit({
    required String token,
    required String packageId,
    required String gmbAddress,
    required String evmTxHash,
    String? payerAddress,
  }) async {
    final data = await _postJson('/v1/square/topup/submit', {
      'token': token,
      'package_id': packageId,
      'gmb_address': gmbAddress,
      'evm_tx_hash': evmTxHash,
      if (payerAddress != null && payerAddress.isNotEmpty)
        'payer_address': payerAddress,
    });
    return TopupSubmitResult.fromJson(data);
  }

  /// 轮询订单状态(按链 + txHash)。
  Future<TopupOrderStatus> status({
    required int chainId,
    required String evmTxHash,
  }) async {
    final data = await _getJson(
      '/v1/square/topup/status?chain_id=$chainId&evm_tx_hash=$evmTxHash',
    );
    return topupOrderStatusFrom(data['status']?.toString());
  }

  Future<Map<String, dynamic>> _getJson(String path) async {
    final response = await _http
        .get(Uri.parse('$baseUrl$path'),
            headers: const {'content-type': 'application/json; charset=utf-8'})
        .timeout(const Duration(seconds: 20));
    return _decode(response);
  }

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, Object?> body,
  ) async {
    final response = await _http
        .post(
          Uri.parse('$baseUrl$path'),
          headers: const {'content-type': 'application/json; charset=utf-8'},
          body: jsonEncode(body),
        )
        .timeout(const Duration(seconds: 20));
    return _decode(response);
  }

  Map<String, dynamic> _decode(http.Response response) {
    final dynamic decoded;
    try {
      decoded = jsonDecode(response.body);
    } catch (_) {
      throw TopupApiException(
        '充值服务响应不是 JSON：${response.statusCode}',
        statusCode: response.statusCode,
      );
    }
    if (decoded is! Map<String, dynamic>) {
      throw TopupApiException(
        '充值服务响应结构不合法：${response.statusCode}',
        statusCode: response.statusCode,
      );
    }
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw TopupApiException(
        decoded['message']?.toString() ?? '充值服务请求失败',
        statusCode: response.statusCode,
        errorCode: decoded['error_code']?.toString(),
      );
    }
    return decoded;
  }

  void close() => _http.close();
}
