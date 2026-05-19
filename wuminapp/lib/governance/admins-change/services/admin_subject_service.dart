import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:isar/isar.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/governance/admins-change/codec/admin_subject_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class AdminSubjectService {
  AdminSubjectService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc(),
        _usePersistedCache =
            chainRpc == null || chainRpc.runtimeType == ChainRpc;

  static const Duration _cacheTtl = Duration(seconds: 30);
  static const Duration _persistedTtl = Duration(minutes: 10);
  static const String _persistedPrefix = 'governance.admin_subject.';
  static final Map<String, _AdminSubjectCacheEntry> _cache = {};
  static final Map<String, Future<AdminSubjectState?>> _inFlight = {};
  static final Set<String> _persistedBypassKeys = {};
  static bool _persistedBypassAll = false;
  static int _cacheGeneration = 0;

  final ChainRpc _rpc;
  final bool _usePersistedCache;

  Future<AdminSubjectState?> fetchByIdentity(
      AdminSubjectIdentity identity) async {
    return fetchBySubjectId(AdminSubjectIdCodec.fromHex(identity.subjectIdHex));
  }

  Future<AdminSubjectState?> fetchBySubjectId(Uint8List subjectId) async {
    final subjectKey = AdminSubjectIdCodec.hexEncode(subjectId);
    final cached = _cache[subjectKey];
    if (cached != null && cached.isFresh(_cacheTtl)) return cached.state;
    final persisted =
        _usePersistedCache ? await _readPersisted(subjectKey) : null;
    if (persisted != null && persisted.isFresh(_persistedTtl)) {
      _cache[subjectKey] = _AdminSubjectCacheEntry(persisted.state);
      return persisted.state;
    }
    final inFlight = _inFlight[subjectKey];
    if (inFlight != null) return inFlight;

    final generation = _cacheGeneration;
    final future = _fetchBySubjectIdUncached(
      subjectId,
      subjectKey,
      generation,
      fallback: persisted?.state,
    );
    _inFlight[subjectKey] = future;
    return future.whenComplete(() {
      if (_inFlight[subjectKey] == future) {
        _inFlight.remove(subjectKey);
      }
    });
  }

  Future<AdminSubjectState?> _fetchBySubjectIdUncached(
    Uint8List subjectId,
    String subjectKey,
    int generation, {
    AdminSubjectState? fallback,
  }) async {
    try {
      final key = AdminSubjectIdCodec.adminSubjectStorageKey(subjectId);
      final data =
          await _rpc.fetchStorage('0x${AdminSubjectIdCodec.hexEncode(key)}');
      if (data == null) return null;
      final decoded = AdminSubjectCodec.decode(subjectId, data);
      final threshold =
          decoded == null ? null : await _resolveThreshold(decoded, subjectId);
      final state = decoded?.copyWith(threshold: threshold ?? 0);
      // 中文注释：管理员主体属于链上动态数据；内存缓存挡住页面短时间重复进入，
      // AppKv 持久化快照则保障重启后首屏不必同步等待链上 storage。
      if (state != null && generation == _cacheGeneration) {
        _cache[subjectKey] = _AdminSubjectCacheEntry(state);
        if (_usePersistedCache) {
          unawaited(_writePersisted(subjectKey, state));
        }
      }
      return state;
    } catch (_) {
      if (fallback != null) {
        _cache[subjectKey] = _AdminSubjectCacheEntry(fallback);
        return fallback;
      }
      rethrow;
    }
  }

  Future<List<String>> fetchAdmins(AdminSubjectIdentity identity) async {
    return (await fetchByIdentity(identity))?.admins ?? const [];
  }

  Future<int?> fetchThreshold(AdminSubjectIdentity identity) async {
    return (await fetchByIdentity(identity))?.threshold;
  }

  Future<bool> isAdmin(String pubkeyHex, AdminSubjectIdentity identity) async {
    final clean = AdminSubjectIdCodec.normalizeHex(pubkeyHex);
    final admins = await fetchAdmins(identity);
    return admins.contains(clean);
  }

  void clearCache([AdminSubjectIdentity? identity]) {
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

  void clearIdentityCache(AdminSubjectIdentity identity) {
    _cacheGeneration++;
    final key = AdminSubjectIdCodec.normalizeHex(identity.subjectIdHex);
    _cache.remove(key);
    _inFlight.remove(key);
    _persistedBypassKeys.add(key);
    unawaited(_clearPersisted(key));
  }

  void clearSubjectCache(String subjectIdHex) {
    _cacheGeneration++;
    final key = AdminSubjectIdCodec.normalizeHex(subjectIdHex);
    _cache.remove(key);
    _inFlight.remove(key);
    _persistedBypassKeys.add(key);
    unawaited(_clearPersisted(key));
  }

  Future<_PersistedAdminSubject?> _readPersisted(String subjectKey) async {
    final key = AdminSubjectIdCodec.normalizeHex(subjectKey);
    if (_persistedBypassAll || _persistedBypassKeys.contains(key)) {
      return null;
    }
    try {
      return WalletIsar.instance.read((isar) async {
        final entity =
            await isar.appKvEntitys.getByKey(_persistedKey(subjectKey));
        return _PersistedAdminSubject.fromJsonString(entity?.stringValue);
      });
    } catch (_) {
      return null;
    }
  }

  Future<void> _writePersisted(
    String subjectKey,
    AdminSubjectState state,
  ) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final persisted = _PersistedAdminSubject(
      state: state,
      updatedAtMillis: now,
    );
    try {
      await WalletIsar.instance.writeTxn((isar) async {
        final entity =
            await isar.appKvEntitys.getByKey(_persistedKey(subjectKey)) ??
                AppKvEntity();
        entity
          ..key = _persistedKey(subjectKey)
          ..stringValue = jsonEncode(persisted.toJson())
          ..intValue = now
          ..boolValue = state.isActive;
        await isar.appKvEntitys.putByKey(entity);
      });
      _persistedBypassKeys.remove(AdminSubjectIdCodec.normalizeHex(subjectKey));
    } catch (_) {
      // 中文注释：管理员主体持久化只是展示加速，写入失败不能阻断链上结果返回。
    }
  }

  Future<void> _clearPersisted([String? subjectKey]) async {
    try {
      await WalletIsar.instance.writeTxn((isar) async {
        if (subjectKey != null) {
          await isar.appKvEntitys
              .where()
              .keyEqualTo(_persistedKey(subjectKey))
              .deleteAll();
          _persistedBypassKeys.remove(
            AdminSubjectIdCodec.normalizeHex(subjectKey),
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

  static String _persistedKey(String subjectKey) =>
      '$_persistedPrefix${AdminSubjectIdCodec.normalizeHex(subjectKey)}';

  Future<int?> _resolveThreshold(
    AdminSubjectState state,
    Uint8List subjectId,
  ) async {
    final fixed = _fixedGovernanceThreshold(state.org);
    if (fixed != null) return fixed;
    final active = await _fetchDynamicThreshold(
      storageName: 'ActiveDynamicThresholds',
      org: state.org,
      subjectId: subjectId,
    );
    if (active != null) return active;
    return _fetchDynamicThreshold(
      storageName: 'PendingDynamicThresholds',
      org: state.org,
      subjectId: subjectId,
    );
  }

  Future<int?> _fetchDynamicThreshold({
    required String storageName,
    required int org,
    required Uint8List subjectId,
  }) async {
    final key = _internalVoteDoubleMapKey(
      storageName: storageName,
      org: org,
      subjectId: subjectId,
    );
    final data =
        await _rpc.fetchStorage('0x${AdminSubjectIdCodec.hexEncode(key)}');
    if (data == null || data.length < 4) return null;
    return ByteData.sublistView(data).getUint32(0, Endian.little);
  }

  Uint8List _internalVoteDoubleMapKey({
    required String storageName,
    required int org,
    required Uint8List subjectId,
  }) {
    final palletHash = Hasher.twoxx128.hashString('InternalVote');
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final orgKey = _blake2128Concat(Uint8List.fromList([org]));
    final subjectKey = _blake2128Concat(subjectId);
    final key = Uint8List(
      palletHash.length +
          storageHash.length +
          orgKey.length +
          subjectKey.length,
    );
    var offset = 0;
    key.setAll(offset, palletHash);
    offset += palletHash.length;
    key.setAll(offset, storageHash);
    offset += storageHash.length;
    key.setAll(offset, orgKey);
    offset += orgKey.length;
    key.setAll(offset, subjectKey);
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

class _AdminSubjectCacheEntry {
  _AdminSubjectCacheEntry(this.state) : createdAt = DateTime.now();

  final AdminSubjectState state;
  final DateTime createdAt;

  bool isFresh(Duration ttl) => DateTime.now().difference(createdAt) < ttl;
}

class _PersistedAdminSubject {
  const _PersistedAdminSubject({
    required this.state,
    required this.updatedAtMillis,
  });

  final AdminSubjectState state;
  final int updatedAtMillis;

  bool isFresh(Duration ttl) {
    return DateTime.now().millisecondsSinceEpoch - updatedAtMillis <
        ttl.inMilliseconds;
  }

  Map<String, Object?> toJson() => {
        'updated_at_millis': updatedAtMillis,
        'state': {
          'subject_id_hex': state.subjectIdHex,
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

  static _PersistedAdminSubject? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final stateRaw = decoded['state'];
      if (stateRaw is! Map<String, dynamic>) return null;
      final subjectIdHex = stateRaw['subject_id_hex']?.toString();
      final org = _toInt(stateRaw['org']);
      final kind = _toInt(stateRaw['kind']);
      final threshold = _toInt(stateRaw['threshold']);
      final createdAt = _toInt(stateRaw['created_at']);
      final updatedAt = _toInt(stateRaw['updated_at']);
      final status = _toInt(stateRaw['status']);
      final updatedAtMillis = _toInt(decoded['updated_at_millis']);
      if (subjectIdHex == null ||
          org == null ||
          kind == null ||
          threshold == null ||
          createdAt == null ||
          updatedAt == null ||
          status == null ||
          updatedAtMillis == null) {
        return null;
      }
      return _PersistedAdminSubject(
        updatedAtMillis: updatedAtMillis,
        state: AdminSubjectState(
          subjectIdHex: AdminSubjectIdCodec.normalizeHex(subjectIdHex),
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
        .map((item) => AdminSubjectIdCodec.normalizeHex(item.toString()))
        .where((item) => item.isNotEmpty)
        .toList(growable: false);
  }
}
