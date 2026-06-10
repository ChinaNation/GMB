// 中文注释:教育机构前端 API。教育委员会(JY)类学校机构(G 公立/S 私立/F 分校)统一从这里调用后端。
// 创建/查重/证件照与私权同一组后端接口,列表用 EDUCATION_INSTITUTION 过滤(= 手动 JY 学校,跨 GOV/PRIVATE)。

import type { AdminAuth } from '../auth/types';
import {
  createPasskeySecurityGrant,
} from '../admins/admin_security_api';
import { adminRequest } from '../utils/http';
import type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  InstitutionListRow,
  LegalRepresentativePhoto,
  ListInstitutionsQuery,
  PageResult,
} from '../subjects/api';

const SECURITY_GRANT_HEADER = 'x-sfid-security-grant';

export type {
  CreateInstitutionInput,
  CreateInstitutionOutput,
  InstitutionListRow,
  LegalRepresentativePhoto,
  PageResult,
} from '../subjects/api';

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
    sub_type: null,
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

export async function listEducationInstitutions(
  auth: AdminAuth,
  query?: Omit<ListInstitutionsQuery, 'category'>,
): Promise<PageResult<InstitutionListRow>> {
  const params = new URLSearchParams();
  // EDUCATION_INSTITUTION 是列表过滤维度(后端 InstitutionListFilter),不是存储 category
  params.set('category', 'EDUCATION_INSTITUTION');
  if (query?.province) params.set('province', query.province);
  if (query?.city) params.set('city', query.city);
  if (query?.q && query.q.trim()) params.set('q', query.q.trim());
  if (query?.cursor) params.set('cursor', query.cursor);
  if (query?.page_size) params.set('page_size', String(query.page_size));
  return adminRequest<PageResult<InstitutionListRow>>(
    `/api/v1/institution/list?${params.toString()}`,
    auth,
  );
}
