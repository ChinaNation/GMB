// 中文注释:SFID 前端角色枚举(ADR-008 起 KEY_ADMIN 已彻底删除)。
// 单一事实源:本文件;client.ts / hooks / views 一律 import 这里。

export type AdminRole = 'SHENG_ADMIN' | 'SHI_ADMIN';

export const AdminRoleLabel: Record<AdminRole, string> = {
  SHENG_ADMIN: '省级管理员',
  SHI_ADMIN: '市级管理员',
};
