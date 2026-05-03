// 中文注释:省级管理员目录的后台只读 API。
// 更换省管理员后续必须走本人签名和链上状态对齐,不再暴露旧本地替换前端入口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

// 省级管理员对外行(API 返回结构)。
//
// SFID 业务语义:机构永久存在(43 个省份),省级管理员只是当前替机构发声的人,
// 不存在停用 / 状态切换的概念。被替换即彻底失效,所以没有 status 字段。
export type ShengAdminRow = {
  id: number;
  province: string;
  admin_pubkey: string;
  admin_name: string;
  built_in: boolean;
  created_at: string;
  /** 最近一次更新时间(含签名密钥 bootstrap),null 表示从未更新。 */
  updated_at?: string | null;
  /** 链上签名 pubkey:未首次登录 bootstrap 时为 null/undefined。 */
  signing_pubkey?: string | null;
  /** 签名密钥生成时间。 */
  signing_created_at?: string | null;
};

export async function listShengAdmins(auth: AdminAuth): Promise<ShengAdminRow[]> {
  return request<ShengAdminRow[]>('/api/v1/admin/sheng-admins', {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}
