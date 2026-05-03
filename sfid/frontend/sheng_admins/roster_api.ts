// 中文注释:注册局-省级管理员页面的一主两备名册查询 API。
// 这里是页面展示和权限状态查询,不是省管理员链上写操作。

import type { AdminAuth } from '../auth/types';
import { adminRequest } from '../utils/http';
import type { ShengSlot } from './types';

/** 名册一行:三槽中某一槽的当前登记状态 */
export interface RosterEntry {
  slot: ShengSlot;
  /** 0x 小写 hex;空字符串/null 表示该槽未登记 */
  admin_pubkey: string | null;
  /** 显示用名字(可选) */
  admin_name?: string | null;
  /** 该槽对应签名密钥的本地状态 */
  signing_status?: 'UNSET' | 'NOT_INITIALIZED' | 'GENERATED' | 'GENERATED_NOT_LOADED' | null;
  /** 已生成的签名公钥(0x 小写 hex);未生成时 null */
  signing_pubkey?: string | null;
  /** 签名密钥生成时间 */
  signing_created_at?: string | null;
  /** 当前 SFID 后端进程是否已加载该签名 keypair */
  cache_loaded?: boolean;
  /** 当前登录管理员是否能操作这一槽自己的签名密钥 */
  can_operate_signing?: boolean;
  /** 当前登录管理员是否可管理一主两备名册 */
  can_manage_roster?: boolean;
}

/** 单省 roster 列表 */
export interface ShengAdminRoster {
  province: string;
  current_admin_pubkey: string;
  entries: RosterEntry[];
}

/** GET /api/v1/admin/sheng-admin/roster */
export async function getRoster(
  auth: AdminAuth,
  province?: string,
): Promise<ShengAdminRoster> {
  const qs = province ? `?province=${encodeURIComponent(province)}` : '';
  return adminRequest<ShengAdminRoster>(`/api/v1/admin/sheng-admin/roster${qs}`, auth);
}

/** POST /api/v1/admin/sheng-admin/backup */
export async function setBackupAdmin(
  auth: AdminAuth,
  input: { slot: Exclude<ShengSlot, 'Main'>; admin_pubkey: string; admin_name: string },
): Promise<ShengAdminRoster> {
  return adminRequest<ShengAdminRoster>('/api/v1/admin/sheng-admin/backup', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}
