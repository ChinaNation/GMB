// 中文注释:登录与角色相关的前端类型集中放在 auth 模块内。
// 中文注释:管理员只有联邦注册局管理员和市注册局管理员两类;Passkey 绑定状态用于首次登录后的强制密钥更新入口。

export type AdminRole = 'FEDERAL_ADMIN' | 'CITY_ADMIN';

export const AdminRoleLabel: Record<AdminRole, string> = {
  FEDERAL_ADMIN: '联邦管理员',
  CITY_ADMIN: '市管理员',
};

export type TokenAdminAuth = {
  access_token: string;
  admin_pubkey: string;
  role: AdminRole;
  admin_name?: string;
  admin_province?: string | null;
  /** 仅 CityAdmin 有值:市管理员所属的市。 */
  admin_city?: string | null;
  /** 当前管理员是否已绑定有效 Passkey。 */
  passkey_bound?: boolean;
};

export type AdminAuth = TokenAdminAuth;

export function isTokenAuth(auth: AdminAuth): auth is TokenAdminAuth {
  return 'access_token' in auth;
}
