import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';

enum AdminSubjectIdentityType {
  governanceInstitution,
  personalDuoqian,
  institutionAccount,
}

class AdminSubjectIdentity {
  const AdminSubjectIdentity({
    required this.type,
    required this.identityKey,
    required this.displayName,
    required this.org,
    required this.kind,
    required this.subjectIdHex,
  });

  factory AdminSubjectIdentity.fromInstitution(InstitutionInfo institution) {
    final personal = personalDuoqianAddressFromIdentity(institution.sfidNumber);
    if (personal != null) {
      return AdminSubjectIdentity.personalDuoqian(
        accountHex: personal,
        displayName: institution.name,
      );
    }

    final account =
        registeredDuoqianAddressFromIdentity(institution.sfidNumber);
    if (account != null) {
      final org = institution.adminSubjectOrg;
      if (org == null) {
        throw ArgumentError('机构账户管理员更换必须提供 adminSubjectOrg');
      }
      return AdminSubjectIdentity.institutionAccount(
        accountHex: account,
        org: org,
        displayName: institution.name,
      );
    }

    return AdminSubjectIdentity.governanceInstitution(
      sfidNumber: institution.sfidNumber,
      org: institution.orgType,
      displayName: institution.name,
    );
  }

  factory AdminSubjectIdentity.governanceInstitution({
    required String sfidNumber,
    required int org,
    required String displayName,
  }) {
    if (org < 0 || org > 2) {
      throw ArgumentError('治理机构 org 必须为 0/1/2');
    }
    final subjectId = AdminSubjectIdCodec.fromBuiltinSfid(sfidNumber);
    return AdminSubjectIdentity(
      type: AdminSubjectIdentityType.governanceInstitution,
      identityKey: 'governance:$org:$sfidNumber',
      displayName: displayName,
      org: org,
      kind: 0,
      subjectIdHex: AdminSubjectIdCodec.hexEncode(subjectId),
    );
  }

  factory AdminSubjectIdentity.personalDuoqian({
    required String accountHex,
    required String displayName,
  }) {
    final account = AdminSubjectIdCodec.normalizeHex(accountHex);
    final subjectId = AdminSubjectIdCodec.fromAccountHex(
      AdminSubjectIdCodec.personalDuoqian,
      account,
    );
    return AdminSubjectIdentity(
      type: AdminSubjectIdentityType.personalDuoqian,
      identityKey: 'personal:$account',
      displayName: displayName,
      org: 3,
      kind: 2,
      subjectIdHex: AdminSubjectIdCodec.hexEncode(subjectId),
    );
  }

  factory AdminSubjectIdentity.institutionAccount({
    required String accountHex,
    required int org,
    required String displayName,
  }) {
    if (org != 4 && org != 5) {
      throw ArgumentError('机构账户 org 必须为 ORG_PUP 或 ORG_OTH');
    }
    final account = AdminSubjectIdCodec.normalizeHex(accountHex);
    final subjectId = AdminSubjectIdCodec.fromAccountHex(
      AdminSubjectIdCodec.institutionAccount,
      account,
    );
    return AdminSubjectIdentity(
      type: AdminSubjectIdentityType.institutionAccount,
      identityKey: 'institution-account:$org:$account',
      displayName: displayName,
      org: org,
      kind: 3,
      subjectIdHex: AdminSubjectIdCodec.hexEncode(subjectId),
    );
  }

  final AdminSubjectIdentityType type;
  final String identityKey;
  final String displayName;
  final int org;
  final int kind;
  final String subjectIdHex;

  String get typeLabel => switch (type) {
        AdminSubjectIdentityType.governanceInstitution => '治理机构',
        AdminSubjectIdentityType.personalDuoqian => '个人多签',
        AdminSubjectIdentityType.institutionAccount => '机构账户',
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

class AdminSubjectState {
  const AdminSubjectState({
    required this.subjectIdHex,
    required this.org,
    required this.kind,
    required this.admins,
    required this.threshold,
    required this.creatorHex,
    required this.createdAt,
    required this.updatedAt,
    required this.status,
  });

  final String subjectIdHex;
  final int org;
  final int kind;
  final List<String> admins;
  final int threshold;
  final String creatorHex;
  final int createdAt;
  final int updatedAt;
  final int status;

  bool get isActive => status == 1;

  String get kindLabel => switch (kind) {
        0 => '内置治理机构',
        1 => 'SFID机构归属',
        2 => '个人多签',
        3 => '机构账户',
        _ => '未知主体',
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
