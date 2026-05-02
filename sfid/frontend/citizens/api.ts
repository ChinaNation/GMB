// 中文注释:公民绑定、解绑、链上同步和 CPMS 状态扫码 API。
// 通用请求能力只从 utils/http.ts 引入,本文件不承接机构或管理员模块接口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

export type CitizenRow = {
  id: number;
  account_pubkey?: string;
  account_address?: string;
  archive_no?: string;
  sfid_code?: string;
  province_code?: string;
  status: 'PENDING' | 'BINDABLE' | 'BOUND' | 'UNLINKED';
};

export type CitizenBindChallengeResult = {
  challenge_id: string;
  challenge_text: string;
  /** WUMIN_QR_V1 签名请求 JSON,前端直接展示为二维码。 */
  sign_request: string;
  expire_at: number;
};

export type CitizenBindResult = {
  id: number;
  account_pubkey?: string;
  account_address?: string;
  archive_no?: string;
  sfid_code?: string;
  province_code?: string;
  status: 'PENDING' | 'BINDABLE' | 'BOUND' | 'UNLINKED';
};

export type CitizenPushChainResult = {
  citizen_id: number;
  tx_hash: string;
};

export type CpmsStatusScanResult = {
  archive_no: string;
  status: 'NORMAL' | 'ABNORMAL';
  message: string;
};

export async function listCitizens(auth: AdminAuth, keyword?: string): Promise<CitizenRow[]> {
  const q = keyword ? `?keyword=${encodeURIComponent(keyword)}` : '';
  return request<CitizenRow[]>(`/api/v1/admin/citizens${q}`, {
    headers: adminHeaders(auth),
  });
}

export async function citizenBindChallenge(auth: AdminAuth): Promise<CitizenBindChallengeResult> {
  return request<CitizenBindChallengeResult>('/api/v1/admin/citizen/bind/challenge', {
    method: 'POST',
    headers: adminHeaders(auth),
  });
}

export async function citizenBind(
  auth: AdminAuth,
  payload: {
    mode: 'bind_archive' | 'bind_pubkey';
    user_address: string;
    qr4_payload?: string;
    citizen_id?: number;
    challenge_id: string;
    signature: string;
  },
): Promise<CitizenBindResult> {
  return request<CitizenBindResult>('/api/v1/admin/citizen/bind', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function citizenUnbind(
  auth: AdminAuth,
  payload: { citizen_id: number; challenge_id: string; signature: string },
): Promise<CitizenBindResult> {
  return request<CitizenBindResult>('/api/v1/admin/citizen/unbind', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function citizenPushChainBind(
  auth: AdminAuth,
  payload: { citizen_id: number },
): Promise<CitizenPushChainResult> {
  return request<CitizenPushChainResult>('/api/v1/admin/citizen/bind/push-chain', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function citizenPushChainUnbind(
  auth: AdminAuth,
  payload: { citizen_id: number },
): Promise<CitizenPushChainResult> {
  return request<CitizenPushChainResult>('/api/v1/admin/citizen/unbind/push-chain', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function scanCpmsStatusQr(
  auth: AdminAuth,
  payload: { qr_payload: string },
): Promise<CpmsStatusScanResult> {
  return request<CpmsStatusScanResult>('/api/v1/admin/cpms-status/scan', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}
