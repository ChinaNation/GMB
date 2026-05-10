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

/// 管理员查询兼容门面。
///
/// 中文注释：管理员主体真源已收口到 `lib/admins_change/`；本类在新目录内保留
/// 原 public API，供机构详情、提案上下文等既有调用方继续复用。
class InstitutionAdminService {
  InstitutionAdminService({ChainRpc? chainRpc})
      : _subjectService = AdminSubjectService(chainRpc: chainRpc);

  final AdminSubjectService _subjectService;

  Future<List<String>> fetchAdmins(String sfidNumber) {
    return _subjectService.fetchAdmins(sfidNumber);
  }

  Future<int?> fetchThreshold(String sfidNumber) {
    return _subjectService.fetchThreshold(sfidNumber);
  }

  Future<bool> isAdmin(String pubkeyHex, String sfidNumber) {
    return _subjectService.isAdmin(pubkeyHex, sfidNumber);
  }

  Future<InstitutionAdminState> fetchState(String sfidNumber) async {
    final subject =
        await _subjectService.fetchByInstitutionIdentity(sfidNumber);
    return InstitutionAdminState(
      admins: subject?.admins ?? const [],
      threshold: subject?.threshold,
    );
  }

  void clearCache([String? sfidNumber]) {
    _subjectService.clearCache(sfidNumber);
  }
}
