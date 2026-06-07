// 中文注释:公民电子护照绑定和 CPMS 状态扫码 API。
// 通用请求能力只从 utils/http.ts 引入,本文件不承接机构或管理员模块接口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';
import { createPasskeySecurityGrant } from '../admins/admin_security_api';

const SECURITY_GRANT_HEADER = 'x-sfid-security-grant';

export type CitizenState = 'NORMAL' | 'REVOKED';

export type CitizenRow = {
  id: number;
  wallet_pubkey?: string;
  wallet_address?: string;
  archive_no?: string;
  sfid_code?: string;
  citizen_status?: CitizenState;
  voting_eligible: boolean;
  vote_status: CitizenState;
  identity_status?: CitizenState;
  valid_from?: string;
  valid_until?: string;
  status_updated_at?: number;
  bind_status: 'PENDING' | 'BOUND';
};

export type PageResult<T> = {
  items: T[];
  page_size: number;
  next_cursor?: string | null;
  has_more: boolean;
};

export type CitizenBindChallengeResult = {
  challenge_id: string;
  challenge_text: string;
  mode: 'create' | 'replace';
  archive_no: string;
  wallet_address: string;
  wallet_pubkey: string;
  citizen_status: CitizenState;
  voting_eligible: boolean;
  valid_from: string;
  valid_until: string;
  status_updated_at: number;
  /** WUMIN_QR_V1 签名请求 JSON,前端直接展示为二维码。 */
  sign_request: string;
  expire_at: number;
};

export type CitizenBindResult = {
  id: number;
  wallet_pubkey?: string;
  wallet_address?: string;
  archive_no?: string;
  sfid_code?: string;
  citizen_status?: CitizenState;
  voting_eligible: boolean;
  vote_status: CitizenState;
  identity_status: CitizenState;
  valid_from?: string;
  valid_until?: string;
  status_updated_at?: number;
  bind_status: 'PENDING' | 'BOUND';
};

export type CpmsStatusExportFile = {
  proto: 'SFID_CPMS_V1';
  type: 'CPMS_STATUS_EXPORT';
  version: number;
  export_year: number;
  sfid_number: string;
  cpms_pubkey: string;
  export_batch_id: string;
  exported_at: number;
  citizen_binding_records_count: number;
  binding_release_records_count: number;
  records_hash: string;
  citizen_binding_records: Array<{
    archive_no: string;
    wallet_address: string;
    wallet_pubkey: string;
    wallet_sig_alg: 'sr25519';
    wallet_bound_at: number;
    citizen_status: CitizenState;
    voting_eligible: boolean;
    status_updated_at: number;
  }>;
  binding_release_records: Array<{
    archive_no: string;
    released_at: number;
    release_reason: 'ARCHIVE_HARD_DELETED_AFTER_100_YEARS';
  }>;
  sig: string;
};

export type CpmsStatusExportImportResult = {
  sfid_number: string;
  export_year: number;
  export_batch_id: string;
  already_imported: boolean;
  imported_binding_records: number;
  updated_binding_records: number;
  wallet_replaced_records: number;
  released_binding_records: number;
  unmatched_binding_records: string[];
  unmatched_release_records: string[];
};

export async function listCitizens(
  auth: AdminAuth,
  keyword: string,
  cursor?: string | null,
  pageSize = 50,
): Promise<PageResult<CitizenRow>> {
  const params = new URLSearchParams({
    keyword,
    page_size: String(pageSize),
  });
  if (cursor) params.set('cursor', cursor);
  return request<PageResult<CitizenRow>>(`/api/v1/admin/citizens?${params.toString()}`, {
    headers: adminHeaders(auth),
  });
}

export async function searchLegalRepresentativeCitizens(
  auth: AdminAuth,
  q: string,
  pageSize = 20,
): Promise<string[]> {
  const params = new URLSearchParams({
    q: q.trim(),
    page_size: String(pageSize),
  });
  return request<string[]>(`/api/v1/admin/citizens/legal-representatives?${params.toString()}`, {
    headers: adminHeaders(auth),
  });
}

export async function citizenBindChallenge(
  auth: AdminAuth,
  payload: {
    mode: 'create' | 'replace';
    archive_code_payload: string;
    citizen_id?: number;
  },
): Promise<CitizenBindChallengeResult> {
  return request<CitizenBindChallengeResult>('/api/v1/admin/citizen/bind/challenge', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function citizenBind(
  auth: AdminAuth,
  payload: {
    challenge_id: string;
    pubkey: string;
    signature: string;
    payload_hash: string;
  },
): Promise<CitizenBindResult> {
  const grant = await createPasskeySecurityGrant(auth, 'CITIZEN_BIND_COMMIT', {
    target: payload.challenge_id,
    challenge_id: payload.challenge_id,
  });
  return request<CitizenBindResult>('/api/v1/admin/citizen/bind', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      [SECURITY_GRANT_HEADER]: grant.grant_id,
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function importCpmsStatusExport(
  auth: AdminAuth,
  exportFile: CpmsStatusExportFile,
): Promise<CpmsStatusExportImportResult> {
  const grant = await createPasskeySecurityGrant(auth, 'CPMS_STATUS_IMPORT_CONFIRM', {
    target: exportFile.sfid_number,
    sfid_number: exportFile.sfid_number,
    export_year: exportFile.export_year,
    export_batch_id: exportFile.export_batch_id,
    records_hash: exportFile.records_hash,
  });
  return request<CpmsStatusExportImportResult>('/api/v1/admin/citizens/cpms-status-export/import', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      [SECURITY_GRANT_HEADER]: grant.grant_id,
      ...adminHeaders(auth),
    },
    body: JSON.stringify({ export_file: exportFile }),
  });
}
