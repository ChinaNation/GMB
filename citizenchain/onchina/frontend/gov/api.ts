// 中文注释:公权机构前端 API。市公安局已折叠为普通公权机构,与自动公权目录同走 gov 模块。
// 手动新增两能力:公权机构(G,ZF/LF/SF/JC,排除储备体系自动目录代码)+ 公权下属非法人(F,挂公法人);
// JY 教育机构归 education 模块。

import type { AdminAuth } from '../auth/types';
import {
  createScanSignSecurityGrant,
  type ScanSignResolver,
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

export type GovCategory = 'GOV_INSTITUTION';

const SECURITY_GRANT_HEADER = 'x-cid-security-grant';

export type { CreateInstitutionOutput, InstitutionDetail } from '../subjects/api';

export interface ListOfficialInstitutionQuery {
  province_name?: string;
  city_name?: string;
  q?: string;
  /** 机构码精确过滤(单源,如市注册局=CREG);省略=不过滤。 */
  institution_code?: string;
  cursor?: string | null;
  page_size?: number;
}

export async function listOfficialInstitutions(
  auth: AdminAuth,
  query?: ListOfficialInstitutionQuery,
): Promise<PageResult<InstitutionListRow>> {
  const params = new URLSearchParams();
  if (query?.province_name) params.set('province_name', query.province_name);
  if (query?.city_name) params.set('city_name', query.city_name);
  if (query?.q && query.q.trim()) params.set('q', query.q.trim());
  if (query?.institution_code && query.institution_code.trim())
    params.set('institution_code', query.institution_code.trim());
  if (query?.cursor) params.set('cursor', query.cursor);
  if (query?.page_size) params.set('page_size', String(query.page_size));
  const qs = params.toString();
  return adminRequest<PageResult<InstitutionListRow>>(
    qs ? `/api/v1/institutions/official?${qs}` : '/api/v1/institutions/official',
    auth,
  );
}

export async function checkCidFullName(
  auth: AdminAuth,
  cidFullName: string,
  subject_property?: string,
  cityName?: string,
): Promise<{ exists: boolean }> {
  const params = new URLSearchParams({ cid_full_name: cidFullName });
  if (subject_property) params.set('subject_property', subject_property);
  if (cityName) params.set('city_name', cityName);
  return adminRequest<{ exists: boolean }>(
    `/api/v1/institution/check-cid-full-name?${params.toString()}`,
    auth,
  );
}

// 中文注释:创建公权机构属 PASSKEY_COLD_SIGN 操作,需冷钱包扫码签名授权;signWithScan 由创建弹窗注入。
export async function createInstitution(
  auth: AdminAuth,
  input: CreateInstitutionInput,
  signWithScan: ScanSignResolver,
): Promise<CreateInstitutionOutput> {
  const grantPayload = {
    subject_property: input.subject_property,
    p1: input.p1 ?? null,
    province_name: input.province_name ?? null,
    city_name: input.city_name,
    institution: input.institution,
    education_type: input.education_type ?? null,
    cid_full_name: input.cid_full_name ?? null,
    parent_cid_number: input.parent_cid_number ?? null,
    private_type: input.private_type ?? null,
    partnership_kind: input.partnership_kind ?? null,
    legal_rep_name: input.legal_rep_name ?? null,
    legal_rep_cid_number: input.legal_rep_cid_number ?? null,
    legal_rep_photo_path: input.legal_rep_photo_path ?? null,
  };
  const grant = await createScanSignSecurityGrant(auth, 'INSTITUTION_CREATE', grantPayload, signWithScan);
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
// 后端按 subjects/unincorporated_org 规则预过滤,前端不再兜底过滤。
export async function searchParentInstitutions(
  auth: AdminAuth,
  q: string,
  opts: SearchParentsOptions,
): Promise<ParentInstitutionRow[]> {
  const params = new URLSearchParams({ q });
  params.set('f_institution', opts.fInstitution);
  params.set('province_name', opts.province_name);
  params.set('city_name', opts.city_name);
  if (opts.parentProperty) params.set('parent_property', opts.parentProperty);
  return adminRequest<ParentInstitutionRow[]>(
    `/api/v1/institution/search-parents?${params.toString()}`,
    auth,
  );
}

export async function getInstitution(
  auth: AdminAuth,
  cidNumber: string,
): Promise<InstitutionDetail> {
  return adminRequest<InstitutionDetail>(
    `/api/v1/institution/${encodeURIComponent(cidNumber)}`,
    auth,
  );
}

/**
 * 联邦注册局机构详情(只读,绕过 scope)。
 * 联邦注册局是全国唯一机构(位于中枢省),其它联邦注册局管理员被普通 getInstitution 的 scope 拦截,
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
