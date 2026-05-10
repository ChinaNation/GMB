import { invoke } from '../core/tauri';
import type {
  GovernanceOverview,
  InstitutionDetail,
  ProposalDisplayMeta,
  ProposalFullInfo,
  ProposalListItem,
  ProposalPageResult,
  UserVoteStatus,
  VoteSignRequestResult,
  VoteSubmitResult,
} from './types';

// 治理模块专用 Tauri API，对齐后端 src/governance。
export const governanceApi = {
  getGovernanceOverview: () => invoke<GovernanceOverview>('get_governance_overview'),
  getInstitutionDetail: (sfidNumber: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { sfidNumber }),
  startGovernanceBalanceWatch: (sfidNumber: string) =>
    invoke<void>('start_governance_balance_watch', { sfidNumber }),
  stopGovernanceBalanceWatch: (sfidNumber: string) =>
    invoke<void>('stop_governance_balance_watch', { sfidNumber }),
  getProposalPage: (startId: number, count: number) =>
    invoke<ProposalPageResult>('get_proposal_page', { startId, count }),
  getProposalDetail: (proposalId: number) =>
    invoke<ProposalFullInfo>('get_proposal_detail', { proposalId }),
  getNextProposalId: () => invoke<number>('get_next_proposal_id'),
  getInstitutionProposals: (sfidNumber: string) =>
    invoke<ProposalListItem[]>('get_institution_proposals', { sfidNumber }),
  getInstitutionProposalPage: (sfidNumber: string, startId: number, count: number) =>
    invoke<ProposalPageResult>('get_institution_proposal_page', { sfidNumber, startId, count }),
  // 双层 ID + 反向索引(spec_version v1)
  getProposalDisplay: (proposalId: number) =>
    invoke<ProposalDisplayMeta | null>('get_proposal_display', { proposalId }),
  listProposalsByOrg: (org: number) =>
    invoke<number[]>('list_proposals_by_org', { org }),
  listProposalsByInstitution: (sfidNumber: string) =>
    invoke<number[]>('list_proposals_by_institution', { sfidNumber }),
  listProposalsByOwner: (moduleTagScaleHex: string) =>
    invoke<number[]>('list_proposals_by_owner', { moduleTagScaleHex }),
  buildVoteRequest: (proposalId: number, pubkeyHex: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_vote_request', { proposalId, pubkeyHex, approve }),
  buildJointVoteRequest: (proposalId: number, pubkeyHex: string, sfidNumber: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_joint_vote_request', { proposalId, pubkeyHex, sfidNumber, approve }),
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
  checkVoteStatus: (proposalId: number, pubkeyHex: string, sfidNumber?: string) =>
    invoke<UserVoteStatus>('check_vote_status', { proposalId, pubkeyHex, sfidNumber: sfidNumber ?? null }),
};
