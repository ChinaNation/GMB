// 行政区字典本地存储 —— Isar 实现(ADR-021 行政区唯一真源)。
//
// 中文注释:全国 5 万条(省43/市3185/镇47574),分块写避免巨型事务卡 UI;
// 查询走唯一索引 divisionKey / scopeKey,UI 显示名零现查。

import 'package:isar_community/isar.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

import 'admin_division_dto.dart';
import 'admin_division_store.dart';

class IsarAdminDivisionStore implements AdminDivisionStore {
  IsarAdminDivisionStore({Isar? isar}) : _injected = isar;

  final Isar? _injected;

  /// 单事务批量上限:5 万条镇级数据分块写,避免巨型事务卡主线程 / 占内存。
  static const int _upsertChunk = 2000;

  Future<Isar> _db() async => _injected ?? await WalletIsar.instance.db();

  Future<T> _write<T>(Future<T> Function(Isar isar) action) async {
    final injected = _injected;
    if (injected != null) {
      return injected.writeTxn(() => action(injected));
    }
    return WalletIsar.instance.writeTxn(action);
  }

  @override
  Future<void> upsertDivisions(
    List<AdminDivisionDto> items, {
    String? dictVersion,
  }) async {
    if (items.isEmpty) return;
    for (var start = 0; start < items.length; start += _upsertChunk) {
      final end = (start + _upsertChunk).clamp(0, items.length);
      final entities = items
          .sublist(start, end)
          .map((dto) => dto.toEntity(dictVersion: dictVersion))
          .toList(growable: false);
      await _write((isar) async {
        await isar.adminDivisionEntitys.putAllByDivisionKey(entities);
      });
    }
  }

  @override
  Future<int> divisionCount() async {
    final isar = await _db();
    return isar.adminDivisionEntitys.count();
  }

  @override
  Future<String> divisionName(
    String level,
    String scopeKey,
    String code,
  ) async {
    // 缺级回退:空 code 没有名字可查,直接回退空(调用方自行决定是否拼段)。
    if (code.isEmpty) return code;
    final isar = await _db();
    // 由 (level, scopeKey, code) 反推完整 divisionKey 唯一命中。
    final key = _divisionKeyFor(level, scopeKey, code);
    final hit = await isar.adminDivisionEntitys
        .filter()
        .divisionKeyEqualTo(key)
        .findFirst();
    // 未命中回退返回 code 本身(绝不崩、绝不空)。
    return hit?.name ?? code;
  }

  @override
  Future<List<AdminDivisionEntity>> divisionsByLevel(
    String level,
    String scopeKey,
  ) async {
    final isar = await _db();
    return isar.adminDivisionEntitys
        .filter()
        .levelEqualTo(level)
        .and()
        .scopeKeyEqualTo(scopeKey)
        .findAll();
  }

  /// 由 (level, scopeKey, code) 还原 divisionKey:
  /// - province: scopeKey 空 → `province|<code>||`
  /// - city: scopeKey=pcode → `city|<pcode>|<code>|`
  /// - town: scopeKey=`pc|cc` → `town|<pc>|<cc>|<code>`
  static String _divisionKeyFor(String level, String scopeKey, String code) {
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
        final pc = parts.isNotEmpty ? parts[0] : '';
        final cc = parts.length > 1 ? parts[1] : '';
        return divisionKeyOf(
          level: level,
          provinceCode: pc,
          cityCode: cc,
          townCode: code,
        );
      default:
        return divisionKeyOf(level: level, provinceCode: code);
    }
  }
}
