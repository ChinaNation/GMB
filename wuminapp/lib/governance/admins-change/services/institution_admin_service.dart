import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_subject_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class InstitutionAdminState {
  const InstitutionAdminState({
    required this.admins,
    this.threshold,
  });

  final List<String> admins;
  final int? threshold;
}

/// 管理员查询门面。
///
/// 中文注释：调用方必须传入明确的 `AdminSubjectIdentity`，不再把
/// `sfidNumber` 当作个人多签、机构账户和治理机构共用的模糊参数。
class InstitutionAdminService {
  InstitutionAdminService({ChainRpc? chainRpc})
      : _subjectService = AdminSubjectService(chainRpc: chainRpc);

  final AdminSubjectService _subjectService;

  Future<List<String>> fetchAdmins(AdminSubjectIdentity identity) {
    return _subjectService.fetchAdmins(identity);
  }

  Future<int?> fetchThreshold(AdminSubjectIdentity identity) {
    return _subjectService.fetchThreshold(identity);
  }

  Future<bool> isAdmin(String pubkeyHex, AdminSubjectIdentity identity) {
    return _subjectService.isAdmin(pubkeyHex, identity);
  }

  Future<InstitutionAdminState> fetchState(
      AdminSubjectIdentity identity) async {
    final subject = await _subjectService.fetchByIdentity(identity);
    return InstitutionAdminState(
      admins: subject?.admins ?? const [],
      threshold: subject?.threshold,
    );
  }

  void clearCache([AdminSubjectIdentity? identity]) {
    _subjectService.clearCache(identity);
  }
}
