// 中文注释:公民直接录入 + 列表查询 API。
// 注册局管理员提交档案字段,后端自动生成身份 CID、护照号和护照有效期。
// 通用请求能力只从 utils/http.ts 引入,本文件不承接机构或管理员模块接口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, adminRequest, request } from '../utils/http';

export type CitizenState = 'NORMAL' | 'REVOKED';
export type CitizenSex = 'MALE' | 'FEMALE';

export type CitizenRow = {
  id: number;
  cid_number: string;
  passport_no: string;
  citizen_family_name: string;
  citizen_given_name: string;
  citizen_sex: CitizenSex;
  citizen_birth_date: string;
  wallet_address?: string | null;
  citizen_status: CitizenState;
  voting_eligible: boolean;
  vote_status: CitizenState;
  identity_status: CitizenState;
  passport_valid_from: string;
  passport_valid_until: string;
  status_updated_at?: number;
  province_code: string;
  city_code: string;
  town_code: string;
  province_name?: string;
  city_name?: string;
  town_name?: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  birth_province_name?: string;
  birth_city_name?: string;
  birth_town_name?: string;
  archive_hash?: string;
  onchain_tx_hash?: string;
  onchain_block_number?: number;
  onchain_at?: string;
};

export type PageResult<T> = {
  items: T[];
  page_size: number;
  next_cursor?: string | null;
  has_more: boolean;
};

/** 直接录入公民请求 DTO,字段与后端 admin_create_citizen 对齐。 */
export type CreateCitizenInput = {
  citizen_family_name: string;
  citizen_given_name: string;
  citizen_sex: CitizenSex;
  citizen_birth_date: string;
  province_name: string;
  city_name: string;
  town_code: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  voting_eligible: boolean;
};

/** 直接录入公民返回 DTO。 */
export type CreateCitizenResult = {
  id: number;
  cid_number: string;
  passport_no: string;
  citizen_family_name: string;
  citizen_given_name: string;
  citizen_sex: CitizenSex;
  citizen_birth_date: string;
  citizen_status: CitizenState;
  voting_eligible: boolean;
  wallet_address?: string | null;
  passport_valid_from: string;
  passport_valid_until: string;
  province_code: string;
  city_code: string;
  town_code: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  archive_hash?: string;
};

export type PrepareCitizenOnchainResult = {
  cid_number: string;
  wallet_address: string;
  wallet_pubkey: string;
  citizen_age_years: number;
  payload_hex: string;
  sign_request: string;
  expires_at: number;
};

export type CompleteCitizenOnchainResult = {
  cid_number: string;
  wallet_address: string;
  chain_action: number;
  call_data_hex: string;
  citizen_signature: string;
  citizen_identity_chain_sign_request: string;
};

export const CITIZEN_DOCUMENT_TYPES = ['护照相片', '出生证明', '监护人护照', '其他材料'] as const;

export type CitizenDocumentType = (typeof CITIZEN_DOCUMENT_TYPES)[number];

export type CitizenDocument = {
  id: number;
  cid_number: string;
  file_name: string;
  document_type: CitizenDocumentType;
  file_size: number;
  file_hash: string;
  uploaded_by: string;
  uploaded_at: string;
};

export interface LegalRepresentativeCitizenSearchContext {
  target_cid_number?: string;
  province_name?: string;
  city_name?: string;
  subject_property?: string;
  institution?: string;
  education_type?: string;
  parent_cid_number?: string;
}

export async function listCitizens(
  auth: AdminAuth,
  keyword: string,
  provinceName: string,
  cityName: string,
  cursor?: string | null,
  pageSize = 50,
): Promise<PageResult<CitizenRow>> {
  const params = new URLSearchParams({
    keyword,
    province_name: provinceName,
    city_name: cityName,
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
  context: LegalRepresentativeCitizenSearchContext,
  pageSize = 20,
): Promise<string[]> {
  const params = new URLSearchParams({
    q: q.trim(),
    page_size: String(pageSize),
  });
  Object.entries(context).forEach(([key, value]) => {
    const trimmed = typeof value === 'string' ? value.trim() : '';
    if (trimmed) params.set(key, trimmed);
  });
  return request<string[]>(`/api/v1/admin/citizens/legal-representatives?${params.toString()}`, {
    headers: adminHeaders(auth),
  });
}

/**
 * 注册局管理员直接录入本地公民档案并发护照。
 * 钱包账户只在后续链上身份推送时录入并由该钱包签名确认。
 */
export async function createCitizen(
  auth: AdminAuth,
  payload: CreateCitizenInput,
): Promise<CreateCitizenResult> {
  return request<CreateCitizenResult>('/api/v1/admin/citizens', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function prepareCitizenOnchainSignature(
  auth: AdminAuth,
  cidNumber: string,
  walletAccount: string,
): Promise<PrepareCitizenOnchainResult> {
  return request<PrepareCitizenOnchainResult>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/onchain/prepare`,
    {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        ...adminHeaders(auth),
      },
      body: JSON.stringify({ wallet_account: walletAccount }),
    },
  );
}

export async function completeCitizenOnchainSignature(
  auth: AdminAuth,
  cidNumber: string,
  walletAccount: string,
  signResponse: string,
): Promise<CompleteCitizenOnchainResult> {
  return request<CompleteCitizenOnchainResult>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/onchain/complete`,
    {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        ...adminHeaders(auth),
      },
      body: JSON.stringify({ wallet_account: walletAccount, sign_response: signResponse }),
    },
  );
}

export async function listCitizenDocuments(
  auth: AdminAuth,
  cidNumber: string,
): Promise<CitizenDocument[]> {
  return adminRequest<CitizenDocument[]>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/documents`,
    auth,
  );
}

// 中文注释:公民资料库独立于机构资料库,字段名使用 document_type,不复用机构 doc_type。
export async function uploadCitizenDocument(
  auth: AdminAuth,
  cidNumber: string,
  file: File,
  documentType: CitizenDocumentType,
): Promise<CitizenDocument> {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('document_type', documentType);
  return adminRequest<CitizenDocument>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/documents`,
    auth,
    {
      method: 'POST',
      body: formData,
    },
  );
}

export async function downloadCitizenDocument(
  auth: AdminAuth,
  cidNumber: string,
  docId: number,
  fileName: string,
): Promise<void> {
  const resp = await fetch(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/documents/${docId}/download`,
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

export async function deleteCitizenDocument(
  auth: AdminAuth,
  cidNumber: string,
  docId: number,
): Promise<void> {
  await adminRequest<string>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/documents/${docId}`,
    auth,
    { method: 'DELETE' },
  );
}
