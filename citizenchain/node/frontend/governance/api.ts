import { invoke } from '../core/tauri';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  GovernanceOverview,
  InstitutionDetail,
  ProposalDisplayMeta,
  ProposalFullInfo,
  ProposalListItem,
  ProposalPageResult,
  ProposeUpgradeRequestResult,
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
  buildActivateAdminRequest: (pubkeyHex: string, sfidNumber: string) =>
    invoke<ActivateRequestResult>('build_activate_admin_request', { pubkeyHex, sfidNumber }),
  verifyActivateAdmin: (
    requestId: string,
    pubkeyHex: string,
    expectedPayloadHash: string,
    payloadHex: string,
    responseJson: string,
  ) =>
    invoke<ActivatedAdmin>('verify_activate_admin', {
      requestId,
      pubkeyHex,
      expectedPayloadHash,
      payloadHex,
      responseJson,
    }),
  getActivatedAdmins: (sfidNumber: string) =>
    invoke<ActivatedAdmin[]>('get_activated_admins', { sfidNumber }),
  deactivateAdmin: (pubkeyHex: string, sfidNumber: string, unlockPassword: string) =>
    invoke<void>('deactivate_admin', { pubkeyHex, sfidNumber, unlockPassword }),
  hasAnyActivatedAdmin: () => invoke<boolean>('has_any_activated_admin'),
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
  buildProposeTransferRequest: (
    pubkeyHex: string,
    sfidNumber: string,
    orgType: number,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_propose_transfer_request', {
      pubkeyHex,
      sfidNumber,
      orgType,
      beneficiaryAddress,
      amountYuan,
      remark,
    }),
  submitProposeTransfer: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidNumber: string,
    orgType: number,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_transfer', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      sfidNumber,
      orgType,
      beneficiaryAddress,
      amountYuan,
      remark,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildProposeUpgradeRequest: (pubkeyHex: string, wasmPath: string, reason: string) =>
    invoke<ProposeUpgradeRequestResult>('build_propose_upgrade_request', { pubkeyHex, wasmPath, reason }),
  submitProposeUpgrade: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    wasmPath: string,
    reason: string,
    eligibleTotal: number,
    snapshotNonce: string,
    snapshotSignature: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_upgrade', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      wasmPath,
      reason,
      eligibleTotal,
      snapshotNonce,
      snapshotSignature,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildDeveloperUpgradeRequest: (pubkeyHex: string, wasmPath: string) =>
    invoke<VoteSignRequestResult>('build_developer_upgrade_request', { pubkeyHex, wasmPath }),
  submitDeveloperUpgrade: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    wasmPath: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_developer_upgrade', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      wasmPath,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildProposeSweepRequest: (pubkeyHex: string, sfidNumber: string, amountYuan: number) =>
    invoke<VoteSignRequestResult>('build_propose_sweep_request', { pubkeyHex, sfidNumber, amountYuan }),
  submitProposeSweep: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidNumber: string,
    amountYuan: number,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_sweep', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      sfidNumber,
      amountYuan,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
  buildProposeSafetyFundRequest: (
    pubkeyHex: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_propose_safety_fund_request', {
      pubkeyHex,
      beneficiaryAddress,
      amountYuan,
      remark,
    }),
  submitProposeSafetyFund: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_safety_fund', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      beneficiaryAddress,
      amountYuan,
      remark,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
