// 中文注释:登录与角色相关的前端类型集中放在 auth 模块内。
// 省管理员槽位类型属于链上名册,放在 chain/sheng_admins/types.ts。

import type { TokenAdminAuth } from '../api/client';
import type { ShengSlot } from '../chain/sheng_admins/types';

export type AdminRole = 'SHENG_ADMIN' | 'SHI_ADMIN';

export const AdminRoleLabel: Record<AdminRole, string> = {
  SHENG_ADMIN: '省级管理员',
  SHI_ADMIN: '市级管理员',
};

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
