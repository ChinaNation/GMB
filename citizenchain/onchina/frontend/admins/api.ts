// 联邦注册局管理员目录 API；岗位换届由治理业务写入 entity，本页只读。

import type { AdminAuth } from '../auth/types';
import type { InstitutionDetail } from '../subjects/api';
import { adminHeaders, request } from '../utils/http';

// 联邦注册局管理员对外行(API 返回结构)。
//
// CID 业务语义:联邦注册局管理员只有存在/更换,不存在新增/删除/停用状态字段。
export type FederalRegistryAdminRow = {
  id: number;
  province_name: string;
  admin_account: string;
  role_code: string;
  role_name: string;
  term_required: boolean;
  term_start: number;
  term_end: number;
  assignment_source: number;
  assignment_source_label: string;
  assignment_source_ref: string;
  balance_fen?: string | null;
  built_in: boolean;
  created_at: string;
  /** 最近一次更新时间,null 表示从未更新。 */
  updated_at?: string | null;
};

export type OwnInstitutionAdminRow = {
  admin_account: string;
  role_code: string;
  role_name: string;
  term_required: boolean;
  term_start: number;
  term_end: number;
  assignment_source: number;
  assignment_source_label: string;
  assignment_source_ref: string;
  balance_fen?: string | null;
  is_self: boolean;
};

export type OwnInstitutionAdminListOutput = {
  institution_code: string;
  cid_short_name?: string | null;
  rows: OwnInstitutionAdminRow[];
};

export async function listFederalRegistryAdmins(auth: AdminAuth): Promise<FederalRegistryAdminRow[]> {
  return request<FederalRegistryAdminRow[]>('/api/v1/admin/federal-registry-admins', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}

export async function listOwnInstitutionAdmins(auth: AdminAuth): Promise<OwnInstitutionAdminListOutput> {
  return request<OwnInstitutionAdminListOutput>('/api/v1/admin/own-institution-admins', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}

export async function getOwnInstitution(auth: AdminAuth): Promise<InstitutionDetail> {
  return request<InstitutionDetail>('/api/v1/admin/own-institution', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}
