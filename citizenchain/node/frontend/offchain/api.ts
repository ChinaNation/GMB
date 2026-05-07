import { invoke } from '../core/tauri';
import type { VoteSignRequestResult, VoteSubmitResult } from '../governance/types';
import type {
  ClearingBankNodeOnChainInfo,
  ConnectivityTestReport,
  DecryptAdminRequestResult,
  DecryptedAdminInfo,
  EligibleClearingBankCandidate,
  InitialAccountInputDto,
  InstitutionDetail,
  InstitutionProposalPage,
  InstitutionRegistrationInfoResp,
} from './types';

// 清算行 offchain 页面专用 Tauri API。全局 api.ts 不再承载清算行业务命令。
export const offchainApi = {
  searchEligibleClearingBanks: (query: string, limit?: number) =>
    invoke<EligibleClearingBankCandidate[]>('search_eligible_clearing_banks', { query, limit }),

  queryClearingBankNodeInfo: (sfidNumber: string) =>
    invoke<ClearingBankNodeOnChainInfo | null>('query_clearing_bank_node_info', { sfidNumber }),

  queryLocalPeerId: () => invoke<string>('query_local_peer_id'),

  testClearingBankEndpointConnectivity: (domain: string, port: number, expectedPeerId: string) =>
    invoke<ConnectivityTestReport>('test_clearing_bank_endpoint_connectivity', {
      domain,
      port,
      expectedPeerId,
    }),

  buildRegisterClearingBankRequest: (
    pubkeyHex: string,
    sfidNumber: string,
    peerId: string,
    rpcDomain: string,
    rpcPort: number,
  ) =>
    invoke<VoteSignRequestResult>('build_register_clearing_bank_request', {
      pubkeyHex,
      sfidNumber,
      peerId,
      rpcDomain,
      rpcPort,
    }),

  submitRegisterClearingBank: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidNumber: string,
    peerId: string,
    rpcDomain: string,
    rpcPort: number,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_register_clearing_bank', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      sfidNumber,
      peerId,
      rpcDomain,
      rpcPort,
      signNonce,
      signBlockNumber,
      responseJson,
    }),

  buildUpdateClearingBankEndpointRequest: (
    pubkeyHex: string,
    sfidNumber: string,
    newDomain: string,
    newPort: number,
  ) =>
    invoke<VoteSignRequestResult>('build_update_clearing_bank_endpoint_request', {
      pubkeyHex,
      sfidNumber,
      newDomain,
      newPort,
    }),

  submitUpdateClearingBankEndpoint: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidNumber: string,
    newDomain: string,
    newPort: number,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_update_clearing_bank_endpoint', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      sfidNumber,
      newDomain,
      newPort,
      signNonce,
      signBlockNumber,
      responseJson,
    }),

  buildUnregisterClearingBankRequest: (pubkeyHex: string, sfidNumber: string) =>
    invoke<VoteSignRequestResult>('build_unregister_clearing_bank_request', { pubkeyHex, sfidNumber }),

  submitUnregisterClearingBank: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidNumber: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_unregister_clearing_bank', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      sfidNumber,
      signNonce,
      signBlockNumber,
      responseJson,
    }),

  buildDecryptAdminRequest: (pubkeyHex: string, sfidNumber: string) =>
    invoke<DecryptAdminRequestResult>('build_decrypt_admin_request', { pubkeyHex, sfidNumber }),

  verifyAndDecryptAdmin: (
    requestId: string,
    pubkeyHex: string,
    expectedPayloadHash: string,
    responseJson: string,
  ) =>
    invoke<DecryptedAdminInfo>('verify_and_decrypt_admin', {
      requestId,
      pubkeyHex,
      expectedPayloadHash,
      responseJson,
    }),

  listDecryptedAdmins: (sfidNumber: string) =>
    invoke<DecryptedAdminInfo[]>('list_decrypted_admins', { sfidNumber }),

  lockDecryptedAdmin: (pubkeyHex: string) =>
    invoke<void>('lock_decrypted_admin', { pubkeyHex }),

  // ── 机构详情(链上 organization-manage::Institutions[sfid_number]) ──

  fetchInstitutionDetail: (sfidNumber: string) =>
    invoke<InstitutionDetail | null>('fetch_clearing_bank_institution_detail', { sfidNumber }),

  fetchInstitutionProposals: (sfidNumber: string, startId: number, pageSize: number) =>
    invoke<InstitutionProposalPage>('fetch_clearing_bank_institution_proposals', {
      sfidNumber,
      startId,
      pageSize,
    }),

  // ── 创建机构多签:拉 SFID registration-info + 构 extrinsic + 提交 ──

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
