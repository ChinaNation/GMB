import 'dart:convert';

import 'package:http/http.dart' as http;

/// 扫码支付清算体系 Step 1 新增:CID 系统**公开**API 客户端。
///
/// 中文注释:
/// - 对接 citizencode-backend 的 `GET /api/v1/app/clearing-banks/search` 端点
///   (实现见 `citizencode/backend/subjects/chain_multisig_info.rs` 的清算行公开查询)。
/// - 该端点**无鉴权**,CitizenApp 在用户绑定清算行前展示列表用。
/// - 仅返回 `is_clearing_bank == true` 的机构,带主账户/费用账户地址。
class CidPublicApi {
  /// [baseUrl] 必须是不带末尾斜杠的根地址,例如 `https://cid.example.com`。
  /// 由调用方根据当前环境(本地 / 测试 / 生产)注入,本类不做硬编码。
  CidPublicApi({required this.baseUrl, http.Client? httpClient})
      : _http = httpClient ?? http.Client();

  final String baseUrl;
  final http.Client _http;

  /// 搜索清算行列表。
  ///
  /// [provinceName] 省份名(如"广东省"),省略=全国。
  /// [cityName]     市名(配合 provinceName 使用)。
  /// [keyword]  子串匹配 cid_number / cid_full_name / cid_short_name。
  /// [page]     页码,从 1 起。
  /// [size]     每页条数,1~100。
  Future<ClearingBankSearchResult> searchClearingBanks({
    String? provinceName,
    String? cityName,
    String? keyword,
    int page = 1,
    int size = 20,
  }) async {
    final params = <String, String>{
      'page': page.toString(),
      'size': size.toString(),
    };
    if (provinceName != null && provinceName.trim().isNotEmpty) {
      params['province_name'] = provinceName.trim();
    }
    if (cityName != null && cityName.trim().isNotEmpty) {
      params['city_name'] = cityName.trim();
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
        'CID 清算行搜索失败:HTTP ${resp.statusCode} ${resp.reasonPhrase}',
      );
    }
    final body = jsonDecode(resp.body) as Map<String, dynamic>;
    final code = body['code'] as int? ?? -1;
    if (code != 0) {
      throw Exception(
        'CID 清算行搜索返回错误:code=$code message=${body['message']}',
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
    required this.cidNumber,
    required this.cidFullName,
    required this.cidShortName,
    required this.subjectProperty,
    required this.subType,
    required this.parentCidNumber,
    required this.parentCidFullName,
    required this.parentSubjectProperty,
    required this.provinceName,
    required this.cityName,
    required this.mainAccount,
    required this.feeAccount,
  });

  /// CID 编码,如 `GD001-SCB0T-123456789-2026`。
  final String cidNumber;

  /// 机构全称(两步式未命名时为空串)。
  final String cidFullName;

  /// 机构简称(两步式未命名时为空串)。
  final String cidShortName;

  /// 主体属性:`S`(私法人)或 `F`(非法人)。
  final String subjectProperty;

  /// 私法人子类型,清算行白名单要求 `JOINT_STOCK`。
  final String? subType;

  /// 非法人主体所属法人信息,用于手机端展示父子结构。
  final String? parentCidNumber;
  final String? parentCidFullName;
  final String? parentSubjectProperty;

  final String provinceName;
  final String cityName;

  /// 主账户链上地址(hex,无 0x 前缀)。未上链时为 null。
  final String? mainAccount;

  /// 费用账户链上地址。
  final String? feeAccount;

  String get displayTitle {
    final cidShort = cidShortName.trim();
    if (cidShort.isNotEmpty) return cidShort;
    final cidFull = cidFullName.trim();
    return cidFull.isEmpty ? cidNumber : cidFull;
  }

  factory ClearingBankInfo.fromJson(Map<String, dynamic> json) {
    return ClearingBankInfo(
      cidNumber: (json['cid_number'] as String?) ?? '',
      cidFullName: (json['cid_full_name'] as String?) ?? '',
      cidShortName: (json['cid_short_name'] as String?) ?? '',
      subjectProperty: (json['subject_property'] as String?) ?? '',
      subType: json['sub_type'] as String?,
      parentCidNumber: json['parent_cid_number'] as String?,
      parentCidFullName: json['parent_cid_full_name'] as String?,
      parentSubjectProperty: json['parent_subject_property'] as String?,
      provinceName: (json['province_name'] as String?) ?? '',
      cityName: (json['city_name'] as String?) ?? '',
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
