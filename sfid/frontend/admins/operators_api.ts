// 中文注释:市级管理员操作员 API。
// 所有操作员列表、创建、编辑、删除都归入 admins 管理员功能目录。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

export type OperatorRow = {
  id: number;
  admin_pubkey: string;
  admin_name: string;
  role: 'SHI_ADMIN';
  built_in: boolean;
  created_by: string;
  created_by_name?: string;
  created_at: string;
  city: string;
};

export async function listOperators(auth: AdminAuth): Promise<OperatorRow[]> {
  const data = await request<{ total: number; rows: OperatorRow[] }>('/api/v1/admin/operators', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
  return data.rows;
}

export async function updateOperatorName(
  auth: AdminAuth,
  id: number,
  adminName: string,
): Promise<OperatorRow> {
  return request<OperatorRow>(`/api/v1/admin/operators/${id}`, {
    method: 'PATCH',
    headers: { 'content-type': 'application/json', ...adminHeaders(auth) },
    body: JSON.stringify({ admin_name: adminName }),
  });
}
