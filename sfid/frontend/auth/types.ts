// 中文注释:登录与角色相关的前端类型集中放在 auth 模块内。
// 省管理员槽位类型属于链上名册,放在 sheng_admins/chain_* 文件内。

import type { ShengSlot } from '../sheng_admins/chain_sheng_admins_types';

export type AdminRole = 'SHENG_ADMIN' | 'SHI_ADMIN';

export const AdminRoleLabel: Record<AdminRole, string> = {
  SHENG_ADMIN: '省级管理员',
  SHI_ADMIN: '市级管理员',
};

export type TokenAdminAuth = {
  access_token: string;
  admin_pubkey: string;
  role: AdminRole;
  admin_name?: string;
  admin_province?: string | null;
  /** 仅 ShiAdmin 有值:操作员所属的市。 */
  admin_city?: string | null;
  /** ADR-008 起 SHENG_ADMIN 三槽自治。 */
  unlocked_slot?: ShengSlot | null;
  /** 当前已解锁的省级签名密钥对应的 admin pubkey。 */
  unlocked_admin_pubkey?: string | null;
};

export type AdminAuth = TokenAdminAuth;

export function isTokenAuth(auth: AdminAuth): auth is TokenAdminAuth {
  return 'access_token' in auth;
}

/** 当前会话:登录的 admin pubkey + 已解锁的槽位(SHENG_ADMIN 三槽自治) */
export interface SessionInfo {
  /** 登录公钥(0x 小写 hex) */
  adminPubkey: string;
  /** 已解锁的省管理员槽位;SHI_ADMIN 时为 null */
  unlockedSlot: ShengSlot | null;
  /** 已解锁省级签名密钥对应的 admin pubkey;默认与 adminPubkey 相同 */
  unlockedAdminPubkey: string;
}

/** 从 TokenAdminAuth 推导 SessionInfo,字段默认值统一在此处理 */
export function deriveSession(auth: TokenAdminAuth): SessionInfo {
  return {
    adminPubkey: auth.admin_pubkey,
    unlockedSlot: auth.unlocked_slot ?? null,
    unlockedAdminPubkey: auth.unlocked_admin_pubkey ?? auth.admin_pubkey,
  };
}
