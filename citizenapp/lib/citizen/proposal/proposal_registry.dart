// 统一「发起提案」能力注册表。
//
//
// - 机构码仍是制度身份输入,但页面不得散落 `NRC/PRC/CREG` 业务判断。
// - 本文件先把 InstitutionInfo + institution_code 解析为 ProposalSubject,再由规则表
//   输出可发起提案。这样个人多签、公权机构、私权机构和创世治理机构共用同一入口。
// - runtime 仍是最终权限边界;这里仅负责端上展示和页面路由。

import 'package:citizenapp/citizen/shared/institution_code_label.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';

/// 一种可发起的提案。
enum ProposalKind {
  transfer, // 转账(资金,proposal/transaction → MultisigTransferPage)
  feeTransfer, // 手续费划转/归集到主账户(资金,proposal/transaction → SweepToMainPage)
  safetyFundTransfer, // 安全基金转账(资金,仅国家储委会)
  adminsChange, // 换管理员(proposal/admins-change)
  resolutionIssuance, // 决议发行(占位)
  resolutionDestroy, // 决议销毁(占位)
  runtimeUpgrade, // 协议升级(类B,proposal/runtime-upgrade)
  grandpaKey, // 验证密钥(占位)
  legislation, // 发起立法/修法/废法(类B,proposal/legislation-yuan)
  election, // 发起选举(占位)
}

/// 发起提案的主体类型。它不是机构码的替代品,而是对机构码+账户身份的解析结果。
enum ProposalSubjectType {
  fixedGovernanceInstitution,
  publicInstitution,
  privateInstitution,
  unincorporatedInstitution,
  personalMultisig,
}

/// 发起提案主体。
class ProposalSubject {
  const ProposalSubject({
    required this.subjectType,
    required this.institutionCode,
    required this.subjectAccountId,
    required this.adminModule,
  });

  factory ProposalSubject.fromInstitution({
    required InstitutionInfo institution,
    required String institutionCode,
  }) {
    final code = institutionCode.toUpperCase();
    final account = institution.mainAccount;
    if (isPersonalAccountIdentity(institution.cidNumber) ||
        InstitutionCodeLabel.isPersonal(code)) {
      return ProposalSubject(
        subjectType: ProposalSubjectType.personalMultisig,
        institutionCode: 'PMUL',
        subjectAccountId: account,
        adminModule: 'PersonalAdmins',
      );
    }
    if (InstitutionCodeLabel.isFixedGovernance(code)) {
      return ProposalSubject(
        subjectType: ProposalSubjectType.fixedGovernanceInstitution,
        institutionCode: code,
        subjectAccountId: account,
        adminModule: 'PublicAdmins',
      );
    }
    if (InstitutionCodeLabel.isPublicAdminCode(code)) {
      return ProposalSubject(
        subjectType: ProposalSubjectType.publicInstitution,
        institutionCode: code,
        subjectAccountId: account,
        adminModule: 'PublicAdmins',
      );
    }
    if (InstitutionCodeLabel.isPrivateAdminCode(code)) {
      return ProposalSubject(
        subjectType: ProposalSubjectType.privateInstitution,
        institutionCode: code,
        subjectAccountId: account,
        adminModule: 'PrivateAdmins',
      );
    }
    if (InstitutionCodeLabel.isUnincorporated(code)) {
      return ProposalSubject(
        subjectType: ProposalSubjectType.unincorporatedInstitution,
        institutionCode: code,
        subjectAccountId: account,
        // 非法人机构码本身不能决定 public/private 管理员模块。
        adminModule: 'Unresolved',
      );
    }
    return ProposalSubject(
      subjectType: ProposalSubjectType.publicInstitution,
      institutionCode: code,
      subjectAccountId: account,
      adminModule: 'PublicAdmins',
    );
  }

  final ProposalSubjectType subjectType;
  final String institutionCode;
  final String subjectAccountId;
  final String adminModule;

  bool get isFixedGovernance =>
      subjectType == ProposalSubjectType.fixedGovernanceInstitution;
  bool get isPersonal => subjectType == ProposalSubjectType.personalMultisig;
  bool get isUnincorporated =>
      subjectType == ProposalSubjectType.unincorporatedInstitution;
  bool get hasResolvedAdminModule => adminModule != 'Unresolved';
}

class ProposalCapability {
  const ProposalCapability({
    required this.kind,
    required this.enabled,
    required this.pallet,
    required this.call,
    required this.voteEngine,
    required this.allows,
  });

  final ProposalKind kind;
  final bool enabled;
  final String pallet;
  final String call;
  final String voteEngine;
  final bool Function(ProposalSubject subject) allows;
}

class ProposalCapabilityRegistry {
  const ProposalCapabilityRegistry._();

  static const Set<String> _jointGovernanceCodes = {'NRC', 'PRC'};
  static const Set<String> _destroyGovernanceCodes = {'NRC', 'PRC', 'PRB'};
  static const Set<String> _sweepCodes = {'NRC', 'PRB'};
  static const Set<String> _legislationProposerCodes = {
    'NRP',
    'NED',
    'PRP',
    'CLEG',
    'CSLF',
    'CEDU',
  };

  static final List<ProposalCapability> _capabilities = [
    ProposalCapability(
      kind: ProposalKind.transfer,
      enabled: true,
      pallet: 'MultisigTransfer',
      call: 'propose_transfer',
      voteEngine: 'InternalVote',
      allows: (subject) => true,
    ),
    ProposalCapability(
      kind: ProposalKind.feeTransfer,
      enabled: true,
      pallet: 'MultisigTransfer',
      call: 'propose_sweep_to_main',
      voteEngine: 'InternalVote',
      allows: (subject) => _sweepCodes.contains(subject.institutionCode),
    ),
    ProposalCapability(
      kind: ProposalKind.adminsChange,
      enabled: true,
      pallet: 'Public/Private/PersonalAdmins',
      call: 'propose_admin_set_change',
      voteEngine: 'InternalVote',
      allows: (subject) => subject.hasResolvedAdminModule,
    ),
    ProposalCapability(
      kind: ProposalKind.safetyFundTransfer,
      enabled: true,
      pallet: 'MultisigTransfer',
      call: 'propose_safety_fund_transfer',
      voteEngine: 'InternalVote',
      allows: (subject) => subject.institutionCode == 'NRC',
    ),
    ProposalCapability(
      kind: ProposalKind.resolutionIssuance,
      enabled: true,
      pallet: 'ResolutionIssuance',
      call: 'propose_resolution_issuance',
      voteEngine: 'JointVote',
      allows: (subject) =>
          _jointGovernanceCodes.contains(subject.institutionCode),
    ),
    ProposalCapability(
      kind: ProposalKind.resolutionDestroy,
      enabled: true,
      pallet: 'ResolutionDestroy',
      call: 'propose_destroy',
      voteEngine: 'InternalVote',
      allows: (subject) =>
          _destroyGovernanceCodes.contains(subject.institutionCode),
    ),
    ProposalCapability(
      kind: ProposalKind.runtimeUpgrade,
      enabled: true,
      pallet: 'RuntimeUpgrade',
      call: 'propose_runtime_upgrade',
      voteEngine: 'JointVote',
      allows: (subject) =>
          _jointGovernanceCodes.contains(subject.institutionCode),
    ),
    ProposalCapability(
      kind: ProposalKind.grandpaKey,
      enabled: true,
      pallet: 'GrandpaKeyChange',
      call: 'propose_replace_grandpa_key',
      voteEngine: 'InternalVote',
      allows: (subject) =>
          _jointGovernanceCodes.contains(subject.institutionCode),
    ),
    ProposalCapability(
      kind: ProposalKind.legislation,
      enabled: true,
      pallet: 'LegislationYuan',
      call: 'propose_enact_law/propose_amend_law/propose_repeal_law',
      voteEngine: 'LegislationVote',
      allows: (subject) =>
          _legislationProposerCodes.contains(subject.institutionCode),
    ),
    ProposalCapability(
      kind: ProposalKind.election,
      enabled: false,
      pallet: 'ElectionVote',
      call: 'pending',
      voteEngine: 'ElectionVote',
      allows: (_) => false,
    ),
  ];

  /// 返回主体当前可展示的提案能力。禁用能力不展示,避免产生假入口。
  static List<ProposalCapability> capabilitiesForSubject(
    ProposalSubject subject,
  ) {
    return _capabilities
        .where((capability) => capability.enabled && capability.allows(subject))
        .toList(growable: false);
  }
}
