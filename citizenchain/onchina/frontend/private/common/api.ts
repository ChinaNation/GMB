// 中文注释:私权机构前端共用 API。六类私权机构必须在各自目录的 api.ts 中传入 routeSegment,
// 本文件只负责封装共用 HTTP、查重、证件照、详情和参与主体共享接口。

import type { AdminAuth } from '../../auth/types';
import {
  createScanSignSecurityGrant,
  type ScanSignResolver,
} from '../../admins/admin_security_api';
import { adminRequest } from '../../utils/http';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  InstitutionCategory,
  InstitutionDetail,
  Institution,
  InstitutionListRow,
  LegalRepresentativePhoto,
  ListInstitutionsQuery,
  PageResult,
  ParentInstitutionRow,
  SearchParentsOptions,
  UpdateInstitutionInput,
} from '../../subjects/api';

const SECURITY_GRANT_HEADER = 'x-cid-security-grant';

export type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  Institution,
  InstitutionCategory,
  InstitutionDetail,
  InstitutionListRow,
  LegalRepresentativePhoto,
  PageResult,
  ParentInstitutionRow,
  UpdateInstitutionInput,
} from '../../subjects/api';

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

// 中文注释:创建机构属 SCAN_SIGN 操作,需冷钱包扫码签名授权;signWithScan 由创建弹窗注入。
export async function createInstitution(
  auth: AdminAuth,
  routeSegment: string,
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
  return adminRequest<CreateInstitutionOutput>(`/api/v1/private/${routeSegment}`, auth, {
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

export async function listPrivateInstitutions(
  auth: AdminAuth,
  routeSegment: string,
  query?: Omit<ListInstitutionsQuery, 'category'>,
): Promise<PageResult<InstitutionListRow>> {
  const params = new URLSearchParams();
  if (query?.private_type) params.set('private_type', query.private_type);
  if (query?.province_name) params.set('province_name', query.province_name);
  if (query?.city_name) params.set('city_name', query.city_name);
  if (query?.q && query.q.trim()) params.set('q', query.q.trim());
  if (query?.cursor) params.set('cursor', query.cursor);
  if (query?.page_size) params.set('page_size', String(query.page_size));
  return adminRequest<PageResult<InstitutionListRow>>(
    `/api/v1/private/${routeSegment}?${params.toString()}`,
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

// 中文注释:所属法人搜索。后端按 subjects/unincorporated_org 规则预过滤
// (分校→本市学校本部;其它 F→私法人全国 ∪ 公法人按层级地域),前端不再兜底过滤。
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

// 中文注释:更新机构属 LOGIN_STATE 操作(仅需有效会话),无需扫码签名授权,直接调用。
export async function updateInstitution(
  auth: AdminAuth,
  cidNumber: string,
  input: UpdateInstitutionInput,
): Promise<Institution> {
  return adminRequest<Institution>(
    `/api/v1/institution/${encodeURIComponent(cidNumber)}`,
    auth,
    {
      method: 'PATCH',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(input),
    },
  );
}
