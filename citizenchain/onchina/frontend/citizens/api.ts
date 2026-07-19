// 公民直接录入 + 列表查询 API。
// 注册局管理员提交档案字段,后端自动生成身份 CID、护照号和护照有效期。
// 通用请求能力只从 utils/http.ts 引入,本文件不承接机构或管理员模块接口。

import type { AdminAuth } from '../auth/types';
import { assertPasskey, PASSKEY_ASSERTION_HEADER } from '../auth/passkey/passkeyClient';
import { adminHeaders, adminRequest, request } from '../utils/http';

export type CitizenState = 'NORMAL' | 'REVOKED';
export type CitizenSex = 'MALE' | 'FEMALE';
export type CitizenOnchainIdentityLevel = 'voting' | 'candidate';

function jsonAdminHeaders(auth: AdminAuth): Record<string, string> {
  return {
    'content-type': 'application/json',
    ...Object.fromEntries(new Headers(adminHeaders(auth)).entries()),
  };
}

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

/** 建档占号请求 DTO,字段与后端 validate_citizen_input 对齐。 */
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
  identity_level: CitizenOnchainIdentityLevel;
  wallet_address: string;
  wallet_pubkey: string;
  citizen_age_years: number;
  payload_hex: string;
  sign_request: string;
  action_label_zh: string;
  expires_at: number;
};

export type CompleteCitizenOnchainResult = {
  request_id: string;
  cid_number: string;
  identity_level: CitizenOnchainIdentityLevel;
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

/** 占号 prepare 返回:冷签 QR + 待占号(此时尚未建档,ADR-031 占号先行)。 */
export type PrepareCitizenOccupyResult = {
  request_id: string;
  cid_number: string;
  sign_request: string;
  expires_at: number;
};

/** 吊销 prepare 返回:冷签 QR。 */
export type PrepareCitizenRevokeResult = {
  request_id: string;
  cid_number: string;
  sign_request: string;
  expires_at: number;
};

/**
 * 建档占号 prepare:后端校验档案并生成号，返回管理员 CitizenWallet 签名的占号 QR。
 * 此步不落任何档案 —— 占号交易经 core 统一提交入口进块后才建档。
 */
export async function prepareCitizenOccupy(
  auth: AdminAuth,
  payload: CreateCitizenInput,
): Promise<PrepareCitizenOccupyResult> {
  return request<PrepareCitizenOccupyResult>('/api/v1/admin/citizens', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

/**
 * 吊销 prepare:登记表墓碑(号永不复用),最严档 PASSKEY_COLD_SIGN grant。
 * 返回冷签 QR,回签后同样走 core 统一提交入口。
 */
export async function prepareCitizenRevoke(
  auth: AdminAuth,
  cidNumber: string,
  walletAccount: string,
): Promise<PrepareCitizenRevokeResult> {
  const assertion = await assertPasskey(auth);
  const headers = { ...jsonAdminHeaders(auth), [PASSKEY_ASSERTION_HEADER]: assertion };
  return request<PrepareCitizenRevokeResult>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/onchain/revoke/prepare`,
    {
      method: 'POST',
      headers,
      body: JSON.stringify({ wallet_account: walletAccount }),
    },
  );
}

export async function prepareCitizenOnchainSignature(
  auth: AdminAuth,
  cidNumber: string,
  walletAccount: string,
  identityLevel: CitizenOnchainIdentityLevel,
): Promise<PrepareCitizenOnchainResult> {
  // 一次业务操作只在创建时消费一次 Passkey；后续回执由短期操作会话关联。
  const assertion = await assertPasskey(auth);
  const headers = { ...jsonAdminHeaders(auth), [PASSKEY_ASSERTION_HEADER]: assertion };
  return request<PrepareCitizenOnchainResult>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/onchain/prepare`,
    {
      method: 'POST',
      headers,
      body: JSON.stringify({ wallet_account: walletAccount, identity_level: identityLevel }),
    },
  );
}

export async function completeCitizenOnchainSignature(
  auth: AdminAuth,
  cidNumber: string,
  walletAccount: string,
  identityLevel: CitizenOnchainIdentityLevel,
  signResponse: string,
): Promise<CompleteCitizenOnchainResult> {
  const headers = jsonAdminHeaders(auth);
  return request<CompleteCitizenOnchainResult>(
    `/api/v1/admin/citizens/${encodeURIComponent(cidNumber)}/onchain/complete`,
    {
      method: 'POST',
      headers,
      body: JSON.stringify({
        wallet_account: walletAccount,
        identity_level: identityLevel,
        sign_response: signResponse,
      }),
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

// 公民资料库独立于机构资料库,字段名使用 document_type,不复用机构 doc_type。
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
