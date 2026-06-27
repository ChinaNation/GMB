// 中文注释:cid 前端登录态 + 能力标志的全局 Context。
// 中文注释:角色仅剩 FEDERAL_REGISTRY / CITY_REGISTRY。

import React, { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import type { AdminAuth } from './types';
import { setOnUnauthorized } from '../utils/http';
import { clearStoredAuth, readStoredAuth, writeStoredAuth } from '../utils/storedAuth';
import { notice } from '../utils/notice';

export type RoleCapabilities = {
  canViewCitizens: boolean;
  canViewInstitutions: boolean;
  canViewPrivate: boolean;
  canViewEducation: boolean;
  canViewFederalRegistryAdmins: boolean;
  canViewCityRegistryAdmins: boolean;
  canCrudCityRegistryAdmins: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canBusinessWrite: boolean;
  canViewCityRegistry: boolean;
  canViewFederalRegistry: boolean;
};

// 中文注释:联邦注册局管理员只管「联邦注册局 + 市注册局」两个 tab——
// 默认进联邦注册局(看本省 5 名联邦管理员),再看市注册局(本省所有市注册局列表)。
// 公民录入、机构(公权/私权/教育)录入都是市注册局的职责,联邦一律不可见。
export function resolveRoleCapabilities(auth: AdminAuth | null): RoleCapabilities {
  const registry_org_code = auth?.registry_org_code;
  const isFederalRegistry = registry_org_code === 'FEDERAL_REGISTRY';
  const isCityRegistry = registry_org_code === 'CITY_REGISTRY';
  return {
    canViewCitizens: isCityRegistry,
    canViewInstitutions: isCityRegistry,
    canViewPrivate: isCityRegistry,
    canViewEducation: isCityRegistry,
    canViewFederalRegistryAdmins: isFederalRegistry,
    canViewCityRegistryAdmins: isFederalRegistry || isCityRegistry,
    canCrudCityRegistryAdmins: isFederalRegistry,
    canManageInstitutions: isCityRegistry,
    canRegisterInstitutions: isCityRegistry,
    canBusinessWrite: isCityRegistry,
    canViewCityRegistry: isFederalRegistry || isCityRegistry,
    canViewFederalRegistry: isFederalRegistry,
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
