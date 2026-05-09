import { invoke } from '../core/tauri';
import type { VoteSignRequestResult, VoteSubmitResult } from '../governance/types';
import type {
  ClearingBankNodeOnChainInfo,
  ConnectivityTestReport,
  DecryptAdminRequestResult,
  DecryptedAdminInfo,
} from './types';

// 清算行 offchain 网络专用 Tauri API。机构多签命令归 governance/organization-manage/api.ts。
export const offchainApi = {
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
};
