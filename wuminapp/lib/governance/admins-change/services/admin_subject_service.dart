import 'dart:typed_data';

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
    final state = AdminSubjectCodec.decode(subjectId, data);
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
}
