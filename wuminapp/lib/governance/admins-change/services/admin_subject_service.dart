import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:wuminapp_mobile/governance/admins-change/codec/admin_subject_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class AdminSubjectService {
  AdminSubjectService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;
  final Map<String, AdminSubjectState> _cache = {};

  Future<AdminSubjectState?> fetchByIdentity(
      AdminSubjectIdentity identity) async {
    return fetchBySubjectId(AdminSubjectIdCodec.fromHex(identity.subjectIdHex));
  }

  Future<AdminSubjectState?> fetchBySubjectId(Uint8List subjectId) async {
    final subjectKey = AdminSubjectIdCodec.hexEncode(subjectId);
    final cached = _cache[subjectKey];
    if (cached != null) return cached;
    final key = AdminSubjectIdCodec.adminSubjectStorageKey(subjectId);
    final data =
        await _rpc.fetchStorage('0x${AdminSubjectIdCodec.hexEncode(key)}');
    if (data == null) return null;
    final decoded = AdminSubjectCodec.decode(subjectId, data);
    final threshold =
        decoded == null ? null : await _resolveThreshold(decoded, subjectId);
    final state = decoded?.copyWith(threshold: threshold ?? 0);
    if (state != null) _cache[subjectKey] = state;
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
      _cache.clear();
    } else {
      clearIdentityCache(identity);
    }
  }

  void clearIdentityCache(AdminSubjectIdentity identity) {
    _cache.remove(AdminSubjectIdCodec.normalizeHex(identity.subjectIdHex));
  }

  void clearSubjectCache(String subjectIdHex) {
    _cache.remove(AdminSubjectIdCodec.normalizeHex(subjectIdHex));
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
