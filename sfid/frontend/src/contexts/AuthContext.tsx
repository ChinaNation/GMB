// 中文注释:sfid 前端登录态 + 能力标志的全局 Context。
// 步 0 目标:让 App.tsx 不再自己管 auth state,改成从 useAuth() 拿。
// 能力标志暂时对齐 App.tsx 原来的 resolveRoleCapabilities(也就是 RoleCapabilities 形状),
// 避免步 0 同时改动 2000+ 行 capabilities 调用点。

import React, { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { message } from 'antd';
import type { AdminAuth } from '../api/client';
import { setOnUnauthorized } from '../api/client';
import { clearStoredAuth, readStoredAuth, writeStoredAuth } from '../utils/storedAuth';

export type RoleCapabilities = {
  canViewInstitutions: boolean;
  canViewMultisig: boolean;
  canViewKeyring: boolean;
  canViewShengAdmins: boolean;
  canViewShiAdmins: boolean;
  canCrudShiAdmins: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canReplaceShengAdmins: boolean;
  canManageKeyring: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
  canViewSystemSettings: boolean;
};

export function resolveRoleCapabilities(auth: AdminAuth | null): RoleCapabilities {
  const role = auth?.role;
  const isKeyAdmin = role === 'KEY_ADMIN';
  const isShengAdmin = role === 'SHENG_ADMIN';
  const isShiAdmin = role === 'SHI_ADMIN';
  return {
    canViewInstitutions: isKeyAdmin || isShengAdmin,
    canViewMultisig: isKeyAdmin || isShengAdmin || isShiAdmin,
    canViewKeyring: isKeyAdmin,
    canViewShengAdmins: isKeyAdmin || isShengAdmin,
    canViewShiAdmins: isKeyAdmin || isShengAdmin || isShiAdmin,
    canCrudShiAdmins: isKeyAdmin || isShengAdmin,
    canManageInstitutions: isKeyAdmin || isShengAdmin,
    canRegisterInstitutions: isKeyAdmin || isShengAdmin,
    canReplaceShengAdmins: isKeyAdmin,
    canManageKeyring: isKeyAdmin,
    canStatusScan: isKeyAdmin || isShengAdmin || isShiAdmin,
    canBusinessWrite: true,
    canViewSystemSettings: isKeyAdmin || isShengAdmin || isShiAdmin,
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
