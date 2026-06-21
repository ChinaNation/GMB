// 中文注释:从 auth.registry_org_code 派生出前端各 tab 的能力标志,方便 业务子组件做条件渲染。
// 中文注释:当前仅保留 FEDERAL_REGISTRY / CITY_REGISTRY。
// 管理员不存在组织级别槽位和状态字段,本 hook 只看角色。

import { useMemo } from 'react';
import type { AdminAuth } from '../auth/types';

export interface Capabilities {
  // 登录角色判断
  isFederalRegistry: boolean;
  isCityRegistry: boolean;

  // tab 可见性
  canViewCitizens: boolean;           // 首页公民身份
  canViewInstitutions: boolean;       // 公安局 / 公权机构 / 六类私权机构入口
  canViewPrivate: boolean;            // 六类私权机构入口
  canViewRegistry: boolean;           // 注册局 tab(原机构管理)
  canViewFederalRegistryAdmins: boolean;        // 联邦注册局管理员列表
  canViewCityRegistryAdmins: boolean;          // 市注册局管理员列表

  // 业务操作权限
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canCrudCityRegistryAdmins: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
}

export function useCapabilities(auth: AdminAuth | null): Capabilities {
  return useMemo<Capabilities>(() => {
    const registry_org_code = auth?.registry_org_code;
    const isFederalRegistry = registry_org_code === 'FEDERAL_REGISTRY';
    const isCityRegistry = registry_org_code === 'CITY_REGISTRY';

    return {
      isFederalRegistry,
      isCityRegistry,

      canViewCitizens: !!registry_org_code,
      canViewInstitutions: isFederalRegistry || isCityRegistry,
      canViewPrivate: isFederalRegistry || isCityRegistry,
      canViewRegistry: isFederalRegistry || isCityRegistry,
      canViewFederalRegistryAdmins: isFederalRegistry,
      canViewCityRegistryAdmins: isFederalRegistry || isCityRegistry,

      canManageInstitutions: isFederalRegistry || isCityRegistry,
      canRegisterInstitutions: isFederalRegistry || isCityRegistry,
      canCrudCityRegistryAdmins: isFederalRegistry,
      canStatusScan: isFederalRegistry || isCityRegistry,
      canBusinessWrite: isFederalRegistry || isCityRegistry,
    };
  }, [auth?.registry_org_code]);
}
