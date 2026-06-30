// 中文注释:联邦注册局管理员目录 API。
// 更换联邦注册局管理员走 REPLACE_GOVERNING_REGISTRY 冷钱包扫码签名动作,不走普通 PATCH。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

// 联邦注册局管理员对外行(API 返回结构)。
//
// CID 业务语义:联邦注册局管理员只有存在/更换,不存在新增/删除/停用状态字段。
export type FederalRegistryAdminRow = {
  id: number;
  province_name: string;
  admin_account: string;
  admin_name: string;
  built_in: boolean;
  created_at: string;
  /** 最近一次更新时间,null 表示从未更新。 */
  updated_at?: string | null;
};

export type OwnInstitutionAdminRow = {
  admin_account: string;
  admin_name: string;
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

export async function updateFederalRegistryName(
  auth: AdminAuth,
  id: number,
  adminName: string,
): Promise<FederalRegistryAdminRow> {
  return request<FederalRegistryAdminRow>(`/api/v1/admin/federal-registry-admins/${id}`, {
    method: 'PATCH',
    headers: { 'content-type': 'application/json', ...adminHeaders(auth) },
    body: JSON.stringify({ admin_name: adminName }),
  });
}
