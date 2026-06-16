// 内存 fake 行政区字典 store —— 测 join/载入逻辑,不依赖 Isar 真库。

import 'package:wuminapp_mobile/citizen/public/data/admin_division_dto.dart';
import 'package:wuminapp_mobile/citizen/public/data/admin_division_store.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

class FakeAdminDivisionStore implements AdminDivisionStore {
  /// divisionKey -> entity。
  final Map<String, AdminDivisionEntity> byKey = {};
  int upsertCalls = 0;

  @override
  Future<void> upsertDivisions(
    List<AdminDivisionDto> items, {
    String? dictVersion,
  }) async {
    upsertCalls++;
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
    return byKey[key]?.name ?? code; // 未命中回退 code 本身。
  }

  @override
  Future<List<AdminDivisionEntity>> divisionsByLevel(
    String level,
    String scopeKey,
  ) async =>
      byKey.values
          .where((e) => e.level == level && e.scopeKey == scopeKey)
          .toList();

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
