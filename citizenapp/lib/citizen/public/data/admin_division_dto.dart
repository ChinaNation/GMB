// 行政区字典 DTO + 键工具(ADR-021 行政区唯一真源)。
//
// 中文注释:对应 `assets/admin_divisions/` 数据包(china.sqlite 直 dump,零映射):
//   provinces.json       = [{code,name}]
//   cities/<pcode>.json  = [{code,name}]
//   towns/<pcode>.json   = [{city_code, code, name}]
// 镇 code 全国不唯一,故所有键一律带完整层级前缀。

import 'package:citizenapp/isar/wallet_isar.dart';

/// 行政区层级。
class AdminDivisionLevel {
  const AdminDivisionLevel._();

  static const String province = 'province';
  static const String city = 'city';
  static const String town = 'town';
}

/// 复合唯一键:`"<level>|<pcode>|<ccode>|<tcode>"`(缺级留空)。
///
/// 例:省=`"province|LN||"`、市=`"city|LN|001|"`、镇=`"town|LN|001|005"`。
String divisionKeyOf({
  required String level,
  required String provinceCode,
  String cityCode = '',
  String townCode = '',
}) =>
    '$level|$provinceCode|$cityCode|$townCode';

/// 父定位键:province 空、city=pcode、town=`"<pcode>|<ccode>"`。
String scopeKeyOf({
  required String level,
  required String provinceCode,
  String cityCode = '',
}) {
  switch (level) {
    case AdminDivisionLevel.city:
      return provinceCode;
    case AdminDivisionLevel.town:
      return '$provinceCode|$cityCode';
    default:
      return '';
  }
}

/// 单条行政区字典记录(解析数据包用)。
class AdminDivisionDto {
  const AdminDivisionDto({
    required this.level,
    required this.provinceCode,
    required this.code,
    required this.name,
    this.cityCode = '',
    this.townCode = '',
  });

  final String level;
  final String provinceCode;

  /// 市级填充;省级/直接挂省时为空。
  final String cityCode;

  /// 镇级填充(= code);省/市级为空。
  final String townCode;

  /// 该层级自身 code。
  final String code;
  final String name;

  /// 省记录:`{code,name}`。
  static AdminDivisionDto province(Map<String, dynamic> json) {
    final code = json['code'] as String? ?? '';
    return AdminDivisionDto(
      level: AdminDivisionLevel.province,
      provinceCode: code,
      code: code,
      name: json['name'] as String? ?? '',
    );
  }

  /// 市记录:`{code,name}`(province code 由分片文件名带入)。
  static AdminDivisionDto city(String provinceCode, Map<String, dynamic> json) {
    final code = json['code'] as String? ?? '';
    return AdminDivisionDto(
      level: AdminDivisionLevel.city,
      provinceCode: provinceCode,
      cityCode: code,
      code: code,
      name: json['name'] as String? ?? '',
    );
  }

  /// 镇记录:`{city_code, code, name}`(province code 由分片文件名带入)。
  static AdminDivisionDto town(String provinceCode, Map<String, dynamic> json) {
    final code = json['code'] as String? ?? '';
    return AdminDivisionDto(
      level: AdminDivisionLevel.town,
      provinceCode: provinceCode,
      cityCode: json['city_code'] as String? ?? '',
      townCode: code,
      code: code,
      name: json['name'] as String? ?? '',
    );
  }

  String get divisionKey => divisionKeyOf(
        level: level,
        provinceCode: provinceCode,
        cityCode: cityCode,
        townCode: townCode,
      );

  String get scopeKey => scopeKeyOf(
        level: level,
        provinceCode: provinceCode,
        cityCode: cityCode,
      );

  AdminDivisionEntity toEntity({String? dictVersion}) => AdminDivisionEntity()
    ..divisionKey = divisionKey
    ..level = level
    ..code = code
    ..scopeKey = scopeKey
    ..name = name
    ..dictVersion = dictVersion;
}
