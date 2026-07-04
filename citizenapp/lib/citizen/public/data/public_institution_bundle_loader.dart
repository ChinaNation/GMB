// 公权机构创世快照包载入 —— 链上快照驱动 reconcile(ADR-018 §九 混合模式 ①)。
//
// 发布期从链上创世状态导出的全量快照打进 assets;客户端把它当本地缓存,
// 公权机构唯一真源仍是链上状态。快照根或省级版本变了就增量刷新——
// 变的换、删的清、没变的不动,零旧数据残留(只读派生数据,无用户数据)。
// 数据包结构:
//   assets/public_institutions/manifest.json =
//     { schema_version, genesis_hash, snapshot_block_hash, state_root,
//       public_institution_root, provinces:[{province_name,manifest_version,shard_hash}] }
//   assets/public_institutions/<省名>.json    = { province_name, manifest_version, institutions:[...] }
// provinces[].manifest_version = 该省机构目录版本,内容一变即变。

import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart' show AssetBundle, rootBundle;
import 'package:citizenapp/isar/wallet_isar.dart';

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
  /// 先 reconcile 行政区字典(机构 join 字典名),再 reconcile 机构。
  /// 机构同步只信省级 manifest_version 表;全局 version 只作完成标记,不能短路省级检查。
  /// 变了的省读取 `<省名>.json` 分片后做行级 diff:只 upsert 变化行、只删 absent cid。
  /// manifest 缺省级版本表时视为无效数据包,直接拒绝写库。
  /// 返回机构部分是否发生写入。
  Future<bool> ensureSynced() async {
    // 字典先于机构(机构 join 字典名);字典自己也走版本驱动增量。
    await divisionLoader.ensureSynced();

    final manifest = await _readManifest();
    if (manifest == null) {
      if (await store.institutionCount() > 0) return false;
      return loadFromBundle();
    }

    final globalVersion = _snapshotVersion(manifest);
    final provinceVers = _parseProvinceVersions(manifest['provinces']);

    // 当前数据包必须提供省级版本表;缺失时不猜测、不回退。
    if (provinceVers.isEmpty) {
      return false;
    }

    final storedGlobal = await versionKv.readGlobalVersion();
    final hasData = await store.institutionCount() > 0;

    // 全局 version 变了才重写省份规范顺序;省内内容仍以省级 manifest_version 决定是否读分片。
    final provinceNames =
        provinceVers.map((p) => p.provinceName).toList(growable: false);
    if (storedGlobal != globalVersion || !hasData) {
      await store.setProvinceOrder(provinceNames);
    }

    final storedProvVers = await versionKv.readProvinceVersions();
    final nextProvVers = Map<String, String>.of(storedProvVers);
    var changed = false;

    for (final p in provinceVers) {
      // 没变的省(manifest_version 相同且本地有数据):不读分片、不碰库。
      if (hasData && storedProvVers[p.provinceName] == p.manifestVersion) {
        continue;
      }

      final provinceChanged = await _reconcileProvince(
        p.provinceName,
        fallbackVersion: globalVersion,
      );
      changed = changed || provinceChanged;

      nextProvVers[p.provinceName] = p.manifestVersion;
      await versionKv.writeProvinceVersions(nextProvVers);
    }

    await versionKv.writeGlobalVersion(globalVersion);
    return changed;
  }

  /// 强制按数据包 reconcile。无数据包时返回 false。
  ///
  /// 首装时逐省 reconcile;只写新增/变更行,并删包内已无的旧 cid。
  Future<bool> loadFromBundle() async {
    final manifest = await _readManifest();
    if (manifest == null) return false;

    final provinceVers = _parseProvinceVersions(manifest['provinces']);
    if (provinceVers.isEmpty) return false;

    final provinceNames =
        provinceVers.map((p) => p.provinceName).toList(growable: false);
    final fallbackVersion = _snapshotVersion(manifest);

    await store.setProvinceOrder(provinceNames);
    var changed = false;
    for (final province in provinceNames) {
      final provinceChanged =
          await _reconcileProvince(province, fallbackVersion: fallbackVersion);
      changed = changed || provinceChanged;
    }

    // 全量灌完落版本游标(供后续走增量),同步字典 manifest_version。
    await versionKv.writeProvinceVersions(
      {for (final p in provinceVers) p.provinceName: p.manifestVersion},
    );
    await versionKv.writeGlobalVersion(fallbackVersion);

    // 末尾确保字典已同步(机构名字 join 唯一真源,ADR-021)。
    await divisionLoader.ensureSynced();
    return changed;
  }

  /// reconcile 单省:读 `<省名>.json` 分片 → 只 upsert 变化行 → 删包里已没有的废 cid。
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

    // 删包里没有的废 cid:provinceCode 取自机构记录自带字段(同省一致)。
    final provinceCode = _provinceCodeOf(
      items,
      fallback: shard['province_name']?.toString(),
    );
    var changed = false;
    if (provinceCode != null) {
      final oldRows = await store.institutionsOfProvince(provinceCode);
      final oldByCid = {for (final e in oldRows) e.cidNumber: e};
      final changedItems = items
          .where((d) => !_sameInstitution(oldByCid[d.cidNumber], d))
          .toList(growable: false);
      if (changedItems.isNotEmpty) {
        await store.upsertInstitutions(changedItems, catalogVersion: version);
        changed = true;
      }

      final newCids = items.map((d) => d.cidNumber).toSet();
      final oldCids = oldByCid.keys;
      final staleCids =
          oldCids.where((s) => !newCids.contains(s)).toList(growable: false);
      if (staleCids.isNotEmpty) {
        await store.deleteByCids(staleCids);
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

  /// 全局快照版本优先由链身份锚点 + 公权机构根哈希组成。
  /// 这样 App 可以区分“同名 version 但实际链快照不同”的发布错误。
  static String _snapshotVersion(Map<String, dynamic> manifest) {
    final genesisHash = manifest['genesis_hash']?.toString() ?? '';
    final snapshotBlockHash = manifest['snapshot_block_hash']?.toString() ?? '';
    final publicInstitutionRoot =
        manifest['public_institution_root']?.toString() ?? '';
    if (genesisHash.isNotEmpty &&
        snapshotBlockHash.isNotEmpty &&
        publicInstitutionRoot.isNotEmpty) {
      return '$genesisHash:$snapshotBlockHash:$publicInstitutionRoot';
    }
    return manifest['version'] as String? ?? '0';
  }

  /// 解析当前 manifest `provinces:[{province_name,manifest_version}]` → 有序列表。
  static List<_ProvinceVer> _parseProvinceVersions(Object? raw) {
    if (raw is! List) return const [];
    final out = <_ProvinceVer>[];
    for (final e in raw) {
      if (e is! Map) continue;
      final provinceName = e['province_name']?.toString();
      final manifestVersion = e['manifest_version']?.toString();
      if (provinceName != null &&
          provinceName.isNotEmpty &&
          manifestVersion != null) {
        out.add(_ProvinceVer(
          provinceName: provinceName,
          manifestVersion: manifestVersion,
        ));
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
    return old.cidNumber == dto.cidNumber &&
        old.cidFullName == (dto.cidFullName ?? dto.cidNumber) &&
        old.cidShortName == dto.cidShortName &&
        old.status == dto.status &&
        old.provinceCode == dto.provinceCode &&
        old.cityCode == dto.cityCode &&
        old.townCode == dto.townCode &&
        old.institutionCode == dto.institutionCode &&
        old.parentCidNumber == dto.parentCidNumber &&
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

/// manifest 省级版本条目(内部用):省名 + 内容 manifestVersion。
class _ProvinceVer {
  const _ProvinceVer({
    required this.provinceName,
    required this.manifestVersion,
  });

  final String provinceName;
  final String manifestVersion;
}
