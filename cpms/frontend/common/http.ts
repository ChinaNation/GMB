// CPMS 前端 HTTP 封装：业务 API 必须放回所属模块目录。

import type { ApiError, ApiResponse } from './types';

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {}),
  };

  const res = await fetch(url, { ...options, headers, credentials: 'same-origin' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText })) as Partial<ApiError>;
    // 中文注释：登录态由 HttpOnly Cookie 承载；401 时只清理前端用户镜像。
    if (res.status === 401) {
      sessionStorage.removeItem('cpms_user');
      if (!['/login', '/install'].includes(window.location.pathname)) {
        window.location.href = '/login';
      }
    }
    throw new Error(err.message || `HTTP ${res.status}`);
  }
  return res.json();
}

export function get<T>(url: string) {
  return request<ApiResponse<T>>(url);
}

export function post<T>(url: string, body?: unknown) {
  return request<ApiResponse<T>>(url, {
    method: 'POST',
    body: body ? JSON.stringify(body) : undefined,
  });
}

export function put<T>(url: string, body?: unknown) {
  return request<ApiResponse<T>>(url, {
    method: 'PUT',
    body: body ? JSON.stringify(body) : undefined,
  });
}

export function del<T>(url: string) {
  return request<ApiResponse<T>>(url, { method: 'DELETE' });
}
