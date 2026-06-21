// 行政区显示路径拼装(ADR-021 行政区唯一真源)。
//
// 中文注释:机构只存 (provinceCode, cityCode, townCode);显示名一律按三元组
// 查字典 join。**不在 widget build 里 await**——在 repository 层预 join 成
// view-model 字段,UI 直接读字符串。
//
// 省名走链上常量(`publicProvinceNames` / `provinceDisplayName`,见
// public_provinces.dart)这一认可的省名源;市/镇名走字典。

import 'admin_division_dto.dart';
import 'admin_division_store.dart';

/// 把 (provinceCode, cityCode, townCode) 拼成「省名·市名·镇名」显示路径。
///
/// 规则(ADR-021):
/// - 有 town:显「省名·市名·镇名」。
/// - 空 town:只显「省名·市名」,**不拼空段、不显 null**。
/// - 字典缺失:回退显 code 本身(绝不崩、绝不空)。
///
/// [provinceName] 省名由调用方传入(链上常量源,认可的省名源);市/镇名查 [store]。
/// 用 ` · ` 连接,与现有 UI 统一。
Future<String> formatAreaPath(
  AdminDivisionStore store, {
  required String provinceName,
  required String provinceCode,
  required String cityCode,
  String townCode = '',
}) async {
  final segments = <String>[];
  if (provinceName.isNotEmpty) segments.add(provinceName);

  if (cityCode.isNotEmpty) {
    final cityScope = scopeKeyOf(
      level: AdminDivisionLevel.city,
      provinceCode: provinceCode,
    );
    segments.add(
      await store.divisionName(AdminDivisionLevel.city, cityScope, cityCode),
    );
  }

  // 空 town 只显到市,不拼空段。
  if (townCode.isNotEmpty) {
    final townScope = scopeKeyOf(
      level: AdminDivisionLevel.town,
      provinceCode: provinceCode,
      cityCode: cityCode,
    );
    segments.add(
      await store.divisionName(AdminDivisionLevel.town, townScope, townCode),
    );
  }

  return segments.join(' · ');
}
