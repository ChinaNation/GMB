// 多签反向索引发现服务(req 3 核心)。
//
// **完全 0 链端改动**:利用现有 `AdminsChange::Subjects` 统一存储
// (个人/SFID 机构/内置治理三类多签共用)+ smoldot 标准 `state_getKeysPaged`
// (verified [smoldot-pow/lib/src/json_rpc/methods.rs:432])。
//
// 数据流:
// 1. state_getKeysPaged 翻页拿全部 institution_id keys
// 2. fetchStorageBatch 批量取 AdminInstitution SCALE bytes
// 3. AdminInstitutionCodec.tryDecode 拿 (org, kind, admins)
// 4. kind ∈ {1=Sfid, 2=Personal} 且 admins ∩ myPubkeys ≠ ∅ 则命中
// 5. 按 kind 反查详情:
//    - Personal:institution_id 前 32B = personal_address;查 PersonalDuoqianInfo
//    - Sfid:institution_id = sfid_number padded;翻页查 SfidRegisteredAddress 拿全部 account
// 6. upsert Isar(标 discoveredViaAdmin=true + matchedAdminPubkeys)
// 7. 反向校验:Isar 中 discoveredViaAdmin=true 但本次未命中的 entity 全部删除

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:isar/isar.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:shared_preferences/shared_preferences.dart';

import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'admin_institution_codec.dart';
import 'duoqian_manage_service.dart';

/// 反向索引扫描统计。
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

class DuoqianDiscoveryService {
  DuoqianDiscoveryService({
    ChainRpc? chainRpc,
    DuoqianManageService? manageService,
    WalletManager? walletManager,
  })  : _rpc = chainRpc ?? ChainRpc(),
        _manage = manageService ?? DuoqianManageService(),
        _wallets = walletManager ?? WalletManager();

  final ChainRpc _rpc;
  final DuoqianManageService _manage;
  final WalletManager _wallets;

  /// 30 分钟节流窗口。
  static const _throttleWindow = Duration(minutes: 30);

  /// SharedPreferences key:最近一次成功扫描的 epochMs。
  static const _prefsLastDiscoveryAt = 'duoqian_discovery_last_at_ms';

  /// 翻页大小。
  static const _pageSize = 256;

  /// 批量 storage 读取一次最多 100 keys(防 RPC 超时)。
  static const _batchSize = 100;

  /// 全量扫描 + 反向校验,把命中的多签 upsert 到 Isar。
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

    // 节流检查
    if (!force) {
      final last = await _readLastDiscoveryAt();
      if (last != null &&
          DateTime.now().difference(last) < _throttleWindow) {
        return DiscoveryStats.empty;
      }
    }

    // 准备 myPubkeys(小写无前缀)
    final myPubkeys = myPubkeysHex ?? await _readMyPubkeys();
    if (myPubkeys.isEmpty) return DiscoveryStats.empty;

    // ── Step 1+2: prefix iter + 批量读 AdminInstitution ──
    final prefixHex = _adminsChangeSubjectsPrefixHex();
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

    // ── Step 3: 批量取 value + 解码 + 过滤命中 ──
    final matchedPersonalAddrs = <String, List<String>>{};   // address → matched admin pubkeys
    final matchedSfidNumbers = <String, List<String>>{};         // sfid_number_hex → matched admin pubkeys
    var matchedCount = 0;

    for (var batchStart = 0; batchStart < allKeys.length; batchStart += _batchSize) {
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
        final decoded = AdminInstitutionCodec.tryDecode(value);
        if (decoded == null) continue;

        // 过滤:仅 Personal / Sfid;Builtin 排除
        if (decoded.kind != AdminInstitutionCodec.kindPersonal &&
            decoded.kind != AdminInstitutionCodec.kindSfid) {
          continue;
        }

        // admins ∩ myPubkeys
        final hits = decoded.adminPubkeysHex
            .where((pk) => myPubkeys.contains(pk))
            .toList();
        if (hits.isEmpty) continue;

        // 提取 institution_id
        final keyBytes = _hexDecode(keyHex);
        final instId =
            AdminInstitutionCodec.extractInstitutionIdFromKey(keyBytes);
        if (instId == null) continue;

        if (decoded.kind == AdminInstitutionCodec.kindPersonal) {
          final addr =
              AdminInstitutionCodec.personalAddressFromInstitutionId(instId);
          if (addr != null) {
            matchedPersonalAddrs[addr] = hits;
            matchedCount++;
          }
        } else {
          // SFID 机构
          final sfidNumber = AdminInstitutionCodec.sfidNumberFromInstitutionId(instId);
          if (sfidNumber != null) {
            matchedSfidNumbers[_hexEncode(sfidNumber)] = hits;
            matchedCount++;
          }
        }
      }
      onProgress?.call(allKeys.length, allKeys.length, matchedCount);
    }

    // ── Step 4+5: 按 kind 反查详情 + upsert Isar ──
    final scannedDuoqianAddrs = <String>{};
    var matchedSfidAccountsCount = 0;
    var newlyAdded = 0;

    final isar = await WalletIsar.instance.db();

    // Personal 反查
    for (final entry in matchedPersonalAddrs.entries) {
      final addr = entry.key;
      final hits = entry.value;
      final meta = await _safeFetchPersonalMeta(addr);
      if (meta == null) continue;
      scannedDuoqianAddrs.add(addr);

      final added = await _upsertPersonal(
        isar: isar,
        duoqianAddrHex: addr,
        name: meta.accountName,
        creatorAddrHex: meta.creatorAddressHex,
        matchedAdmins: hits,
      );
      if (added) newlyAdded++;
    }

    // SFID 机构反查
    for (final entry in matchedSfidNumbers.entries) {
      final sfidNumberHex = entry.key;
      final hits = entry.value;
      final sfidNumber = _hexDecode(sfidNumberHex);

      List<({String accountName, String duoqianAddressHex})> accounts;
      try {
        accounts = await _manage.listSfidAccounts(Uint8List.fromList(sfidNumber));
      } catch (e) {
        debugPrint('[DuoqianDiscovery] listSfidAccounts $sfidNumberHex 失败: $e');
        partialFailure = true;
        continue;
      }

      for (final acc in accounts) {
        scannedDuoqianAddrs.add(acc.duoqianAddressHex);
        final added = await _upsertInstitution(
          isar: isar,
          duoqianAddrHex: acc.duoqianAddressHex,
          name: acc.accountName,
          sfidNumberUtf8: _utf8FromBytes(Uint8List.fromList(sfidNumber)),
          matchedAdmins: hits,
        );
        if (added) newlyAdded++;
        matchedSfidAccountsCount++;
      }
    }

    // ── Step 6: 反向校验删除孤立 entity ──
    final orphans = await _reverseValidateAndDelete(isar, scannedDuoqianAddrs);

    // 节流时间戳更新(仅成功完成时)
    await _writeLastDiscoveryAt(DateTime.now());

    return DiscoveryStats(
      institutionsScanned: allKeys.length,
      matchedPersonals: matchedPersonalAddrs.length,
      matchedSfidAccounts: matchedSfidAccountsCount,
      newlyAdded: newlyAdded,
      orphansRemoved: orphans,
      elapsed: DateTime.now().difference(start),
      partialFailure: partialFailure,
    );
  }

  /// 上次成功扫描时间(SharedPreferences 持久化)。
  Future<DateTime?> lastDiscoveryAt() => _readLastDiscoveryAt();

  // ──── 内部 ─────────────────────────────────────────────

  Future<({String creatorAddressHex, String accountName})?>
      _safeFetchPersonalMeta(String addrHex) async {
    try {
      return await _manage.fetchPersonalMeta(addrHex);
    } catch (e) {
      debugPrint('[DuoqianDiscovery] fetchPersonalMeta $addrHex 失败: $e');
      return null;
    }
  }

  /// 返回 true = 新增 entity;false = 已存在(含本机创建的 discoveredViaAdmin=false)。
  /// 本机创建的 entity 不被覆盖,但若已是 discoveredViaAdmin=true 则更新 matchedAdminPubkeys。
  Future<bool> _upsertPersonal({
    required Isar isar,
    required String duoqianAddrHex,
    required String name,
    required String creatorAddrHex,
    required List<String> matchedAdmins,
  }) async {
    final exists = await isar.personalDuoqianEntitys
        .filter()
        .duoqianAddressEqualTo(duoqianAddrHex)
        .findFirst();

    if (exists != null) {
      // 本机创建的(discoveredViaAdmin=false)→ 不动,保持原状
      if (!exists.discoveredViaAdmin) return false;
      // 已是反向索引发现的 → 更新 matchedAdminPubkeys 快照
      await isar.writeTxn(() async {
        exists.matchedAdminPubkeys = matchedAdmins;
        await isar.personalDuoqianEntitys.put(exists);
      });
      return false;
    }

    // 反向索引发现新 entity
    String creatorSs58;
    try {
      creatorSs58 =
          Keyring().encodeAddress(Uint8List.fromList(_hexDecode(creatorAddrHex)), 2027);
    } catch (_) {
      creatorSs58 = '';
    }

    await isar.writeTxn(() async {
      final entity = PersonalDuoqianEntity()
        ..duoqianAddress = duoqianAddrHex
        ..name = name
        ..creatorAddress = creatorSs58
        ..addedAtMillis = DateTime.now().millisecondsSinceEpoch
        ..discoveredViaAdmin = true
        ..matchedAdminPubkeys = matchedAdmins;
      await isar.personalDuoqianEntitys.put(entity);
    });
    return true;
  }

  Future<bool> _upsertInstitution({
    required Isar isar,
    required String duoqianAddrHex,
    required String name,
    required String sfidNumberUtf8,
    required List<String> matchedAdmins,
  }) async {
    final exists = await isar.duoqianInstitutionEntitys
        .filter()
        .duoqianAddressEqualTo(duoqianAddrHex)
        .findFirst();

    if (exists != null) {
      if (!exists.discoveredViaAdmin) return false;
      await isar.writeTxn(() async {
        exists.matchedAdminPubkeys = matchedAdmins;
        await isar.duoqianInstitutionEntitys.put(exists);
      });
      return false;
    }

    await isar.writeTxn(() async {
      final entity = DuoqianInstitutionEntity()
        ..duoqianAddress = duoqianAddrHex
        ..sfidNumber = sfidNumberUtf8
        ..name = name
        ..addedAtMillis = DateTime.now().millisecondsSinceEpoch
        ..discoveredViaAdmin = true
        ..matchedAdminPubkeys = matchedAdmins;
      await isar.duoqianInstitutionEntitys.put(entity);
    });
    return true;
  }

  /// 反向校验:删除 Isar 中 discoveredViaAdmin=true 但本次扫描未命中的 entity。
  /// 用户 discoveredViaAdmin=false 的 entity(本机创建)永不被删除。
  Future<int> _reverseValidateAndDelete(
    Isar isar,
    Set<String> scannedAddrs,
  ) async {
    var orphans = 0;

    await isar.writeTxn(() async {
      final stalePersonals = await isar.personalDuoqianEntitys
          .filter()
          .discoveredViaAdminEqualTo(true)
          .findAll();
      for (final p in stalePersonals) {
        if (!scannedAddrs.contains(p.duoqianAddress)) {
          await isar.personalDuoqianEntitys.delete(p.id);
          orphans++;
        }
      }
      final staleInstitutions = await isar.duoqianInstitutionEntitys
          .filter()
          .discoveredViaAdminEqualTo(true)
          .findAll();
      for (final i in staleInstitutions) {
        if (!scannedAddrs.contains(i.duoqianAddress)) {
          await isar.duoqianInstitutionEntitys.delete(i.id);
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
    } catch (_) {/* 持久化失败不阻断主流程 */}
  }

  // ──── 编码工具 ────

  /// `AdminsChange::Subjects` 双 prefix(twox128 || twox128)的 hex 形式。
  /// C 阶段(命名修正,2026-05-06)起,storage 已从 Institutions 改名 Subjects。
  String _adminsChangeSubjectsPrefixHex() {
    final palletHash = Hasher.twoxx128.hashString('AdminsChange');
    final storageHash = Hasher.twoxx128.hashString('Subjects');
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

  String _utf8FromBytes(Uint8List bytes) {
    try {
      return String.fromCharCodes(bytes);
    } catch (_) {
      return '';
    }
  }
}
