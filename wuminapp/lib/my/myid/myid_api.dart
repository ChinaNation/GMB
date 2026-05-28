import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:wuminapp_mobile/sfid_api_config.dart';

class MyIdStatusResponse {
  const MyIdStatusResponse({
    required this.bindStatus,
    this.walletAddress,
    this.sfidCode,
    this.citizenStatus,
    this.votingEligible,
    this.voteStatus,
    this.identityStatus,
    this.validFrom,
    this.validUntil,
    this.statusUpdatedAt,
  });

  /// "pending" | "bound" | "unset"
  final String bindStatus;
  final String? walletAddress;
  final String? sfidCode;
  final String? citizenStatus;
  final bool? votingEligible;
  final String? voteStatus;
  final String? identityStatus;
  final String? validFrom;
  final String? validUntil;
  final int? statusUpdatedAt;
}

class MyIdApi {
  MyIdApi() : _baseUrl = SfidApiConfig.defaultBaseUrl;

  final String _baseUrl;

  /// 查询电子护照绑定状态。
  Future<MyIdStatusResponse> queryMyIdStatus(String walletAddress) async {
    final addr = walletAddress.trim();
    if (addr.isEmpty) {
      throw Exception('walletAddress is empty');
    }
    final uri = Uri.parse('$_baseUrl/api/v1/app/myid/status')
        .replace(queryParameters: {'wallet_address': addr});
    http.Response response;
    try {
      response = await http.get(uri).timeout(const Duration(seconds: 15));
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
      bindStatus: (data['bind_status']?.toString() ?? 'unset').trim(),
      walletAddress: data['wallet_address']?.toString(),
      sfidCode: data['sfid_code']?.toString(),
      citizenStatus: data['citizen_status']?.toString(),
      votingEligible: _parseBool(data['voting_eligible']),
      voteStatus: data['vote_status']?.toString(),
      identityStatus: data['identity_status']?.toString(),
      validFrom: data['valid_from']?.toString(),
      validUntil: data['valid_until']?.toString(),
      statusUpdatedAt: data['status_updated_at'] is int
          ? data['status_updated_at'] as int
          : int.tryParse(data['status_updated_at']?.toString() ?? ''),
    );
  }

  static bool? _parseBool(Object? value) {
    if (value is bool) return value;
    final text = value?.toString().trim().toLowerCase();
    if (text == 'true') return true;
    if (text == 'false') return false;
    return null;
  }
}
