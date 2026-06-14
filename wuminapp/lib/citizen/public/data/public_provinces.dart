// 公权机构省份导航来源 —— 复用治理省储会的同一行政区(不新建)。
//
// 中文注释:省份是固定行政区划(43 省),与机构数据是否加载无关,必须始终全显
// (对称治理 tab 的 43 省储会编译期常量)。省名 = 省储会名去掉 `公民储备委员会`
// 后缀,**保留"省"字**,与 SFID `province` 字段(china.sqlite 省名,如 `中枢省`)
// 逐字对齐——否则点省查不到机构。

import 'package:wuminapp_mobile/governance/organization-manage/institution_registry.dart';

const String _kCouncilSuffix = '公民储备委员会';

/// 公权机构左栏的 43 个省份(规范行政区**全名**,含"省",用于匹配/查询 SFID)。
List<String> publicProvinceNames() {
  return kProvincialCouncils
      .map((c) => c.name.endsWith(_kCouncilSuffix)
          ? c.name.substring(0, c.name.length - _kCouncilSuffix.length)
          : c.name)
      .toList(growable: false);
}

/// 省份**展示名**:去掉末尾"省"字。
///
/// 中文注释:展示一律不带"省"(中枢/岭南/广东);但匹配/查询仍用 [publicProvinceNames]
/// 的全名(中枢省),与 SFID `province` 字段对齐。两者职责分离,不可混用。
String provinceDisplayName(String fullProvince) => fullProvince.endsWith('省')
    ? fullProvince.substring(0, fullProvince.length - 1)
    : fullProvince;
