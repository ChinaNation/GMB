// 中文注释:公权机构前端 API。公安局和自动公权目录都归 gov 模块调用。

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
  a3?: string,
  city?: string,
): Promise<{ exists: boolean }> {
  const params = new URLSearchParams({ name });
  if (a3) params.set('a3', a3);
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
    a3: input.a3,
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

export async function getInstitution(
  auth: AdminAuth,
  sfidNumber: string,
): Promise<InstitutionDetail> {
  return adminRequest<InstitutionDetail>(
    `/api/v1/institution/${encodeURIComponent(sfidNumber)}`,
    auth,
  );
}
