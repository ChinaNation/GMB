// CPMS 前端 HTTP 封装：业务 API 必须放回所属模块目录。

import type { ApiError, ApiResponse } from './types';

async function readNonJsonMessage(res: Response, url: string): Promise<string> {
  const text = await res.text().catch(() => '');
  const trimmed = text.trim();
  if (trimmed.startsWith('<!DOCTYPE') || trimmed.startsWith('<html')) {
    const requestPath = new URL(url, window.location.origin).pathname;
    return `接口返回了页面HTML，请确认 ${requestPath} 已命中 API 路由`;
  }
  return trimmed || res.statusText || `HTTP ${res.status}`;
}

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {}),
  };

  const res = await fetch(url, { ...options, headers, credentials: 'same-origin' });
  const contentType = res.headers.get('content-type') || '';
  const isJson = contentType.includes('application/json');
  if (!res.ok) {
    const err = isJson
      ? await res.json().catch(() => ({ message: res.statusText })) as Partial<ApiError>
      : { message: await readNonJsonMessage(res, url) };
    // 中文注释：登录态由 HttpOnly Cookie 承载；401 只通知认证上下文清理用户镜像，页面去向交给路由判断。
    if (res.status === 401) {
      sessionStorage.removeItem('cpms_user');
      window.dispatchEvent(new Event('cpms-auth-expired'));
    }
    throw new Error(err.message || `HTTP ${res.status}`);
  }
  if (!isJson) {
    throw new Error(await readNonJsonMessage(res, url));
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
