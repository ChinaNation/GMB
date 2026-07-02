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
  getInstitutionDetail: (cidNumber: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { cidNumber }),
  startGovernanceBalanceWatch: (cidNumber: string) =>
    invoke<void>('start_governance_balance_watch', { cidNumber }),
  stopGovernanceBalanceWatch: (cidNumber: string) =>
    invoke<void>('stop_governance_balance_watch', { cidNumber }),
  getProposalPage: (startId: number, count: number) =>
    invoke<ProposalPageResult>('get_proposal_page', { startId, count }),
  getProposalDetail: (proposalId: number) =>
    invoke<ProposalFullInfo>('get_proposal_detail', { proposalId }),
  getNextProposalId: () => invoke<number>('get_next_proposal_id'),
  getInstitutionProposals: (cidNumber: string) =>
    invoke<ProposalListItem[]>('get_institution_proposals', { cidNumber }),
  getInstitutionProposalPage: (cidNumber: string, startId: number, count: number) =>
    invoke<ProposalPageResult>('get_institution_proposal_page', { cidNumber, startId, count }),
  // 双层 ID + 反向索引(spec_version v1)
  getProposalDisplay: (proposalId: number) =>
    invoke<ProposalDisplayMeta | null>('get_proposal_display', { proposalId }),
  listProposalsByInstitutionCode: (institutionCode: string) =>
    invoke<number[]>('list_proposals_by_institution_code', { institutionCode }),
  listProposalsByCid: (cidNumber: string) =>
    invoke<number[]>('list_proposals_by_cid', { cidNumber }),
  listProposalsByOwner: (moduleTagScaleHex: string) =>
    invoke<number[]>('list_proposals_by_owner', { moduleTagScaleHex }),
  buildVoteRequest: (proposalId: number, pubkeyHex: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_vote_request', { proposalId, pubkeyHex, approve }),
  buildJointVoteRequest: (proposalId: number, pubkeyHex: string, cidNumber: string, approve: boolean) =>
    invoke<VoteSignRequestResult>('build_joint_vote_request', { proposalId, pubkeyHex, cidNumber, approve }),
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
  checkVoteStatus: (proposalId: number, pubkeyHex: string, cidNumber?: string) =>
    invoke<UserVoteStatus>('check_vote_status', { proposalId, pubkeyHex, cidNumber: cidNumber ?? null }),
};
