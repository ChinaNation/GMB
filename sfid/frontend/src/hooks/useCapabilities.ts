// 中文注释:从 auth.role 派生出前端各 tab 的能力标志,方便 views/ 子组件做条件渲染。
// 对齐 App.tsx 现有的 capabilities 对象,新代码用本 hook,老 App.tsx 暂不切换。

import { useMemo } from 'react';
import type { AdminAuth } from '../api/client';

export interface Capabilities {
  // 登录角色判断
  isKeyAdmin: boolean;
  isShengAdmin: boolean;
  isShiAdmin: boolean;

  // tab 可见性
  canViewCitizens: boolean;           // 首页公民身份
  canViewInstitutions: boolean;       // 公安局 / 公权机构 / 私权机构 tab
  canViewMultisig: boolean;           // 多签管理(legacy tab,任务卡 4 会并入私权机构)
  canViewKeyring: boolean;            // 密钥管理
  canViewRegistry: boolean;           // 注册局 tab(原机构管理)
  canViewShengAdmins: boolean;        // 省级管理员列表
  canViewShiAdmins: boolean;          // 市级管理员列表

  // 业务操作权限
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canManageKeyring: boolean;
  canCrudShiAdmins: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
}

export function useCapabilities(auth: AdminAuth | null): Capabilities {
  return useMemo<Capabilities>(() => {
    const role = auth?.role;
    const isKeyAdmin = role === 'KEY_ADMIN';
    const isShengAdmin = role === 'SHENG_ADMIN';
    const isShiAdmin = role === 'SHI_ADMIN';

    return {
      isKeyAdmin,
      isShengAdmin,
      isShiAdmin,

      canViewCitizens: !!role,
      canViewInstitutions: isKeyAdmin || isShengAdmin,
      canViewMultisig: isKeyAdmin || isShengAdmin || isShiAdmin,
      canViewKeyring: isKeyAdmin,
      canViewRegistry: isKeyAdmin || isShengAdmin || isShiAdmin,
      canViewShengAdmins: isKeyAdmin || isShengAdmin,
      canViewShiAdmins: isKeyAdmin || isShengAdmin || isShiAdmin,

      canManageInstitutions: isKeyAdmin || isShengAdmin,
      canRegisterInstitutions: isKeyAdmin || isShengAdmin,
      canManageKeyring: isKeyAdmin,
      canCrudShiAdmins: isKeyAdmin || isShengAdmin,
      canStatusScan: isKeyAdmin || isShengAdmin || isShiAdmin,
      canBusinessWrite: isKeyAdmin || isShengAdmin || isShiAdmin,
    };
  }, [auth?.role]);
}
