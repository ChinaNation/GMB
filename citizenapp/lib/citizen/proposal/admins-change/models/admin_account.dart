import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';

enum AdminAccountIdentityType {
  governanceInstitution,
  personalAccount,
  institutionAccount,
}

class AdminAccountIdentity {
  const AdminAccountIdentity({
    required this.type,
    required this.identityKey,
    required this.accountLabel,
    required this.institutionCode,
    required this.kind,
    required this.accountHex,
  });

  factory AdminAccountIdentity.fromInstitution(InstitutionInfo institution) {
    final fixedCode = institution.adminAccountCode?.toUpperCase();
    if (fixedCode != null &&
        InstitutionCodeLabel.isFixedGovernance(fixedCode)) {
      return AdminAccountIdentity.fixedGovernanceInstitution(
        accountHex: institution.mainAccount,
        institutionCode: fixedCode,
        accountLabel: institution.cidShortName,
      );
    }

    final personal = personalAccountHexFromIdentity(institution.cidNumber);
    if (personal != null) {
      return AdminAccountIdentity.personalAccount(
        accountHex: personal,
        accountLabel: institution.cidShortName,
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
        accountLabel: institution.cidShortName,
      );
    }

    return AdminAccountIdentity.governanceInstitution(
      accountHex: institution.mainAccount,
      orgType: institution.orgType,
      accountLabel: institution.cidShortName,
    );
  }

  factory AdminAccountIdentity.fixedGovernanceInstitution({
    required String accountHex,
    required String institutionCode,
    required String accountLabel,
  }) {
    final code = institutionCode.toUpperCase();
    if (!InstitutionCodeLabel.isFixedGovernance(code)) {
      throw ArgumentError('固定治理机构 institutionCode 无效: $institutionCode');
    }
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    AdminAccountIdCodec.fromAccountHex(account);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.governanceInstitution,
      identityKey: 'fixed-governance:$code:$account',
      accountLabel: accountLabel,
      institutionCode: code,
      kind: InstitutionCodeLabel.adminAccountKind(code),
      accountHex: account,
    );
  }

  factory AdminAccountIdentity.governanceInstitution({
    required String accountHex,
    required int orgType,
    required String accountLabel,
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
      accountLabel: accountLabel,
      institutionCode: code,
      kind: InstitutionCodeLabel.adminAccountKind(code),
      accountHex: account,
    );
  }

  factory AdminAccountIdentity.personalAccount({
    required String accountHex,
    required String accountLabel,
  }) {
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    AdminAccountIdCodec.fromAccountHex(account);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.personalAccount,
      identityKey: 'personal-account:$account',
      accountLabel: accountLabel,
      institutionCode: 'PMUL',
      kind: InstitutionCodeLabel.adminAccountKind('PMUL'),
      accountHex: account,
    );
  }

  factory AdminAccountIdentity.institutionAccount({
    required String accountHex,
    required String institutionCode,
    required String accountLabel,
    int? kind,
  }) {
    if (!InstitutionCodeLabel.isInstitution(institutionCode)) {
      throw ArgumentError(
        '机构账户 institutionCode 必须为注册机构码，收到: $institutionCode',
      );
    }
    final resolvedKind = kind ??
        _deriveInstitutionKind(
          institutionCode,
        );
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    AdminAccountIdCodec.fromAccountHex(account);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.institutionAccount,
      identityKey: 'institution-account:$institutionCode:$account',
      accountLabel: accountLabel,
      institutionCode: institutionCode,
      kind: resolvedKind,
      accountHex: account,
    );
  }

  static int _deriveInstitutionKind(String institutionCode) {
    if (InstitutionCodeLabel.isUnincorporatedAdminCode(institutionCode)) {
      throw ArgumentError(
        '非法人机构码不能自动选择管理员模块，必须显式传入 kind=0(public) 或 kind=1(private)',
      );
    }
    return InstitutionCodeLabel.adminAccountKind(institutionCode);
  }

  final AdminAccountIdentityType type;
  final String identityKey;

  /// 管理员账户选择/提示用标签;不是机构全称或简称的第二字段。
  final String accountLabel;

  /// 4 字节机构码字符串（"NRC"/"PRC"/"PRB"/"PMUL"/"CGOV" 等）。
  final String institutionCode;
  final int kind;
  final String accountHex;

  String get typeLabel => switch (type) {
        AdminAccountIdentityType.governanceInstitution => '创世治理机构',
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

  /// 管理员钱包账户集合；机构岗位不在 admins 模块重复保存。
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
        0 => '公权机构',
        1 => '私权机构',
        2 => '个人多签',
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
