// 治理模块前端类型定义，与后端 governance::types 和 governance::activation 对应。

// ── 签名请求/响应 ──

export type VoteSignRequestResult = {
  requestJson: string;
  requestId: string;
  expectedPayloadHash: string;
  signNonce: number;
  signBlockNumber: number;
};

export type VoteSubmitResult = {
  txHash: string;
};

export type ProposeUpgradeRequestResult = {
  requestJson: string;
  requestId: string;
  expectedPayloadHash: string;
  signNonce: number;
  signBlockNumber: number;
  eligibleTotal: number;
  snapshotNonce: string;
  snapshotSignature: string;
};

// ── 投票状态 ──

export type UserVoteStatus = {
  proposalId: number;
  kind: number;
  stage: number;
  internalVote: boolean | null;
  jointVote: boolean | null;
};

// ── 签名管理员 ──

export interface SigningAdminInfo {
  pubkeyHex: string;
  shenfenId: string;
  shenfenName: string;
}

// ── 管理员匹配（旧，保留兼容投票签名流程） ──

export type AdminWalletMatch = {
  address: string;
  pubkeyHex: string;
  name: string;
};

// ── 管理员激活 ──

export type ActivatedAdmin = {
  pubkeyHex: string;
  shenfenId: string;
  activatedAtMs: number;
};

export type ActivateRequestResult = {
  requestJson: string;
  requestId: string;
  expectedPayloadHash: string;
  payloadHex: string;
};

// ── 机构 ──

export type InstitutionListItem = {
  name: string;
  shenfenId: string;
  orgType: number;
  orgTypeLabel: string;
  duoqianAddress: string;
};

export type GovernanceOverview = {
  nationalCouncils: InstitutionListItem[];
  provincialCouncils: InstitutionListItem[];
  provincialBanks: InstitutionListItem[];
  warning: string | null;
};

export type AdminInfo = {
  pubkeyHex: string;
  balanceFen: string | null;
};

export type InstitutionDetail = {
  name: string;
  shenfenId: string;
  orgType: number;
  orgTypeLabel: string;
  duoqianAddress: string;
  balanceFen: string | null;
  admins: AdminInfo[];
  internalThreshold: number;
  jointVoteWeight: number;
  stakingAddress: string | null;
  stakingBalanceFen: string | null;
  feeAddress: string | null;
  feeBalanceFen: string | null;
  cbFeeAddress: string | null;
  cbFeeBalanceFen: string | null;
  nrcFeeAddress: string | null;
  nrcFeeBalanceFen: string | null;
  nrcAnquanAddress: string | null;
  nrcAnquanBalanceFen: string | null;
  warning: string | null;
};

// ── 提案相关类型 ──

export type ProposalListItem = {
  proposalId: number;
  displayId: string;
  kind: number;
  kindLabel: string;
  stage: number;
  stageLabel: string;
  status: number;
  statusLabel: string;
  institutionName: string | null;
  summary: string;
};

export type ProposalPageResult = {
  items: ProposalListItem[];
  hasMore: boolean;
  warning: string | null;
};

export type VoteTally = {
  yes: number;
  no: number;
};

export type ProposalMeta = {
  proposalId: number;
  kind: number;
  stage: number;
  status: number;
  internalOrg: number | null;
  institutionHex: string | null;
};

export type TransferProposalDetail = {
  proposalId: number;
  institutionHex: string;
  beneficiaryHex: string;
  amountFen: string;
  remark: string;
  proposerHex: string;
};

export type RuntimeUpgradeDetail = {
  proposalId: number;
  proposerHex: string;
  reason: string;
  codeHashHex: string;
  status: number;
};

export type FeeRateProposalDetail = {
  proposalId: number;
  institutionHex: string;
  newRateBp: number;
};

export type SweepProposalDetail = {
  proposalId: number;
  institutionHex: string;
  amountFen: string;
};

export type SafetyFundProposalDetail = {
  proposalId: number;
  beneficiaryHex: string;
  amountFen: string;
  remark: string;
};

export type ProposalFullInfo = {
  meta: ProposalMeta;
  transferDetail: TransferProposalDetail | null;
  runtimeUpgradeDetail: RuntimeUpgradeDetail | null;
  feeRateDetail: FeeRateProposalDetail | null;
  safetyFundDetail: SafetyFundProposalDetail | null;
  sweepDetail: SweepProposalDetail | null;
  internalTally: VoteTally | null;
  jointTally: VoteTally | null;
  citizenTally: { yes: number; no: number } | null;
  institutionName: string | null;
};
