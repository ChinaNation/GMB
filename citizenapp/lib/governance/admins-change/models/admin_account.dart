import 'package:citizenapp/governance/shared/institution_code_label.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/governance/admins-change/codec/account_id_codec.dart';

enum AdminAccountIdentityType {
  governanceInstitution,
  personalAccount,
  institutionAccount,
}

class AdminAccountIdentity {
  const AdminAccountIdentity({
    required this.type,
    required this.identityKey,
    required this.displayName,
    required this.institutionCode,
    required this.kind,
    required this.accountHex,
  });

  factory AdminAccountIdentity.fromInstitution(InstitutionInfo institution) {
    final personal = personalAccountHexFromIdentity(institution.cidNumber);
    if (personal != null) {
      return AdminAccountIdentity.personalAccount(
        accountHex: personal,
        displayName: institution.cidShortName,
      );
    }

    final account = registeredAccountHexFromIdentity(institution.cidNumber);
    if (account != null) {
      final code = institution.adminAccountCode;
      if (code == null) {
        throw ArgumentError('机构账户管理员更换必须提供 adminAccountCode');
      }
      return AdminAccountIdentity.institutionAccount(
        accountHex: account,
        institutionCode: code,
        displayName: institution.cidShortName,
      );
    }

    return AdminAccountIdentity.governanceInstitution(
      accountHex: institution.mainAccount,
      orgType: institution.orgType,
      displayName: institution.cidShortName,
    );
  }

  factory AdminAccountIdentity.governanceInstitution({
    required String accountHex,
    required int orgType,
    required String displayName,
  }) {
    if (orgType < 0 || orgType > 2) {
      throw ArgumentError('治理机构 orgType 必须为 0/1/2 (NRC/PRC/PRB)');
    }
    final code = switch (orgType) {
      0 => 'NRC',
      1 => 'PRC',
      2 => 'PRB',
      _ => throw ArgumentError('治理机构 orgType 无效: $orgType'),
    };
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.governanceInstitution,
      identityKey: 'governance:$code:$account',
      displayName: displayName,
      institutionCode: code,
      kind: 0,
      accountHex: account,
    );
  }

  factory AdminAccountIdentity.personalAccount({
    required String accountHex,
    required String displayName,
  }) {
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    AdminAccountIdCodec.fromAccountHex(account);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.personalAccount,
      identityKey: 'personal-account:$account',
      displayName: displayName,
      institutionCode: 'PMUL',
      kind: 1,
      accountHex: account,
    );
  }

  factory AdminAccountIdentity.institutionAccount({
    required String accountHex,
    required String institutionCode,
    required String displayName,
  }) {
    if (!InstitutionCodeLabel.isInstitution(institutionCode)) {
      throw ArgumentError(
        '机构账户 institutionCode 必须为注册机构码，收到: $institutionCode',
      );
    }
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    AdminAccountIdCodec.fromAccountHex(account);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.institutionAccount,
      identityKey: 'institution-account:$institutionCode:$account',
      displayName: displayName,
      institutionCode: institutionCode,
      kind: 2,
      accountHex: account,
    );
  }

  final AdminAccountIdentityType type;
  final String identityKey;
  final String displayName;

  /// 4 字节机构码字符串（"NRC"/"PRC"/"PRB"/"PMUL"/"CGOV" 等）。
  final String institutionCode;
  final int kind;
  final String accountHex;

  String get typeLabel => switch (type) {
        AdminAccountIdentityType.governanceInstitution => '治理机构',
        AdminAccountIdentityType.personalAccount => '个人多签',
        AdminAccountIdentityType.institutionAccount => '机构账户',
      };

  String get orgLabel => InstitutionCodeLabel.codeLabel(institutionCode);
}

class AdminAccountState {
  const AdminAccountState({
    required this.accountHex,
    required this.institutionCode,
    required this.kind,
    required this.admins,
    required this.threshold,
    required this.creatorHex,
    required this.createdAt,
    required this.updatedAt,
    required this.status,
  });

  final String accountHex;

  /// 4 字节机构码字符串（"NRC"/"PRC"/"PRB"/"PMUL"/"CGOV" 等）。
  final String institutionCode;
  final int kind;
  final List<String> admins;
  final int threshold;
  final String creatorHex;
  final int createdAt;
  final int updatedAt;
  final int status;

  bool get isActive => status == 1;

  AdminAccountState copyWith({int? threshold}) {
    return AdminAccountState(
      accountHex: accountHex,
      institutionCode: institutionCode,
      kind: kind,
      admins: admins,
      threshold: threshold ?? this.threshold,
      creatorHex: creatorHex,
      createdAt: createdAt,
      updatedAt: updatedAt,
      status: status,
    );
  }

  String get kindLabel => switch (kind) {
        0 => '内置治理机构',
        1 => '个人多签',
        2 => '机构账户',
        _ => '未知账户',
      };

  String get orgLabel => InstitutionCodeLabel.codeLabel(institutionCode);

  String get statusLabel => switch (status) {
        0 => '待激活',
        1 => '已激活',
        2 => '已关闭',
        _ => '未知状态',
      };
}
