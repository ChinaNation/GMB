// 中文注释:机构账户前端 API。账户创建、列表和删除都归 accounts 模块。

import type { AdminAuth } from '../auth/types';
import {
  createPasskeySecurityGrant,
  type AdminSecurityGrantOutput,
} from '../admins/admin_security_api';
import { adminRequest } from '../utils/http';
import type { CreateAccountOutput, MultisigAccount } from '../subjects/api';

const SECURITY_GRANT_HEADER = 'x-sfid-security-grant';

export type { CreateAccountOutput, MultisigAccount, MultisigChainStatus } from '../subjects/api';

export async function createAccount(
  auth: AdminAuth,
  sfidNumber: string,
  accountName: string,
): Promise<CreateAccountOutput> {
  const grantPayload = { target: sfidNumber, sfid_number: sfidNumber, account_name: accountName };
  const grant = await createPasskeySecurityGrant(auth, 'INSTITUTION_CREATE_ACCOUNT', grantPayload);
  return adminRequest<CreateAccountOutput>(
    `/api/v1/institution/${encodeURIComponent(sfidNumber)}/account/create`,
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
  sfidNumber: string,
): Promise<MultisigAccount[]> {
  return adminRequest<MultisigAccount[]>(
    `/api/v1/institution/${encodeURIComponent(sfidNumber)}/accounts`,
    auth,
  );
}

export async function deleteAccount(
  auth: AdminAuth,
  sfidNumber: string,
  accountName: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<{ deleted: boolean }> {
  return adminRequest<{ deleted: boolean }>(
    `/api/v1/institution/${encodeURIComponent(sfidNumber)}/account/${encodeURIComponent(accountName)}`,
    auth,
    { method: 'DELETE', headers: { [SECURITY_GRANT_HEADER]: securityGrant.grant_id } },
  );
}
