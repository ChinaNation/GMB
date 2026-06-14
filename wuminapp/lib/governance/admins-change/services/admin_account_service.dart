import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:isar_community/isar.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/governance/admins-change/codec/admin_account_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/account_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_account.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class AdminAccountService {
  AdminAccountService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc(),
        _usePersistedCache =
            chainRpc == null || chainRpc.runtimeType == ChainRpc;

  static const Duration _cacheTtl = Duration(seconds: 30);
  static const Duration _persistedTtl = Duration(minutes: 10);
  static const String _persistedPrefix = 'governance.admin_account.';
  static final Map<String, _AdminAccountCacheEntry> _cache = {};
  static final Map<String, Future<AdminAccountState?>> _inFlight = {};
  static final Set<String> _persistedBypassKeys = {};
  static bool _persistedBypassAll = false;
  static int _cacheGeneration = 0;

  final ChainRpc _rpc;
  final bool _usePersistedCache;

  Future<AdminAccountState?> fetchByIdentity(
      AdminAccountIdentity identity) async {
    return fetchByAccountId(AdminAccountIdCodec.fromHex(identity.accountHex));
  }

  Future<AdminAccountState?> fetchByAccountId(Uint8List accountId) async {
    final accountKey = AdminAccountIdCodec.hexEncode(accountId);
    final cached = _cache[accountKey];
    if (cached != null && cached.isFresh(_cacheTtl)) return cached.state;
    final persisted =
        _usePersistedCache ? await _readPersisted(accountKey) : null;
    if (persisted != null && persisted.isFresh(_persistedTtl)) {
      _cache[accountKey] = _AdminAccountCacheEntry(persisted.state);
      return persisted.state;
    }
    final inFlight = _inFlight[accountKey];
    if (inFlight != null) return inFlight;

    final generation = _cacheGeneration;
    final future = _fetchByAccountIdUncached(
      accountId,
      accountKey,
      generation,
      fallback: persisted?.state,
    );
    _inFlight[accountKey] = future;
    return future.whenComplete(() {
      if (_inFlight[accountKey] == future) {
        _inFlight.remove(accountKey);
      }
    });
  }

  Future<AdminAccountState?> _fetchByAccountIdUncached(
    Uint8List accountId,
    String accountKey,
    int generation, {
    AdminAccountState? fallback,
  }) async {
    try {
      final key = AdminAccountIdCodec.adminAccountStorageKey(accountId);
      final data =
          await _rpc.fetchStorage('0x${AdminAccountIdCodec.hexEncode(key)}');
      if (data == null) return null;
      final decoded = AdminAccountCodec.decode(accountId, data);
      final threshold =
          decoded == null ? null : await _resolveThreshold(decoded, accountId);
      final state = decoded?.copyWith(threshold: threshold ?? 0);
      // 中文注释：管理员账户属于链上动态数据；内存缓存挡住页面短时间重复进入，
      // AppKv 持久化快照则保障重启后首屏不必同步等待链上 storage。
      if (state != null && generation == _cacheGeneration) {
        _cache[accountKey] = _AdminAccountCacheEntry(state);
        if (_usePersistedCache) {
          unawaited(_writePersisted(accountKey, state));
        }
      }
      return state;
    } catch (_) {
      if (fallback != null) {
        _cache[accountKey] = _AdminAccountCacheEntry(fallback);
        return fallback;
      }
      rethrow;
    }
  }

  Future<List<String>> fetchAdmins(AdminAccountIdentity identity) async {
    return (await fetchByIdentity(identity))?.admins ?? const [];
  }

  Future<int?> fetchThreshold(AdminAccountIdentity identity) async {
    return (await fetchByIdentity(identity))?.threshold;
  }

  Future<bool> isAdmin(String pubkeyHex, AdminAccountIdentity identity) async {
    final clean = AdminAccountIdCodec.normalizeHex(pubkeyHex);
    final admins = await fetchAdmins(identity);
    return admins.contains(clean);
  }

  void clearCache([AdminAccountIdentity? identity]) {
    if (identity == null) {
      _cacheGeneration++;
      _cache.clear();
      _inFlight.clear();
      _persistedBypassAll = true;
      unawaited(_clearPersisted());
    } else {
      clearIdentityCache(identity);
    }
  }

  void clearIdentityCache(AdminAccountIdentity identity) {
    _cacheGeneration++;
    final key = AdminAccountIdCodec.normalizeHex(identity.accountHex);
    _cache.remove(key);
    _inFlight.remove(key);
    _persistedBypassKeys.add(key);
    unawaited(_clearPersisted(key));
  }

  void clearAccountCache(String accountHex) {
    _cacheGeneration++;
    final key = AdminAccountIdCodec.normalizeHex(accountHex);
    _cache.remove(key);
    _inFlight.remove(key);
    _persistedBypassKeys.add(key);
    unawaited(_clearPersisted(key));
  }

  Future<_PersistedAdminAccount?> _readPersisted(String accountKey) async {
    final key = AdminAccountIdCodec.normalizeHex(accountKey);
    if (_persistedBypassAll || _persistedBypassKeys.contains(key)) {
      return null;
    }
    try {
      return WalletIsar.instance.read((isar) async {
        final entity =
            await isar.appKvEntitys.getByKey(_persistedKey(accountKey));
        return _PersistedAdminAccount.fromJsonString(entity?.stringValue);
      });
    } catch (_) {
      return null;
    }
  }

  Future<void> _writePersisted(
    String accountKey,
    AdminAccountState state,
  ) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final persisted = _PersistedAdminAccount(
      state: state,
      updatedAtMillis: now,
    );
    try {
      await WalletIsar.instance.writeTxn((isar) async {
        final entity =
            await isar.appKvEntitys.getByKey(_persistedKey(accountKey)) ??
                AppKvEntity();
        entity
          ..key = _persistedKey(accountKey)
          ..stringValue = jsonEncode(persisted.toJson())
          ..intValue = now
          ..boolValue = state.isActive;
        await isar.appKvEntitys.putByKey(entity);
      });
      _persistedBypassKeys.remove(AdminAccountIdCodec.normalizeHex(accountKey));
    } catch (_) {
      // 中文注释：管理员账户持久化只是展示加速，写入失败不能阻断链上结果返回。
    }
  }

  Future<void> _clearPersisted([String? accountKey]) async {
    try {
      await WalletIsar.instance.writeTxn((isar) async {
        if (accountKey != null) {
          await isar.appKvEntitys
              .where()
              .keyEqualTo(_persistedKey(accountKey))
              .deleteAll();
          _persistedBypassKeys.remove(
            AdminAccountIdCodec.normalizeHex(accountKey),
          );
          return;
        }
        final rows = await isar.appKvEntitys
            .filter()
            .keyStartsWith(_persistedPrefix)
            .findAll();
        await isar.appKvEntitys.deleteAll(rows.map((row) => row.id).toList());
        _persistedBypassKeys.clear();
        _persistedBypassAll = false;
      });
    } catch (_) {
      // 本地缓存清理失败不影响链上提交路径。
    }
  }

  static String _persistedKey(String accountKey) =>
      '$_persistedPrefix${AdminAccountIdCodec.normalizeHex(accountKey)}';

  Future<int?> _resolveThreshold(
    AdminAccountState state,
    Uint8List accountId,
  ) async {
    final fixed = _fixedGovernanceThreshold(state.org);
    if (fixed != null) return fixed;
    final active = await _fetchDynamicThreshold(
      storageName: 'ActiveDynamicThresholds',
      org: state.org,
      accountId: accountId,
    );
    if (active != null) return active;
    return _fetchDynamicThreshold(
      storageName: 'PendingDynamicThresholds',
      org: state.org,
      accountId: accountId,
    );
  }

  Future<int?> _fetchDynamicThreshold({
    required String storageName,
    required int org,
    required Uint8List accountId,
  }) async {
    final key = _internalVoteDoubleMapKey(
      storageName: storageName,
      org: org,
      accountId: accountId,
    );
    final data =
        await _rpc.fetchStorage('0x${AdminAccountIdCodec.hexEncode(key)}');
    if (data == null || data.length < 4) return null;
    return ByteData.sublistView(data).getUint32(0, Endian.little);
  }

  Uint8List _internalVoteDoubleMapKey({
    required String storageName,
    required int org,
    required Uint8List accountId,
  }) {
    final palletHash = Hasher.twoxx128.hashString('InternalVote');
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final orgKey = _blake2128Concat(Uint8List.fromList([org]));
    final accountKey = _blake2128Concat(accountId);
    final key = Uint8List(
      palletHash.length +
          storageHash.length +
          orgKey.length +
          accountKey.length,
    );
    var offset = 0;
    key.setAll(offset, palletHash);
    offset += palletHash.length;
    key.setAll(offset, storageHash);
    offset += storageHash.length;
    key.setAll(offset, orgKey);
    offset += orgKey.length;
    key.setAll(offset, accountKey);
    return key;
  }

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  int? _fixedGovernanceThreshold(int org) {
    return switch (org) {
      0 => 13,
      1 || 2 => 6,
      _ => null,
    };
  }
}

class _AdminAccountCacheEntry {
  _AdminAccountCacheEntry(this.state) : createdAt = DateTime.now();

  final AdminAccountState state;
  final DateTime createdAt;

  bool isFresh(Duration ttl) => DateTime.now().difference(createdAt) < ttl;
}

class _PersistedAdminAccount {
  const _PersistedAdminAccount({
    required this.state,
    required this.updatedAtMillis,
  });

  final AdminAccountState state;
  final int updatedAtMillis;

  bool isFresh(Duration ttl) {
    return DateTime.now().millisecondsSinceEpoch - updatedAtMillis <
        ttl.inMilliseconds;
  }

  Map<String, Object?> toJson() => {
        'updated_at_millis': updatedAtMillis,
        'state': {
          'account_id_hex': state.accountHex,
          'org': state.org,
          'kind': state.kind,
          'admins': state.admins,
          'threshold': state.threshold,
          'creator_hex': state.creatorHex,
          'created_at': state.createdAt,
          'updated_at': state.updatedAt,
          'status': state.status,
        },
      };

  static _PersistedAdminAccount? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final stateRaw = decoded['state'];
      if (stateRaw is! Map<String, dynamic>) return null;
      final accountHex = stateRaw['account_id_hex']?.toString();
      final org = _toInt(stateRaw['org']);
      final kind = _toInt(stateRaw['kind']);
      final threshold = _toInt(stateRaw['threshold']);
      final createdAt = _toInt(stateRaw['created_at']);
      final updatedAt = _toInt(stateRaw['updated_at']);
      final status = _toInt(stateRaw['status']);
      final updatedAtMillis = _toInt(decoded['updated_at_millis']);
      if (accountHex == null ||
          org == null ||
          kind == null ||
          threshold == null ||
          createdAt == null ||
          updatedAt == null ||
          status == null ||
          updatedAtMillis == null) {
        return null;
      }
      return _PersistedAdminAccount(
        updatedAtMillis: updatedAtMillis,
        state: AdminAccountState(
          accountHex: AdminAccountIdCodec.normalizeHex(accountHex),
          org: org,
          kind: kind,
          admins: _stringList(stateRaw['admins']),
          threshold: threshold,
          creatorHex: stateRaw['creator_hex']?.toString() ?? '',
          createdAt: createdAt,
          updatedAt: updatedAt,
          status: status,
        ),
      );
    } catch (_) {
      return null;
    }
  }

  static int? _toInt(Object? value) {
    if (value == null) return null;
    if (value is int) return value;
    return int.tryParse(value.toString());
  }

  static List<String> _stringList(Object? value) {
    if (value is! List) return const [];
    return value
        .map((item) => AdminAccountIdCodec.normalizeHex(item.toString()))
        .where((item) => item.isNotEmpty)
        .toList(growable: false);
  }
}
