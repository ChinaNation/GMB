// 认证上下文：管理 token、user、角色状态

import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import type { SessionUser } from './types';

interface AuthState {
  token: string | null;
  user: SessionUser | null;
  login: (token: string, user: SessionUser) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthState | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem('cpms_token'));
  const [user, setUser] = useState<SessionUser | null>(() => {
    const raw = localStorage.getItem('cpms_user');
    return raw ? JSON.parse(raw) : null;
  });

  const login = useCallback((t: string, u: SessionUser) => {
    localStorage.setItem('cpms_token', t);
    localStorage.setItem('cpms_user', JSON.stringify(u));
    setToken(t);
    setUser(u);
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem('cpms_token');
    localStorage.removeItem('cpms_user');
    setToken(null);
    setUser(null);
  }, []);

  return (
    <AuthContext.Provider value={{ token, user, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be inside AuthProvider');
  return ctx;
}
