// 中文注释:市注册局管理员 API。
// 市注册局管理员列表、创建、编辑、删除都归入 admins 管理员功能目录。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

export type CityRegistryAdminRow = {
  id: number;
  admin_account: string;
  admin_name: string;
  institution_code: string;
  built_in: boolean;
  created_by: string;
  created_by_name?: string;
  created_at: string;
  city_name: string;
};

export async function listCityRegistryAdmins(auth: AdminAuth): Promise<CityRegistryAdminRow[]> {
  const data = await request<{ total: number; rows: CityRegistryAdminRow[] }>('/api/v1/admin/city-registry-admins', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
  return data.rows;
}

export async function updateCityRegistryName(
  auth: AdminAuth,
  id: number,
  adminName: string,
): Promise<CityRegistryAdminRow> {
  return request<CityRegistryAdminRow>(`/api/v1/admin/city-registry-admins/${id}`, {
    method: 'PATCH',
    headers: { 'content-type': 'application/json', ...adminHeaders(auth) },
    body: JSON.stringify({ admin_name: adminName }),
  });
}
