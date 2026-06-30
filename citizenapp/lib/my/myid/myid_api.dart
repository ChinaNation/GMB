import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:citizenapp/cid_api_config.dart';

class MyIdStatusResponse {
  const MyIdStatusResponse({
    required this.found,
    this.walletAddress,
    this.cidNumber,
    this.passportNo,
    this.citizenStatus,
    this.votingEligible,
    this.voteStatus,
    this.identityStatus,
    this.passportValidFrom,
    this.passportValidUntil,
    this.statusUpdatedAt,
  });

  /// 后端已找到当前钱包对应的公民档案。
  final bool found;
  final String? walletAddress;
  final String? cidNumber;
  final String? passportNo;
  final String? citizenStatus;
  final bool? votingEligible;
  final String? voteStatus;
  final String? identityStatus;
  final String? passportValidFrom;
  final String? passportValidUntil;
  final int? statusUpdatedAt;
}

class MyIdApi {
  MyIdApi() : _baseUrl = CidApiConfig.defaultBaseUrl;

  final String _baseUrl;

  /// 查询电子护照状态。
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
        throw Exception(CidApiConfig.connectionErrorMessage(_baseUrl));
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
      found: _parseBool(data['found']) ?? false,
      walletAddress: data['wallet_address']?.toString(),
      cidNumber: data['cid_number']?.toString(),
      passportNo: data['passport_no']?.toString(),
      citizenStatus: data['citizen_status']?.toString(),
      votingEligible: _parseBool(data['voting_eligible']),
      voteStatus: data['vote_status']?.toString(),
      identityStatus: data['identity_status']?.toString(),
      passportValidFrom: data['passport_valid_from']?.toString(),
      passportValidUntil: data['passport_valid_until']?.toString(),
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
