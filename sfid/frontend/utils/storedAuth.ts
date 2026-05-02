// 中文注释:sfid 前端管理员登录态持久化工具。
// 从 App.tsx 里抽出来的 readStoredAuth / writeStoredAuth / clearStoredAuth,
// 语义保持一致(sessionStorage + 结构校验),供 AuthContext 和老 App.tsx 共用。

import type { AdminAuth } from '../api/client';

const AUTH_STORAGE_KEY = 'sfid_admin_auth_v1';

export function readStoredAuth(): AdminAuth | null {
  try {
    const raw = sessionStorage.getItem(AUTH_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<AdminAuth>;
    if (
      typeof parsed === 'object' &&
      parsed &&
      'access_token' in parsed &&
      typeof parsed.access_token === 'string' &&
      typeof parsed.admin_pubkey === 'string' &&
      typeof parsed.role === 'string'
    ) {
      return parsed as AdminAuth;
    }
    return null;
  } catch {
    return null;
  }
}

export function writeStoredAuth(auth: AdminAuth) {
  try {
    sessionStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(auth));
  } catch {
    // 静默:隐身模式或 sessionStorage 不可用时不阻塞流程
  }
}

// 中文注释:保留 saveStoredAuth 别名,任务卡要求里提到的名字之一。
export const saveStoredAuth = writeStoredAuth;

export function clearStoredAuth() {
  try {
    sessionStorage.removeItem(AUTH_STORAGE_KEY);
  } catch {
    // 静默
  }
}
