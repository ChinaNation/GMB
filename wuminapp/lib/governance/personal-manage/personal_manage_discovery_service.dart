// 个人多签反向索引发现服务。
//
// 只处理 AdminsChange::Subjects 中的 PersonalDuoqian 主体，发现后写入
// PersonalDuoqianEntity。机构账户发现继续留在 organization-manage 目录。

import 'package:flutter/foundation.dart';
import 'package:isar/isar.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/common/admin_institution_codec.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

import 'personal_manage_service.dart';

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
    WalletManager? walletManager,
  })  : _rpc = chainRpc ?? ChainRpc(),
        _personalManage =
            personalManageService ?? PersonalManageService(chainRpc: chainRpc),
        _wallets = walletManager ?? WalletManager();

  final ChainRpc _rpc;
  final PersonalManageService _personalManage;
  final WalletManager _wallets;

  static const _throttleWindow = Duration(minutes: 30);
  static const _prefsLastDiscoveryAt = 'personal_manage_discovery_last_at_ms';
  static const _pageSize = 256;
  static const _batchSize = 100;

  Future<PersonalManageDiscoveryStats> discoverByMyWallets({
    Set<String>? myPubkeysHex,
    bool force = false,
    void Function(int scanned, int? total, int matched)? onProgress,
  }) async {
    final start = DateTime.now();

    if (!force) {
      final last = await _readLastDiscoveryAt();
      if (last != null && DateTime.now().difference(last) < _throttleWindow) {
        return PersonalManageDiscoveryStats.empty;
      }
    }

    final myPubkeys = myPubkeysHex ?? await _readMyPubkeys();
    if (myPubkeys.isEmpty) return PersonalManageDiscoveryStats.empty;

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
        debugPrint('[PersonalManageDiscovery] getKeysPaged 失败: $e');
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

    final matchedPersonalAddrs = <String, List<String>>{};
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
        debugPrint('[PersonalManageDiscovery] fetchStorageBatch 失败: $e');
        partialFailure = true;
        continue;
      }

      for (final keyHex in batchKeys) {
        final value = values[keyHex];
        if (value == null) continue;
        final decoded = AdminInstitutionCodec.tryDecode(value);
        if (decoded == null ||
            decoded.kind != AdminInstitutionCodec.kindPersonal) {
          continue;
        }

        final hits = decoded.adminPubkeysHex
            .where((pk) => myPubkeys.contains(pk))
            .toList();
        if (hits.isEmpty) continue;

        final keyBytes = _hexDecode(keyHex);
        final subjectId =
            AdminInstitutionCodec.extractInstitutionIdFromKey(keyBytes);
        if (subjectId == null) continue;
        final addr =
            AdminInstitutionCodec.personalAddressFromInstitutionId(subjectId);
        if (addr == null) continue;
        matchedPersonalAddrs[addr] = hits;
        matchedCount++;
      }
      onProgress?.call(allKeys.length, allKeys.length, matchedCount);
    }

    final isar = await WalletIsar.instance.db();
    final scannedAddrs = <String>{};
    var newlyAdded = 0;

    for (final entry in matchedPersonalAddrs.entries) {
      final addr = entry.key;
      final meta = await _safeFetchPersonalMeta(addr);
      if (meta == null) continue;
      scannedAddrs.add(addr);
      final added = await _upsertPersonal(
        isar: isar,
        duoqianAddrHex: addr,
        name: meta.accountName,
        creatorAddrHex: meta.creatorAddressHex,
        matchedAdmins: entry.value,
      );
      if (added) newlyAdded++;
    }

    final orphans = await _reverseValidateAndDelete(isar, scannedAddrs);
    await _writeLastDiscoveryAt(DateTime.now());

    return PersonalManageDiscoveryStats(
      subjectsScanned: allKeys.length,
      matchedPersonals: matchedPersonalAddrs.length,
      newlyAdded: newlyAdded,
      orphansRemoved: orphans,
      elapsed: DateTime.now().difference(start),
      partialFailure: partialFailure,
    );
  }

  Future<DateTime?> lastDiscoveryAt() => _readLastDiscoveryAt();

  Future<({String creatorAddressHex, String accountName})?>
      _safeFetchPersonalMeta(String addrHex) async {
    try {
      return await _personalManage.fetchPersonalMeta(addrHex);
    } catch (e) {
      debugPrint('[PersonalManageDiscovery] fetchPersonalMeta $addrHex 失败: $e');
      return null;
    }
  }

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
      if (!exists.discoveredViaAdmin) return false;
      await isar.writeTxn(() async {
        exists.matchedAdminPubkeys = matchedAdmins;
        await isar.personalDuoqianEntitys.put(exists);
      });
      return false;
    }

    String creatorSs58;
    try {
      creatorSs58 = Keyring()
          .encodeAddress(Uint8List.fromList(_hexDecode(creatorAddrHex)), 2027);
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
      // 中文注释：节流时间写入失败不影响本次发现结果。
    }
  }

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
}
