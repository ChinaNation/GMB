import 'dart:typed_data';

import 'package:wuminapp_mobile/admins-change/codec/admin_subject_codec.dart';
import 'package:wuminapp_mobile/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class AdminSubjectService {
  AdminSubjectService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;
  final Map<String, AdminSubjectState> _cache = {};

  Future<AdminSubjectState?> fetchByInstitutionIdentity(String identity) async {
    final cached = _cache[identity];
    if (cached != null) return cached;
    final subjectId = _subjectIdFromInstitutionIdentity(identity);
    final state = await fetchBySubjectId(subjectId);
    if (state != null) _cache[identity] = state;
    return state;
  }

  Future<AdminSubjectState?> fetchBySubjectId(Uint8List subjectId) async {
    final key = AdminSubjectIdCodec.adminSubjectStorageKey(subjectId);
    final data =
        await _rpc.fetchStorage('0x${AdminSubjectIdCodec.hexEncode(key)}');
    if (data == null) return null;
    return AdminSubjectCodec.decode(subjectId, data);
  }

  Future<List<String>> fetchAdmins(String identity) async {
    return (await fetchByInstitutionIdentity(identity))?.admins ?? const [];
  }

  Future<int?> fetchThreshold(String identity) async {
    return (await fetchByInstitutionIdentity(identity))?.threshold;
  }

  Future<bool> isAdmin(String pubkeyHex, String identity) async {
    final clean = AdminSubjectIdCodec.normalizeHex(pubkeyHex);
    final admins = await fetchAdmins(identity);
    return admins.contains(clean);
  }

  void clearCache([String? identity]) {
    if (identity == null) {
      _cache.clear();
    } else {
      _cache.remove(identity);
    }
  }

  Uint8List _subjectIdFromInstitutionIdentity(String identity) {
    final registered = registeredDuoqianAddressFromIdentity(identity);
    if (registered != null) {
      return AdminSubjectIdCodec.fromAccountHex(
        AdminSubjectIdCodec.institutionAccount,
        registered,
      );
    }
    final personal = personalDuoqianAddressFromIdentity(identity);
    if (personal != null) {
      return AdminSubjectIdCodec.fromAccountHex(
        AdminSubjectIdCodec.personalDuoqian,
        personal,
      );
    }
    return AdminSubjectIdCodec.fromBuiltinSfid(identity);
  }
}
