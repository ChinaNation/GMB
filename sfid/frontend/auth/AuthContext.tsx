// 中文注释:sfid 前端登录态 + 能力标志的全局 Context。
// 中文注释:角色仅剩 SHENG_ADMIN / SHI_ADMIN。
// Passkey 绑定状态只用于引导管理员进入注册局更新密钥,不改变角色能力。

import React, { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { message } from 'antd';
import type { AdminAuth } from './types';
import { setOnUnauthorized } from '../utils/http';
import { clearStoredAuth, readStoredAuth, writeStoredAuth } from '../utils/storedAuth';

export type RoleCapabilities = {
  canViewInstitutions: boolean;
  canViewPrivate: boolean;
  canViewShengAdmins: boolean;
  canViewShiAdmins: boolean;
  canCrudShiAdmins: boolean;
  canManageInstitutions: boolean;
  canRegisterInstitutions: boolean;
  canStatusScan: boolean;
  canBusinessWrite: boolean;
  canViewSystemSettings: boolean;
};

export function resolveRoleCapabilities(auth: AdminAuth | null): RoleCapabilities {
  const role = auth?.role;
  const isShengAdmin = role === 'SHENG_ADMIN';
  const isShiAdmin = role === 'SHI_ADMIN';
  return {
    canViewInstitutions: isShengAdmin || isShiAdmin,
    canViewPrivate: isShengAdmin || isShiAdmin,
    canViewShengAdmins: isShengAdmin,
    canViewShiAdmins: isShengAdmin || isShiAdmin,
    canCrudShiAdmins: isShengAdmin,
    canManageInstitutions: isShengAdmin || isShiAdmin,
    canRegisterInstitutions: isShengAdmin || isShiAdmin,
    canStatusScan: isShengAdmin || isShiAdmin,
    canBusinessWrite: isShengAdmin || isShiAdmin,
    canViewSystemSettings: isShengAdmin || isShiAdmin,
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
