// 中文注释:CPMS 系统管理前端 API。
// 后端对应:backend/cpms/handler.rs。CPMS 站点挂在公安局机构详情页中展示,
// 但安装、注册、匿名证书、站点状态治理属于 CPMS 系统管理功能。

import type { AdminAuth } from '../auth/types';
import { adminRequest } from '../utils/http';

export type GenerateCpmsInstitutionSfidResult = {
  site_sfid: string;
  qr1_payload: string;
};

export type CpmsSiteRow = {
  site_sfid: string;
  install_token_status: 'PENDING' | 'USED' | 'REVOKED';
  status?: 'PENDING' | 'ACTIVE' | 'DISABLED' | 'REVOKED';
  version?: number;
  province_code?: string;
  admin_province?: string;
  city_name?: string;
  institution_code?: string;
  institution_name?: string;
  qr1_payload?: string;
  qr3_payload?: string | null;
  created_by: string;
  created_by_name?: string;
  created_at: string;
  updated_by?: string | null;
  updated_at?: string | null;
};

export type CpmsRegisterResult = {
  qr3_payload: string;
};

export type CpmsArchiveImportResult = {
  archive_no: string;
  province_code: string;
  status: string;
};

/**
 * 任务卡 `20260408-sfid-public-security-cpms-embed`:
 * 按机构 sfid_id 反查其 CPMS 站点。
 * 后端通过 `(province, city, institution_code)` 三元组匹配,无则返回 null。
 */
export async function getCpmsSiteByInstitution(
  auth: AdminAuth,
  sfidId: string
): Promise<CpmsSiteRow | null> {
  return adminRequest<CpmsSiteRow | null>(
    `/api/v1/admin/cpms-keys/by-institution/${encodeURIComponent(sfidId)}`,
    auth
  );
}

/** 生成公安局 CPMS 站点 SFID 和安装 QR1。 */
export async function generateCpmsInstitutionSfid(
  auth: AdminAuth,
  payload: { province?: string; city: string; institution: string; institution_name: string },
): Promise<GenerateCpmsInstitutionSfidResult> {
  return adminRequest<GenerateCpmsInstitutionSfidResult>('/api/v1/admin/cpms-keys/sfid/generate', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

/** 扫 CPMS 设备返回的 QR2,完成站点匿名证书注册并返回 QR3。 */
export async function registerCpms(
  auth: AdminAuth,
  payload: { qr_payload: string },
): Promise<CpmsRegisterResult> {
  return adminRequest<CpmsRegisterResult>('/api/v1/admin/cpms/register', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

/** 导入 CPMS 档案二维码。 */
export async function importArchive(
  auth: AdminAuth,
  payload: { qr_payload: string },
): Promise<CpmsArchiveImportResult> {
  return adminRequest<CpmsArchiveImportResult>('/api/v1/admin/cpms/archive/import', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

/** 注销未使用安装令牌。 */
export async function revokeInstallToken(auth: AdminAuth, siteSfid: string): Promise<string> {
  return adminRequest<string>(
    `/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/revoke-token`,
    auth,
    { method: 'POST' },
  );
}

/** 重发安装令牌,用于 PENDING/REVOKED 后重新生成 QR1。 */
export async function reissueInstallToken(
  auth: AdminAuth,
  siteSfid: string,
): Promise<GenerateCpmsInstitutionSfidResult> {
  return adminRequest<GenerateCpmsInstitutionSfidResult>(
    `/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/reissue`,
    auth,
    { method: 'POST' },
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
  siteSfid: string,
  reason?: string,
): Promise<CpmsSiteRow> {
  return adminRequest<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/disable`, auth, {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ reason }),
  });
}

/** 启用已禁用的 CPMS 站点密钥。 */
export async function enableCpmsKeys(auth: AdminAuth, siteSfid: string): Promise<CpmsSiteRow> {
  return adminRequest<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/enable`, auth, {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({}),
  });
}

/** 吊销 CPMS 站点密钥。 */
export async function revokeCpmsKeys(
  auth: AdminAuth,
  siteSfid: string,
  reason?: string,
): Promise<CpmsSiteRow> {
  return adminRequest<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/revoke`, auth, {
    method: 'PUT',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ reason }),
  });
}

/** 删除 CPMS 站点密钥记录。 */
export async function deleteCpmsKeys(auth: AdminAuth, siteSfid: string): Promise<string> {
  return adminRequest<string>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}`, auth, {
    method: 'DELETE',
  });
}
