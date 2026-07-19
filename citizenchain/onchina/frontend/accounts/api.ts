// 机构账户前端 API。账户创建、列表和删除都归 accounts 模块。

import type { AdminAuth } from '../auth/types';
import {
  createColdSignSubmitHeaders,
  securityGrantSubmitHeaders,
  type AdminSecurityGrantOutput,
  type ScanSignResolver,
} from '../admins/securityApi';
import { adminRequest } from '../utils/http';
import type { CreateAccountOutput, InstitutionAccount } from '../subjects/api';

export type { CreateAccountOutput, InstitutionAccount, MultisigChainStatus } from '../subjects/api';

// 新增机构账户属 PASSKEY_COLD_SIGN 操作，需 CitizenWallet 扫码签名授权；signWithScan 由创建弹窗注入。
export async function createAccount(
  auth: AdminAuth,
  cidNumber: string,
  accountName: string,
  signWithScan: ScanSignResolver,
): Promise<CreateAccountOutput> {
  const grantPayload = { target: cidNumber, cid_number: cidNumber, account_name: accountName };
  const headers = await createColdSignSubmitHeaders(
    auth,
    'INSTITUTION_CREATE_ACCOUNT',
    grantPayload,
    signWithScan,
    { 'content-type': 'application/json' },
  );
  return adminRequest<CreateAccountOutput>(
    `/api/v1/institutions/${encodeURIComponent(cidNumber)}/account/create`,
    auth,
    {
      method: 'POST',
      headers,
      body: JSON.stringify({ account_name: accountName }),
    },
  );
}

export async function listAccounts(
  auth: AdminAuth,
  cidNumber: string,
): Promise<InstitutionAccount[]> {
  return adminRequest<InstitutionAccount[]>(
    `/api/v1/institutions/${encodeURIComponent(cidNumber)}/accounts`,
    auth,
  );
}

export async function deleteAccount(
  auth: AdminAuth,
  cidNumber: string,
  accountName: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<{ deleted: boolean }> {
  const headers = await securityGrantSubmitHeaders(auth, securityGrant);
  return adminRequest<{ deleted: boolean }>(
    `/api/v1/institutions/${encodeURIComponent(cidNumber)}/account/${encodeURIComponent(accountName)}`,
    auth,
    { method: 'DELETE', headers },
  );
}
