// 中文注释:sfid 前端登录态 + 能力标志的全局 Context。
// ADR-008(2026-05-01)起 KEY_ADMIN 已彻底删除,角色仅剩 SHENG_ADMIN / SHI_ADMIN。
// 省管理员三槽自治(Main/Backup1/Backup2)由 sheng_admin 视图自身处理,本 context 只看 role + 三 槽位字段。

import React, { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { message } from 'antd';
import type { AdminAuth } from './types';
import { setOnUnauthorized } from '../utils/http';
import { clearStoredAuth, readStoredAuth, writeStoredAuth } from '../utils/storedAuth';

export type RoleCapabilities = {
  canViewInstitutions: boolean;
  canViewMultisig: boolean;
  canViewShengAdmins: boolean;
  canViewShiAdmins: boolean;
  canCrudShiAdmins: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
  canViewSystemSettings: boolean;
  /** 当前 SHENG_ADMIN 是否处于 main 槽(可对名册做加/删 backup) */
  isShengMainSlot: boolean;
};

export function resolveRoleCapabilities(auth: AdminAuth | null): RoleCapabilities {
  const role = auth?.role;
  const isShengAdmin = role === 'SHENG_ADMIN';
  const isShiAdmin = role === 'SHI_ADMIN';
  const isShengMainSlot = isShengAdmin && (auth?.unlocked_slot === 'Main');
  return {
    canViewInstitutions: isShengAdmin,
    canViewMultisig: isShengAdmin || isShiAdmin,
    canViewShengAdmins: isShengAdmin,
    canViewShiAdmins: isShengAdmin || isShiAdmin,
    canCrudShiAdmins: isShengAdmin,
    canManageInstitutions: isShengAdmin,
    canRegisterInstitutions: isShengAdmin,
    canStatusScan: isShengAdmin || isShiAdmin,
    canBusinessWrite: true,
    canViewSystemSettings: isShengAdmin || isShiAdmin,
    isShengMainSlot,
  };
}

export interface AuthContextValue {
  auth: AdminAuth | null;
  setAuth: (auth: AdminAuth | null) => void;
  logout: () => void;
  capabilities: RoleCapabilities;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export interface AuthProviderProps {
  children: React.ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [auth, setAuthState] = useState<AdminAuth | null>(() => readStoredAuth());

  // 中文注释:auth 变化时同步 sessionStorage。null 时走 clearStoredAuth。
  useEffect(() => {
    if (auth) {
      writeStoredAuth(auth);
    } else {
      clearStoredAuth();
    }
  }, [auth]);

  const setAuth = useCallback((next: AdminAuth | null) => {
    setAuthState(next);
  }, []);

  const logout = useCallback(() => {
    setAuthState(null);
  }, []);

  // ── 401 拦截：token 失效时自动登出 + 提示 ──
  const logoutRef = useRef(logout);
  logoutRef.current = logout;
  useEffect(() => {
    setOnUnauthorized(() => {
      message.warning('登录已过期，请重新登录');
      logoutRef.current();
    });
    return () => setOnUnauthorized(null);
  }, []);

  const capabilities = useMemo(() => resolveRoleCapabilities(auth), [auth]);

  const value = useMemo<AuthContextValue>(
    () => ({ auth, setAuth, logout, capabilities }),
    [auth, setAuth, logout, capabilities],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

export function useAuthContext(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error('useAuthContext 必须在 <AuthProvider> 内部使用');
  }
  return ctx;
}
