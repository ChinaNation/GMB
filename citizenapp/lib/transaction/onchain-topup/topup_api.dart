import 'dart:convert';

import 'package:http/http.dart' as http;

import 'package:citizenapp/8964/services/square_api_client.dart'
    show SquareApiConfig;
import 'topup_models.dart';

/// 稳定币充值 Worker 客户端(/v1/square/topup/*)。
///
/// 充值 = 付款人自掏稳定币给某个公民链账户打公民币,收款方无需证明账户所有权(同转账),
/// 故全程不带广场会话：充值目标由 `account_id` 直接指定，任意钱包账户（含冷钱包、
/// 含他人账户）均可作目标。Worker 从 finalized 链双向绑定自行确定目标当时是否归属
/// 某个 `cid_number`，客户端不得自行声明身份归属；付款鉴权由 WalletConnect 完成。
/// confirm / status 凭 Worker 用 HMAC 签发的付款意图自证(不可伪造,内部钉死付款人、
/// 收款地址、金额、目标账户与签发时间)。
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
  })  : baseUrl = SquareApiConfig.normalizeBaseUrl(
          baseUrl ?? SquareApiConfig.defaultBaseUrl,
        ),
        _http = httpClient ?? http.Client();

  final String baseUrl;
  final http.Client _http;

  Future<TopupConfig> fetchConfig() async {
    final data = await _getJson('/v1/square/topup/config');
    return TopupConfig.fromJson(data);
  }

  /// 钱包连接后、付款前创建短期意图；[accountId] 即公民币充值目标账户。
  Future<TopupPaymentIntent> createIntent({
    required String token,
    required String packageId,
    required String accountId,
    required String payerAddress,
  }) async {
    final data = await _postJson('/v1/square/topup/intent', {
      'account_id': accountId,
      'token': token,
      'package_id': packageId,
      'payer_address': payerAddress,
    });
    return TopupPaymentIntent.fromJson(data);
  }

  /// 付款后提交交易哈希；Worker 从 HMAC 意图恢复充值目标、付款人与精确报价。
  Future<TopupConfirmResult> confirm({
    required String paymentIntent,
    required String evmTxHash,
  }) async {
    final data = await _postJson('/v1/square/topup/confirm', {
      'payment_intent': paymentIntent,
      'evm_tx_hash': evmTxHash,
    });
    return TopupConfirmResult.fromJson(data);
  }

  /// 按订单 ID 轮询；付款意图即查询凭证，故走 POST——凭证不能出现在 URL 里。
  Future<TopupOrderStatus> status({
    required String orderId,
    required String paymentIntent,
  }) async {
    final data = await _postJson('/v1/square/topup/status', {
      'order_id': orderId,
      'payment_intent': paymentIntent,
    });
    return topupOrderStatusFrom(data['status']?.toString());
  }

  Future<Map<String, dynamic>> _getJson(String path) async {
    final response = await _http.get(Uri.parse('$baseUrl$path'), headers: {
      'content-type': 'application/json; charset=utf-8',
    }).timeout(const Duration(seconds: 20));
    return _decode(response);
  }

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, Object?> body,
  ) async {
    final response = await _http
        .post(
          Uri.parse('$baseUrl$path'),
          headers: {'content-type': 'application/json; charset=utf-8'},
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
