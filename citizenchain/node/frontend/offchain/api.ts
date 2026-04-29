import { invoke } from '@tauri-apps/api/core';
import type { VoteSignRequestResult, VoteSubmitResult } from '../governance/governance-types';
import type {
  ClearingBankNodeOnChainInfo,
  ConnectivityTestReport,
  DecryptAdminRequestResult,
  DecryptedAdminInfo,
  EligibleClearingBankCandidate,
} from './types';

// 清算行 offchain 页面专用 Tauri API。全局 api.ts 不再承载清算行业务命令。
export const offchainApi = {
  searchEligibleClearingBanks: (query: string, limit?: number) =>
    invoke<EligibleClearingBankCandidate[]>('search_eligible_clearing_banks', { query, limit }),

  queryClearingBankNodeInfo: (sfidId: string) =>
    invoke<ClearingBankNodeOnChainInfo | null>('query_clearing_bank_node_info', { sfidId }),

  queryLocalPeerId: () => invoke<string>('query_local_peer_id'),

  testClearingBankEndpointConnectivity: (domain: string, port: number, expectedPeerId: string) =>
    invoke<ConnectivityTestReport>('test_clearing_bank_endpoint_connectivity', {
      domain,
      port,
      expectedPeerId,
    }),

  buildRegisterClearingBankRequest: (
    pubkeyHex: string,
    sfidId: string,
    peerId: string,
    rpcDomain: string,
    rpcPort: number,
  ) =>
    invoke<VoteSignRequestResult>('build_register_clearing_bank_request', {
      pubkeyHex,
      sfidId,
      peerId,
      rpcDomain,
      rpcPort,
    }),

  submitRegisterClearingBank: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidId: string,
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
      sfidId,
      peerId,
      rpcDomain,
      rpcPort,
      signNonce,
      signBlockNumber,
      responseJson,
    }),

  buildUpdateClearingBankEndpointRequest: (
    pubkeyHex: string,
    sfidId: string,
    newDomain: string,
    newPort: number,
  ) =>
    invoke<VoteSignRequestResult>('build_update_clearing_bank_endpoint_request', {
      pubkeyHex,
      sfidId,
      newDomain,
      newPort,
    }),

  submitUpdateClearingBankEndpoint: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidId: string,
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
      sfidId,
      newDomain,
      newPort,
      signNonce,
      signBlockNumber,
      responseJson,
    }),

  buildUnregisterClearingBankRequest: (pubkeyHex: string, sfidId: string) =>
    invoke<VoteSignRequestResult>('build_unregister_clearing_bank_request', { pubkeyHex, sfidId }),

  submitUnregisterClearingBank: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    sfidId: string,
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_unregister_clearing_bank', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      sfidId,
      signNonce,
      signBlockNumber,
      responseJson,
    }),

  buildDecryptAdminRequest: (pubkeyHex: string, sfidId: string) =>
    invoke<DecryptAdminRequestResult>('build_decrypt_admin_request', { pubkeyHex, sfidId }),

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

  listDecryptedAdmins: (sfidId: string) =>
    invoke<DecryptedAdminInfo[]>('list_decrypted_admins', { sfidId }),

  lockDecryptedAdmin: (pubkeyHex: string) =>
    invoke<void>('lock_decrypted_admin', { pubkeyHex }),
};
