/// 机构岗位状态，序号与 entity runtime 枚举一致。
enum InstitutionRoleStatus { active, inactive }

/// 管理员任职来源，序号与 entity runtime 枚举一致。
enum InstitutionAssignmentSource {
  genesis,
  registry,
  popularElection,
  mutualElection,
  nominationAppointment,
}

extension InstitutionAssignmentSourceLabel on InstitutionAssignmentSource {
  String get label => switch (this) {
        InstitutionAssignmentSource.genesis => '创世',
        InstitutionAssignmentSource.registry => '注册局',
        InstitutionAssignmentSource.popularElection => '普选',
        InstitutionAssignmentSource.mutualElection => '互选',
        InstitutionAssignmentSource.nominationAppointment => '提名任免',
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
    required this.adminAccount,
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
  final String adminAccount;
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
        adminAccount: adminAccount,
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

/// admins pallet 的机构管理员账户值；只含钱包集合，不含岗位资料。
class InstitutionAdminAccountStorage {
  const InstitutionAdminAccountStorage({
    required this.institutionCode,
    required this.admins,
  });

  final String institutionCode;
  final List<String> admins;
}
