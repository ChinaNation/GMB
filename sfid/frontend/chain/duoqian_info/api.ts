// 中文注释:DUOQIAN 链交互 API 入口。机构创建仍走 ../institution.ts,
// 本文件只承载备案上链、链侧机构信息状态等能力。

import { adminRequest, type AdminAuth } from '../../api/client';

export type InstitutionFilingStatus =
  | 'NOT_FILED'
  | 'FILING_PENDING'
  | 'FILED_ON_CHAIN'
  | 'FILING_FAILED';

export interface InstitutionFilingPayload {
  sfid_id: string;
  institution_name: string;
  account_name: string;
}

export interface InstitutionFilingRecord extends InstitutionFilingPayload {
  status: InstitutionFilingStatus;
  tx_hash?: string | null;
  block_number?: number | null;
  filed_at?: string | null;
  last_error?: string | null;
}

export async function submitInstitutionFiling(
  auth: AdminAuth,
  payload: InstitutionFilingPayload,
): Promise<InstitutionFilingRecord> {
  return adminRequest<InstitutionFilingRecord>('/api/v1/chain/duoqian-info/institution-filing', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

export async function getInstitutionFilingStatus(
  auth: AdminAuth,
  sfidId: string,
): Promise<InstitutionFilingRecord> {
  const encoded = encodeURIComponent(sfidId);
  return adminRequest<InstitutionFilingRecord>(`/api/v1/chain/duoqian-info/institution-filing/${encoded}`, auth);
}
