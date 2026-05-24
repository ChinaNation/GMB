import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:wuminapp_mobile/sfid_api_config.dart';

class MyIdStatusResponse {
  const MyIdStatusResponse({
    required this.status,
    this.address,
    this.sfidCode,
    this.identityStatus,
  });

  /// "pending" | "bound" | "unset"
  final String status;
  final String? address;
  final String? sfidCode;
  final String? identityStatus;
}

class MyIdApi {
  MyIdApi() : _baseUrl = SfidApiConfig.defaultBaseUrl;

  final String _baseUrl;

  Map<String, String> _headers({
    bool includeContentType = false,
  }) {
    final out = <String, String>{};
    if (includeContentType) {
      out['Content-Type'] = 'application/json';
    }
    return out;
  }

  /// 注册电子护照账户。
  ///
  /// 中文注释：后端协议暂时沿用既有 vote-account 路由，页面和模块归属
  /// 已改为“电子护照”。后续后端协议重命名时，只需要收敛改这里。
  Future<void> registerMyId({
    required String address,
    required String pubkeyHex,
    required String signatureHex,
    required String signMessage,
  }) async {
    final normalized = _normalizePubkeyHex(pubkeyHex);
    final uri = Uri.parse('$_baseUrl/api/v1/app/vote-account/register');
    http.Response response;
    try {
      response = await http
          .post(
            uri,
            headers: _headers(includeContentType: true),
            body: jsonEncode({
              'address': address,
              'pubkey': normalized,
              'signature': signatureHex.startsWith('0x')
                  ? signatureHex
                  : '0x$signatureHex',
              'sign_message': signMessage,
            }),
          )
          .timeout(const Duration(seconds: 15));
    } on TimeoutException catch (_) {
      throw Exception('电子护照注册请求超时，请检查网络连接');
    } on SocketException catch (_) {
      if (Platform.isAndroid || Platform.isIOS) {
        throw Exception(SfidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('电子护照注册失败：${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('电子护照注册被拒绝：code=$code message=$message');
    }
  }

  /// 查询电子护照绑定状态。
  Future<MyIdStatusResponse> queryMyIdStatus(String walletAddress) async {
    final addr = walletAddress.trim();
    if (addr.isEmpty) {
      throw Exception('walletAddress is empty');
    }
    final uri =
        Uri.parse('$_baseUrl/api/v1/app/vote-account/status?address=$addr');
    http.Response response;
    try {
      response = await http
          .get(uri, headers: _headers())
          .timeout(const Duration(seconds: 15));
    } on TimeoutException catch (_) {
      throw Exception('电子护照状态查询超时，请检查网络连接');
    } on SocketException catch (_) {
      if (Platform.isAndroid || Platform.isIOS) {
        throw Exception(SfidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('电子护照状态查询失败：${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('电子护照状态查询被拒绝：code=$code message=$message');
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('电子护照状态响应缺少 data');
    }

    return MyIdStatusResponse(
      status: (data['status']?.toString() ?? 'unset').trim(),
      address: data['address']?.toString(),
      sfidCode: data['sfid_code']?.toString(),
      identityStatus: data['identity_status']?.toString(),
    );
  }

  String _normalizePubkeyHex(String value) {
    final trimmed = value.trim();
    if (trimmed.isEmpty) {
      throw Exception('pubkey is empty');
    }
    return trimmed.startsWith('0x') ? trimmed : '0x$trimmed';
  }
}
