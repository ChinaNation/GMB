// 认证上下文：HttpOnly Cookie 承载登录态，前端只保存用户镜像

import { createContext, useContext, useEffect, useState, useCallback, type ReactNode } from 'react';
import { authMe } from '../login/api';
import type { SessionUser } from '../common/types';

interface AuthState {
  user: SessionUser | null;
  ready: boolean;
  login: (user: SessionUser) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [ready, setReady] = useState(false);
  const [user, setUser] = useState<SessionUser | null>(() => {
    const raw = sessionStorage.getItem('cpms_user');
    return raw ? JSON.parse(raw) : null;
  });

  useEffect(() => {
    authMe()
      .then(res => {
        if (res.data) {
          sessionStorage.setItem('cpms_user', JSON.stringify(res.data));
          setUser(res.data);
        }
      })
      .catch(() => {
        sessionStorage.removeItem('cpms_user');
        setUser(null);
      })
      .finally(() => setReady(true));
  }, []);

  const login = useCallback((u: SessionUser) => {
    sessionStorage.setItem('cpms_user', JSON.stringify(u));
    setUser(u);
  }, []);

  const logout = useCallback(() => {
    sessionStorage.removeItem('cpms_user');
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider value={{ user, ready, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be inside AuthProvider');
  return ctx;
}
