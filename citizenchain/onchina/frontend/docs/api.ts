// 机构资料库前端 API。资料上传、下载、删除都归 docs 模块。

import type { AdminAuth } from '../auth/types';
import type { AdminSecurityGrantOutput } from '../admins/admin_security_api';
import { adminHeaders, adminRequest } from '../utils/http';
import type { InstitutionDocument } from '../subjects/api';

const SECURITY_GRANT_HEADER = 'x-cid-security-grant';

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
    `/api/v1/institution/${encodeURIComponent(cidNumber)}/documents`,
    auth,
  );
}

// 上传资料属 SESSION 操作(仅需有效会话),无需扫码签名授权,直接调用。
export async function uploadDocument(
  auth: AdminAuth,
  cidNumber: string,
  file: File,
  docType: string,
): Promise<InstitutionDocument> {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('doc_type', docType);
  return adminRequest<InstitutionDocument>(
    `/api/v1/institution/${encodeURIComponent(cidNumber)}/documents`,
    auth,
    {
      method: 'POST',
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
    `/api/v1/institution/${encodeURIComponent(cidNumber)}/documents/${docId}/download`,
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
  await adminRequest<string>(
    `/api/v1/institution/${encodeURIComponent(cidNumber)}/documents/${docId}`,
    auth,
    { method: 'DELETE', headers: { [SECURITY_GRANT_HEADER]: securityGrant.grant_id } },
  );
}
