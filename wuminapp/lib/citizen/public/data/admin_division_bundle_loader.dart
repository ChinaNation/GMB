// 行政区字典数据包载入(ADR-021 §A2/A3)。
//
// 中文注释:发布期由 `tools/generate_admin_division_bundle.mjs` 直接 dump china.sqlite
// 生成的静态数据包,首启(库空)分批灌进 Isar AdminDivisionEntity 作行政区名字唯一真源。
// 数据包结构:
//   assets/admin_divisions/manifest.json      = { version, china_sqlite_sha256, ... }
//   assets/admin_divisions/provinces.json      = [{code,name}]
//   assets/admin_divisions/cities/<pcode>.json = [{code,name}]
//   assets/admin_divisions/towns/<pcode>.json  = [{city_code,code,name}]

import 'dart:convert';

import 'package:flutter/foundation.dart' show FlutterError;
import 'package:flutter/services.dart' show AssetBundle, rootBundle;

import 'admin_division_dto.dart';
import 'admin_division_store.dart';

class AdminDivisionBundleLoader {
  AdminDivisionBundleLoader({
    required this.store,
    AssetBundle? bundle,
  }) : bundle = bundle ?? rootBundle;

  final AdminDivisionStore store;
  final AssetBundle bundle;

  static const String _dir = 'assets/admin_divisions';
  static const String _manifestPath = '$_dir/manifest.json';
  static const String _provincesPath = '$_dir/provinces.json';

  /// 库空才从数据包灌字典(幂等);已有字典跳过。返回是否灌入。
  ///
  /// 中文注释:5 万条镇级,逐省分片 + store 内分块事务写入,适合首启后台调用;
  /// 字典是机构名字唯一真源,机构包载入前先灌好。
  Future<bool> ensureDictionaryLoaded() async {
    if (await store.divisionCount() > 0) return false;
    return loadFromBundle();
  }

  /// 强制从数据包灌字典(幂等 upsert)。无数据包时返回 false。
  Future<bool> loadFromBundle() async {
    final provincesRaw = await _tryLoadString(_provincesPath);
    if (provincesRaw == null) return false;

    final version = await _readVersion();

    // 省:一份全量。
    final provinceJson = jsonDecode(provincesRaw) as List<dynamic>;
    final provinces = provinceJson
        .map((e) => AdminDivisionDto.province(e as Map<String, dynamic>))
        .toList(growable: false);
    await store.upsertDivisions(provinces, dictVersion: version);

    // 市 + 镇:按省分片懒加载(逐省灌,避免一次性 5 万条占内存)。
    for (final p in provinces) {
      final pcode = p.code;

      final citiesRaw = await _tryLoadString('$_dir/cities/$pcode.json');
      if (citiesRaw != null) {
        final cities = (jsonDecode(citiesRaw) as List<dynamic>)
            .map((e) =>
                AdminDivisionDto.city(pcode, e as Map<String, dynamic>))
            .toList(growable: false);
        await store.upsertDivisions(cities, dictVersion: version);
      }

      final townsRaw = await _tryLoadString('$_dir/towns/$pcode.json');
      if (townsRaw != null) {
        final towns = (jsonDecode(townsRaw) as List<dynamic>)
            .map((e) =>
                AdminDivisionDto.town(pcode, e as Map<String, dynamic>))
            .toList(growable: false);
        await store.upsertDivisions(towns, dictVersion: version);
      }
    }
    return true;
  }

  Future<String?> _readVersion() async {
    final manifestRaw = await _tryLoadString(_manifestPath);
    if (manifestRaw == null) return null;
    try {
      final manifest = jsonDecode(manifestRaw) as Map<String, dynamic>;
      return manifest['version'] as String?;
    } on FormatException {
      return null;
    }
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
}
