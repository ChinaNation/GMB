// 中文注释:省级管理员目录的后台只读 API。
// 更换省管理员后续必须走本人签名和链上状态对齐,不再暴露旧本地替换前端入口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

// 省级管理员对外行(API 返回结构)。
//
// SFID 业务语义:省级管理员只有存在/删除,不存在停用/启用状态字段。
export type ShengAdminRow = {
  id: number;
  province: string;
  admin_pubkey: string;
  admin_name: string;
  built_in: boolean;
  created_at: string;
  /** 最近一次更新时间,null 表示从未更新。 */
  updated_at?: string | null;
};

export async function listShengAdmins(auth: AdminAuth): Promise<ShengAdminRow[]> {
  return request<ShengAdminRow[]>('/api/v1/admin/sheng-admins', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}
