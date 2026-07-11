// 内存 fake 行政区字典 store —— 测 join/载入逻辑,不依赖 Isar 真库。

import 'package:citizenapp/citizen/public/data/admin_division_dto.dart';
import 'package:citizenapp/citizen/public/data/admin_division_store.dart';
import 'package:citizenapp/isar/app_isar.dart';

class FakeAdminDivisionStore implements AdminDivisionStore {
  /// divisionKey -> entity。
  final Map<String, AdminDivisionEntity> byKey = {};
  int upsertCalls = 0;
  int upsertItemCount = 0;
  int deleteCalls = 0;
  List<String> lastUpsertKeys = const [];

  @override
  Future<void> upsertDivisions(
    List<AdminDivisionDto> items, {
    String? dictVersion,
  }) async {
    upsertCalls++;
    upsertItemCount += items.length;
    lastUpsertKeys = items.map((d) => d.divisionKey).toList(growable: false);
    for (final d in items) {
      byKey[d.divisionKey] = d.toEntity(dictVersion: dictVersion);
    }
  }

  @override
  Future<int> divisionCount() async => byKey.length;

  @override
  Future<String> divisionName(
    String level,
    String scopeKey,
    String code,
  ) async {
    if (code.isEmpty) return code;
    final key = _keyFor(level, scopeKey, code);
    return byKey[key]?.divisionName ?? code; // 未命中回退 code 本身。
  }

  @override
  Future<List<AdminDivisionEntity>> divisionsByLevel(
    String level,
    String scopeKey,
  ) async =>
      byKey.values
          .where((e) => e.level == level && e.scopeKey == scopeKey)
          .toList();

  @override
  Future<List<AdminDivisionEntity>> divisionsOfProvince(
    String provinceCode,
  ) async {
    // 三段前缀并起来,每段以 `|` 收口(与 Isar 实现一致)。
    final prefixes = [
      '${AdminDivisionLevel.province}|$provinceCode|',
      '${AdminDivisionLevel.city}|$provinceCode|',
      '${AdminDivisionLevel.town}|$provinceCode|',
    ];
    return byKey.entries
        .where((e) => prefixes.any(e.key.startsWith))
        .map((e) => e.value)
        .toList(growable: false);
  }

  @override
  Future<List<String>> divisionKeysOfProvince(String provinceCode) async {
    final rows = await divisionsOfProvince(provinceCode);
    return rows.map((e) => e.divisionKey).toList(growable: false);
  }

  @override
  Future<void> deleteByKeys(List<String> divisionKeys) async {
    deleteCalls++;
    for (final key in divisionKeys) {
      byKey.remove(key);
    }
  }

  /// 便捷直接 seed 一条字典记录。
  void seed(AdminDivisionDto dto) {
    byKey[dto.divisionKey] = dto.toEntity();
  }

  static String _keyFor(String level, String scopeKey, String code) {
    switch (level) {
      case AdminDivisionLevel.province:
        return divisionKeyOf(level: level, provinceCode: code);
      case AdminDivisionLevel.city:
        return divisionKeyOf(
          level: level,
          provinceCode: scopeKey,
          cityCode: code,
        );
      case AdminDivisionLevel.town:
        final parts = scopeKey.split('|');
        return divisionKeyOf(
          level: level,
          provinceCode: parts.isNotEmpty ? parts[0] : '',
          cityCode: parts.length > 1 ? parts[1] : '',
          townCode: code,
        );
      default:
        return divisionKeyOf(level: level, provinceCode: code);
    }
  }
}
