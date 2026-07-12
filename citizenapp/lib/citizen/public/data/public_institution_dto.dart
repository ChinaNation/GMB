// 公权机构 finalized 链快照 DTO。

import 'package:citizenapp/isar/app_isar.dart';

/// 公权机构目录行；只解析发布期从 finalized 链状态生成的数据包。
class PublicInstitutionDto {
  const PublicInstitutionDto({
    required this.cidNumber,
    required this.status,
    required this.provinceCode,
    required this.cityCode,
    required this.institutionCode,
    required this.accountCount,
    this.cidFullName,
    this.cidShortName,
    this.townCode = '',
    this.parentCidNumber,
    this.hasLegalPersonality,
    this.legalRepName,
    this.customAccountNames = const [],
  });

  final String cidNumber;
  final String? cidFullName;
  final String? cidShortName;
  final String status;

  /// 所属省 code(行政区唯一真源键;名字由字典 join,见 ADR-021)。
  final String provinceCode;

  /// 所属市 code(名字走字典 join)。
  final String cityCode;

  /// 所属镇 code(空串=只定位到市级)。
  final String townCode;
  final String institutionCode;
  final String? parentCidNumber;
  final bool? hasLegalPersonality;

  /// 法定代表人姓名；当前链快照没有该字段时为 null。
  final String? legalRepName;
  final int accountCount;
  final List<String> customAccountNames;

  static PublicInstitutionDto fromJson(Map<String, dynamic> json) {
    return PublicInstitutionDto(
      cidNumber: json['cid_number'] as String,
      cidFullName: json['cid_full_name'] as String?,
      cidShortName: json['cid_short_name'] as String?,
      status: json['status'] as String? ?? 'ACTIVE',
      // 行政区只吃 code(province_code/city_code/town_code);无名字 fallback
      // (ADR-021 死规则:名字唯一来自字典,不留旧方案)。
      provinceCode: json['province_code'] as String? ?? '',
      cityCode: json['city_code'] as String? ?? '',
      townCode: json['town_code'] as String? ?? '',
      institutionCode: json['institution_code'] as String? ?? '',
      parentCidNumber: json['parent_cid_number'] as String?,
      hasLegalPersonality: json['has_legal_personality'] as bool?,
      legalRepName: json['legal_rep_name'] as String?,
      accountCount: (json['account_count'] as num?)?.toInt() ?? 0,
      customAccountNames:
          (json['custom_account_names'] as List<dynamic>? ?? const [])
              .map((e) => e as String)
              .toList(growable: false),
    );
  }

  /// 映射为 Isar 实体(catalogVersion / updatedAtMillis 由 repo 在落库时补)。
  PublicInstitutionEntity toEntity({
    required String catalogVersion,
    required int updatedAtMillis,
  }) {
    return PublicInstitutionEntity()
      ..cidNumber = cidNumber
      ..cidFullName = cidFullName ?? cidNumber
      ..cidShortName = cidShortName
      ..status = status
      ..provinceCode = provinceCode
      ..cityCode = cityCode
      ..townCode = townCode
      ..institutionCode = institutionCode
      ..parentCidNumber = parentCidNumber
      ..hasLegalPersonality = hasLegalPersonality
      ..legalRepName = legalRepName
      ..accountCount = accountCount
      ..customAccountNames = customAccountNames
      ..catalogVersion = catalogVersion
      ..updatedAtMillis = updatedAtMillis;
  }
}
