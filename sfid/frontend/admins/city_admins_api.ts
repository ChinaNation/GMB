// 中文注释:市注册局管理员 API。
// 市管理员列表、创建、编辑、删除都归入 admins 管理员功能目录。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

export type CityAdminRow = {
  id: number;
  admin_pubkey: string;
  admin_name: string;
  role: 'CITY_ADMIN';
  built_in: boolean;
  created_by: string;
  created_by_name?: string;
  created_at: string;
  city: string;
};

export async function listCityAdmins(auth: AdminAuth): Promise<CityAdminRow[]> {
  const data = await request<{ total: number; rows: CityAdminRow[] }>('/api/v1/admin/city-admins', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
  return data.rows;
}

export async function updateCityAdminName(
  auth: AdminAuth,
  id: number,
  adminName: string,
): Promise<CityAdminRow> {
  return request<CityAdminRow>(`/api/v1/admin/city-admins/${id}`, {
    method: 'PATCH',
    headers: { 'content-type': 'application/json', ...adminHeaders(auth) },
    body: JSON.stringify({ admin_name: adminName }),
  });
}
