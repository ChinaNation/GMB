import 'dart:convert';

import 'package:http/http.dart' as http;

/// 扫码支付清算体系 Step 1 新增:SFID 系统**公开**API 客户端。
///
/// 中文注释:
/// - 对接 sfid-backend 的 `GET /api/v1/app/clearing-banks/search` 端点
///   (实现见 `sfid/backend/src/institutions/handler.rs::app_search_clearing_banks`)。
/// - 该端点**无鉴权**,wuminapp 在用户绑定清算行前展示列表用。
/// - 仅返回 `is_clearing_bank == true` 的机构,带主账户/费用账户地址。
class SfidPublicApi {
  /// [baseUrl] 必须是不带末尾斜杠的根地址,例如 `https://sfid.example.com`。
  /// 由调用方根据当前环境(本地 / 测试 / 生产)注入,本类不做硬编码。
  SfidPublicApi({required this.baseUrl, http.Client? httpClient})
      : _http = httpClient ?? http.Client();

  final String baseUrl;
  final http.Client _http;

  /// 搜索清算行列表。
  ///
  /// [province] 省份名(如"广东省"),省略=全国。
  /// [city]     市名(配合 province 使用)。
  /// [keyword]  子串匹配 sfid_id / institution_name。
  /// [page]     页码,从 1 起。
  /// [size]     每页条数,1~100。
  Future<ClearingBankSearchResult> searchClearingBanks({
    String? province,
    String? city,
    String? keyword,
    int page = 1,
    int size = 20,
  }) async {
    final params = <String, String>{
      'page': page.toString(),
      'size': size.toString(),
    };
    if (province != null && province.trim().isNotEmpty) {
      params['province'] = province.trim();
    }
    if (city != null && city.trim().isNotEmpty) {
      params['city'] = city.trim();
    }
    if (keyword != null && keyword.trim().isNotEmpty) {
      params['keyword'] = keyword.trim();
    }
    final uri = Uri.parse('$baseUrl/api/v1/app/clearing-banks/search')
        .replace(queryParameters: params);

    final resp = await _http.get(uri, headers: const {
      'accept': 'application/json'
    }).timeout(const Duration(seconds: 10));
    if (resp.statusCode != 200) {
      throw Exception(
        'SFID 清算行搜索失败:HTTP ${resp.statusCode} ${resp.reasonPhrase}',
      );
    }
    final body = jsonDecode(resp.body) as Map<String, dynamic>;
    final code = body['code'] as int? ?? -1;
    if (code != 0) {
      throw Exception(
        'SFID 清算行搜索返回错误:code=$code message=${body['message']}',
      );
    }
    final data = (body['data'] as Map<String, dynamic>?) ?? const {};
    return ClearingBankSearchResult.fromJson(data);
  }

  void close() => _http.close();
}

/// 单条清算行展示数据(脱敏,无管理员/创建人)。
class ClearingBankInfo {
  const ClearingBankInfo({
    required this.sfidId,
    required this.institutionName,
    required this.a3,
    required this.subType,
    required this.parentSfidId,
    required this.parentInstitutionName,
    required this.parentA3,
    required this.province,
    required this.city,
    required this.mainAccount,
    required this.feeAccount,
  });

  /// SFID 编码,如 `SFR-GD-SZ01-CB01-N9-D8`。
  final String sfidId;

  /// 机构中文名(两步式未命名时为空串)。
  final String institutionName;

  /// 主体属性:`SFR`(私法人)或 `FFR`(非法人)。
  final String a3;

  /// 私法人子类型,清算行白名单要求 `JOINT_STOCK`。
  final String? subType;

  /// FFR 所属法人信息,用于手机端展示父子结构。
  final String? parentSfidId;
  final String? parentInstitutionName;
  final String? parentA3;

  final String province;
  final String city;

  /// 主账户链上地址(hex,无 0x 前缀)。未上链时为 null。
  final String? mainAccount;

  /// 费用账户链上地址。
  final String? feeAccount;

  factory ClearingBankInfo.fromJson(Map<String, dynamic> json) {
    return ClearingBankInfo(
      sfidId: (json['sfid_id'] as String?) ?? '',
      institutionName: (json['institution_name'] as String?) ?? '',
      a3: (json['a3'] as String?) ?? '',
      subType: json['sub_type'] as String?,
      parentSfidId: json['parent_sfid_id'] as String?,
      parentInstitutionName: json['parent_institution_name'] as String?,
      parentA3: json['parent_a3'] as String?,
      province: (json['province'] as String?) ?? '',
      city: (json['city'] as String?) ?? '',
      mainAccount: json['main_account'] as String?,
      feeAccount: json['fee_account'] as String?,
    );
  }
}

/// 分页响应。
class ClearingBankSearchResult {
  const ClearingBankSearchResult({
    required this.total,
    required this.items,
    required this.page,
    required this.size,
  });

  final int total;
  final List<ClearingBankInfo> items;
  final int page;
  final int size;

  factory ClearingBankSearchResult.fromJson(Map<String, dynamic> json) {
    final raw = (json['items'] as List?) ?? const [];
    return ClearingBankSearchResult(
      total: (json['total'] as int?) ?? 0,
      items: raw
          .whereType<Map<String, dynamic>>()
          .map(ClearingBankInfo.fromJson)
          .toList(growable: false),
      page: (json['page'] as int?) ?? 1,
      size: (json['size'] as int?) ?? 20,
    );
  }
}
