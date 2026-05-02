// 中文注释:市级管理员操作员 API。
// 所有操作员列表、创建、编辑、状态切换都归入 shi_admins 功能目录。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

export type OperatorRow = {
  id: number;
  admin_pubkey: string;
  admin_name: string;
  role: 'SHI_ADMIN';
  status: 'ACTIVE' | 'DISABLED';
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

export async function createOperator(
  auth: AdminAuth,
  payload: { admin_pubkey: string; admin_name: string; city: string; created_by?: string },
): Promise<OperatorRow> {
  return request<OperatorRow>('/api/v1/admin/operators', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function updateOperator(
  auth: AdminAuth,
  id: number,
  payload: { admin_pubkey?: string; admin_name?: string; city?: string },
): Promise<OperatorRow> {
  return request<OperatorRow>(`/api/v1/admin/operators/${id}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function updateOperatorStatus(
  auth: AdminAuth,
  id: number,
  status: 'ACTIVE' | 'DISABLED',
): Promise<OperatorRow> {
  return request<OperatorRow>(`/api/v1/admin/operators/${id}/status`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify({ status }),
  });
}

export async function deleteOperator(auth: AdminAuth, id: number): Promise<string> {
  return request<string>(`/api/v1/admin/operators/${id}`, {
    method: 'DELETE',
    headers: adminHeaders(auth),
  });
}
