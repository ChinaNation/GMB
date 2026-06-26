// 公权机构「发起提案」注册表——机构码 → 可发起提案种类(单一真源)。
//
// 中文注释:proposal_entry_page 按机构码 `kindsForCode(institutionCode)` 渲染提案卡片,
// 取代原 GovernanceProposalsPage 里散落的 `if (orgType == nrc || prc)` 硬判断。
// **机构维度一律用机构码,不另建机构类型枚举**(用户铁律 2026-06-25)。仅公权机构。
// 提案的具体发起页在 `citizen/proposal/<type>/`。

/// 一种可发起的提案。
enum ProposalKind {
  transfer, // 转账(资金,proposal/transaction → MultisigTransferPage)
  feeTransfer, // 手续费划转/归集到主账户(资金,proposal/transaction → SweepToMainPage)
  safetyFundTransfer, // 安全基金转账(资金,仅国储会)
  adminsChange, // 换管理员(proposal/admins-change)
  resolutionIssuance, // 决议发行(占位)
  resolutionDestroy, // 决议销毁(占位)
  runtimeUpgrade, // 协议升级(类B,proposal/runtime-upgrade)
  grandpaKey, // 验证密钥(占位)
  legislation, // 发起立法/修法/废法(类B,proposal/legislation-yuan)
  election, // 发起选举(占位)
}

/// 各提案种类 → 可发起的机构码集合(单一真源)。
/// `null` = 全部公权机构通用(转账/手续费/归集/换管理员)。
const Map<ProposalKind, Set<String>?> _eligibleCodes = {
  ProposalKind.transfer: null,
  ProposalKind.feeTransfer: null,
  ProposalKind.adminsChange: null,
  ProposalKind.safetyFundTransfer: {'NRC'}, // 仅国储会
  ProposalKind.resolutionIssuance: {'NRC', 'PRC'}, // 国储会/省储会
  ProposalKind.resolutionDestroy: {'NRC', 'PRC', 'PRB'}, // 治理三类
  ProposalKind.runtimeUpgrade: {'NRC', 'PRC'},
  ProposalKind.grandpaKey: {'NRC', 'PRC'},
  // 立法发起院:国家众议会/国家教委会/省众议会/市立法会/市自治会/市教委会
  // (参议会 NSN/PSN、立法院 NLG/PLG 是表决院,不发起)。
  ProposalKind.legislation: {'NRP', 'NED', 'PRP', 'CLEG', 'CSLF', 'CEDU'},
  ProposalKind.election: <String>{}, // 选举机构码待定义(空=暂无机构可发起)
};

/// 某机构码可发起的提案种类(顺序同 ProposalKind 声明)。
/// `codes == null` → 通用,任何公权机构可发起;否则机构码命中才可发起。
List<ProposalKind> kindsForCode(String institutionCode) {
  return ProposalKind.values.where((kind) {
    final codes = _eligibleCodes[kind];
    return codes == null || codes.contains(institutionCode);
  }).toList(growable: false);
}
