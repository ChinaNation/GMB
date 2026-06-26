import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_account_service.dart';

class AdminSetChangeController {
  AdminSetChangeController({AdminAccountService? accountService})
      : _accountService = accountService ?? AdminAccountService();

  final AdminAccountService _accountService;

  Future<AdminAccountState?> loadAccount(AdminAccountIdentity identity) {
    return _accountService.fetchByIdentity(identity);
  }
}
