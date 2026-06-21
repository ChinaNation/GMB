// 中文注释:联邦注册局管理员目录的后台只读 API。
// 更换联邦注册局管理员必须走本人签名和链上状态对齐,不暴露本地替换前端入口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

// 联邦注册局管理员对外行(API 返回结构)。
//
// SFID 业务语义:联邦注册局管理员只有存在/删除,不存在停用/启用状态字段。
export type FederalRegistryAdminRow = {
  id: number;
  province_name: string;
  admin_account: string;
  admin_display_name: string;
  built_in: boolean;
  created_at: string;
  /** 最近一次更新时间,null 表示从未更新。 */
  updated_at?: string | null;
};

export async function listFederalRegistryAdmins(auth: AdminAuth): Promise<FederalRegistryAdminRow[]> {
  return request<FederalRegistryAdminRow[]>('/api/v1/admin/federal-registry-admins', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}

export async function updateFederalRegistryName(
  auth: AdminAuth,
  id: number,
  adminDisplayName: string,
): Promise<FederalRegistryAdminRow> {
  return request<FederalRegistryAdminRow>(`/api/v1/admin/federal-registry-admins/${id}`, {
    method: 'PATCH',
    headers: { 'content-type': 'application/json', ...adminHeaders(auth) },
    body: JSON.stringify({ admin_display_name: adminDisplayName }),
  });
}
