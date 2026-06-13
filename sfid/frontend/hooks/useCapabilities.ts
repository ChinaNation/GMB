// 中文注释:从 auth.role 派生出前端各 tab 的能力标志,方便 业务子组件做条件渲染。
// 中文注释:当前仅保留 FEDERAL_ADMIN / CITY_ADMIN。
// 管理员不存在组织级别槽位和状态字段,本 hook 只看角色。

import { useMemo } from 'react';
import type { AdminAuth } from '../auth/types';

export interface Capabilities {
  // 登录角色判断
  isFederalAdmin: boolean;
  isCityAdmin: boolean;

  // tab 可见性
  canViewCitizens: boolean;           // 首页公民身份
  canViewInstitutions: boolean;       // 公安局 / 公权机构 / 六类私权机构入口
  canViewPrivate: boolean;            // 六类私权机构入口
  canViewRegistry: boolean;           // 注册局 tab(原机构管理)
  canViewFederalAdmins: boolean;        // 联邦管理员列表
  canViewCityAdmins: boolean;          // 市管理员列表

  // 业务操作权限
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canCrudCityAdmins: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
}

export function useCapabilities(auth: AdminAuth | null): Capabilities {
  return useMemo<Capabilities>(() => {
    const role = auth?.role;
    const isFederalAdmin = role === 'FEDERAL_ADMIN';
    const isCityAdmin = role === 'CITY_ADMIN';

    return {
      isFederalAdmin,
      isCityAdmin,

      canViewCitizens: !!role,
      canViewInstitutions: isFederalAdmin || isCityAdmin,
      canViewPrivate: isFederalAdmin || isCityAdmin,
      canViewRegistry: isFederalAdmin || isCityAdmin,
      canViewFederalAdmins: isFederalAdmin,
      canViewCityAdmins: isFederalAdmin || isCityAdmin,

      canManageInstitutions: isFederalAdmin || isCityAdmin,
      canRegisterInstitutions: isFederalAdmin || isCityAdmin,
      canCrudCityAdmins: isFederalAdmin,
      canStatusScan: isFederalAdmin || isCityAdmin,
      canBusinessWrite: isFederalAdmin || isCityAdmin,
    };
  }, [auth?.role]);
}
