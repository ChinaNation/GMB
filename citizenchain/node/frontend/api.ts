import { invoke } from '@tauri-apps/api/core';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  AdminWalletMatch,
  GovernanceOverview,
  ProposeUpgradeRequestResult,
  SigningAdminInfo,
  UserVoteStatus,
  VoteSignRequestResult,
  VoteSubmitResult,
  InstitutionDetail,
  ProposalFullInfo,
  ProposalListItem,
  ProposalPageResult,
} from './governance/governance-types';
import type {
  BootnodeKey,
  BootnodeOption,
  ChainStatus,
  GrandpaKey,
  MiningDashboard,
  NetworkOverview,
  NodeIdentity,
  NodeStatus,
  OtherTabsPayload,
  RewardWallet,
  TotalIssuance,
  TotalStake,
} from './types';

const ERROR_MAX_LENGTH = 500;

/** 提取错误消息并截断，防止超长或异常内容影响 UI。 */
export function sanitizeError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  return raw.length > ERROR_MAX_LENGTH
    ? raw.slice(0, ERROR_MAX_LENGTH) + '…'
    : raw;
}

// 统一封装所有 Tauri 命令调用，避免组件里散落 invoke 字符串。
export const api = {
  getNodeStatus: () => invoke<NodeStatus>('get_node_status'),
  startNode: (unlockPassword: string) =>
    invoke<NodeStatus>('start_node', { unlockPassword }),
  stopNode: (unlockPassword: string) =>
    invoke<NodeStatus>('stop_node', { unlockPassword }),

  getRewardWallet: () => invoke<RewardWallet>('get_reward_wallet'),
  setRewardWallet: (address: string, unlockPassword: string) =>
    invoke<RewardWallet>('set_reward_wallet', { address, unlockPassword }),

  getBootnodeKey: () => invoke<BootnodeKey>('get_bootnode_key'),
  getGrandpaKey: () => invoke<GrandpaKey>('get_grandpa_key'),
  setBootnodeKey: (nodeKey: string, unlockPassword: string) =>
    invoke<BootnodeKey>('set_bootnode_key', { nodeKey, unlockPassword }),
  setGrandpaKey: (key: string, unlockPassword: string) =>
    invoke<GrandpaKey>('set_grandpa_key', { key, unlockPassword }),
  getGenesisBootnodeOptions: () =>
    invoke<BootnodeOption[]>('get_genesis_bootnode_options'),

  getChainStatus: () => invoke<ChainStatus>('get_chain_status'),
  getNodeIdentity: () => invoke<NodeIdentity>('get_node_identity'),
  getTotalIssuance: () => invoke<TotalIssuance>('get_total_issuance'),
  getTotalStake: () => invoke<TotalStake>('get_total_stake'),
  getMiningDashboard: () => invoke<MiningDashboard>('get_mining_dashboard'),
  getNetworkOverview: () => invoke<NetworkOverview>('get_network_overview'),
  getOtherTabsContent: () => invoke<OtherTabsPayload>('get_other_tabs_content'),
  getGovernanceOverview: () => invoke<GovernanceOverview>('get_governance_overview'),
  getInstitutionDetail: (shenfenId: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { shenfenId }),
  getProposalPage: (startId: number, count: number) =>
    invoke<ProposalPageResult>('get_proposal_page', { startId, count }),
  getProposalDetail: (proposalId: number) =>
    invoke<ProposalFullInfo>('get_proposal_detail', { proposalId }),
  getNextProposalId: () => invoke<number>('get_next_proposal_id'),
  getInstitutionProposals: (shenfenId: string) =>
    invoke<ProposalListItem[]>('get_institution_proposals', { shenfenId }),
  getInstitutionProposalPage: (shenfenId: string, startId: number, count: number) =>
    invoke<ProposalPageResult>('get_institution_proposal_page', { shenfenId, startId, count }),
  setSigningAdmin: (pubkeyHex: string, privateKeyHex: string, unlockPassword: string) =>
    invoke<SigningAdminInfo | null>('set_signing_admin', { pubkeyHex, privateKeyHex, unlockPassword }),
  getSigningAdmin: () => invoke<SigningAdminInfo | null>('get_signing_admin'),
  // 管理员激活
  buildActivateAdminRequest: (pubkeyHex: string, shenfenId: string) =>
    invoke<ActivateRequestResult>('build_activate_admin_request', { pubkeyHex, shenfenId }),
  verifyActivateAdmin: (
    requestId: string, pubkeyHex: string, expectedPayloadHash: string,
    payloadHex: string, responseJson: string,
  ) =>
    invoke<ActivatedAdmin>('verify_activate_admin', {
      requestId, pubkeyHex, expectedPayloadHash, payloadHex, responseJson,
    }),
  getActivatedAdmins: (shenfenId: string) =>
    invoke<ActivatedAdmin[]>('get_activated_admins', { shenfenId }),
  deactivateAdmin: (pubkeyHex: string, shenfenId: string, unlockPassword: string) =>
    invoke<void>('deactivate_admin', { pubkeyHex, shenfenId, unlockPassword }),
  hasAnyActivatedAdmin: () => invoke<boolean>('has_any_activated_admin'),
  buildVoteRequest: (proposalId: number, pubkeyHex: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_vote_request', { proposalId, pubkeyHex, approve }),
  buildJointVoteRequest: (proposalId: number, pubkeyHex: string, shenfenId: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_joint_vote_request', { proposalId, pubkeyHex, shenfenId, approve }),
  submitVote: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    callDataHex: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_vote', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      callDataHex,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  checkVoteStatus: (proposalId: number, pubkeyHex: string, shenfenId?: string) =>
    invoke<UserVoteStatus>('check_vote_status', { proposalId, pubkeyHex, shenfenId: shenfenId ?? null }),
  buildProposeTransferRequest: (
    pubkeyHex: string, shenfenId: string, orgType: number,
    beneficiaryAddress: string, amountYuan: number, remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_propose_transfer_request', {
      pubkeyHex, shenfenId, orgType, beneficiaryAddress, amountYuan, remark,
    }),
  submitProposeTransfer: (
    requestId: string, expectedPubkeyHex: string, expectedPayloadHash: string,
    shenfenId: string, orgType: number, beneficiaryAddress: string,
    amountYuan: number, remark: string,
    signNonce: number, signBlockNumber: number, responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_transfer', {
      requestId, expectedPubkeyHex, expectedPayloadHash,
      shenfenId, orgType, beneficiaryAddress, amountYuan, remark,
      signNonce, signBlockNumber, responseJson,
    }),
  buildProposeUpgradeRequest: (pubkeyHex: string, wasmPath: string, reason: string) =>
    invoke<ProposeUpgradeRequestResult>('build_propose_upgrade_request', { pubkeyHex, wasmPath, reason }),
  submitProposeUpgrade: (
    requestId: string, expectedPubkeyHex: string, expectedPayloadHash: string,
    wasmPath: string, reason: string, eligibleTotal: number,
    snapshotNonce: string, snapshotSignature: string,
    signNonce: number, signBlockNumber: number, responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_upgrade', {
      requestId, expectedPubkeyHex, expectedPayloadHash,
      wasmPath, reason, eligibleTotal, snapshotNonce, snapshotSignature,
      signNonce, signBlockNumber, responseJson,
    }),
  buildDeveloperUpgradeRequest: (pubkeyHex: string, wasmPath: string) =>
    invoke<VoteSignRequestResult>('build_developer_upgrade_request', { pubkeyHex, wasmPath }),
  submitDeveloperUpgrade: (
    requestId: string, expectedPubkeyHex: string, expectedPayloadHash: string,
    wasmPath: string, signNonce: number, signBlockNumber: number, responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_developer_upgrade', {
      requestId, expectedPubkeyHex, expectedPayloadHash,
      wasmPath, signNonce, signBlockNumber, responseJson,
    }),
  buildProposeSweepRequest: (pubkeyHex: string, shenfenId: string, amountYuan: number) =>
    invoke<VoteSignRequestResult>('build_propose_sweep_request', { pubkeyHex, shenfenId, amountYuan }),
  submitProposeSweep: (
    requestId: string, expectedPubkeyHex: string, expectedPayloadHash: string,
    shenfenId: string, amountYuan: number,
    signNonce: number, signBlockNumber: number, responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_sweep', {
      requestId, expectedPubkeyHex, expectedPayloadHash,
      shenfenId, amountYuan, signNonce, signBlockNumber, responseJson,
    }),
  buildSweepVoteRequest: (proposalId: number, pubkeyHex: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_sweep_vote_request', { proposalId, pubkeyHex, approve }),
  buildProposeSafetyFundRequest: (
    pubkeyHex: string, beneficiaryAddress: string, amountYuan: number, remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_propose_safety_fund_request', {
      pubkeyHex, beneficiaryAddress, amountYuan, remark,
    }),
  submitProposeSafetyFund: (
    requestId: string, expectedPubkeyHex: string, expectedPayloadHash: string,
    beneficiaryAddress: string, amountYuan: number, remark: string,
    signNonce: number, signBlockNumber: number, responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_safety_fund', {
      requestId, expectedPubkeyHex, expectedPayloadHash,
      beneficiaryAddress, amountYuan, remark,
      signNonce, signBlockNumber, responseJson,
    }),
  buildSafetyFundVoteRequest: (proposalId: number, pubkeyHex: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_safety_fund_vote_request', { proposalId, pubkeyHex, approve }),
  buildRateVoteRequest: (proposalId: number, pubkeyHex: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_rate_vote_request', { proposalId, pubkeyHex, approve }),
  queryInstitutionRateBp: (shenfenId: string) =>
    invoke<number>('query_institution_rate_bp', { shenfenId }),
  buildProposeRateRequest: (pubkeyHex: string, shenfenId: string, newRateBp: number) =>
    invoke<VoteSignRequestResult>('build_propose_rate_request', { pubkeyHex, shenfenId, newRateBp }),
  submitProposeRate: (
    requestId: string, expectedPubkeyHex: string, expectedPayloadHash: string,
    shenfenId: string, newRateBp: number,
    signNonce: number, signBlockNumber: number, responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_rate', {
      requestId, expectedPubkeyHex, expectedPayloadHash,
      shenfenId, newRateBp, signNonce, signBlockNumber, responseJson,
    }),
};
