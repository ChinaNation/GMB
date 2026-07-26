import 'dart:convert';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareApiConfig, SquareSession;
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'topup_models.dart';

/// 稳定币充值 Worker 客户端(/v1/square/topup/*)。
///
/// config 公开只读；intent / confirm / status 使用默认热钱包的静默设备会话，
/// 使充值目标只能取 Worker 会话内 account_id，不接受客户端另传账户。
class TopupApiException implements Exception {
  const TopupApiException(this.message, {this.statusCode, this.errorCode});

  final String message;
  final int? statusCode;
  final String? errorCode;

  @override
  String toString() => message;
}

class TopupApi {
  TopupApi({
    String? baseUrl,
    http.Client? httpClient,
    SquareSessionProvider? sessionProvider,
    Future<SquareSession?> Function()? sessionResolver,
  })  : baseUrl = SquareApiConfig.normalizeBaseUrl(
          baseUrl ?? SquareApiConfig.defaultBaseUrl,
        ),
        _http = httpClient ?? http.Client(),
        _sessionResolver = sessionResolver ??
            (sessionProvider ?? SquareSessionProvider.instance).ensureSession;

  final String baseUrl;
  final http.Client _http;
  final Future<SquareSession?> Function() _sessionResolver;

  Future<TopupConfig> fetchConfig() async {
    final data = await _getJson('/v1/square/topup/config');
    return TopupConfig.fromJson(data);
  }

  /// 钱包连接后、付款前创建短期意图；accountId 仅用于本地核对默认热钱包。
  Future<TopupPaymentIntent> createIntent({
    required String token,
    required String packageId,
    required String accountId,
    required String payerAddress,
  }) async {
    final sessionToken = await _sessionTokenFor(accountId);
    final data = await _postJson(
        '/v1/square/topup/intent',
        {
          'token': token,
          'package_id': packageId,
          'payer_address': payerAddress,
        },
        sessionToken: sessionToken);
    return TopupPaymentIntent.fromJson(data);
  }

  /// 付款后提交交易哈希；Worker 从 HMAC 意图恢复付款人与精确报价。
  Future<TopupConfirmResult> confirm({
    required String accountId,
    required String paymentIntent,
    required String evmTxHash,
  }) async {
    final sessionToken = await _sessionTokenFor(accountId);
    final data = await _postJson(
        '/v1/square/topup/confirm',
        {
          'payment_intent': paymentIntent,
          'evm_tx_hash': evmTxHash,
        },
        sessionToken: sessionToken);
    return TopupConfirmResult.fromJson(data);
  }

  /// 按订单 ID 轮询；Worker 同时校验订单归属当前会话账户。
  Future<TopupOrderStatus> status({
    required String accountId,
    required String orderId,
  }) async {
    final sessionToken = await _sessionTokenFor(accountId);
    final data = await _getJson(
      '/v1/square/topup/status/${Uri.encodeComponent(orderId)}',
      sessionToken: sessionToken,
    );
    return topupOrderStatusFrom(data['status']?.toString());
  }

  Future<String> _sessionTokenFor(String expectedAccountId) async {
    final session = await _sessionResolver();
    if (session == null) {
      throw const TopupApiException(
        '当前钱包无法建立设备会话，请确认正在使用已注册的热钱包',
        statusCode: 401,
        errorCode: 'topup_session_unavailable',
      );
    }
    if (session.accountId != expectedAccountId) {
      throw const TopupApiException(
        '充值目标不是当前默认热钱包',
        statusCode: 403,
        errorCode: 'topup_account_mismatch',
      );
    }
    return session.sessionToken;
  }

  Future<Map<String, dynamic>> _getJson(
    String path, {
    String? sessionToken,
  }) async {
    final response = await _http.get(Uri.parse('$baseUrl$path'), headers: {
      'content-type': 'application/json; charset=utf-8',
      if (sessionToken != null) 'authorization': 'Bearer $sessionToken',
    }).timeout(const Duration(seconds: 20));
    return _decode(response);
  }

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, Object?> body, {
    String? sessionToken,
  }) async {
    final response = await _http
        .post(
          Uri.parse('$baseUrl$path'),
          headers: {
            'content-type': 'application/json; charset=utf-8',
            if (sessionToken != null) 'authorization': 'Bearer $sessionToken',
          },
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
