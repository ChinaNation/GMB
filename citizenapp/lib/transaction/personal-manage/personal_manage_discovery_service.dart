// 个人多签反向索引发现:后处理(ADR-018 §九)。
//
// 从 AdminAccounts 单次扫描结果(AdminAccountsScanService)里筛出个人多签
// (kind=Personal,且管理员含本地钱包),反查发起人 / 账户名后 upsert
// `PersonalAccountEntity`。机构账户登记与展示不走本服务。

import 'package:flutter/foundation.dart';
import 'package:isar_community/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/citizen/shared/admin_account_storage_codec.dart';
import 'package:citizenapp/citizen/shared/admin_accounts_scan_service.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

import 'personal_manage_service.dart';

/// 个人多签后处理统计。
class PersonalManageDiscoveryStats {
  const PersonalManageDiscoveryStats({
    required this.subjectsScanned,
    required this.matchedPersonals,
    required this.newlyAdded,
    required this.orphansRemoved,
    required this.elapsed,
    this.partialFailure = false,
  });

  final int subjectsScanned;
  final int matchedPersonals;
  final int newlyAdded;
  final int orphansRemoved;
  final Duration elapsed;
  final bool partialFailure;

  static const empty = PersonalManageDiscoveryStats(
    subjectsScanned: 0,
    matchedPersonals: 0,
    newlyAdded: 0,
    orphansRemoved: 0,
    elapsed: Duration.zero,
  );
}

class PersonalManageDiscoveryService {
  PersonalManageDiscoveryService({
    ChainRpc? chainRpc,
    PersonalManageService? personalManageService,
  }) : _personalManage =
            personalManageService ?? PersonalManageService(chainRpc: chainRpc);

  final PersonalManageService _personalManage;

  /// 处理一次共享扫描结果:筛出我的个人多签 → 批量反查发起人/账户名 → upsert Isar + 孤儿校验。
  Future<PersonalManageDiscoveryStats> processScanned(
    AdminAccountsScanResult scan, {
    required Set<String> myPubkeys,
  }) async {
    final start = DateTime.now();

    final mine = AdminAccountsScanService.filterMine(
      scan,
      myPubkeysHex: myPubkeys,
      kind: AdminAccountStorageCodec.kindPersonal,
      codeWhitelist: const {'PMUL'},
    );

    // 批量反查发起人/账户名(PersonalAccounts 精确整键)。
    Map<String, ({String creatorAccountHex, String accountName})?> metas;
    try {
      metas = await _personalManage.fetchPersonalMetasBatch(
        mine.map((a) => a.addrHex),
      );
    } catch (e) {
      debugPrint('[PersonalManageDiscovery] 批量反查个人多签元数据失败: $e');
      // 反查整体失败时不做孤儿状态变更,避免把瞬时 RPC 失败误判为注销。
      return PersonalManageDiscoveryStats(
        subjectsScanned: scan.totalKeys,
        matchedPersonals: mine.length,
        newlyAdded: 0,
        orphansRemoved: 0,
        elapsed: DateTime.now().difference(start),
        partialFailure: true,
      );
    }

    final scannedAccounts = <String>{};
    var newlyAdded = 0;

    for (final acc in mine) {
      final meta = metas[acc.addrHex];
      if (meta == null) continue;
      scannedAccounts.add(acc.addrHex);
      final added = await _upsertPersonal(
        accountHex: acc.addrHex,
        name: meta.accountName,
        creatorAccountHex: meta.creatorAccountHex,
        matchedAdmins:
            acc.adminsHex.where(myPubkeys.contains).toList(growable: false),
      );
      if (added) newlyAdded++;
    }

    final orphans = await _reverseValidateAndDelete(scannedAccounts);

    return PersonalManageDiscoveryStats(
      subjectsScanned: scan.totalKeys,
      matchedPersonals: mine.length,
      newlyAdded: newlyAdded,
      orphansRemoved: orphans,
      elapsed: DateTime.now().difference(start),
      partialFailure: scan.partialFailure,
    );
  }

  Future<bool> _upsertPersonal({
    required String accountHex,
    required String name,
    required String creatorAccountHex,
    required List<String> matchedAdmins,
  }) async {
    String creatorSs58;
    try {
      creatorSs58 = Keyring().encodeAddress(
          Uint8List.fromList(_hexDecode(creatorAccountHex)), 2027);
    } catch (_) {
      creatorSs58 = '';
    }

    return WalletIsar.instance.writeTxn((isar) async {
      final exists = await isar.personalAccountEntitys
          .filter()
          .accountEqualTo(accountHex)
          .findFirst();

      if (exists != null) {
        if (!exists.discoveredViaAdmin) return false;
        exists.matchedAdminPubkeys = matchedAdmins;
        await isar.personalAccountEntitys.put(exists);
        await PersonalMultisigLocalState.putStatusInTxn(
          isar,
          accountHex,
          PersonalMultisigLocalState.statusActive,
        );
        return false;
      }

      final entity = PersonalAccountEntity()
        ..account = accountHex
        ..accountName = name
        ..creatorAddress = creatorSs58
        ..addedAtMillis = DateTime.now().millisecondsSinceEpoch
        ..discoveredViaAdmin = true
        ..matchedAdminPubkeys = matchedAdmins;
      await isar.personalAccountEntitys.put(entity);
      await PersonalMultisigLocalState.putStatusInTxn(
        isar,
        accountHex,
        PersonalMultisigLocalState.statusActive,
      );
      return true;
    });
  }

  Future<int> _reverseValidateAndDelete(Set<String> scannedAccounts) async {
    var closed = 0;
    await WalletIsar.instance.writeTxn((isar) async {
      final stalePersonals = await isar.personalAccountEntitys
          .filter()
          .discoveredViaAdminEqualTo(true)
          .findAll();
      for (final p in stalePersonals) {
        if (!scannedAccounts.contains(p.account)) {
          // 链上注销后仍保留本地账户入口，只把状态标成已注销；
          // 用户在详情页点“删除”时才真正清空本机数据。
          await PersonalMultisigLocalState.putStatusInTxn(
            isar,
            p.account,
            PersonalMultisigLocalState.statusClosed,
          );
          closed++;
        }
      }
    });
    return closed;
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final bytes = Uint8List(h.length ~/ 2);
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return bytes;
  }
}
