// 中文注释:公权机构前端 API。公安局和自动公权目录都归 gov 模块调用。
// 手动新增两能力:公权机构(G,ZF/LF/SF/JC,排除央行CB)+ 公权下属非法人(F,挂公法人);
// JY 学校机构归 education 模块。

import type { AdminAuth } from '../auth/types';
import {
  createPasskeySecurityGrant,
} from '../admins/admin_security_api';
import { adminRequest } from '../utils/http';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  InstitutionDetail,
  InstitutionListRow,
  LegalRepresentativePhoto,
  PageResult,
  ParentInstitutionRow,
  SearchParentsOptions,
} from '../subjects/api';

export type GovCategory = 'PUBLIC_SECURITY' | 'GOV_INSTITUTION';

const SECURITY_GRANT_HEADER = 'x-sfid-security-grant';

export type { CreateInstitutionOutput, InstitutionDetail } from '../subjects/api';

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

export async function checkInstitutionName(
  auth: AdminAuth,
  name: string,
  subject_property?: string,
  city?: string,
): Promise<{ exists: boolean }> {
  const params = new URLSearchParams({ name });
  if (subject_property) params.set('subject_property', subject_property);
  if (city) params.set('city', city);
  return adminRequest<{ exists: boolean }>(
    `/api/v1/institution/check-name?${params.toString()}`,
    auth,
  );
}

export async function createInstitution(
  auth: AdminAuth,
  input: CreateInstitutionInput,
): Promise<CreateInstitutionOutput> {
  const grantPayload = {
    subject_property: input.subject_property,
    p1: input.p1 ?? null,
    province: input.province ?? null,
    city: input.city,
    institution: input.institution,
    institution_name: input.institution_name ?? null,
    parent_sfid_number: input.parent_sfid_number ?? null,
    private_type: input.private_type ?? null,
    partnership_kind: input.partnership_kind ?? null,
    legal_rep_name: input.legal_rep_name ?? null,
    legal_rep_sfid_number: input.legal_rep_sfid_number ?? null,
    legal_rep_photo_path: input.legal_rep_photo_path ?? null,
  };
  const grant = await createPasskeySecurityGrant(auth, 'INSTITUTION_CREATE', grantPayload);
  return adminRequest<CreateInstitutionOutput>('/api/v1/institution/create', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json', [SECURITY_GRANT_HEADER]: grant.grant_id },
    body: JSON.stringify(input),
  });
}

export async function uploadLegalRepresentativePhoto(
  auth: AdminAuth,
  file: File,
): Promise<LegalRepresentativePhoto> {
  const form = new FormData();
  form.append('file', file);
  return adminRequest<LegalRepresentativePhoto>(
    '/api/v1/institution/legal-representative/photo',
    auth,
    {
      method: 'POST',
      body: form,
    },
  );
}

// 中文注释:所属法人搜索(公权入口 parentProperty=G → 本市市级/本省省级/国家级公法人)。
// 后端按 subjects/uninorg 规则预过滤,前端不再兜底过滤。
export async function searchParentInstitutions(
  auth: AdminAuth,
  q: string,
  opts: SearchParentsOptions,
): Promise<ParentInstitutionRow[]> {
  const params = new URLSearchParams({ q });
  params.set('f_institution', opts.fInstitution);
  params.set('province', opts.province);
  params.set('city', opts.city);
  if (opts.parentProperty) params.set('parent_property', opts.parentProperty);
  return adminRequest<ParentInstitutionRow[]>(
    `/api/v1/institution/search-parents?${params.toString()}`,
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
 * 联邦注册局是全国唯一机构(位于中枢省),其它联邦管理员被普通 getInstitution 的 scope 拦截,
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
