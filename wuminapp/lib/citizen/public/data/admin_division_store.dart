// 行政区字典本地存储抽象(ADR-021 行政区唯一真源)。
//
// 中文注释:抽象出存储接口,使载入/查询逻辑可用内存 fake 单测,不依赖 Isar 真库;
// 生产实现见 isar_admin_division_store.dart。全部本地读写,UI 显示名零链读零现查。

import 'package:wuminapp_mobile/isar/wallet_isar.dart';

import 'admin_division_dto.dart';

abstract interface class AdminDivisionStore {
  /// 幂等 upsert 一批行政区字典(按 divisionKey 唯一)。
  Future<void> upsertDivisions(
    List<AdminDivisionDto> items, {
    String? dictVersion,
  });

  /// 字典记录总数(判断是否需要首次灌库)。
  Future<int> divisionCount();

  /// 按 (level, scopeKey, code) 命中字典返回名字。
  ///
  /// **未命中回退返回 code 本身**(绝不崩、绝不空):字典缺失时 UI 仍可用 code 兜底。
  Future<String> divisionName(String level, String scopeKey, String code);

  /// 某 (level, scopeKey) 下全部行政区(省=scopeKey 空、市=pcode、镇=`pc|cc`)。
  Future<List<AdminDivisionEntity>> divisionsByLevel(
    String level,
    String scopeKey,
  );
}
