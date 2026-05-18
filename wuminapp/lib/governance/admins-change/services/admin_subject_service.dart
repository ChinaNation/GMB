import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/governance/admins-change/codec/admin_subject_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class AdminSubjectService {
  AdminSubjectService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  static const Duration _cacheTtl = Duration(seconds: 30);
  static final Map<String, _AdminSubjectCacheEntry> _cache = {};
  static final Map<String, Future<AdminSubjectState?>> _inFlight = {};
  static int _cacheGeneration = 0;

  final ChainRpc _rpc;

  Future<AdminSubjectState?> fetchByIdentity(
      AdminSubjectIdentity identity) async {
    return fetchBySubjectId(AdminSubjectIdCodec.fromHex(identity.subjectIdHex));
  }

  Future<AdminSubjectState?> fetchBySubjectId(Uint8List subjectId) async {
    final subjectKey = AdminSubjectIdCodec.hexEncode(subjectId);
    final cached = _cache[subjectKey];
    if (cached != null && cached.isFresh(_cacheTtl)) return cached.state;
    final inFlight = _inFlight[subjectKey];
    if (inFlight != null) return inFlight;

    final generation = _cacheGeneration;
    final future = _fetchBySubjectIdUncached(
      subjectId,
      subjectKey,
      generation,
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
    int generation,
  ) async {
    final key = AdminSubjectIdCodec.adminSubjectStorageKey(subjectId);
    final data =
        await _rpc.fetchStorage('0x${AdminSubjectIdCodec.hexEncode(key)}');
    if (data == null) return null;
    final decoded = AdminSubjectCodec.decode(subjectId, data);
    final threshold =
        decoded == null ? null : await _resolveThreshold(decoded, subjectId);
    final state = decoded?.copyWith(threshold: threshold ?? 0);
    // 中文注释：管理员主体属于链上动态数据，短缓存只用于避免页面间反复进入时重复 RPC。
    if (state != null && generation == _cacheGeneration) {
      _cache[subjectKey] = _AdminSubjectCacheEntry(state);
    }
    return state;
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
    } else {
      clearIdentityCache(identity);
    }
  }

  void clearIdentityCache(AdminSubjectIdentity identity) {
    _cacheGeneration++;
    final key = AdminSubjectIdCodec.normalizeHex(identity.subjectIdHex);
    _cache.remove(key);
    _inFlight.remove(key);
  }

  void clearSubjectCache(String subjectIdHex) {
    _cacheGeneration++;
    final key = AdminSubjectIdCodec.normalizeHex(subjectIdHex);
    _cache.remove(key);
    _inFlight.remove(key);
  }

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
