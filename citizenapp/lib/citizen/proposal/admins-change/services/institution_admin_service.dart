import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_account_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

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
/// 中文注释：调用方必须传入明确的 `AdminAccountIdentity`，不再把
/// `cidNumber` 当作个人多签、机构账户和治理机构共用的模糊参数。
class InstitutionAdminService {
  InstitutionAdminService({ChainRpc? chainRpc})
      : _accountService = AdminAccountService(chainRpc: chainRpc);

  final AdminAccountService _accountService;

  Future<List<String>> fetchAdmins(AdminAccountIdentity identity) {
    return _accountService.fetchAdmins(identity);
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
      threshold: account?.threshold,
    );
  }

  void clearCache([AdminAccountIdentity? identity]) {
    _accountService.clearCache(identity);
  }
}
