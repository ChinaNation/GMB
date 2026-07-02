import 'dart:typed_data';

import 'package:citizenapp/transaction/personal-manage/personal_manage_models.dart';

/// 提案展示号(双层 ID 设计,spec_version v1)。
///
/// 链上 `ProposalDisplayId[u64] = ProposalDisplayMeta { year: u16, seq_in_year: u32 }`
/// 反查表的客户端镜像。主键 `proposal_id` 全局单调与展示号解耦,渲染层基于
/// 本结构拼接 "2026000123" 类格式。
class ProposalDisplayMeta {
  const ProposalDisplayMeta({required this.year, required this.seqInYear});

  /// 创建年份(UTC 公历)。
  final int year;

  /// 年内序号(每年从 0 重置)。u32 上限,实质无上限。
  final int seqInYear;
}

/// 提案链上元数据（从 VotingEngine::Proposals Storage 解码）。
class ProposalMeta {
  const ProposalMeta({
    required this.proposalId,
    required this.kind,
    required this.stage,
    required this.status,
    this.internalCode,
    this.institutionBytes,
    this.subjectCidNumbers = const [],
    this.displayMeta,
  });

  final int proposalId;
  final int kind; // 0=internal, 1=joint
  final int stage; // 0=internal, 1=joint, 2=citizen
  final int status; // 0=voting, 1=passed, 2=rejected
  final String? internalCode;

  /// 链上 Proposal.account_context。机构归属真源不看这里,只看 [subjectCidNumbers]。
  final Uint8List? institutionBytes;

  /// 链上 Proposal.subject_cid_numbers。机构类提案归属和订阅过滤统一用 CID。
  final List<String> subjectCidNumbers;

  /// 展示号(双层 ID:主键 `proposalId` 单调,展示号年份+序号通过 `ProposalDisplayId` 反查)。
  /// 列表页 batch fetch 后填充;详情页解码 ProposalMeta 时同步查询。
  final ProposalDisplayMeta? displayMeta;
}

/// Runtime upgrade 提案链上数据。
class RuntimeUpgradeProposalInfo {
  const RuntimeUpgradeProposalInfo({
    required this.proposalId,
    required this.proposer,
    required this.reason,
    required this.codeHashHex,
  });

  final int proposalId;
  final String proposer; // SS58 (ss58Format 2027)
  final String reason; // UTF-8 decoded
  final String codeHashHex; // 32-byte hash as hex
}

/// 提案 + 业务详情（用于全局提案列表与机构投票事件展示）。
class ProposalWithDetail {
  const ProposalWithDetail({
    required this.meta,
    this.runtimeUpgradeDetail,
    this.createMultisigDetail,
    this.closeMultisigDetail,
    this.businessDetails = const {},
    this.resolutionIssuanceSummary,
    this.resolutionDestroySummary,
  });

  final ProposalMeta meta;

  /// 协议升级提案详情（非升级提案为 null）。
  final RuntimeUpgradeProposalInfo? runtimeUpgradeDetail;

  /// 创建多签账户提案详情。
  final CreateMultisigProposalInfo? createMultisigDetail;

  /// 关闭多签账户提案详情。
  final CloseMultisigProposalInfo? closeMultisigDetail;

  /// 业务模块详情集合。proposal/shared 只按字符串键保存不透明对象，
  /// 具体键名、模型和页面跳转由所属业务模块自己定义。
  final Map<String, Object?> businessDetails;

  /// 决议发行提案摘要（仅列表展示用）。
  final String? resolutionIssuanceSummary;

  /// 决议销毁提案摘要（仅列表展示用）。
  final String? resolutionDestroySummary;
}
