// 业务字段必须与链上管理员任职的 `admin_account` 逐字一致。
// ignore_for_file: non_constant_identifier_names

import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';

/// 机构岗位状态，序号与 entity runtime 枚举一致。
enum InstitutionRoleStatus { active, inactive }

/// 管理员任职来源，序号与 entity runtime 枚举一致。
enum InstitutionAssignmentSource {
  genesis,
  registry,
  popularElection,
  mutualElection,
  nominationAppointment,
  institutionGovernance,
}

extension InstitutionAssignmentSourceLabel on InstitutionAssignmentSource {
  String get label => switch (this) {
        InstitutionAssignmentSource.genesis => '创世',
        InstitutionAssignmentSource.registry => '注册局',
        InstitutionAssignmentSource.popularElection => '普选',
        InstitutionAssignmentSource.mutualElection => '互选',
        InstitutionAssignmentSource.nominationAppointment => '提名任免',
        InstitutionAssignmentSource.institutionGovernance => '机构内部治理',
      };
}

/// 机构岗位定义；岗位只在所属机构 CID 内唯一。
class InstitutionRole {
  const InstitutionRole({
    required this.cidNumber,
    required this.roleCode,
    required this.roleName,
    required this.termRequired,
    required this.status,
  });

  final String cidNumber;
  final String roleCode;
  final String roleName;
  final bool termRequired;
  final InstitutionRoleStatus status;
}

/// 管理员钱包与机构岗位的一条任职关系。
class InstitutionAdminAssignment {
  const InstitutionAdminAssignment({
    required this.cidNumber,
    required this.admin_account,
    required this.roleCode,
    required this.termStart,
    required this.termEnd,
    required this.source,
    required this.sourceRef,
    required this.active,
    this.roleName = '',
    this.termRequired = false,
  });

  final String cidNumber;
  final String admin_account;
  final String roleCode;
  final String roleName;
  final bool termRequired;
  final int termStart;
  final int termEnd;
  final InstitutionAssignmentSource source;
  final String sourceRef;
  final bool active;

  String get termLabel => termRequired ? '$termStart ~ $termEnd（自纪元日序）' : '无任期';

  InstitutionAdminAssignment withRole(InstitutionRole role) =>
      InstitutionAdminAssignment(
        cidNumber: cidNumber,
        admin_account: admin_account,
        roleCode: roleCode,
        roleName: role.roleName,
        termRequired: role.termRequired,
        termStart: termStart,
        termEnd: termEnd,
        source: source,
        sourceRef: sourceRef,
        active: active,
      );
}

/// admins pallet 的机构管理员值；岗位资料仍由 entity 独立保存。
class InstitutionAdminsStorage {
  const InstitutionAdminsStorage({
    required this.institutionCode,
    required this.admins,
  });

  final String institutionCode;
  final List<AdminPerson> admins;
}

/// 一名机构管理员及其零到多条岗位任职。
///
/// 管理员人员集合是主记录；岗位任职只是附加展示。没有岗位的管理员仍保留本行。
class InstitutionAdminView {
  const InstitutionAdminView({
    required this.admin,
    this.assignments = const [],
  });

  final AdminPerson admin;
  final List<InstitutionAdminAssignment> assignments;
}
