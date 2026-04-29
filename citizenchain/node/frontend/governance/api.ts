import { invoke } from '../core/tauri';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  GovernanceOverview,
  InstitutionDetail,
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
  getInstitutionDetail: (shenfenId: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { shenfenId }),
  startGovernanceBalanceWatch: (shenfenId: string) =>
    invoke<void>('start_governance_balance_watch', { shenfenId }),
  stopGovernanceBalanceWatch: (shenfenId: string) =>
    invoke<void>('stop_governance_balance_watch', { shenfenId }),
  getProposalPage: (startId: number, count: number) =>
    invoke<ProposalPageResult>('get_proposal_page', { startId, count }),
  getProposalDetail: (proposalId: number) =>
    invoke<ProposalFullInfo>('get_proposal_detail', { proposalId }),
  getNextProposalId: () => invoke<number>('get_next_proposal_id'),
  getInstitutionProposals: (shenfenId: string) =>
    invoke<ProposalListItem[]>('get_institution_proposals', { shenfenId }),
  getInstitutionProposalPage: (shenfenId: string, startId: number, count: number) =>
    invoke<ProposalPageResult>('get_institution_proposal_page', { shenfenId, startId, count }),
  buildActivateAdminRequest: (pubkeyHex: string, shenfenId: string) =>
    invoke<ActivateRequestResult>('build_activate_admin_request', { pubkeyHex, shenfenId }),
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
    pubkeyHex: string,
    shenfenId: string,
    orgType: number,
    beneficiaryAddress: string,
    amountYuan: number,
    remark: string,
  ) =>
    invoke<VoteSignRequestResult>('build_propose_transfer_request', {
      pubkeyHex,
      shenfenId,
      orgType,
      beneficiaryAddress,
      amountYuan,
      remark,
    }),
  submitProposeTransfer: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    shenfenId: string,
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
      shenfenId,
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
  buildProposeSweepRequest: (pubkeyHex: string, shenfenId: string, amountYuan: number) =>
    invoke<VoteSignRequestResult>('build_propose_sweep_request', { pubkeyHex, shenfenId, amountYuan }),
  submitProposeSweep: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    shenfenId: string,
    amountYuan: number,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_propose_sweep', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      shenfenId,
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
