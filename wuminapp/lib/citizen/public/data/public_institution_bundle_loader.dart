// 公权机构目录数据包载入 —— 版本驱动增量 reconcile(ADR-018 §九 混合模式 ①)。
//
// 中文注释:发布期生成的全量目录打进 assets;客户端无服务端,数据靠 assets 包
// 分发。包版本变了就增量刷新——变的换、删的清、没变的不动,零旧数据残留
// (只读派生数据,无用户数据)。数据包结构:
//   assets/public_institutions/manifest.json = { version, provinces:[{name,ver}] }
//   assets/public_institutions/<省名>.json    = { province, manifest_version, institutions:[...] }
// provinces[].ver = 该省机构目录 manifest_version,内容一变即变。

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show AssetBundle, rootBundle;
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

import 'admin_division_bundle_loader.dart';
import 'admin_division_store.dart';
import 'data_version_kv.dart';
import 'isar_admin_division_store.dart';
import 'public_institution_dto.dart';
import 'public_institution_store.dart';

class PublicInstitutionBundleLoader {
  PublicInstitutionBundleLoader({
    required this.store,
    AssetBundle? bundle,
    AdminDivisionStore? divisionStore,
    AdminDivisionBundleLoader? divisionLoader,
    DataVersionKv? versionKv,
  })  : bundle = bundle ?? rootBundle,
        divisionLoader = divisionLoader ??
            AdminDivisionBundleLoader(
              store: divisionStore ?? IsarAdminDivisionStore(),
              bundle: bundle ?? rootBundle,
            ),
        versionKv = versionKv ?? DataVersionKv(namespace: 'public_institution');

  final PublicInstitutionStore store;
  final AssetBundle bundle;

  /// 行政区字典载入器(ADR-021):机构名字唯一真源,机构 reconcile 前先同步。
  final AdminDivisionBundleLoader divisionLoader;

  /// 版本游标存储(与 schemaVersion 解耦,独立管数据新鲜度)。
  final DataVersionKv versionKv;

  static const String _dir = 'assets/public_institutions';
  static const String _manifestPath = '$_dir/manifest.json';

  /// 版本驱动增量 reconcile:包版本变了就增量刷新,变的换、删的清、没变的不动。
  ///
  /// 中文注释:先 reconcile 行政区字典(机构 join 字典名),再 reconcile 机构。
  /// 机构同步只信省级 ver 表;全局 version 只作完成标记,不能短路省级检查。
  /// 变了的省读取 `<省名>.json` 分片后做行级 diff:只 upsert 变化行、只删 absent sfid。
  /// manifest 缺 provinces 版本表(旧格式包)→ 回退 [loadFromBundle] reconcile。
  /// 返回机构部分是否发生写入。
  Future<bool> ensureSynced() async {
    // 字典先于机构(机构 join 字典名);字典自己也走版本驱动增量。
    await divisionLoader.ensureSynced();

    final manifest = await _readManifest();
    if (manifest == null) {
      if (await store.institutionCount() > 0) return false;
      return loadFromBundle();
    }

    final globalVersion = manifest['version'] as String? ?? '0';
    final provinceVers = _parseProvinceVersions(manifest['provinces']);

    // 缺省级版本表(旧格式包)→ 无法确认本地是否干净,强制按包 reconcile。
    if (provinceVers.isEmpty) {
      return loadFromBundle();
    }

    final storedGlobal = await versionKv.readGlobalVersion();
    final hasData = await store.institutionCount() > 0;

    // 全局 version 变了才重写省份规范顺序;省内内容仍以省级 ver 表决定是否读分片。
    final provinceNames =
        provinceVers.map((p) => p.name).toList(growable: false);
    if (storedGlobal != globalVersion || !hasData) {
      await store.setProvinceOrder(provinceNames);
    }

    final storedProvVers = await versionKv.readProvinceVersions();
    final nextProvVers = Map<String, String>.of(storedProvVers);
    var changed = false;

    for (final p in provinceVers) {
      // 没变的省(ver 相同且本地有数据):不读分片、不碰库。
      if (hasData && storedProvVers[p.name] == p.ver) continue;

      final provinceChanged =
          await _reconcileProvince(p.name, fallbackVersion: globalVersion);
      changed = changed || provinceChanged;

      nextProvVers[p.name] = p.ver;
      await versionKv.writeProvinceVersions(nextProvVers);
    }

    await versionKv.writeGlobalVersion(globalVersion);
    return changed;
  }

  /// 强制按数据包 reconcile。无数据包时返回 false。
  ///
  /// 中文注释:首装 / 版本表缺失时逐省 reconcile;只写新增/变更行,并删包内已无的旧 sfid。
  Future<bool> loadFromBundle() async {
    final manifest = await _readManifest();
    if (manifest == null) return false;

    final provinceNames = _parseProvinceNames(manifest['provinces']);
    final fallbackVersion = manifest['version'] as String? ?? '0';

    await store.setProvinceOrder(provinceNames);
    var changed = false;
    for (final province in provinceNames) {
      final provinceChanged =
          await _reconcileProvince(province, fallbackVersion: fallbackVersion);
      changed = changed || provinceChanged;
    }

    // 全量灌完落版本游标(供后续走增量),同步字典 ver。
    final provinceVers = _parseProvinceVersions(manifest['provinces']);
    if (provinceVers.isNotEmpty) {
      await versionKv.writeProvinceVersions(
        {for (final p in provinceVers) p.name: p.ver},
      );
    }
    await versionKv.writeGlobalVersion(fallbackVersion);

    // 末尾确保字典已同步(机构名字 join 唯一真源,ADR-021)。
    await divisionLoader.ensureSynced();
    return changed;
  }

  /// reconcile 单省:读 `<省名>.json` 分片 → 只 upsert 变化行 → 删包里已没有的废 sfid。
  Future<bool> _reconcileProvince(
    String provinceName, {
    required String fallbackVersion,
  }) async {
    final shardRaw = await _tryLoadString('$_dir/$provinceName.json');
    if (shardRaw == null) return false;

    final shard = jsonDecode(shardRaw) as Map<String, dynamic>;
    final version = shard['manifest_version'] as String? ?? fallbackVersion;
    final items = (shard['institutions'] as List<dynamic>? ?? const [])
        .map((e) => PublicInstitutionDto.fromJson(e as Map<String, dynamic>))
        .toList(growable: false);

    // 删包里没有的废 sfid:provinceCode 取自机构记录自带字段(同省一致)。
    final provinceCode = _provinceCodeOf(
      items,
      fallback: shard['province']?.toString(),
    );
    var changed = false;
    if (provinceCode != null) {
      final oldRows = await store.institutionsOfProvince(provinceCode);
      final oldBySfid = {for (final e in oldRows) e.sfidNumber: e};
      final changedItems = items
          .where((d) => !_sameInstitution(oldBySfid[d.sfidNumber], d))
          .toList(growable: false);
      if (changedItems.isNotEmpty) {
        await store.upsertInstitutions(changedItems, catalogVersion: version);
        changed = true;
      }

      final newSfids = items.map((d) => d.sfidNumber).toSet();
      final oldSfids = oldBySfid.keys;
      final staleSfids =
          oldSfids.where((s) => !newSfids.contains(s)).toList(growable: false);
      if (staleSfids.isNotEmpty) {
        await store.deleteBySfids(staleSfids);
        changed = true;
      }
    }

    await store.setProvinceVersion(provinceName, version);
    return changed;
  }

  /// 取该省机构记录自带的 provinceCode(同省一致);空分片返回 null。
  static String? _provinceCodeOf(
    List<PublicInstitutionDto> items, {
    String? fallback,
  }) {
    for (final d in items) {
      if (d.provinceCode.isNotEmpty) return d.provinceCode;
    }
    if (fallback != null && fallback.isNotEmpty) return fallback;
    return null;
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

  /// 解析 manifest `provinces:[{name,ver}]`(新格式)→ 有序列表;旧格式返回空。
  static List<_ProvinceVer> _parseProvinceVersions(Object? raw) {
    if (raw is! List) return const [];
    final out = <_ProvinceVer>[];
    for (final e in raw) {
      if (e is! Map) continue;
      final name = e['name']?.toString();
      final ver = e['ver']?.toString();
      if (name != null && name.isNotEmpty && ver != null) {
        out.add(_ProvinceVer(name: name, ver: ver));
      }
    }
    return out;
  }

  /// 省名有序列表:新格式 `[{name,ver}]` 取 name;旧格式 `[省名...]` 直取。
  static List<String> _parseProvinceNames(Object? raw) {
    if (raw is! List) return const [];
    final out = <String>[];
    for (final e in raw) {
      if (e is Map) {
        final name = e['name']?.toString();
        if (name != null && name.isNotEmpty) out.add(name);
      } else if (e is String && e.isNotEmpty) {
        out.add(e);
      }
    }
    return out;
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

  static bool _sameInstitution(
    PublicInstitutionEntity? old,
    PublicInstitutionDto dto,
  ) {
    if (old == null) return false;
    return old.sfidNumber == dto.sfidNumber &&
        old.institutionName ==
            (dto.institutionName ?? dto.sfidName ?? dto.sfidNumber) &&
        old.sfidName == dto.sfidName &&
        old.shortName == dto.shortName &&
        old.status == dto.status &&
        old.provinceCode == dto.provinceCode &&
        old.cityCode == dto.cityCode &&
        old.townCode == dto.townCode &&
        old.institutionCode == dto.institutionCode &&
        old.orgCode == dto.orgCode &&
        old.parentSfidNumber == dto.parentSfidNumber &&
        old.hasLegalPersonality == dto.hasLegalPersonality &&
        old.legalRepName == dto.legalRepName &&
        old.accountCount == dto.accountCount &&
        _sameStringList(old.customAccountNames, dto.customAccountNames);
  }

  static bool _sameStringList(List<String> a, List<String> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}

/// manifest 省级版本条目(内部用):省名 + 内容 ver。
class _ProvinceVer {
  const _ProvinceVer({required this.name, required this.ver});

  final String name;
  final String ver;
}
