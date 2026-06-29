import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_account_service.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

class InstitutionAdminState {
  const InstitutionAdminState({
    required this.admins,
    this.profiles = const [],
    this.threshold,
  });

  final List<String> admins;

  /// 管理员完整资料(A2:cid/姓名/职务/任期/来源;个人多签仅 account)。
  final List<AdminProfile> profiles;
  final int? threshold;
}

/// 管理员查询门面。
///
/// 中文注释：调用方必须传入明确的 `AdminAccountIdentity`，不再把
/// `cidNumber` 当作个人多签、机构账户和治理机构共用的模糊参数。
class InstitutionAdminService {
  InstitutionAdminService({ChainRpc? chainRpc})
      : _accountService = AdminAccountService(chainRpc: chainRpc);

  final AdminAccountService _accountService;

  Future<List<String>> fetchAdmins(AdminAccountIdentity identity) {
    return _accountService.fetchAdmins(identity);
  }

  /// 取管理员完整资料(cid/姓名/职务/任期/来源)。供机构详情管理员展示。
  Future<List<AdminProfile>> fetchAdminProfiles(AdminAccountIdentity identity) {
    return _accountService.fetchAdminProfiles(identity);
  }

  Future<int?> fetchThreshold(AdminAccountIdentity identity) {
    return _accountService.fetchThreshold(identity);
  }

  Future<bool> isAdmin(String pubkeyHex, AdminAccountIdentity identity) {
    return _accountService.isAdmin(pubkeyHex, identity);
  }

  Future<InstitutionAdminState> fetchState(
      AdminAccountIdentity identity) async {
    final account = await _accountService.fetchByIdentity(identity);
    return InstitutionAdminState(
      admins: account?.admins ?? const [],
      profiles: account?.profiles ?? const [],
      threshold: account?.threshold,
    );
  }

  void clearCache([AdminAccountIdentity? identity]) {
    _accountService.clearCache(identity);
  }
}
