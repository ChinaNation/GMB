// 行政区字典数据包载入 —— 版本驱动增量 reconcile(ADR-021 §A2/A3)。
//
// 中文注释:发布期由 `tools/generate_admin_division_bundle.mjs` 直接 dump china.sqlite
// 生成静态数据包。客户端无服务端,数据靠 assets 包分发;包版本变了就增量刷新——
// 变的换、删的清、没变的不动,零旧数据残留(只读派生数据,无用户数据)。
// 数据包结构:
//   assets/admin_divisions/manifest.json      = { version, provinces:[{code,ver}], ... }
//   assets/admin_divisions/provinces.json      = [{code,name}]
//   assets/admin_divisions/cities/<pcode>.json = [{code,name}]
//   assets/admin_divisions/towns/<pcode>.json  = [{city_code,code,name}]
// provinces[].ver = 该省内容(市分片+镇分片)hash,改名/删码/重排一变即变。

import 'dart:convert';

import 'package:flutter/foundation.dart' show FlutterError;
import 'package:flutter/services.dart' show AssetBundle, rootBundle;
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

import 'admin_division_dto.dart';
import 'admin_division_store.dart';
import 'data_version_kv.dart';

class AdminDivisionBundleLoader {
  AdminDivisionBundleLoader({
    required this.store,
    AssetBundle? bundle,
    DataVersionKv? versionKv,
  })  : bundle = bundle ?? rootBundle,
        versionKv = versionKv ?? DataVersionKv(namespace: 'admin_division');

  final AdminDivisionStore store;
  final AssetBundle bundle;

  /// 版本游标存储(与 schemaVersion 解耦,独立管数据新鲜度)。
  final DataVersionKv versionKv;

  static const String _dir = 'assets/admin_divisions';
  static const String _manifestPath = '$_dir/manifest.json';
  static const String _provincesPath = '$_dir/provinces.json';

  /// 版本驱动增量 reconcile:包版本变了就增量刷新,变的换、删的清、没变的不动。
  ///
  /// 中文注释:每次先看省级 ver 表,只 reconcile ver 变了或本地缺游标的省;
  /// 没变的省连分片都不读。全局 version 只作最终完成标记,不能短路省级检查。
  /// manifest 缺省级版本表时视为无效数据包,直接拒绝写库。
  /// 返回是否发生了写入。
  Future<bool> ensureSynced() async {
    final manifest = await _readManifest();
    if (manifest == null) {
      // 无 manifest:库空才全量兜底(loadFromBundle 内部自判数据包是否存在)。
      if (await store.divisionCount() > 0) return false;
      return loadFromBundle();
    }

    final globalVersion = manifest['version'] as String?;
    final provinceVers = _parseProvinceVersions(manifest['provinces']);

    // 当前数据包必须提供省级版本表;缺失时不猜测、不回退。
    if (provinceVers.isEmpty) {
      return false;
    }

    final hasData = await store.divisionCount() > 0;
    final storedProvVers = await versionKv.readProvinceVersions();
    final nextProvVers = Map<String, String>.of(storedProvVers);
    var changed = false;

    for (final entry in provinceVers.entries) {
      final code = entry.key;
      final ver = entry.value;
      // 没变的省(ver 相同且本地有数据):不读分片、不碰库。
      if (hasData && storedProvVers[code] == ver) continue;

      final provinceChanged =
          await _reconcileProvince(code, dictVersion: globalVersion);
      changed = changed || provinceChanged;

      // 逐省落 ver,中断可续(下次启动从这里继续 reconcile 剩余省)。
      nextProvVers[code] = ver;
      await versionKv.writeProvinceVersions(nextProvVers);
    }

    if (globalVersion != null) {
      await versionKv.writeGlobalVersion(globalVersion);
    }
    return changed;
  }

  /// 强制按数据包 reconcile 字典。无数据包时返回 false。
  ///
  /// 中文注释:首装时逐省 reconcile;只写新增/变更行,并删包内已无的旧键。
  Future<bool> loadFromBundle() async {
    final provincesRaw = await _tryLoadString(_provincesPath);
    if (provincesRaw == null) return false;

    final manifest = await _readManifest();
    final version = manifest?['version'] as String?;
    final provinceVers = _parseProvinceVersions(manifest?['provinces']);
    if (provinceVers.isEmpty) return false;

    final provinceJson = jsonDecode(provincesRaw) as List<dynamic>;
    final provinces = provinceJson
        .map((e) => AdminDivisionDto.province(e as Map<String, dynamic>))
        .where((dto) => provinceVers.containsKey(dto.code))
        .toList(growable: false);

    var changed = false;
    for (final p in provinces) {
      final provinceChanged =
          await _reconcileProvince(p.code, dictVersion: version);
      changed = changed || provinceChanged;
    }

    // 全量灌完落版本游标:同步省级 ver 表 + 全局 version,后续走增量。
    await versionKv.writeProvinceVersions(provinceVers);
    if (version != null) {
      await versionKv.writeGlobalVersion(version);
    }
    return changed;
  }

  /// reconcile 单省:读该省分片 → 只 upsert 变化行 → 删包里已没有的废键。
  Future<bool> _reconcileProvince(
    String provinceCode, {
    required String? dictVersion,
  }) async {
    final newDtos = <AdminDivisionDto>[];

    // 省级:从全量 provinces.json 取该省一条(分片不带省级记录)。
    final provincesRaw = await _tryLoadString(_provincesPath);
    if (provincesRaw != null) {
      final provinceJson = jsonDecode(provincesRaw) as List<dynamic>;
      for (final e in provinceJson) {
        final dto = AdminDivisionDto.province(e as Map<String, dynamic>);
        if (dto.code == provinceCode) {
          newDtos.add(dto);
          break;
        }
      }
    }

    final shard = await _loadProvinceShards(provinceCode);
    newDtos
      ..addAll(shard.cities)
      ..addAll(shard.towns);
    if (newDtos.isEmpty) return false;

    final newByKey = {for (final d in newDtos) d.divisionKey: d};
    final oldRows = await store.divisionsOfProvince(provinceCode);
    final oldByKey = {for (final e in oldRows) e.divisionKey: e};

    // 只写新增/改名行;dictVersion 是排错字段,不作为内容变化条件,避免整省重写。
    final changedDtos = newDtos
        .where((d) => !_sameDivision(oldByKey[d.divisionKey], d))
        .toList(growable: false);
    if (changedDtos.isNotEmpty) {
      await store.upsertDivisions(changedDtos, dictVersion: dictVersion);
    }

    // 删包里没有的(被删码 / 重排掉的旧键)。
    final newKeys = newByKey.keys.toSet();
    final oldKeys = oldByKey.keys;
    final staleKeys =
        oldKeys.where((k) => !newKeys.contains(k)).toList(growable: false);
    if (staleKeys.isNotEmpty) {
      await store.deleteByKeys(staleKeys);
    }
    return changedDtos.isNotEmpty || staleKeys.isNotEmpty;
  }

  Future<_ProvinceShards> _loadProvinceShards(String provinceCode) async {
    final cities = <AdminDivisionDto>[];
    final towns = <AdminDivisionDto>[];

    final citiesRaw = await _tryLoadString('$_dir/cities/$provinceCode.json');
    if (citiesRaw != null) {
      cities.addAll(
        (jsonDecode(citiesRaw) as List<dynamic>).map(
          (e) => AdminDivisionDto.city(
            provinceCode,
            e as Map<String, dynamic>,
          ),
        ),
      );
    }

    final townsRaw = await _tryLoadString('$_dir/towns/$provinceCode.json');
    if (townsRaw != null) {
      towns.addAll(
        (jsonDecode(townsRaw) as List<dynamic>).map(
          (e) => AdminDivisionDto.town(
            provinceCode,
            e as Map<String, dynamic>,
          ),
        ),
      );
    }
    return _ProvinceShards(cities: cities, towns: towns);
  }

  Future<Map<String, dynamic>?> _readManifest() async {
    final manifestRaw = await _tryLoadString(_manifestPath);
    if (manifestRaw == null) return null;
    try {
      return jsonDecode(manifestRaw) as Map<String, dynamic>;
    } on FormatException {
      return null;
    }
  }

  /// 解析当前 manifest `provinces:[{code,ver}]` → `{code: ver}`。
  static Map<String, String> _parseProvinceVersions(Object? raw) {
    if (raw is! List) return <String, String>{};
    final out = <String, String>{};
    for (final e in raw) {
      if (e is! Map) continue;
      final code = e['code']?.toString();
      final ver = e['ver']?.toString();
      if (code != null && code.isNotEmpty && ver != null) {
        out[code] = ver;
      }
    }
    return out;
  }

  Future<String?> _tryLoadString(String path) async {
    try {
      return await bundle.loadString(path);
    } on FlutterError {
      // 资源不存在(rootBundle 抛 FlutterError)——返回 null 走空,不崩。
      return null;
    } on Exception {
      return null;
    }
  }

  static bool _sameDivision(AdminDivisionEntity? old, AdminDivisionDto dto) {
    if (old == null) return false;
    return old.divisionKey == dto.divisionKey &&
        old.level == dto.level &&
        old.code == dto.code &&
        old.scopeKey == dto.scopeKey &&
        old.name == dto.name;
  }
}

/// 某省的市/镇分片解析结果(内部用)。
class _ProvinceShards {
  const _ProvinceShards({required this.cities, required this.towns});

  final List<AdminDivisionDto> cities;
  final List<AdminDivisionDto> towns;
}
