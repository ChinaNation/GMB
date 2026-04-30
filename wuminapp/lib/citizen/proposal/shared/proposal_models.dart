import 'dart:typed_data';

import 'package:wuminapp_mobile/duoqian/shared/duoqian_manage_models.dart';

/// 转账提案链上数据。
class TransferProposalInfo {
  const TransferProposalInfo({
    required this.proposalId,
    required this.institutionBytes,
    required this.beneficiary,
    required this.amountFen,
    required this.remark,
    required this.proposer,
    this.status,
  });

  final int proposalId;
  final Uint8List institutionBytes;
  final String beneficiary; // SS58
  final BigInt amountFen;
  final String remark;
  final String proposer; // SS58

  /// 0=voting, 1=passed, 2=rejected, null=unknown
  final int? status;

  double get amountYuan => amountFen.toDouble() / 100;

  TransferProposalInfo copyWithStatus(int? newStatus) {
    return TransferProposalInfo(
      proposalId: proposalId,
      institutionBytes: institutionBytes,
      beneficiary: beneficiary,
      amountFen: amountFen,
      remark: remark,
      proposer: proposer,
      status: newStatus,
    );
  }
}

/// 提案链上元数据（从 VotingEngine::Proposals Storage 解码）。
class ProposalMeta {
  const ProposalMeta({
    required this.proposalId,
    required this.kind,
    required this.stage,
    required this.status,
    this.internalOrg,
    this.institutionBytes,
  });

  final int proposalId;
  final int kind; // 0=internal, 1=joint
  final int stage; // 0=internal, 1=joint, 2=citizen
  final int status; // 0=voting, 1=passed, 2=rejected
  final int? internalOrg;
  final Uint8List? institutionBytes;
}

/// Runtime upgrade 提案链上数据。
class RuntimeUpgradeProposalInfo {
  const RuntimeUpgradeProposalInfo({
    required this.proposalId,
    required this.proposer,
    required this.reason,
    required this.codeHashHex,
    required this.status,
  });

  final int proposalId;
  final String proposer; // SS58 (ss58Format 2027)
  final String reason; // UTF-8 decoded
  final String codeHashHex; // 32-byte hash as hex
  final int status; // 0=Voting, 1=Passed, 2=Rejected, 3=ExecutionFailed
}

/// 安全基金转账提案详情（从 SafetyFundProposalActions 存储解码）。
class SafetyFundProposalInfo {
  const SafetyFundProposalInfo({
    required this.proposalId,
    required this.beneficiary,
    required this.amountFen,
    required this.remark,
    required this.proposer,
    this.status,
  });

  final int proposalId;
  final String beneficiary; // SS58
  final BigInt amountFen;
  final String remark;
  final String proposer; // SS58
  final int? status;

  double get amountYuan => amountFen.toDouble() / 100;
}

/// 手续费划转提案详情（从 SweepProposalActions 存储解码）。
class SweepProposalInfo {
  const SweepProposalInfo({
    required this.proposalId,
    required this.institutionBytes,
    required this.amountFen,
    this.status,
  });

  final int proposalId;
  final Uint8List institutionBytes;
  final BigInt amountFen;
  final int? status;

  double get amountYuan => amountFen.toDouble() / 100;
}

/// 提案 + 业务详情（用于全局提案列表与机构投票事件展示）。
class ProposalWithDetail {
  const ProposalWithDetail({
    required this.meta,
    this.transferDetail,
    this.runtimeUpgradeDetail,
    this.createDuoqianDetail,
    this.closeDuoqianDetail,
    this.safetyFundDetail,
    this.sweepDetail,
    this.resolutionIssuanceSummary,
    this.resolutionDestroySummary,
  });

  final ProposalMeta meta;

  /// 转账提案详情（非转账提案为 null）。
  final TransferProposalInfo? transferDetail;

  /// Runtime 升级提案详情（非升级提案为 null）。
  final RuntimeUpgradeProposalInfo? runtimeUpgradeDetail;

  /// 创建多签账户提案详情。
  final CreateDuoqianProposalInfo? createDuoqianDetail;

  /// 关闭多签账户提案详情。
  final CloseDuoqianProposalInfo? closeDuoqianDetail;

  /// 安全基金转账提案详情。
  final SafetyFundProposalInfo? safetyFundDetail;

  /// 手续费划转提案详情。
  final SweepProposalInfo? sweepDetail;

  /// 决议发行提案摘要（仅列表展示用）。
  final String? resolutionIssuanceSummary;

  /// 决议销毁提案摘要（仅列表展示用）。
  final String? resolutionDestroySummary;
}
