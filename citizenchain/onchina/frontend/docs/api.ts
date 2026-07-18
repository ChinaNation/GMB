// 机构资料库前端 API。资料上传、下载、删除都归 docs 模块。

import type { AdminAuth } from '../auth/types';
import {
  securityGrantSubmitHeaders,
  type AdminSecurityGrantOutput,
} from '../admins/securityApi';
import { adminHeaders, adminRequest } from '../utils/http';
import type { InstitutionDocument } from '../subjects/api';

export type { InstitutionDocument } from '../subjects/api';

export const DOC_TYPE_OPTIONS = [
  '公司章程',
  '营业许可证',
  '股东会决议',
  '法人授权书',
  '其他',
] as const;

export async function listDocuments(
  auth: AdminAuth,
  cidNumber: string,
): Promise<InstitutionDocument[]> {
  return adminRequest<InstitutionDocument[]>(
    `/api/v1/institutions/${encodeURIComponent(cidNumber)}/docs`,
    auth,
  );
}

// 上传资料属 PASSKEY_COLD_SIGN 操作:
// 前端 prepare 的授权 payload 必须与后端 require_admin_security_grant 中的 grant_payload 完全同形,
// 正式提交再同时携带冷签 grant 与 Passkey assertion,缺一律由后端 fail-closed。
export async function uploadDocument(
  auth: AdminAuth,
  cidNumber: string,
  file: File,
  docType: string,
  securityGrant: AdminSecurityGrantOutput,
): Promise<InstitutionDocument> {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('doc_type', docType);
  const headers = await securityGrantSubmitHeaders(auth, securityGrant);
  return adminRequest<InstitutionDocument>(
    `/api/v1/institutions/${encodeURIComponent(cidNumber)}/docs`,
    auth,
    {
      method: 'POST',
      headers,
      body: formData,
    },
  );
}

export async function downloadDocument(
  auth: AdminAuth,
  cidNumber: string,
  docId: number,
  fileName: string,
): Promise<void> {
  const resp = await fetch(
    `/api/v1/institutions/${encodeURIComponent(cidNumber)}/docs/${docId}/download`,
    { headers: adminHeaders(auth) },
  );
  if (!resp.ok) throw new Error(`下载失败 (${resp.status})`);
  const blob = await resp.blob();
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName;
  a.click();
  URL.revokeObjectURL(url);
}

export async function deleteDocument(
  auth: AdminAuth,
  cidNumber: string,
  docId: number,
  securityGrant: AdminSecurityGrantOutput,
): Promise<void> {
  const headers = await securityGrantSubmitHeaders(auth, securityGrant);
  await adminRequest<string>(
    `/api/v1/institutions/${encodeURIComponent(cidNumber)}/docs/${docId}`,
    auth,
    { method: 'DELETE', headers },
  );
}
