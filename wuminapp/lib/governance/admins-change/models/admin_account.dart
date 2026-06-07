import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/account_id_codec.dart';

enum AdminAccountIdentityType {
  governanceInstitution,
  personalDuoqian,
  institutionAccount,
}

class AdminAccountIdentity {
  const AdminAccountIdentity({
    required this.type,
    required this.identityKey,
    required this.displayName,
    required this.org,
    required this.kind,
    required this.accountHex,
  });

  factory AdminAccountIdentity.fromInstitution(InstitutionInfo institution) {
    final personal = personalDuoqianAddressFromIdentity(institution.sfidNumber);
    if (personal != null) {
      return AdminAccountIdentity.personalDuoqian(
        accountHex: personal,
        displayName: institution.name,
      );
    }

    final account =
        registeredDuoqianAddressFromIdentity(institution.sfidNumber);
    if (account != null) {
      final org = institution.adminAccountOrg;
      if (org == null) {
        throw ArgumentError('机构账户管理员更换必须提供 adminAccountOrg');
      }
      return AdminAccountIdentity.institutionAccount(
        accountHex: account,
        org: org,
        displayName: institution.name,
      );
    }

    return AdminAccountIdentity.governanceInstitution(
      accountHex: institution.mainAddress,
      org: institution.orgType,
      displayName: institution.name,
    );
  }

  factory AdminAccountIdentity.governanceInstitution({
    required String accountHex,
    required int org,
    required String displayName,
  }) {
    if (org < 0 || org > 2) {
      throw ArgumentError('治理机构 org 必须为 0/1/2');
    }
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.governanceInstitution,
      identityKey: 'governance:$org:$account',
      displayName: displayName,
      org: org,
      kind: 0,
      accountHex: account,
    );
  }

  factory AdminAccountIdentity.personalDuoqian({
    required String accountHex,
    required String displayName,
  }) {
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    AdminAccountIdCodec.fromAccountHex(account);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.personalDuoqian,
      identityKey: 'personal:$account',
      displayName: displayName,
      org: 3,
      kind: 1,
      accountHex: account,
    );
  }

  factory AdminAccountIdentity.institutionAccount({
    required String accountHex,
    required int org,
    required String displayName,
  }) {
    if (org != 4 && org != 5) {
      throw ArgumentError('机构账户 org 必须为 ORG_PUP 或 ORG_OTH');
    }
    final account = AdminAccountIdCodec.normalizeHex(accountHex);
    AdminAccountIdCodec.fromAccountHex(account);
    return AdminAccountIdentity(
      type: AdminAccountIdentityType.institutionAccount,
      identityKey: 'institution-account:$org:$account',
      displayName: displayName,
      org: org,
      kind: 2,
      accountHex: account,
    );
  }

  final AdminAccountIdentityType type;
  final String identityKey;
  final String displayName;
  final int org;
  final int kind;
  final String accountHex;

  String get typeLabel => switch (type) {
        AdminAccountIdentityType.governanceInstitution => '治理机构',
        AdminAccountIdentityType.personalDuoqian => '个人多签',
        AdminAccountIdentityType.institutionAccount => '机构账户',
      };

  String get orgLabel => switch (org) {
        0 => '国储会',
        1 => '省储会',
        2 => '省储行',
        3 => '个人多签',
        4 => '公权机构账户',
        5 => '其他机构账户',
        _ => '未知组织',
      };
}

class AdminAccountState {
  const AdminAccountState({
    required this.accountHex,
    required this.org,
    required this.kind,
    required this.admins,
    required this.threshold,
    required this.creatorHex,
    required this.createdAt,
    required this.updatedAt,
    required this.status,
  });

  final String accountHex;
  final int org;
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
      org: org,
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

  String get orgLabel => switch (org) {
        0 => '国储会',
        1 => '省储会',
        2 => '省储行',
        3 => '个人多签',
        4 => '公权机构账户',
        5 => '其他机构账户',
        _ => '未知组织',
      };

  String get statusLabel => switch (status) {
        0 => '待激活',
        1 => '已激活',
        2 => '已关闭',
        _ => '未知状态',
      };
}
