// 公权机构目录 repo 门面(ADR-018 §九,混合模式)。
//
// 中文注释:card B/C 的统一入口。**读全部走本地 store(零链读零网络、秒开)**;
// 数据包基线由 [ensureBundleLoaded] 在首启后台灌入;某省的在线增量由
// [refreshProvince] 后台跑(TTL 节流 + 失败上抛供 UI 决定提示)。UI 一律先读本地、
// 再后台刷新,绝不阻塞在网络同步上(消除"一直转圈")。

import 'package:flutter/foundation.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

import 'isar_public_institution_store.dart';
import 'public_institution_bundle_loader.dart';
import 'public_institution_store.dart';
import 'public_institution_sync_service.dart';

class PublicInstitutionRepository {
  PublicInstitutionRepository({
    PublicInstitutionStore? store,
    PublicInstitutionSyncService? sync,
    PublicInstitutionBundleLoader? loader,
    Duration? syncTtl,
  })  : store = store ?? IsarPublicInstitutionStore(),
        _syncTtl = syncTtl ?? const Duration(minutes: 2) {
    this.sync = sync ?? PublicInstitutionSyncService(store: this.store);
    this.loader = loader ?? PublicInstitutionBundleLoader(store: this.store);
  }

  final PublicInstitutionStore store;
  late final PublicInstitutionSyncService sync;
  late final PublicInstitutionBundleLoader loader;

  final Duration _syncTtl;
  final Map<String, int> _lastSyncMs = {};

  // ── 读(本地,零网络,秒开)──
  Future<List<String>> listProvinces() => store.listProvinces();
  Future<List<String>> listCities(String province) =>
      store.listCities(province);
  Future<List<PublicInstitutionEntity>> listInstitutionsByCity(
    String province,
    String city,
  ) =>
      store.listInstitutionsByCity(province, city);
  Future<PublicInstitutionEntity?> getBySfid(String sfidNumber) =>
      store.getBySfid(sfidNumber);

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
