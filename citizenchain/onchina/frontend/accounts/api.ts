// 中文注释:机构账户前端 API。账户创建、列表和删除都归 accounts 模块。

import type { AdminAuth } from '../auth/types';
import {
  createScanSignSecurityGrant,
  type AdminSecurityGrantOutput,
  type ScanSignResolver,
} from '../admins/admin_security_api';
import { adminRequest } from '../utils/http';
import type { CreateAccountOutput, InstitutionAccount } from '../subjects/api';

const SECURITY_GRANT_HEADER = 'x-cid-security-grant';

export type { CreateAccountOutput, InstitutionAccount, MultisigChainStatus } from '../subjects/api';

// 中文注释:新增机构账户属 PASSKEY_COLD_SIGN 操作,需冷钱包扫码签名授权;signWithScan 由创建弹窗注入。
export async function createAccount(
  auth: AdminAuth,
  cidNumber: string,
  accountName: string,
  signWithScan: ScanSignResolver,
): Promise<CreateAccountOutput> {
  const grantPayload = { target: cidNumber, cid_number: cidNumber, account_name: accountName };
  const grant = await createScanSignSecurityGrant(auth, 'INSTITUTION_CREATE_ACCOUNT', grantPayload, signWithScan);
  return adminRequest<CreateAccountOutput>(
    `/api/v1/institution/${encodeURIComponent(cidNumber)}/account/create`,
    auth,
    {
      method: 'POST',
      headers: { 'content-type': 'application/json', [SECURITY_GRANT_HEADER]: grant.grant_id },
      body: JSON.stringify({ account_name: accountName }),
    },
  );
}

export async function listAccounts(
  auth: AdminAuth,
  cidNumber: string,
): Promise<InstitutionAccount[]> {
  return adminRequest<InstitutionAccount[]>(
    `/api/v1/institution/${encodeURIComponent(cidNumber)}/accounts`,
    auth,
  );
}

export async function deleteAccount(
  auth: AdminAuth,
  cidNumber: string,
  accountName: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<{ deleted: boolean }> {
  return adminRequest<{ deleted: boolean }>(
    `/api/v1/institution/${encodeURIComponent(cidNumber)}/account/${encodeURIComponent(accountName)}`,
    auth,
    { method: 'DELETE', headers: { [SECURITY_GRANT_HEADER]: securityGrant.grant_id } },
  );
}
