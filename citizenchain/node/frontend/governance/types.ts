// 治理模块前端类型定义，与后端 governance::types 和 admins_change::activation 对应。
import type { DuoqianTransferProposalDetails } from '../transaction/duoqian-transfer/types';

// ── 签名请求/响应 ──

export type VoteSignRequestResult = {
  requestJson: string;
  callDataHex: string;
  requestId: string;
  expectedPayloadHash: string;
  signNonce: number;
  signBlockNumber: number;
};

export type VoteSubmitResult = {
  txHash: string;
};

// ── 投票状态 ──

export type UserVoteStatus = {
  proposalId: number;
  kind: number;
  stage: number;
  internalVote: boolean | null;
  jointVote: boolean | null;
};

// ── 管理员匹配（提案发起/投票签名流程共用） ──

export type AdminWalletMatch = {
  address: string;
  pubkeyHex: string;
  name: string;
};

// ── 管理员激活 ──

export type ActivatedAdmin = {
  pubkeyHex: string;
  accountHex: string;
  org: number;
  kind: number;
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
  sfidNumber: string;
  orgType: number;
  orgTypeLabel: string;
  mainAccount: string;
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
  sfidNumber: string;
  orgType: number;
  orgTypeLabel: string;
  mainAccount: string;
  balanceFen: string | null;
  admins: AdminInfo[];
  internalThreshold: number;
  jointVoteWeight: number;
  stakeAccount: string | null;
  stakingBalanceFen: string | null;
  feeAccount: string | null;
  feeBalanceFen: string | null;
  cbFeeAccount: string | null;
  cbFeeBalanceFen: string | null;
  nrcFeeAccount: string | null;
  nrcFeeBalanceFen: string | null;
  nrcAnquanAccount: string | null;
  nrcAnquanBalanceFen: string | null;
  warning: string | null;
};

export type InstitutionBalanceUpdate = {
  sfidNumber: string;
  balanceFen: string | null;
  stakingBalanceFen: string | null;
  feeBalanceFen: string | null;
  cbFeeBalanceFen: string | null;
  nrcFeeBalanceFen: string | null;
  nrcAnquanBalanceFen: string | null;
  warning: string | null;
};

// ── 提案相关类型 ──

/// 双层 ID v1:展示号反查值。主键 `proposalId` 与展示号解耦。
export type ProposalDisplayMeta = {
  year: number;
  seqInYear: number;
};

export type ProposalListItem = {
  proposalId: number;
  displayId: string;
  kind: number;
  kindLabel: string;
  stage: number;
  stageLabel: string;
  status: number;
  statusLabel: string;
  sfidFullName: string | null;
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

export type RuntimeUpgradeDetail = {
  proposalId: number;
  proposerHex: string;
  reason: string;
  codeHashHex: string;
};

export type ProposalFullInfo = DuoqianTransferProposalDetails & {
  meta: ProposalMeta;
  runtimeUpgradeDetail: RuntimeUpgradeDetail | null;
  internalTally: VoteTally | null;
  jointTally: VoteTally | null;
  citizenTally: { yes: number; no: number } | null;
  sfidFullName: string | null;
};
