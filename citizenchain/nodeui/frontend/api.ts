import { invoke } from '@tauri-apps/api/core';
import type {
  AdminWalletMatch,
  ColdWalletList,
  GovernanceOverview,
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
  setNodeName: (nodeName: string, unlockPassword: string) =>
    invoke<NodeIdentity>('set_node_name', { nodeName, unlockPassword }),
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
  getColdWallets: () => invoke<ColdWalletList>('get_cold_wallets'),
  addColdWallet: (address: string, name: string, unlockPassword: string) =>
    invoke<ColdWalletList>('add_cold_wallet', { address, name, unlockPassword }),
  removeColdWallet: (pubkeyHex: string, unlockPassword: string) =>
    invoke<ColdWalletList>('remove_cold_wallet', { pubkeyHex, unlockPassword }),
  checkAdminWallets: (shenfenId: string) =>
    invoke<AdminWalletMatch[]>('check_admin_wallets', { shenfenId }),
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
};
