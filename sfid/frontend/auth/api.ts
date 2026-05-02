// 中文注释:登录、登出、会话校验与二维码登录 API。
// 通用 HTTP 能力在 utils/http.ts;本文件只放 auth 模块自己的后端接口。

import { adminHeaders, request } from '../utils/http';
import type { AdminAuth } from './types';

export type AdminAuthCheck = {
  ok: boolean;
  admin_pubkey: string;
  role: 'SHENG_ADMIN' | 'SHI_ADMIN';
  admin_name: string;
  admin_province?: string | null;
  admin_city?: string | null;
};

export type AdminIdentifyResult = {
  admin_pubkey: string;
  role: 'SHENG_ADMIN' | 'SHI_ADMIN';
  status: 'ACTIVE' | 'DISABLED';
  admin_name: string;
  admin_province?: string | null;
  admin_city?: string | null;
};

export type AdminChallengeResult = {
  challenge_id: string;
  challenge_payload: string;
  origin: string;
  domain: string;
  session_id: string;
  nonce: string;
  expire_at: number;
};

export type AdminQrChallengeResult = {
  challenge_id: string;
  challenge_payload: string;
  login_qr_payload: string;
  origin: string;
  domain: string;
  session_id: string;
  nonce: string;
  expire_at: number;
};

export type AdminVerifyResult = {
  access_token: string;
  expire_at: number;
  admin: AdminIdentifyResult;
};

export type AdminQrLoginStatus = {
  status: 'PENDING' | 'SUCCESS' | 'EXPIRED';
  message: string;
  access_token?: string;
  expire_at?: number;
  admin?: AdminIdentifyResult;
};

export type AdminDemoSignResult = {
  challenge_id: string;
  admin_pubkey: string;
  signature: string;
};

export async function identifyAdmin(identityQr: string): Promise<AdminIdentifyResult> {
  return request<AdminIdentifyResult>('/api/v1/admin/auth/identify', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ identity_qr: identityQr }),
  });
}

export async function createAdminChallenge(input: {
  admin_pubkey: string;
  origin: string;
  session_id: string;
}): Promise<AdminChallengeResult> {
  return request<AdminChallengeResult>('/api/v1/admin/auth/challenge', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export async function createAdminQrChallenge(input: {
  origin: string;
  session_id: string;
}): Promise<AdminQrChallengeResult> {
  return request<AdminQrChallengeResult>('/api/v1/admin/auth/qr/challenge', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export async function queryAdminQrLoginResult(
  challengeId: string,
  sessionId: string,
): Promise<AdminQrLoginStatus> {
  const q = `?challenge_id=${encodeURIComponent(challengeId)}&session_id=${encodeURIComponent(sessionId)}`;
  return request<AdminQrLoginStatus>(`/api/v1/admin/auth/qr/result${q}`, {
    method: 'GET',
  });
}

export async function completeAdminQrLogin(input: {
  challenge_id: string;
  session_id?: string;
  admin_pubkey: string;
  signer_pubkey?: string;
  signature: string;
}): Promise<string> {
  return request<string>('/api/v1/admin/auth/qr/complete', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export async function verifyAdminChallenge(input: {
  challenge_id: string;
  origin: string;
  domain: string;
  session_id: string;
  nonce: string;
  signature: string;
}): Promise<AdminVerifyResult> {
  return request<AdminVerifyResult>('/api/v1/admin/auth/verify', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export async function demoSignChallenge(input: {
  challenge_id: string;
  admin_pubkey: string;
}): Promise<AdminDemoSignResult> {
  return request<AdminDemoSignResult>('/api/v1/admin/auth/demo-sign', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

/** 主动登出:通知后端销毁 session。best-effort,不阻塞前端退出流程。 */
export async function adminLogout(auth: AdminAuth): Promise<void> {
  try {
    await request<string>('/api/v1/admin/auth/logout', {
      method: 'POST',
      headers: adminHeaders(auth),
    });
  } catch {
    // 静默:即使后端不可达也不影响前端退出。
  }
}

export async function checkAdminAuth(auth: AdminAuth): Promise<AdminAuthCheck> {
  return request<AdminAuthCheck>('/api/v1/admin/auth/check', {
    headers: adminHeaders(auth),
  });
}
