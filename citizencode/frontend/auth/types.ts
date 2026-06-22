// 中文注释:登录与角色相关的前端类型集中放在 auth 模块内。
// 中文注释:管理员只有联邦注册局管理员和市注册局管理员两类;Passkey 绑定状态用于首次登录后的强制密钥更新入口。

export type RegistryOrgCode = 'FEDERAL_REGISTRY' | 'CITY_REGISTRY';

export type TokenAdminAuth = {
  access_token: string;
  admin_account: string;
  registry_org_code: RegistryOrgCode;
  admin_display_name?: string;
  scope_province_name?: string | null;
  /** 仅 CityRegistry 有值:市注册局管理员所属的市。 */
  scope_city_name?: string | null;
  /** 当前管理员是否已绑定有效 Passkey。 */
  passkey_bound?: boolean;
  /** 当前管理员所属机构的简称(取自 subjects.cid_short_name 单一真源);右上角徽标显示用。 */
  institution_short_name?: string | null;
};

export type AdminAuth = TokenAdminAuth;

export function isTokenAuth(auth: AdminAuth): auth is TokenAdminAuth {
  return 'access_token' in auth;
}
