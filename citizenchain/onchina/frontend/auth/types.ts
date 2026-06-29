// 中文注释:登录与角色相关的前端类型集中放在 auth 模块内。
// 管理员按机构码(institution_code)归属机构;能力位 capabilities 由后端会话下发,前端镜像渲染 tab。

import type { RoleCapabilities } from '../platform/capabilityMap';

export type TokenAdminAuth = {
  access_token: string;
  admin_account: string;
  /** 所属机构码(3/4 字符文本,如 FRG/CREG)。 */
  institution_code: string;
  /** 行政层级标签(NATIONAL/PROVINCE/CITY/TOWN);私权法人/非法人为空。 */
  admin_level?: string | null;
  /** 机构能力位(后端单源下发);前端据此渲染 tab。 */
  capabilities?: RoleCapabilities;
  admin_name?: string;
  scope_province_name?: string | null;
  /** 市级及以下机构所属的市。 */
  scope_city_name?: string | null;
  /** 镇级机构所属的镇。 */
  scope_town_name?: string | null;
  /** 当前管理员所属机构简称,字段名与 subjects.cid_short_name 保持唯一命名。 */
  cid_short_name?: string | null;
};

export type AdminAuth = TokenAdminAuth;

export function isTokenAuth(auth: AdminAuth): auth is TokenAdminAuth {
  return 'access_token' in auth;
}
