// 公权机构目录数据包载入(ADR-018 §九 混合模式 ①)。
//
// 中文注释:发布期生成的全量目录打进 assets;首次启动(库空)载入 Isar 作基线,
// 之后增量交给 sync。数据包结构:
//   assets/public_institutions/manifest.json = { version, provinces: [省名...] }
//   assets/public_institutions/<省名>.json    = { province, manifest_version, institutions: [...] }

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show AssetBundle, rootBundle;

import 'admin_division_bundle_loader.dart';
import 'admin_division_store.dart';
import 'isar_admin_division_store.dart';
import 'public_institution_dto.dart';
import 'public_institution_store.dart';

class PublicInstitutionBundleLoader {
  PublicInstitutionBundleLoader({
    required this.store,
    AssetBundle? bundle,
    AdminDivisionStore? divisionStore,
    AdminDivisionBundleLoader? divisionLoader,
  })  : bundle = bundle ?? rootBundle,
        divisionLoader = divisionLoader ??
            AdminDivisionBundleLoader(
              store: divisionStore ?? IsarAdminDivisionStore(),
              bundle: bundle ?? rootBundle,
            );

  final PublicInstitutionStore store;
  final AssetBundle bundle;

  /// 行政区字典载入器(ADR-021):机构名字唯一真源,基线灌库时一并灌好。
  final AdminDivisionBundleLoader divisionLoader;

  static const String _dir = 'assets/public_institutions';
  static const String _manifestPath = '$_dir/manifest.json';

  /// 库空才从数据包载入基线;已有数据跳过(增量交给 sync)。返回是否载入。
  ///
  /// 中文注释:数据包可达数十万条,逐省分片 + store 内分块事务写入,适合首启
  /// 后台调用;不阻塞 UI。(真隔离 isolate 导入留 follow-up。)
  /// 末尾一并灌行政区字典(库空才灌,幂等),保证机构名字 join 有数据。
  Future<bool> ensureBundleLoaded() async {
    // 字典独立判空灌入(幂等):即便机构库非空,字典空也补灌,二者解耦。
    await divisionLoader.ensureDictionaryLoaded();
    if (await store.institutionCount() > 0) return false;
    return loadFromBundle();
  }

  /// 强制从数据包载入(幂等 upsert)。无数据包时返回 false。
  Future<bool> loadFromBundle() async {
    final manifestRaw = await _tryLoadString(_manifestPath);
    if (manifestRaw == null) return false;

    final manifest = jsonDecode(manifestRaw) as Map<String, dynamic>;
    final provinces = (manifest['provinces'] as List<dynamic>? ?? const [])
        .map((e) => e as String)
        .toList(growable: false);
    final fallbackVersion = manifest['version'] as String? ?? '0';

    await store.setProvinceOrder(provinces);
    for (final province in provinces) {
      final shardRaw = await _tryLoadString('$_dir/$province.json');
      if (shardRaw == null) continue;
      final shard = jsonDecode(shardRaw) as Map<String, dynamic>;
      final version = shard['manifest_version'] as String? ?? fallbackVersion;
      final items = (shard['institutions'] as List<dynamic>? ?? const [])
          .map((e) => PublicInstitutionDto.fromJson(e as Map<String, dynamic>))
          .toList(growable: false);
      await store.upsertInstitutions(items, catalogVersion: version);
      await store.setProvinceVersion(province, version);
    }
    // 末尾一并灌行政区字典(幂等),机构名字 join 唯一真源(ADR-021)。
    await divisionLoader.ensureDictionaryLoaded();
    return true;
  }

  Future<String?> _tryLoadString(String path) async {
    try {
      return await bundle.loadString(path);
    } on FlutterError {
      // 资源不存在(数据包尚未生成)——返回 null 走空,不崩。
      return null;
    } on Exception {
      return null;
    }
  }
}
