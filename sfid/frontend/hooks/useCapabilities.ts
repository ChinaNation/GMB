// 中文注释:从 auth.role 派生出前端各 tab 的能力标志,方便 业务子组件做条件渲染。
// 中文注释:当前仅保留 SHENG_ADMIN / SHI_ADMIN。
// SHENG_ADMIN 三槽自治(Main/Backup1/Backup2)由 sheng_admin 视图自身处理,本 hook 只看角色不看槽位。

import { useMemo } from 'react';
import type { AdminAuth } from '../auth/types';

export interface Capabilities {
  // 登录角色判断
  isShengAdmin: boolean;
  isShiAdmin: boolean;

  // tab 可见性
  canViewCitizens: boolean;           // 首页公民身份
  canViewInstitutions: boolean;       // 公安局 / 公权机构 / 私权机构 tab
  canViewMultisig: boolean;           // 多签管理(legacy tab,任务卡 4 会并入私权机构)
  canViewRegistry: boolean;           // 注册局 tab(原机构管理)
  canViewShengAdmins: boolean;        // 省级管理员列表 / 名册 / 激活 / rotate
  canViewShiAdmins: boolean;          // 市级管理员列表

  // 业务操作权限
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canCrudShiAdmins: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
}

export function useCapabilities(auth: AdminAuth | null): Capabilities {
  return useMemo<Capabilities>(() => {
    const role = auth?.role;
    const isShengAdmin = role === 'SHENG_ADMIN';
    const isShiAdmin = role === 'SHI_ADMIN';

    return {
      isShengAdmin,
      isShiAdmin,

      canViewCitizens: !!role,
      canViewInstitutions: isShengAdmin,
      canViewMultisig: isShengAdmin || isShiAdmin,
      canViewRegistry: isShengAdmin || isShiAdmin,
      canViewShengAdmins: isShengAdmin,
      canViewShiAdmins: isShengAdmin || isShiAdmin,

      canManageInstitutions: isShengAdmin,
      canRegisterInstitutions: isShengAdmin,
      canCrudShiAdmins: isShengAdmin,
      canStatusScan: isShengAdmin || isShiAdmin,
      canBusinessWrite: isShengAdmin || isShiAdmin,
    };
  }, [auth?.role]);
}
