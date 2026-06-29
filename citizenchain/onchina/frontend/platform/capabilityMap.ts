// 中文注释:控制台能力位类型 + 空能力兜底。
//
// 能力单源在【后端】(registry src/platform/capability.rs),由登录会话下发(auth.capabilities)。
// 前端只镜像渲染 tab,不在此硬编码权限——render-gating 非安全边界,后端始终对越权独立拒绝。
// 新增机构类型只需在后端 capability.rs 补能力位,前端无需改动。

export type RoleCapabilities = {
  canViewCitizens: boolean;
  canViewInstitutions: boolean;
  canViewPrivate: boolean;
  canViewEducation: boolean;
  canViewFederalRegistryAdmins: boolean;
  canViewCityRegistryAdmins: boolean;
  canCrudCityRegistryAdmins: boolean;
  /** 只读「本机构管理员」位:非注册局法人可查看本机构链上管理员列表(只读)。 */
  canViewOwnAdmins: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canBusinessWrite: boolean;
  canViewCityRegistry: boolean;
  canViewFederalRegistry: boolean;
};

/** 空能力:未登录、能力未下发或未知机构码时的兜底(不显示任何受限 tab)。 */
export const EMPTY_CAPABILITIES: RoleCapabilities = {
  canViewCitizens: false,
  canViewInstitutions: false,
  canViewPrivate: false,
  canViewEducation: false,
  canViewFederalRegistryAdmins: false,
  canViewCityRegistryAdmins: false,
  canCrudCityRegistryAdmins: false,
  canViewOwnAdmins: false,
  canManageInstitutions: false,
  canRegisterInstitutions: false,
  canBusinessWrite: false,
  canViewCityRegistry: false,
  canViewFederalRegistry: false,
};
