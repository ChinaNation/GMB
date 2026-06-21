import { invoke } from '../../core/tauri';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  AdminAccountRef,
  AdminAccountState,
  InstitutionDetail,
  VoteSignRequestResult,
  VoteSubmitResult,
} from './types';

// 管理员更换模块前端 API。激活、更换与账户读取统一聚合到 admins_change。
const accountRefParams = (accountRef: AdminAccountRef) => ({
  sfidNumber: accountRef.sfidNumber ?? null,
  accountHex: accountRef.accountHex ?? null,
  expectedOrg: accountRef.org ?? null,
});

export const adminsChangeApi = {
  getInstitutionDetail: (sfidNumber: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { sfidNumber }),
  buildActivateAdminRequest: (pubkeyHex: string, sfidNumber: string, accountRef?: AdminAccountRef) =>
    invoke<ActivateRequestResult>('build_activate_admin_request', {
      pubkeyHex,
      sfidNumber,
      accountHex: accountRef?.accountHex ?? null,
      expectedOrg: accountRef?.org ?? null,
    }),
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
  getActivatedAdmins: (sfidNumber: string, accountRef?: AdminAccountRef) =>
    invoke<ActivatedAdmin[]>('get_activated_admins', {
      sfidNumber,
      accountHex: accountRef?.accountHex ?? null,
      expectedOrg: accountRef?.org ?? null,
    }),
  deactivateAdmin: (pubkeyHex: string, sfidNumber: string, accountRef: AdminAccountRef, unlockPassword: string) =>
    invoke<void>('deactivate_admin', {
      pubkeyHex,
      sfidNumber,
      accountHex: accountRef.accountHex ?? null,
      expectedOrg: accountRef.org ?? null,
      unlockPassword,
    }),
  hasAnyActivatedAdmin: () => invoke<boolean>('has_any_activated_admin'),
  getAdminAccountState: (accountRef: AdminAccountRef) =>
    invoke<AdminAccountState | null>('get_admin_account_state', accountRefParams(accountRef)),
  buildAdminSetChangeRequest: (pubkeyHex: string, accountRef: AdminAccountRef, admins: string[]) =>
    invoke<VoteSignRequestResult>('build_admin_set_change_request', {
      pubkeyHex,
      ...accountRefParams(accountRef),
      admins,
    }),
  submitAdminSetChange: (
    requestId: string,
    expectedPubkeyHex: string,
    expectedPayloadHash: string,
    accountRef: AdminAccountRef,
    admins: string[],
    signNonce: number,
    signBlockNumber: number,
    responseJson: string,
  ) =>
    invoke<VoteSubmitResult>('submit_admin_set_change', {
      requestId,
      expectedPubkeyHex,
      expectedPayloadHash,
      ...accountRefParams(accountRef),
      admins,
      signNonce,
      signBlockNumber,
      responseJson,
    }),
};
