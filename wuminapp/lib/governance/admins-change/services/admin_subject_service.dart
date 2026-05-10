import 'dart:typed_data';

import 'package:wuminapp_mobile/governance/admins-change/codec/admin_subject_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class AdminSubjectService {
  AdminSubjectService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;
  final Map<String, AdminSubjectState> _cache = {};

  Future<AdminSubjectState?> fetchByInstitutionIdentity(String identity) async {
    final subjectId = _subjectIdFromInstitutionIdentity(identity);
    return fetchBySubjectId(subjectId);
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
      try {
        _cache.remove(AdminSubjectIdCodec.hexEncode(
          _subjectIdFromInstitutionIdentity(identity),
        ));
      } catch (_) {
        // 中文注释：旧调用可能传入的不是机构身份，保守忽略即可。
      }
    }
  }

  void clearSubjectCache(String subjectIdHex) {
    _cache.remove(AdminSubjectIdCodec.normalizeHex(subjectIdHex));
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
