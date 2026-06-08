// 机构多签反向索引发现服务(req 3 核心)。
//
// 利用链上 `AdminsChange::AdminAccounts` 统一存储
// + smoldot 标准 `state_getKeysPaged` 扫描管理员反向索引。
//
// 本文件只处理机构多签 AdminAccountKind=InstitutionAccount。个人多签发现
// 已迁移到 `lib/personal-manage/personal_manage_discovery_service.dart`。

import 'package:flutter/foundation.dart';
import 'package:isar/isar.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'package:wuminapp_mobile/governance/shared/admin_account_storage_codec.dart';
import 'institution_manage_service.dart';
import 'duoqian_storage_codec.dart';

/// 机构多签反向索引扫描统计。
class DiscoveryStats {
  const DiscoveryStats({
    required this.institutionsScanned,
    required this.matchedPersonals,
    required this.matchedSfidAccounts,
    required this.newlyAdded,
    required this.orphansRemoved,
    required this.elapsed,
    this.partialFailure = false,
    this.errorMessage,
  });

  final int institutionsScanned;

  /// 中文注释：兼容旧测试/调用方字段；机构发现服务不再处理个人多签。
  final int matchedPersonals;
  final int matchedSfidAccounts;
  final int newlyAdded;
  final int orphansRemoved;
  final Duration elapsed;
  final bool partialFailure;
  final String? errorMessage;

  static const empty = DiscoveryStats(
    institutionsScanned: 0,
    matchedPersonals: 0,
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
    WalletManager? walletManager,
  })  : _rpc = chainRpc ?? ChainRpc(),
        _manage = manageService ?? InstitutionManageService(chainRpc: chainRpc),
        _wallets = walletManager ?? WalletManager();

  final ChainRpc _rpc;
  final InstitutionManageService _manage;
  final WalletManager _wallets;

  /// 30 分钟节流窗口。
  static const _throttleWindow = Duration(minutes: 30);

  /// SharedPreferences key:最近一次成功扫描的 epochMs。
  static const _prefsLastDiscoveryAt = 'duoqian_discovery_last_at_ms';

  /// 翻页大小。
  static const _pageSize = 256;

  /// 批量 storage 读取一次最多 100 keys(防 RPC 超时)。
  static const _batchSize = 100;

  /// 全量扫描 + 反向校验,把命中的机构多签 upsert 到 Isar。
  ///
  /// [myPubkeysHex] 若 null 自动从 WalletManager 取本地全部钱包公钥(小写 hex,无 0x)。
  /// [force] true 跳过 30 分钟节流。
  /// [onProgress] 进度回调:(已扫描页数, 总条数估值若已知, 命中数)。
  Future<DiscoveryStats> discoverByMyWallets({
    Set<String>? myPubkeysHex,
    bool force = false,
    void Function(int scanned, int? total, int matched)? onProgress,
  }) async {
    final start = DateTime.now();

    if (!force) {
      final last = await _readLastDiscoveryAt();
      if (last != null && DateTime.now().difference(last) < _throttleWindow) {
        return DiscoveryStats.empty;
      }
    }

    final myPubkeys = myPubkeysHex ?? await _readMyPubkeys();
    if (myPubkeys.isEmpty) return DiscoveryStats.empty;

    final prefixHex = _adminsChangeAdminAccountsPrefixHex();
    final allKeys = <String>[];
    String? startKey;
    var partialFailure = false;

    while (true) {
      List<dynamic>? page;
      try {
        page = await SmoldotClientManager.instance.request(
          'state_getKeysPaged',
          [prefixHex, _pageSize, startKey, null],
        ) as List<dynamic>?;
      } catch (e) {
        debugPrint('[DuoqianDiscovery] getKeysPaged 失败: $e');
        partialFailure = true;
        break;
      }
      if (page == null || page.isEmpty) break;
      final keys = page.cast<String>();
      allKeys.addAll(keys);
      onProgress?.call(allKeys.length, null, 0);
      if (keys.length < _pageSize) break;
      startKey = keys.last;
    }

    final matchedInstitutionAddrs = <String, List<String>>{};
    final matchedInstitutionOrgs = <String, int>{};
    var matchedCount = 0;

    for (var batchStart = 0;
        batchStart < allKeys.length;
        batchStart += _batchSize) {
      final batchEnd = (batchStart + _batchSize).clamp(0, allKeys.length);
      final batchKeys = allKeys.sublist(batchStart, batchEnd);

      Map<String, Uint8List?> values;
      try {
        values = await _rpc.fetchStorageBatch(batchKeys);
      } catch (e) {
        debugPrint('[DuoqianDiscovery] fetchStorageBatch 失败: $e');
        partialFailure = true;
        continue;
      }

      for (final keyHex in batchKeys) {
        final value = values[keyHex];
        if (value == null) continue;
        final decoded = AdminAccountStorageCodec.tryDecode(value);
        if (decoded == null ||
            decoded.kind != AdminAccountStorageCodec.kindInstitutionAccount ||
            (decoded.org != 4 && decoded.org != 5)) {
          continue;
        }

        final hits = decoded.adminPubkeysHex
            .where((pk) => myPubkeys.contains(pk))
            .toList();
        if (hits.isEmpty) continue;

        final keyBytes = _hexDecode(keyHex);
        final accountId =
            AdminAccountStorageCodec.extractAccountIdFromKey(keyBytes);
        if (accountId == null) continue;
        final addr =
            AdminAccountStorageCodec.accountHexFromAccountId(accountId);
        if (addr == null) continue;
        matchedInstitutionAddrs[addr] = hits;
        matchedInstitutionOrgs[addr] = decoded.org;
        matchedCount++;
      }
      onProgress?.call(allKeys.length, allKeys.length, matchedCount);
    }

    final scannedDuoqianAddrs = <String>{};
    var matchedSfidAccountsCount = 0;
    var newlyAdded = 0;

    for (final entry in matchedInstitutionAddrs.entries) {
      final duoqianAddrHex = entry.key;
      final hits = entry.value;

      RegisteredInstitutionRef? ref;
      try {
        ref = await _manage.fetchRegisteredInstitutionRef(duoqianAddrHex);
      } catch (e) {
        debugPrint(
          '[DuoqianDiscovery] fetchRegisteredInstitutionRef $duoqianAddrHex 失败: $e',
        );
        partialFailure = true;
        continue;
      }
      if (ref == null) continue;

      scannedDuoqianAddrs.add(duoqianAddrHex);
      final added = await _upsertInstitution(
        duoqianAddrHex: duoqianAddrHex,
        name: ref.accountNameText,
        sfidNumberUtf8: ref.sfidNumberText,
        adminAccountOrg: matchedInstitutionOrgs[duoqianAddrHex],
        matchedAdmins: hits,
      );
      if (added) newlyAdded++;
      matchedSfidAccountsCount++;
    }

    final orphans = await _reverseValidateAndDelete(scannedDuoqianAddrs);
    await _writeLastDiscoveryAt(DateTime.now());

    return DiscoveryStats(
      institutionsScanned: allKeys.length,
      matchedPersonals: 0,
      matchedSfidAccounts: matchedSfidAccountsCount,
      newlyAdded: newlyAdded,
      orphansRemoved: orphans,
      elapsed: DateTime.now().difference(start),
      partialFailure: partialFailure,
    );
  }

  /// 上次成功扫描时间(SharedPreferences 持久化)。
  Future<DateTime?> lastDiscoveryAt() => _readLastDiscoveryAt();

  Future<bool> _upsertInstitution({
    required String duoqianAddrHex,
    required String name,
    required String sfidNumberUtf8,
    required int? adminAccountOrg,
    required List<String> matchedAdmins,
  }) async {
    return WalletIsar.instance.writeTxn((isar) async {
      final exists = await isar.institutionEntitys
          .filter()
          .duoqianAddressEqualTo(duoqianAddrHex)
          .findFirst();

      if (exists != null) {
        if (!exists.discoveredViaAdmin) return false;
        exists.adminAccountOrg = adminAccountOrg;
        exists.matchedAdminPubkeys = matchedAdmins;
        await isar.institutionEntitys.put(exists);
        return false;
      }

      final entity = InstitutionEntity()
        ..duoqianAddress = duoqianAddrHex
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
  Future<int> _reverseValidateAndDelete(Set<String> scannedAddrs) async {
    var orphans = 0;

    await WalletIsar.instance.writeTxn((isar) async {
      final staleInstitutions = await isar.institutionEntitys
          .filter()
          .discoveredViaAdminEqualTo(true)
          .findAll();
      for (final i in staleInstitutions) {
        if (!scannedAddrs.contains(i.duoqianAddress)) {
          await isar.institutionEntitys.delete(i.id);
          orphans++;
        }
      }
    });

    return orphans;
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
      // 中文注释：节流时间写入失败不阻断机构发现结果。
    }
  }

  /// `AdminsChange::AdminAccounts` 双 prefix(twox128 || twox128)的 hex 形式。
  String _adminsChangeAdminAccountsPrefixHex() {
    final palletHash = Hasher.twoxx128.hashString('AdminsChange');
    final storageHash = Hasher.twoxx128.hashString('AdminAccounts');
    final prefix = Uint8List(palletHash.length + storageHash.length);
    prefix.setAll(0, palletHash);
    prefix.setAll(palletHash.length, storageHash);
    return '0x${_hexEncode(prefix)}';
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final bytes = Uint8List(h.length ~/ 2);
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return bytes;
  }

  String _hexEncode(Uint8List bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
