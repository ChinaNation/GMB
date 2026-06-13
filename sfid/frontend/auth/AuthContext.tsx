// 中文注释:sfid 前端登录态 + 能力标志的全局 Context。
// 中文注释:角色仅剩 FEDERAL_ADMIN / CITY_ADMIN。
// Passkey 绑定状态只用于引导管理员进入注册局更新密钥,不改变角色能力。

import React, { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import type { AdminAuth } from './types';
import { setOnUnauthorized } from '../utils/http';
import { clearStoredAuth, readStoredAuth, writeStoredAuth } from '../utils/storedAuth';
import { notice } from '../utils/notice';

export type RoleCapabilities = {
  canViewInstitutions: boolean;
  canViewPrivate: boolean;
  canViewEducation: boolean;
  canViewFederalAdmins: boolean;
  canViewCityAdmins: boolean;
  canCrudCityAdmins: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
  canViewCityRegistry: boolean;
  canViewFederalRegistry: boolean;
};

export function resolveRoleCapabilities(auth: AdminAuth | null): RoleCapabilities {
  const role = auth?.role;
  const isFederalAdmin = role === 'FEDERAL_ADMIN';
  const isCityAdmin = role === 'CITY_ADMIN';
  return {
    canViewInstitutions: isFederalAdmin || isCityAdmin,
    canViewPrivate: isFederalAdmin || isCityAdmin,
    canViewEducation: isFederalAdmin || isCityAdmin,
    canViewFederalAdmins: isFederalAdmin,
    canViewCityAdmins: isFederalAdmin || isCityAdmin,
    canCrudCityAdmins: isFederalAdmin,
    canManageInstitutions: isFederalAdmin || isCityAdmin,
    canRegisterInstitutions: isFederalAdmin || isCityAdmin,
    canStatusScan: isFederalAdmin || isCityAdmin,
    canBusinessWrite: isFederalAdmin || isCityAdmin,
    canViewCityRegistry: isFederalAdmin || isCityAdmin,
    canViewFederalRegistry: isFederalAdmin || isCityAdmin,
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
      notice.warning('登录已过期，请重新登录');
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
