// 中文注释:公民直接录入 + 列表查询 API。
// 注册局管理员提交档案字段,后端自动生成身份 CID、护照号和护照有效期。
// 通用请求能力只从 utils/http.ts 引入,本文件不承接机构或管理员模块接口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

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
  wallet_address: string;
  citizen_status: CitizenState;
  voting_eligible: boolean;
  vote_status: CitizenState;
  identity_status: CitizenState;
  passport_valid_from: string;
  passport_valid_until: string;
  status_updated_at?: number;
  residence_province_code: string;
  residence_city_code: string;
  residence_town_code: string;
  residence_province_name?: string;
  residence_city_name?: string;
  residence_town_name?: string;
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
  residence_province_name: string;
  residence_city_name: string;
  residence_town_code: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  voting_eligible: boolean;
  /** SS58 地址或扫码得到的 0x 公钥;UI 只展示 SS58 地址。 */
  wallet_account: string;
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
  wallet_address: string;
  passport_valid_from: string;
  passport_valid_until: string;
  residence_province_code: string;
  residence_city_code: string;
  residence_town_code: string;
  birth_province_code: string;
  birth_city_code: string;
  birth_town_code: string;
  archive_hash?: string;
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
 * 注册局管理员直接录入公民并直接发护照。
 * 成功即本地公民档案、身份 CID、护照号、钱包账户全部落库。
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
