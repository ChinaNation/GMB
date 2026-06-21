// 公权机构目录 CID 公开接口客户端(匿名只读)。
//
// 对应 CID BFF(混合模式:keyset 翻页 + updated_at 增量):
//   GET /api/v1/app/public-institutions?province_name=&city_name=&since_version=&after_cid=&page_size=
//   GET /api/v1/app/public-institutions/version?province_name=&city_name=
// 走 CidApiConfig 唯一地址策略,无鉴权头,带超时(杜绝无限转)。

import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:citizenapp/cid_api_config.dart';

import 'public_institution_dto.dart';

/// 某省/市目录版本(增量比对)。
class PublicInstitutionVersion {
  const PublicInstitutionVersion({
    required this.provinceName,
    this.cityName,
    this.manifestVersion,
    this.count = 0,
  });

  final String provinceName;
  final String? cityName;

  /// 目录版本 = MAX(updated_at) RFC3339;增量同步 since 用。
  final String? manifestVersion;
  final int count;
}

class PublicInstitutionApi {
  PublicInstitutionApi({
    String? baseUrl,
    http.Client? client,
    Duration? timeout,
  })  : _baseUrl = baseUrl ?? CidApiConfig.defaultBaseUrl,
        _client = client ?? http.Client(),
        _timeout = timeout ?? const Duration(seconds: 15);

  final String _baseUrl;
  final http.Client _client;
  final Duration _timeout;

  /// 拉取某省(可选市)公权机构目录一页(keyset + 可选 since 增量)。
  Future<PublicInstitutionPage> fetchPage({
    required String provinceName,
    String? cityName,
    String? sinceVersion,
    String? afterCid,
    int pageSize = 500,
  }) async {
    final params = <String, String>{
      'province_name': provinceName,
      'page_size': pageSize.clamp(1, 500).toString(),
    };
    if (cityName != null && cityName.isNotEmpty) {
      params['city_name'] = cityName;
    }
    if (sinceVersion != null && sinceVersion.isNotEmpty) {
      params['since_version'] = sinceVersion;
    }
    if (afterCid != null && afterCid.isNotEmpty) {
      params['after_cid'] = afterCid;
    }

    final uri = Uri.parse('$_baseUrl/api/v1/app/public-institutions')
        .replace(queryParameters: params);
    final response = await _client.get(uri).timeout(_timeout);
    if (response.statusCode != 200) {
      throw Exception(
        'public institutions list failed: ${response.statusCode}',
      );
    }
    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final data = payload['data'] as Map<String, dynamic>;
    return PublicInstitutionPage.fromData(data);
  }

  /// 拉取某省(可选市)目录版本戳。
  Future<PublicInstitutionVersion> fetchVersion({
    required String provinceName,
    String? cityName,
  }) async {
    final params = <String, String>{'province_name': provinceName};
    if (cityName != null && cityName.isNotEmpty) {
      params['city_name'] = cityName;
    }
    final uri = Uri.parse('$_baseUrl/api/v1/app/public-institutions/version')
        .replace(queryParameters: params);
    final response = await _client.get(uri).timeout(_timeout);
    if (response.statusCode != 200) {
      throw Exception(
        'public institutions version failed: ${response.statusCode}',
      );
    }
    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final data = payload['data'] as Map<String, dynamic>;
    return PublicInstitutionVersion(
      provinceName: data['province_name'] as String? ?? provinceName,
      cityName: data['city_name'] as String?,
      manifestVersion: data['manifest_version'] as String?,
      count: (data['count'] as num?)?.toInt() ?? 0,
    );
  }
}
