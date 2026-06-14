// 公权机构目录 DTO —— 对应 SFID BFF `PublicInstitutionRow` / `PageResult`
// (`GET /api/v1/app/public-institutions`)。数据包 JSON 与接口响应共用本解析。

import 'package:wuminapp_mobile/isar/wallet_isar.dart';

/// 公权机构目录行(白名单字段,与后端 PublicInstitutionRow 一一对应)。
class PublicInstitutionDto {
  const PublicInstitutionDto({
    required this.sfidNumber,
    required this.status,
    required this.province,
    required this.city,
    required this.institutionCode,
    required this.accountCount,
    this.institutionName,
    this.sfidName,
    this.shortName,
    this.town = '',
    this.orgCode,
    this.parentSfidNumber,
    this.hasLegalPersonality,
    this.legalRepName,
    this.customAccountNames = const [],
  });

  final String sfidNumber;
  final String? institutionName;
  final String? sfidName;
  final String? shortName;
  final String status;
  final String province;
  final String city;
  final String town;
  final String institutionCode;
  final String? orgCode;
  final String? parentSfidNumber;
  final bool? hasLegalPersonality;

  /// 法定代表人姓名(公开目录字段,来自 SFID subjects.legal_rep_name);无则 null → 留空。
  final String? legalRepName;
  final int accountCount;
  final List<String> customAccountNames;

  static PublicInstitutionDto fromJson(Map<String, dynamic> json) {
    return PublicInstitutionDto(
      sfidNumber: json['sfid_number'] as String,
      institutionName: json['institution_name'] as String?,
      sfidName: json['sfid_name'] as String?,
      shortName: json['short_name'] as String?,
      status: json['status'] as String? ?? 'ACTIVE',
      province: json['province'] as String? ?? '',
      city: json['city'] as String? ?? '',
      town: json['town'] as String? ?? '',
      institutionCode: json['institution_code'] as String? ?? '',
      orgCode: json['org_code'] as String?,
      parentSfidNumber: json['parent_sfid_number'] as String?,
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
      ..sfidNumber = sfidNumber
      ..institutionName = institutionName ?? sfidName ?? sfidNumber
      ..sfidName = sfidName
      ..shortName = shortName
      ..status = status
      ..province = province
      ..city = city
      ..town = town
      ..institutionCode = institutionCode
      ..orgCode = orgCode
      ..parentSfidNumber = parentSfidNumber
      ..hasLegalPersonality = hasLegalPersonality
      ..legalRepName = legalRepName
      ..accountCount = accountCount
      ..customAccountNames = customAccountNames
      ..catalogVersion = catalogVersion
      ..updatedAtMillis = updatedAtMillis;
  }
}

/// 目录分页结果(对应后端 PageResult)。
class PublicInstitutionPage {
  const PublicInstitutionPage({
    required this.items,
    required this.hasMore,
    this.nextCursor,
    this.manifestVersion,
  });

  final List<PublicInstitutionDto> items;
  final bool hasMore;
  final String? nextCursor;
  final String? manifestVersion;

  static PublicInstitutionPage fromData(Map<String, dynamic> data) {
    final rawItems = data['items'] as List<dynamic>? ?? const [];
    return PublicInstitutionPage(
      items: rawItems
          .map((e) => PublicInstitutionDto.fromJson(e as Map<String, dynamic>))
          .toList(growable: false),
      hasMore: data['has_more'] as bool? ?? false,
      nextCursor: data['next_cursor'] as String?,
      manifestVersion: data['manifest_version'] as String?,
    );
  }
}
