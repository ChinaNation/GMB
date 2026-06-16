// 公权机构目录 repo 门面(ADR-018 §九,混合模式)。
//
// 中文注释:card B/C 的统一入口。**读全部走本地 store(零链读零网络、秒开)**;
// 数据包基线由 [ensureBundleLoaded] 在首启后台灌入;某省的在线增量由
// [refreshProvince] 后台跑(TTL 节流 + 失败上抛供 UI 决定提示)。UI 一律先读本地、
// 再后台刷新,绝不阻塞在网络同步上(消除"一直转圈")。

import 'package:flutter/foundation.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

import 'admin_division_dto.dart';
import 'admin_division_store.dart';
import 'area_path_formatter.dart';
import 'isar_admin_division_store.dart';
import 'isar_public_institution_store.dart';
import 'public_institution_bundle_loader.dart';
import 'public_institution_store.dart';
import 'public_institution_sync_service.dart';
import 'public_provinces.dart';

class PublicInstitutionRepository {
  PublicInstitutionRepository({
    PublicInstitutionStore? store,
    AdminDivisionStore? divisionStore,
    PublicInstitutionSyncService? sync,
    PublicInstitutionBundleLoader? loader,
    Duration? syncTtl,
  })  : store = store ?? IsarPublicInstitutionStore(),
        divisionStore = divisionStore ?? IsarAdminDivisionStore(),
        _syncTtl = syncTtl ?? const Duration(minutes: 2) {
    this.sync = sync ?? PublicInstitutionSyncService(store: this.store);
    this.loader = loader ??
        PublicInstitutionBundleLoader(
          store: this.store,
          divisionStore: this.divisionStore,
        );
  }

  final PublicInstitutionStore store;

  /// 行政区字典(ADR-021 行政区唯一真源):机构显示名按 code join 此字典。
  final AdminDivisionStore divisionStore;
  late final PublicInstitutionSyncService sync;
  late final PublicInstitutionBundleLoader loader;

  final Duration _syncTtl;
  final Map<String, int> _lastSyncMs = {};

  // ── 读(本地,零网络,秒开)──
  Future<List<String>> listProvinces() => store.listProvinces();
  Future<List<String>> listCities(String provinceCode) =>
      store.listCities(provinceCode);
  Future<List<PublicInstitutionEntity>> listInstitutionsByCity(
    String provinceCode,
    String cityCode,
  ) =>
      store.listInstitutionsByCity(provinceCode, cityCode);
  Future<PublicInstitutionEntity?> getBySfid(String sfidNumber) =>
      store.getBySfid(sfidNumber);

  // ── 行政区字典 join(ADR-021;UI 显示名唯一来自字典/链上常量省名)──

  /// 某市 code → 市名(查字典;未命中回退 code 本身,绝不崩)。
  Future<String> cityName(String provinceCode, String cityCode) {
    final scope = scopeKeyOf(
      level: AdminDivisionLevel.city,
      provinceCode: provinceCode,
    );
    return divisionStore.divisionName(
      AdminDivisionLevel.city,
      scope,
      cityCode,
    );
  }

  /// (provinceCode, cityCode, townCode) → 「省名·市名[·镇名]」显示路径。
  ///
  /// 省名走链上常量(认可的省名源);空 town 只显到市;字典缺失回退 code。
  /// **不在 widget build 里调**:在 repository / state 层预 join 成 view-model。
  Future<String> areaPath({
    required String provinceCode,
    required String cityCode,
    String townCode = '',
  }) {
    return formatAreaPath(
      divisionStore,
      provinceName: provinceDisplayNameByCode(provinceCode),
      provinceCode: provinceCode,
      cityCode: cityCode,
      townCode: townCode,
    );
  }

  /// 机构所属地显示路径(详情页 所属地行用;省名带"省"全名)。
  Future<String> institutionAreaPath(PublicInstitutionEntity inst) {
    return formatAreaPath(
      divisionStore,
      provinceName: provinceFullNameByCode(inst.provinceCode),
      provinceCode: inst.provinceCode,
      cityCode: inst.cityCode,
      townCode: inst.townCode,
    );
  }

  // ── 订阅("关注")──
  Future<void> subscribe(String walletPubkeyHex, String sfidNumber) =>
      store.subscribe(walletPubkeyHex, sfidNumber);
  Future<void> unsubscribe(String walletPubkeyHex, String sfidNumber) =>
      store.unsubscribe(walletPubkeyHex, sfidNumber);
  Future<bool> isSubscribed(String walletPubkeyHex, String sfidNumber) =>
      store.isSubscribed(walletPubkeyHex, sfidNumber);
  Future<List<PublicInstitutionEntity>> listSubscribed(
    String walletPubkeyHex,
  ) =>
      store.listSubscribed(walletPubkeyHex);

  /// 首启后台灌入数据包基线(库空才灌)。返回是否灌入。
  Future<bool> ensureBundleLoaded() => loader.ensureBundleLoaded();

  /// 后台刷新某省的在线增量。**非阻塞调用方**:UI 先读本地再调本方法。
  /// TTL 内重复调跳过;失败上抛(UI 自行 catch 决定是否提示),失败不计入节流以便重试。
  Future<void> refreshProvince(String province) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final last = _lastSyncMs[province];
    if (last != null && now - last < _syncTtl.inMilliseconds) return;
    _lastSyncMs[province] = now;
    try {
      await sync.syncProvince(province);
    } on Exception catch (e) {
      _lastSyncMs.remove(province);
      debugPrint('[public-institution] sync "$province" failed: $e');
      rethrow;
    }
  }
}
