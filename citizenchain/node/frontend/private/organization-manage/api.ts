import { invoke } from '../../core/tauri';
import type { VoteSignRequestResult, VoteSubmitResult } from '../../governance/types';
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

  fetchInstitutionDetail: (cidNumber: string) =>
    invoke<InstitutionDetail | null>('fetch_clearing_bank_institution_detail', { cidNumber }),

  fetchInstitutionProposals: (cidNumber: string, startId: number, pageSize: number) =>
    invoke<InstitutionProposalPage>('fetch_clearing_bank_institution_proposals', {
      cidNumber,
      startId,
      pageSize,
    }),

  fetchInstitutionRegistrationInfo: (cidNumber: string) =>
    invoke<InstitutionRegistrationInfoResp>(
      'fetch_clearing_bank_institution_registration_info',
      { cidNumber },
    ),

  buildProposeCreateInstitutionRequest: (params: {
    pubkeyHex: string;
    cidNumber: string;
    cidFullName: string;
    accounts: InitialAccountInputDto[];
    institutionCode: string;
    admins: string[];
    threshold: number;
    registerNonce: string;
    signatureHex: string;
    issuerCidNumber: string;
    issuerMainAccount: string;
    signerPubkey: string;
    scopeProvinceName: string;
    scopeCityName: string;
  }) =>
    invoke<VoteSignRequestResult>('build_propose_create_institution_request', {
      pubkeyHex: params.pubkeyHex,
      cidNumber: params.cidNumber,
      cidFullName: params.cidFullName,
      accounts: params.accounts,
      institutionCode: params.institutionCode,
      admins: params.admins,
      threshold: params.threshold,
      registerNonce: params.registerNonce,
      signatureHex: params.signatureHex,
      issuerCidNumber: params.issuerCidNumber,
      issuerMainAccount: params.issuerMainAccount,
      signerPubkey: params.signerPubkey,
      scopeProvinceName: params.scopeProvinceName,
      scopeCityName: params.scopeCityName,
    }),

  submitProposeCreateInstitution: (params: {
    requestId: string;
    expectedPubkeyHex: string;
    expectedPayloadHash: string;
    cidNumber: string;
    cidFullName: string;
    accounts: InitialAccountInputDto[];
    institutionCode: string;
    admins: string[];
    threshold: number;
    registerNonce: string;
    signatureHex: string;
    issuerCidNumber: string;
    issuerMainAccount: string;
    signerPubkey: string;
    scopeProvinceName: string;
    scopeCityName: string;
    signNonce: number;
    signBlockNumber: number;
    responseJson: string;
  }) =>
    invoke<VoteSubmitResult>('submit_propose_create_institution', {
      requestId: params.requestId,
      expectedPubkeyHex: params.expectedPubkeyHex,
      expectedPayloadHash: params.expectedPayloadHash,
      cidNumber: params.cidNumber,
      cidFullName: params.cidFullName,
      accounts: params.accounts,
      institutionCode: params.institutionCode,
      admins: params.admins,
      threshold: params.threshold,
      registerNonce: params.registerNonce,
      signatureHex: params.signatureHex,
      issuerCidNumber: params.issuerCidNumber,
      issuerMainAccount: params.issuerMainAccount,
      signerPubkey: params.signerPubkey,
      scopeProvinceName: params.scopeProvinceName,
      scopeCityName: params.scopeCityName,
      signNonce: params.signNonce,
      signBlockNumber: params.signBlockNumber,
      responseJson: params.responseJson,
    }),
};
