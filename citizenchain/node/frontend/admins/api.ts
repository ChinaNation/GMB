import { invoke } from '../tauri';
import type {
  ActivateRequestResult,
  ActivatedAdmin,
  AdminAccountRef,
  AdminAccountState,
  InstitutionDetail,
} from './types';

// 管理员只读与本机激活 API；岗位任职变更由对应业务模块提交，不在 Node 直接改集合。
const accountRefParams = (accountRef: AdminAccountRef) => ({
  cidNumber: accountRef.cidNumber ?? null,
  accountHex: accountRef.accountHex ?? null,
  expectedInstitutionCode: accountRef.institutionCode ?? null,
});

export const adminsChangeApi = {
  getInstitutionDetail: (cidNumber: string) =>
    invoke<InstitutionDetail>('get_institution_detail', { cidNumber }),
  buildActivateAdminRequest: (pubkeyHex: string, cidNumber: string, accountRef?: AdminAccountRef) =>
    invoke<ActivateRequestResult>('build_activate_admin_request', {
      pubkeyHex,
      cidNumber,
      accountHex: accountRef?.accountHex ?? null,
      expectedInstitutionCode: accountRef?.institutionCode ?? null,
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
  getActivatedAdmins: (cidNumber: string, accountRef?: AdminAccountRef) =>
    invoke<ActivatedAdmin[]>('get_activated_admins', {
      cidNumber,
      accountHex: accountRef?.accountHex ?? null,
      expectedInstitutionCode: accountRef?.institutionCode ?? null,
    }),
  deactivateAdmin: (pubkeyHex: string, cidNumber: string, accountRef: AdminAccountRef, unlockPassword: string) =>
    invoke<void>('deactivate_admin', {
      pubkeyHex,
      cidNumber,
      accountHex: accountRef.accountHex ?? null,
      expectedInstitutionCode: accountRef.institutionCode ?? null,
      unlockPassword,
    }),
  hasAnyActivatedAdmin: () => invoke<boolean>('has_any_activated_admin'),
  getAdminAccountState: (accountRef: AdminAccountRef) =>
    invoke<AdminAccountState | null>('get_admin_account_state', accountRefParams(accountRef)),
  getAdminAccountBalances: (accountHexes: string[]) =>
    invoke<Record<string, string | null>>('get_admin_account_balances', { accountHexes }),
};
