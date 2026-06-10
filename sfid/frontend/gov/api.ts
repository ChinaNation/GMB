// 中文注释:公权机构前端 API。公安局和自动公权目录都归 gov 模块调用。
// 公权机构全部由后端自动生成,本模块无创建接口;JY 学校机构归 education 模块。

import type { AdminAuth } from '../auth/types';
import { adminRequest } from '../utils/http';
import type {
  InstitutionDetail,
  InstitutionListRow,
  PageResult,
} from '../subjects/api';

export type GovCategory = 'PUBLIC_SECURITY' | 'GOV_INSTITUTION';

export type { InstitutionDetail } from '../subjects/api';

export interface ListPublicSecurityQuery {
  cursor?: string | null;
  page_size?: number;
}

export interface ListOfficialInstitutionQuery {
  province?: string;
  city?: string;
  q?: string;
  cursor?: string | null;
  page_size?: number;
}

export async function listPublicSecurityInstitutions(
  auth: AdminAuth,
  query?: ListPublicSecurityQuery,
): Promise<PageResult<InstitutionListRow>> {
  const params = new URLSearchParams();
  if (query?.cursor) params.set('cursor', query.cursor);
  if (query?.page_size) params.set('page_size', String(query.page_size));
  const qs = params.toString();
  return adminRequest<PageResult<InstitutionListRow>>(
    qs ? `/api/v1/institutions/public-security?${qs}` : '/api/v1/institutions/public-security',
    auth,
  );
}

export async function listOfficialInstitutions(
  auth: AdminAuth,
  query?: ListOfficialInstitutionQuery,
): Promise<PageResult<InstitutionListRow>> {
  const params = new URLSearchParams();
  if (query?.province) params.set('province', query.province);
  if (query?.city) params.set('city', query.city);
  if (query?.q && query.q.trim()) params.set('q', query.q.trim());
  if (query?.cursor) params.set('cursor', query.cursor);
  if (query?.page_size) params.set('page_size', String(query.page_size));
  const qs = params.toString();
  return adminRequest<PageResult<InstitutionListRow>>(
    qs ? `/api/v1/institutions/official?${qs}` : '/api/v1/institutions/official',
    auth,
  );
}

export async function getInstitution(
  auth: AdminAuth,
  sfidNumber: string,
): Promise<InstitutionDetail> {
  return adminRequest<InstitutionDetail>(
    `/api/v1/institution/${encodeURIComponent(sfidNumber)}`,
    auth,
  );
}

/**
 * 联邦注册局机构详情(只读,绕过 scope)。
 * 联邦注册局是全国唯一机构(位于中枢省),其它省管理员被普通 getInstitution 的 scope 拦截,
 * 故走专用只读接口。返回结构与 getInstitution 完全一致。
 */
export async function getFederalRegistry(
  auth: AdminAuth,
): Promise<InstitutionDetail> {
  return adminRequest<InstitutionDetail>(
    '/api/v1/institutions/federal-registry',
    auth,
  );
}
