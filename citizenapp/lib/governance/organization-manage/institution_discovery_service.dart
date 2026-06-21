// 机构多签反向索引发现:后处理(ADR-018 §九)。
//
// 只负责"后处理":从共享的 AdminAccounts 单次扫描结果(AdminAccountsScanService)
// 里筛出机构多签(kind=InstitutionAccount,org ∈ {PUP,OTH},且管理员含本地钱包),
// 反查 SFID 归属后 upsert 到 Isar。扫描、节流、本地钱包读取统一收口在
// `MultisigDiscoveryCoordinator`,本服务不再各自扫链。
//
// 个人多签后处理见 lib/governance/personal-manage/personal_manage_discovery_service.dart。

import 'package:flutter/foundation.dart';
import 'package:isar_community/isar.dart';
import 'package:citizenapp/governance/shared/admin_account_storage_codec.dart';
import 'package:citizenapp/governance/shared/admin_accounts_scan_service.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

import 'duoqian_storage_codec.dart';
import 'institution_manage_service.dart';

/// 机构多签后处理统计。
class DiscoveryStats {
  const DiscoveryStats({
    required this.institutionsScanned,
    required this.matchedSfidAccounts,
    required this.newlyAdded,
    required this.orphansRemoved,
    required this.elapsed,
    this.partialFailure = false,
  });

  /// 本轮扫描到的 AdminAccounts key 总数。
  final int institutionsScanned;

  /// 命中并成功反查 SFID 的机构账户数。
  final int matchedSfidAccounts;

  /// 新增到 Isar 的机构数。
  final int newlyAdded;

  /// 反向校验删除的孤儿机构数。
  final int orphansRemoved;

  final Duration elapsed;
  final bool partialFailure;

  static const empty = DiscoveryStats(
    institutionsScanned: 0,
    matchedSfidAccounts: 0,
    newlyAdded: 0,
    orphansRemoved: 0,
    elapsed: Duration.zero,
  );
}

class InstitutionDiscoveryService {
  InstitutionDiscoveryService({
    ChainRpc? chainRpc,
    InstitutionManageService? manageService,
  }) : _manage = manageService ?? InstitutionManageService(chainRpc: chainRpc);

  final InstitutionManageService _manage;

  /// 机构多签 org 白名单:PUP(4) / OTH(5)。
  static const _institutionOrgWhitelist = {4, 5};

  /// 处理一次共享扫描结果:筛出我的机构多签 → 批量反查 SFID → upsert Isar + 孤儿校验。
  Future<DiscoveryStats> processScanned(
    AdminAccountsScanResult scan, {
    required Set<String> myPubkeys,
  }) async {
    final start = DateTime.now();

    final mine = AdminAccountsScanService.filterMine(
      scan,
      myPubkeysHex: myPubkeys,
      kind: AdminAccountStorageCodec.kindInstitutionAccount,
      orgWhitelist: _institutionOrgWhitelist,
    );

    // 批量反查 SFID 归属(AccountRegisteredSfid 精确整键),取代循环内逐条(ADR-018 R2)。
    Map<String, RegisteredInstitutionRef?> refs;
    try {
      refs = await _manage.fetchRegisteredInstitutionRefsBatch(
        mine.map((a) => a.addrHex),
      );
    } catch (e) {
      debugPrint('[DuoqianDiscovery] 批量反查 SFID 失败: $e');
      // 中文注释:反查整体失败时不做孤儿删除,避免把一次瞬时 RPC 失败误判为账户注销。
      return DiscoveryStats(
        institutionsScanned: scan.totalKeys,
        matchedSfidAccounts: 0,
        newlyAdded: 0,
        orphansRemoved: 0,
        elapsed: DateTime.now().difference(start),
        partialFailure: true,
      );
    }

    final scannedDuoqianAddrs = <String>{};
    var matchedSfidAccountsCount = 0;
    var newlyAdded = 0;

    for (final acc in mine) {
      final ref = refs[acc.addrHex];
      if (ref == null) continue;

      scannedDuoqianAddrs.add(acc.addrHex);
      final added = await _upsertInstitution(
        duoqianAccountHex: acc.addrHex,
        name: ref.accountNameText,
        sfidNumberUtf8: ref.sfidNumberText,
        adminAccountOrg: acc.org,
        matchedAdmins: acc.adminPubkeysHex
            .where(myPubkeys.contains)
            .toList(growable: false),
      );
      if (added) newlyAdded++;
      matchedSfidAccountsCount++;
    }

    final orphans = await _reverseValidateAndDelete(scannedDuoqianAddrs);

    return DiscoveryStats(
      institutionsScanned: scan.totalKeys,
      matchedSfidAccounts: matchedSfidAccountsCount,
      newlyAdded: newlyAdded,
      orphansRemoved: orphans,
      elapsed: DateTime.now().difference(start),
      partialFailure: scan.partialFailure,
    );
  }

  Future<bool> _upsertInstitution({
    required String duoqianAccountHex,
    required String name,
    required String sfidNumberUtf8,
    required int? adminAccountOrg,
    required List<String> matchedAdmins,
  }) async {
    return WalletIsar.instance.writeTxn((isar) async {
      final exists = await isar.institutionEntitys
          .filter()
          .duoqianAccountEqualTo(duoqianAccountHex)
          .findFirst();

      if (exists != null) {
        if (!exists.discoveredViaAdmin) return false;
        exists.adminAccountOrg = adminAccountOrg;
        exists.matchedAdminPubkeys = matchedAdmins;
        await isar.institutionEntitys.put(exists);
        return false;
      }

      final entity = InstitutionEntity()
        ..duoqianAccount = duoqianAccountHex
        ..sfidNumber = sfidNumberUtf8
        ..adminAccountOrg = adminAccountOrg
        ..name = name
        ..addedAtMillis = DateTime.now().millisecondsSinceEpoch
        ..discoveredViaAdmin = true
        ..matchedAdminPubkeys = matchedAdmins;
      await isar.institutionEntitys.put(entity);
      return true;
    });
  }

  /// 反向校验:删除 Isar 中 discoveredViaAdmin=true 但本次扫描未命中的机构 entity。
  /// 用户 discoveredViaAdmin=false 的 entity(本机创建)永不被删除。
  Future<int> _reverseValidateAndDelete(Set<String> scannedAccounts) async {
    var orphans = 0;

    await WalletIsar.instance.writeTxn((isar) async {
      final staleInstitutions = await isar.institutionEntitys
          .filter()
          .discoveredViaAdminEqualTo(true)
          .findAll();
      for (final i in staleInstitutions) {
        if (!scannedAccounts.contains(i.duoqianAccount)) {
          await isar.institutionEntitys.delete(i.id);
          orphans++;
        }
      }
    });

    return orphans;
  }
}
