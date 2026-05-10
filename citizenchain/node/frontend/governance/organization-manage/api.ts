import { invoke } from '../../core/tauri';
import type { VoteSignRequestResult, VoteSubmitResult } from '../types';
import type {
  EligibleClearingBankCandidate,
  InitialAccountInputDto,
  InstitutionDetail,
  InstitutionProposalPage,
  InstitutionRegistrationInfoResp,
} from './types';

// 机构多签管理专用 Tauri API。offchain/api.ts 不再承载 OrganizationManage 业务命令。
export const organizationManageApi = {
  searchEligibleClearingBanks: (query: string, limit?: number) =>
    invoke<EligibleClearingBankCandidate[]>('search_eligible_clearing_banks', { query, limit }),

  fetchInstitutionDetail: (sfidNumber: string) =>
    invoke<InstitutionDetail | null>('fetch_clearing_bank_institution_detail', { sfidNumber }),

  fetchInstitutionProposals: (sfidNumber: string, startId: number, pageSize: number) =>
    invoke<InstitutionProposalPage>('fetch_clearing_bank_institution_proposals', {
      sfidNumber,
      startId,
      pageSize,
    }),

  fetchInstitutionRegistrationInfo: (sfidNumber: string) =>
    invoke<InstitutionRegistrationInfoResp>(
      'fetch_clearing_bank_institution_registration_info',
      { sfidNumber },
    ),

  buildProposeCreateInstitutionRequest: (params: {
    pubkeyHex: string;
    sfidNumber: string;
    institutionName: string;
    accounts: InitialAccountInputDto[];
    adminOrg: number;
    adminPubkeys: string[];
    threshold: number;
    registerNonce: string;
    signatureHex: string;
    signingProvince: string;
    signerAdminPubkey: string;
  }) =>
    invoke<VoteSignRequestResult>('build_propose_create_institution_request', {
      pubkeyHex: params.pubkeyHex,
      sfidNumber: params.sfidNumber,
      institutionName: params.institutionName,
      accounts: params.accounts,
      adminOrg: params.adminOrg,
      adminPubkeys: params.adminPubkeys,
      threshold: params.threshold,
      registerNonce: params.registerNonce,
      signatureHex: params.signatureHex,
      signingProvince: params.signingProvince,
      signerAdminPubkey: params.signerAdminPubkey,
    }),

  submitProposeCreateInstitution: (params: {
    requestId: string;
    expectedPubkeyHex: string;
    expectedPayloadHash: string;
    sfidNumber: string;
    institutionName: string;
    accounts: InitialAccountInputDto[];
    adminOrg: number;
    adminPubkeys: string[];
    threshold: number;
    registerNonce: string;
    signatureHex: string;
    signingProvince: string;
    signerAdminPubkey: string;
    signNonce: number;
    signBlockNumber: number;
    responseJson: string;
  }) =>
    invoke<VoteSubmitResult>('submit_propose_create_institution', {
      requestId: params.requestId,
      expectedPubkeyHex: params.expectedPubkeyHex,
      expectedPayloadHash: params.expectedPayloadHash,
      sfidNumber: params.sfidNumber,
      institutionName: params.institutionName,
      accounts: params.accounts,
      adminOrg: params.adminOrg,
      adminPubkeys: params.adminPubkeys,
      threshold: params.threshold,
      registerNonce: params.registerNonce,
      signatureHex: params.signatureHex,
      signingProvince: params.signingProvince,
      signerAdminPubkey: params.signerAdminPubkey,
      signNonce: params.signNonce,
      signBlockNumber: params.signBlockNumber,
      responseJson: params.responseJson,
    }),
};
