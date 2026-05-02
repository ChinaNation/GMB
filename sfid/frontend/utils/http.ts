// 中文注释:前端通用 HTTP 工具。这里只做请求、鉴权头和 401 拦截,
// 不放任何业务 API;业务 API 必须放回所属功能模块目录。

import type { AdminAuth } from '../auth/types';

let onUnauthorized: (() => void) | null = null;
let unauthorizedFired = false;

/** AuthProvider 启动时注册回调;卸载时传 null 清除。 */
export function setOnUnauthorized(cb: (() => void) | null) {
  onUnauthorized = cb;
  unauthorizedFired = false;
}

/** 所有请求使用相对路径,由 Vite(开发) / Nginx(生产) 统一代理到后端。 */
export async function adminRequest<T>(
  path: string,
  auth: AdminAuth,
  init?: RequestInit,
): Promise<T> {
  return request<T>(path, {
    ...init,
    headers: {
      ...adminHeaders(auth),
      ...(init?.headers || {}),
    },
  });
}

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let resp: Response;
  try {
    resp = await fetch(path, init);
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    throw new Error(`无法连接服务器：${msg}`);
  }

  const text = await resp.text();
  let body: any = null;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    const snippet = text.slice(0, 120);
    throw new Error(
      `服务响应格式错误(${resp.status})：${snippet || 'empty body'}，请确认后端已重启到最新版本`,
    );
  }

  if (resp.status === 401) {
    if (onUnauthorized && !unauthorizedFired) {
      unauthorizedFired = true;
      onUnauthorized();
    }
    return undefined as unknown as T;
  }

  if (!resp.ok || !body || body.code !== 0) {
    throw new Error(body?.message ?? `request failed (${resp.status})`);
  }
  return body.data as T;
}

export function adminHeaders(auth: AdminAuth): HeadersInit {
  return {
    authorization: `Bearer ${auth.access_token}`,
  };
}
