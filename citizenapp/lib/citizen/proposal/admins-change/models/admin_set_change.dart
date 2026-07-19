import 'admin_account.dart';

class AdminsChangeDraft {
  const AdminsChangeDraft({
    required this.accountHex,
    required this.institutionCode,
    required this.currentAdmins,
    required this.admins,
  });

  final String accountHex;

  /// 4 字节机构码字符串（"NRC"/"PRC"/"PRB"/"PMUL"/"CGOV" 等）。
  final String institutionCode;
  final List<AdminPerson> currentAdmins;
  final List<AdminPerson> admins;
}
