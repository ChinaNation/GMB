// 中文注释:CPMS 系统管理前端 API。
// 后端对应:backend/cpms/handler.rs。CPMS 站点挂在公安局机构详情页中展示,
// SFID 侧只负责安装码授权、档案码验收和站点状态治理。

import type { AdminAuth } from '../auth/types';
import { adminRequest } from '../utils/http';
import type { AdminSecurityGrantOutput } from '../admins/admin_security_api';

const SECURITY_GRANT_HEADER = 'x-sfid-security-grant';

export type GenerateCpmsInstallResult = {
  sfid_number: string;
  qr1_payload: string;
};

export type CpmsSiteRow = {
  sfid_number: string;
  install_token_status: 'PENDING' | 'USED' | 'REVOKED';
  status?: 'PENDING' | 'ACTIVE' | 'DISABLED' | 'REVOKED';
  version?: number;
  province_code?: string;
  admin_province?: string;
  city_name?: string;
  city_code?: string;
  institution_code?: string;
  sfid_full_name?: string;
  qr1_payload?: string;
  cpms_pubkey_bound?: boolean;
  created_by: string;
  created_by_name?: string;
  created_at: string;
  updated_by?: string | null;
  updated_at?: string | null;
};

export type CpmsArchiveVerifyResult = {
  archive_no: string;
  province_code: string;
  city_code: string;
  residence_province_code: string;
  residence_city_code?: string | null;
  residence_town_code?: string | null;
  birth_province_code: string;
  birth_city_code?: string | null;
  birth_town_code?: string | null;
  election_scope_level: 'PROVINCE' | 'CITY' | 'TOWN';
  sfid_number: string;
  status: string;
};

/**
 * 任务卡 `20260408-sfid-public-security-cpms-embed`:
 * 按机构 sfid_number 反查其 CPMS 站点。
 * 后端以 `cpms_sites.sfid_number`(= 机构自身 sfid_number)为主键查询,无则返回 null。
 */
export async function getCpmsSiteByInstitution(
  auth: AdminAuth,
  sfidNumber: string
): Promise<CpmsSiteRow | null> {
  return adminRequest<CpmsSiteRow | null>(
    `/api/v1/admin/cpms-keys/by-institution/${encodeURIComponent(sfidNumber)}`,
    auth
  );
}

/**
 * 生成公安局 CPMS 站点安装码。
 * 写入键 = 机构自身 `sfid_number`(= 详情页读取键);province_name/city_name/institution 仅供安全授权绑定。
 */
export async function generateCpmsInstallQr(
  auth: AdminAuth,
  payload: { sfid_number: string; province_name?: string; city_name: string; institution: string },
  securityGrant: AdminSecurityGrantOutput,
): Promise<GenerateCpmsInstallResult> {
  return adminRequest<GenerateCpmsInstallResult>('/api/v1/admin/cpms-keys/sfid/generate', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json', [SECURITY_GRANT_HEADER]: securityGrant.grant_id },
    body: JSON.stringify(payload),
  });
}

/** 验真 CPMS 档案码；正式绑定必须走公民绑定流程。 */
export async function verifyArchive(
  auth: AdminAuth,
  payload: { qr_payload: string },
): Promise<CpmsArchiveVerifyResult> {
  return adminRequest<CpmsArchiveVerifyResult>('/api/v1/admin/cpms/archive/verify', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

/** 注销未使用安装令牌。 */
export async function revokeInstallToken(
  auth: AdminAuth,
  sfidNumber: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<string> {
  return adminRequest<string>(
    `/api/v1/admin/cpms-keys/${encodeURIComponent(sfidNumber)}/revoke-token`,
    auth,
    { method: 'POST', headers: { [SECURITY_GRANT_HEADER]: securityGrant.grant_id } },
  );
}

/** 重发安装令牌,用于 PENDING/REVOKED 后重新生成 QR1。 */
export async function reissueInstallToken(
  auth: AdminAuth,
  sfidNumber: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<GenerateCpmsInstallResult> {
  return adminRequest<GenerateCpmsInstallResult>(
    `/api/v1/admin/cpms-keys/${encodeURIComponent(sfidNumber)}/reissue`,
    auth,
    { method: 'POST', headers: { [SECURITY_GRANT_HEADER]: securityGrant.grant_id } },
  );
}

/** 列出 CPMS 站点。 */
export async function listCpmsSites(auth: AdminAuth): Promise<CpmsSiteRow[]> {
  const result = await adminRequest<{ total: number; limit: number; offset: number; rows: CpmsSiteRow[] }>(
    '/api/v1/admin/cpms-keys',
    auth,
    { method: 'GET' },
  );
  return result.rows ?? [];
}

/** 禁用 CPMS 站点密钥。 */
export async function disableCpmsKeys(
  auth: AdminAuth,
  sfidNumber: string,
  reason: string | undefined,
  securityGrant: AdminSecurityGrantOutput,
): Promise<CpmsSiteRow> {
  return adminRequest<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(sfidNumber)}/disable`, auth, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      [SECURITY_GRANT_HEADER]: securityGrant.grant_id,
    },
    body: JSON.stringify({ reason }),
  });
}

/** 启用已禁用的 CPMS 站点密钥。 */
export async function enableCpmsKeys(
  auth: AdminAuth,
  sfidNumber: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<CpmsSiteRow> {
  return adminRequest<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(sfidNumber)}/enable`, auth, {
    method: 'PUT',
    headers: { 'content-type': 'application/json', [SECURITY_GRANT_HEADER]: securityGrant.grant_id },
    body: JSON.stringify({}),
  });
}

/** 吊销 CPMS 站点密钥。 */
export async function revokeCpmsKeys(
  auth: AdminAuth,
  sfidNumber: string,
  reason: string | undefined,
  securityGrant: AdminSecurityGrantOutput,
): Promise<CpmsSiteRow> {
  return adminRequest<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(sfidNumber)}/revoke`, auth, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      [SECURITY_GRANT_HEADER]: securityGrant.grant_id,
    },
    body: JSON.stringify({ reason }),
  });
}

/** 删除 CPMS 站点密钥记录。 */
export async function deleteCpmsKeys(
  auth: AdminAuth,
  sfidNumber: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<string> {
  return adminRequest<string>(`/api/v1/admin/cpms-keys/${encodeURIComponent(sfidNumber)}`, auth, {
    method: 'DELETE',
    headers: { [SECURITY_GRANT_HEADER]: securityGrant.grant_id },
  });
}
