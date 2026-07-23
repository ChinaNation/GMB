// 业务字段必须与 Rust 管理员结构逐字一致。
// ignore_for_file: non_constant_identifier_names

import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';

enum AdminAccountIdentityType { institution, personalAccount }

/// 全仓统一的管理员人员记录。
///
/// `account_id` 是人员记录唯一标识与去重字段；姓、名只用于人员姓名展示，
/// 页面需要姓名时现场按中文顺序组合，不保存合并姓名。机构业务授权必须另查
/// `CID + 岗位码`，个人多签才直接以管理员集合授权。
class AdminPerson {
  const AdminPerson({
    required this.account_id,
    this.cid_number = '',
    required this.family_name,
    required this.given_name,
  });

  final String account_id;

  /// 所有机构管理员均保存公民 CID；个人多签不是机构，其管理员固定为空。
  final String cid_number;
  final String family_name;
  final String given_name;

  AdminPerson copyWith({
    String? account_id,
    String? cid_number,
    String? family_name,
    String? given_name,
  }) =>
      AdminPerson(
        account_id: account_id ?? this.account_id,
        cid_number: cid_number ?? this.cid_number,
        family_name: family_name ?? this.family_name,
        given_name: given_name ?? this.given_name,
      );
}

/// 管理员查询主体。机构只以 CID 为主键；个人多签只以 personal_account 为主键。
class AdminAccountIdentity {
  const AdminAccountIdentity._({
    required this.type,
    required this.identityKey,
    required this.accountLabel,
    required this.institutionCode,
    required this.kind,
    this.cidNumber,
    this.personalAccountId,
  });

  factory AdminAccountIdentity.fromInstitution(InstitutionInfo institution) {
    final personal = personalAccountIdFromIdentity(institution.cidNumber);
    if (personal != null) {
      return AdminAccountIdentity.personalAccount(
        personalAccountId: personal,
        accountLabel: institution.cidShortName,
      );
    }
    final code = institution.adminAccountCode?.toUpperCase() ??
        switch (institution.orgType) {
          0 => 'NRC',
          1 => 'PRC',
          2 => 'PRB',
          _ => throw ArgumentError('机构管理员查询必须提供 institutionCode'),
        };
    return AdminAccountIdentity.institution(
      cidNumber: institution.cidNumber,
      institutionCode: code,
      accountLabel: institution.cidShortName,
    );
  }

  factory AdminAccountIdentity.institution({
    required String cidNumber,
    required String institutionCode,
    required String accountLabel,
    int? kind,
  }) {
    final cid = cidNumber.trim();
    if (cid.isEmpty) throw ArgumentError('机构 cidNumber 不能为空');
    final normalizedCode = institutionCode.toUpperCase();
    if (!InstitutionCodeLabel.isInstitution(normalizedCode) &&
        !InstitutionCodeLabel.isFixedGovernance(normalizedCode)) {
      throw ArgumentError('institutionCode 必须是机构码: $institutionCode');
    }
    final resolvedKind = kind ?? _deriveInstitutionKind(normalizedCode);
    return AdminAccountIdentity._(
      type: AdminAccountIdentityType.institution,
      identityKey: 'institution:$cid',
      accountLabel: accountLabel,
      institutionCode: normalizedCode,
      kind: resolvedKind,
      cidNumber: cid,
    );
  }

  factory AdminAccountIdentity.personalAccount({
    required String personalAccountId,
    required String accountLabel,
  }) {
    final account = AdminAccountIdCodec.requireAccountId(personalAccountId);
    AdminAccountIdCodec.fromAccountIdText(account);
    return AdminAccountIdentity._(
      type: AdminAccountIdentityType.personalAccount,
      identityKey: 'personal-account:$account',
      accountLabel: accountLabel,
      institutionCode: 'PMUL',
      kind: 2,
      personalAccountId: account,
    );
  }

  static int _deriveInstitutionKind(String institutionCode) {
    if (InstitutionCodeLabel.isUnincorporatedAdminCode(institutionCode)) {
      throw ArgumentError('非法人机构码必须显式传入所属法人 kind');
    }
    return InstitutionCodeLabel.adminAccountKind(institutionCode);
  }

  final AdminAccountIdentityType type;
  final String identityKey;
  final String accountLabel;
  final String institutionCode;
  final int kind;
  final String? cidNumber;
  final String? personalAccountId;

  String get typeLabel => switch (type) {
        AdminAccountIdentityType.institution => '机构',
        AdminAccountIdentityType.personalAccount => '个人多签',
      };

  String get orgLabel => InstitutionCodeLabel.codeLabel(institutionCode);
}

class AdminAccountState {
  const AdminAccountState({
    this.cidNumber,
    this.personalAccountId,
    required this.institutionCode,
    required this.kind,
    required this.admins,
    required this.threshold,
    this.personalCreatorAccountId,
    this.personalCreatedAt,
    this.personalUpdatedAt,
    this.personalStatus,
  }) : assert((cidNumber == null) != (personalAccountId == null));

  final String? cidNumber;
  final String? personalAccountId;
  final String institutionCode;
  final int kind;
  final List<AdminPerson> admins;
  final int threshold;

  /// 以下字段只属于 PersonalAdmins 的个人多签生命周期。
  final String? personalCreatorAccountId;
  final int? personalCreatedAt;
  final int? personalUpdatedAt;
  final int? personalStatus;

  bool get isActive => cidNumber != null || personalStatus == 1;

  AdminAccountState copyWith({int? threshold}) => AdminAccountState(
        cidNumber: cidNumber,
        personalAccountId: personalAccountId,
        institutionCode: institutionCode,
        kind: kind,
        admins: admins,
        threshold: threshold ?? this.threshold,
        personalCreatorAccountId: personalCreatorAccountId,
        personalCreatedAt: personalCreatedAt,
        personalUpdatedAt: personalUpdatedAt,
        personalStatus: personalStatus,
      );

  String get kindLabel => switch (kind) {
        0 => '公权机构',
        1 => '私权机构',
        2 => '个人多签',
        _ => '未知账户',
      };

  String get orgLabel => InstitutionCodeLabel.codeLabel(institutionCode);

  String get statusLabel {
    if (cidNumber != null) return '机构管理员';
    return switch (personalStatus) {
      0 => '待激活',
      1 => '已激活',
      2 => '已关闭',
      _ => '未知状态',
    };
  }
}
