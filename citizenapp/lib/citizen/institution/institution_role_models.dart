// 业务字段必须与链上管理员任职的 `account_id` 逐字一致。
// ignore_for_file: non_constant_identifier_names

import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';

/// 机构岗位状态，序号与 entity runtime 枚举一致。
enum InstitutionRoleStatus { active, inactive }

/// 岗位对一个业务动作拥有的权限操作，序号与 runtime SCALE 枚举一致。
enum RolePermissionOperation { propose, vote }

/// 业务模块静态指定的投票引擎，序号与 runtime SCALE 枚举一致。
enum VotingEngineKind { internal, joint, election, legislation }

/// 完整机构岗位授权主体；CID 和岗位码缺一不可。
class RoleSubject {
  const RoleSubject({required this.cidNumber, required this.roleCode});

  final String cidNumber;
  final String roleCode;
}

/// 业务模块与模块内动作的稳定标识。
class BusinessActionId {
  const BusinessActionId({required this.moduleTag, required this.actionCode});

  final String moduleTag;
  final int actionCode;
}

/// 一条完整岗位业务权限。
class RoleBusinessPermission {
  const RoleBusinessPermission({
    required this.roleSubject,
    required this.businessActionId,
    required this.operation,
  });

  final RoleSubject roleSubject;
  final BusinessActionId businessActionId;
  final RolePermissionOperation operation;
}

/// 机构岗位或个人多签授权主体。
///
/// 两个字段只允许存在其一；构造器固定分支，避免调用方混合主体。
class AuthorizationSubject {
  const AuthorizationSubject.institution(this.roleSubject)
      : personalAccountId = null;

  const AuthorizationSubject.personalMultisig(this.personalAccountId)
      : roleSubject = null;

  final RoleSubject? roleSubject;
  final String? personalAccountId;

  bool get isInstitution => roleSubject != null;
}

/// 业务模块创建提案时绑定的投票计划。
class VotePlan {
  const VotePlan({
    required this.businessActionId,
    required this.proposalOwner,
    required this.proposerSubject,
    required this.voterSubjects,
    required this.votingEngine,
    required this.businessObjectHash,
  });

  final BusinessActionId businessActionId;
  final String proposalOwner;
  final AuthorizationSubject proposerSubject;
  final List<AuthorizationSubject> voterSubjects;
  final VotingEngineKind votingEngine;
  final String businessObjectHash;
}

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
    required this.account_id,
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
  final String account_id;
  final String roleCode;
  final String roleName;
  final bool termRequired;
  final int termStart;
  final int termEnd;
  final InstitutionAssignmentSource source;
  final String sourceRef;
  final bool active;

  String get termLabel => termRequired ? '$termStart ~ $termEnd（自纪元日序）' : '无任期';

  /// 与 runtime 一致按 UTC 纪元日判断有效任职；起止日均包含在任期内。
  bool isEffectiveOnDay(int currentDay) {
    if (!active) return false;
    if (!termRequired) return termStart == 0 && termEnd == 0;
    return termStart > 0 && termStart <= currentDay && currentDay <= termEnd;
  }

  InstitutionAdminAssignment withRole(InstitutionRole role) =>
      InstitutionAdminAssignment(
        cidNumber: cidNumber,
        account_id: account_id,
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
