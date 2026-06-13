// 多签发现统一编排(ADR-018 §九 L1)。
//
// 单次扫描 `AdminsChange::AdminAccounts`,把结果分发给机构多签与个人多签两个
// 后处理服务做过滤 / 落库。取代两个服务各自全表扫一遍(2 次 → 1 次)。
// 节流、本地钱包读取、扫描入口统一收口在本协调器,业务服务只做 processScanned。

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/governance/organization-manage/institution_discovery_service.dart';
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_discovery_service.dart';
import 'package:wuminapp_mobile/governance/shared/admin_accounts_scan_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 编排结果:两类发现统计 + 派生的"是否有变更 / 是否完整完成"。
@immutable
class MultisigDiscoveryResult {
  const MultisigDiscoveryResult({
    required this.institution,
    required this.personal,
  });

  final DiscoveryStats institution;
  final PersonalManageDiscoveryStats personal;

  /// 本轮有新增或孤儿删除 → 列表需要刷新。
  bool get anyChanged =>
      institution.newlyAdded > 0 ||
      institution.orphansRemoved > 0 ||
      personal.newlyAdded > 0 ||
      personal.orphansRemoved > 0;

  /// 两类后处理都未发生部分失败 → 本轮扫描结果完整。
  bool get completed =>
      !institution.partialFailure && !personal.partialFailure;

  static const empty = MultisigDiscoveryResult(
    institution: DiscoveryStats.empty,
    personal: PersonalManageDiscoveryStats.empty,
  );
}

/// 机构多签 + 个人多签发现协调器:一次扫描,分发两类后处理。
class MultisigDiscoveryCoordinator {
  MultisigDiscoveryCoordinator({
    AdminAccountsScanService? scanService,
    InstitutionDiscoveryService? institutionService,
    PersonalManageDiscoveryService? personalService,
    WalletManager? walletManager,
  })  : _scan = scanService ?? AdminAccountsScanService(),
        _institution = institutionService ?? InstitutionDiscoveryService(),
        _personal = personalService ?? PersonalManageDiscoveryService(),
        _wallets = walletManager ?? WalletManager();

  final AdminAccountsScanService _scan;
  final InstitutionDiscoveryService _institution;
  final PersonalManageDiscoveryService _personal;
  final WalletManager _wallets;

  /// 30 分钟节流窗口(机构 + 个人共用一次扫描,故共用一个节流)。
  static const _throttleWindow = Duration(minutes: 30);

  /// SharedPreferences key:最近一次成功扫描的 epochMs(统一口径,
  /// 取代旧的 `duoqian_discovery_last_at_ms` / `personal_manage_discovery_last_at_ms`)。
  static const _prefsLastDiscoveryAt = 'multisig_discovery_last_at_ms';

  Future<DateTime?> lastDiscoveryAt() => _readLastDiscoveryAt();

  /// 单次扫描 + 分发。空钱包或节流命中时直接返回 empty,不发链。
  ///
  /// [myPubkeysHex] 若 null 自动从 WalletManager 取本地全部钱包公钥(小写 hex,无 0x)。
  /// [force] true 跳过 30 分钟节流。
  /// [onProgress] 扫描进度回调:(已扫描 key 数, 已知总数或 null, 已解码条目数)。
  Future<MultisigDiscoveryResult> discoverAll({
    Set<String>? myPubkeysHex,
    bool force = false,
    void Function(int scanned, int? total, int decoded)? onProgress,
  }) async {
    if (!force) {
      final last = await _readLastDiscoveryAt();
      if (last != null && DateTime.now().difference(last) < _throttleWindow) {
        return MultisigDiscoveryResult.empty;
      }
    }

    final myPubkeys = myPubkeysHex ?? await _readMyPubkeys();
    if (myPubkeys.isEmpty) return MultisigDiscoveryResult.empty;

    final scan = await _scan.scanAll(onProgress: onProgress);

    final institutionStats =
        await _institution.processScanned(scan, myPubkeys: myPubkeys);
    final personalStats =
        await _personal.processScanned(scan, myPubkeys: myPubkeys);

    await _writeLastDiscoveryAt(DateTime.now());

    return MultisigDiscoveryResult(
      institution: institutionStats,
      personal: personalStats,
    );
  }

  Future<Set<String>> _readMyPubkeys() async {
    try {
      final wallets = await _wallets.getWallets();
      return wallets.map((w) {
        var pk = w.pubkeyHex.toLowerCase();
        if (pk.startsWith('0x')) pk = pk.substring(2);
        return pk;
      }).toSet();
    } catch (_) {
      return <String>{};
    }
  }

  Future<DateTime?> _readLastDiscoveryAt() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final ms = prefs.getInt(_prefsLastDiscoveryAt);
      return ms == null ? null : DateTime.fromMillisecondsSinceEpoch(ms);
    } catch (_) {
      return null;
    }
  }

  Future<void> _writeLastDiscoveryAt(DateTime t) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setInt(_prefsLastDiscoveryAt, t.millisecondsSinceEpoch);
    } catch (_) {
      // 中文注释:节流时间写入失败不阻断本次发现结果。
    }
  }
}
