// 中文注释:cid 前端管理员登录态持久化工具。
// 从 App.tsx 里抽出来的 readStoredAuth / writeStoredAuth / clearStoredAuth,
// 语义保持一致(sessionStorage + 结构校验),供 AuthContext 和老 App.tsx 共用。

import type { AdminAuth } from '../auth/types';

// 中文注释:缓存版本号。删除 passkey/passkey_bound 后 bump v1→v2,
// 让带有已删除 passkey_bound 字段的旧缓存对象自动失效。
const AUTH_STORAGE_KEY = 'cid_admin_auth_v2';

export function readStoredAuth(): AdminAuth | null {
  try {
    const raw = sessionStorage.getItem(AUTH_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<AdminAuth> & { passkey_bound?: unknown };
    if (
      typeof parsed === 'object' &&
      parsed &&
      'access_token' in parsed &&
      typeof parsed.access_token === 'string' &&
      typeof parsed.admin_account === 'string' &&
      typeof parsed.registry_org_code === 'string'
    ) {
      // 中文注释:自愈——即便旧缓存仍带已删除的 passkey_bound,也只丢弃该字段,
      // 不让结构校验失败而清空整个登录态。
      const { passkey_bound: _drop, ...rest } = parsed;
      void _drop;
      return rest as AdminAuth;
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
