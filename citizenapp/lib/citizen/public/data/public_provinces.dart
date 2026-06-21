// 公权机构省份导航来源 —— 复用治理省储会的同一行政区(不新建)。
//
// 中文注释:省份是固定行政区划(43 省),与机构数据是否加载无关,必须始终全显
// (对称治理 tab 的 43 省储会编译期常量)。
//
// ADR-021 行政区唯一真源:机构记录只存 province/city/town code;省名走链上常量
// (`kProvincialCouncils`,认可的省名源),市/镇名走 china.sqlite 派生字典。
// 省 code 从省储会 cidNumber 前缀派生(`ZS001-...` → `ZS`),与字典 provinces.json
// 的 code 一一对应。`publicProvinceNamesSet()` 给单测断言「链上省名集合==字典省名集合」
// 用,把"逐字对齐"变守卫。

import 'package:citizenapp/governance/organization-manage/institution_registry.dart';

const String _kCouncilSuffix = '公民储备委员会';

/// 省导航条目:code(查询/落库键)+ 全名(含"省")+ 展示名(去"省")。
class PublicProvinceItem {
  const PublicProvinceItem({
    required this.code,
    required this.fullName,
    required this.displayName,
  });

  /// 省 code(= 省储会 cidNumber 前 2 字符),与字典 provinces.json code 对齐。
  final String code;

  /// 规范全名(含"省"),与 china.sqlite 省名逐字对齐。
  final String fullName;

  /// 展示名:去掉末尾"省"字。
  final String displayName;
}

String _fullNameOf(String councilName) => councilName.endsWith(_kCouncilSuffix)
    ? councilName.substring(0, councilName.length - _kCouncilSuffix.length)
    : councilName;

String _codeOf(String cidNumber) {
  // 省储会 cidNumber 形如 `ZS001-GCB0R-...`,前 2 字符为省 code。
  return cidNumber.length >= 2 ? cidNumber.substring(0, 2) : cidNumber;
}

String _displayOf(String fullName) => fullName.endsWith('省')
    ? fullName.substring(0, fullName.length - 1)
    : fullName;

/// 公权机构左栏的 43 个省份导航条目(code + 全名 + 展示名,来自链上省储会常量)。
List<PublicProvinceItem> publicProvinceItems() {
  return kProvincialCouncils.map((c) {
    final fullName = _fullNameOf(c.name);
    return PublicProvinceItem(
      code: _codeOf(c.cidNumber),
      fullName: fullName,
      displayName: _displayOf(fullName),
    );
  }).toList(growable: false);
}

/// 公权机构左栏的 43 个省份规范**全名**(含"省")。
List<String> publicProvinceNames() =>
    publicProvinceItems().map((p) => p.fullName).toList(growable: false);

/// 链上省名集合(去"省"前的全名);单测用作「链上==字典」守卫断言。
Set<String> publicProvinceNamesSet() =>
    publicProvinceItems().map((p) => p.fullName).toSet();

/// 省 code → 全名(含"省");未知 code 回退 code 本身(绝不崩)。
String provinceFullNameByCode(String code) {
  for (final p in publicProvinceItems()) {
    if (p.code == code) return p.fullName;
  }
  return code;
}

/// 省 code → 展示名(去"省");未知 code 回退 code 本身。
String provinceDisplayNameByCode(String code) {
  for (final p in publicProvinceItems()) {
    if (p.code == code) return p.displayName;
  }
  return code;
}

/// 省份**展示名**:去掉末尾"省"字。
///
/// 中文注释:展示一律不带"省"(中枢/岭南/广东);但匹配/查询仍用 code。
/// 名字与 code 职责分离,不可混用。
String provinceDisplayName(String fullProvince) => fullProvince.endsWith('省')
    ? fullProvince.substring(0, fullProvince.length - 1)
    : fullProvince;
