// 中文注释:公民直接录入 + 列表查询 API。
// 中文注释:注册局管理员直接录入公民并直接发护照。
// 通用请求能力只从 utils/http.ts 引入,本文件不承接机构或管理员模块接口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

export type CitizenState = 'NORMAL' | 'REVOKED';
export type ElectionScopeLevel = 'PROVINCE' | 'CITY' | 'TOWN';

export type CitizenRow = {
  id: number;
  wallet_pubkey?: string;
  wallet_address?: string;
  cid_number?: string;
  citizen_status?: CitizenState;
  voting_eligible: boolean;
  vote_status: CitizenState;
  identity_status?: CitizenState;
  valid_from?: string;
  valid_until?: string;
  status_updated_at?: number;
  residence_province_code?: string;
  residence_city_code?: string;
  residence_town_code?: string;
  residence_province_name?: string;
  residence_city_name?: string;
  residence_town_name?: string;
  birth_province_code?: string;
  birth_city_code?: string;
  birth_town_code?: string;
  birth_province_name?: string;
  birth_city_name?: string;
  birth_town_name?: string;
  election_scope_level?: ElectionScopeLevel;
  bind_status: 'PENDING' | 'BOUND';
};

export type PageResult<T> = {
  items: T[];
  page_size: number;
  next_cursor?: string | null;
  has_more: boolean;
};

/** 直接录入公民请求 DTO,字段与后端 admin_create_citizen 对齐。 */
export type CreateCitizenInput = {
  cid_number: string;
  residence_province_code: string;
  residence_city_code?: string;
  residence_town_code?: string;
  birth_province_code: string;
  birth_city_code?: string;
  birth_town_code?: string;
  voting_eligible: boolean;
  election_scope_level: ElectionScopeLevel;
  /** YYYY-MM-DD */
  valid_from: string;
  /** YYYY-MM-DD */
  valid_until: string;
  wallet_pubkey?: string;
  wallet_address?: string;
};

/** 直接录入公民返回 DTO。 */
export type CreateCitizenResult = {
  id: number;
  cid_number: string;
  citizen_status: CitizenState;
  voting_eligible: boolean;
  bind_status: 'PENDING' | 'BOUND';
  wallet_pubkey?: string;
  wallet_address?: string;
  valid_from: string;
  valid_until: string;
  election_scope_level: ElectionScopeLevel;
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
 * 注册局管理员直接录入公民并直接发护照。
 * 中文注释:属 LOGIN_STATE 操作(仅需有效会话),无需扫码签名授权,直接调用。
 * 成功即「已发护照」(公民 NORMAL + 有效期)。
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
