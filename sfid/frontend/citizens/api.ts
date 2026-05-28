// 中文注释:公民电子护照绑定和 CPMS 状态扫码 API。
// 通用请求能力只从 utils/http.ts 引入,本文件不承接机构或管理员模块接口。

import type { AdminAuth } from '../auth/types';
import { adminHeaders, request } from '../utils/http';

export type CitizenRow = {
  id: number;
  wallet_pubkey?: string;
  wallet_address?: string;
  archive_no?: string;
  sfid_code?: string;
  province_code?: string;
  city_code?: string;
  archive_status?: 'NORMAL' | 'ABNORMAL';
  identity_status?: 'NORMAL' | 'ABNORMAL';
  valid_from?: string;
  valid_until?: string;
  status_updated_at?: number;
  bind_status: 'PENDING' | 'BOUND';
};

export type CitizenBindChallengeResult = {
  challenge_id: string;
  challenge_text: string;
  mode: 'create' | 'replace';
  archive_no: string;
  wallet_address: string;
  wallet_pubkey: string;
  archive_status: 'NORMAL' | 'ABNORMAL';
  valid_from: string;
  valid_until: string;
  status_updated_at: number;
  /** WUMIN_QR_V1 签名请求 JSON,前端直接展示为二维码。 */
  sign_request: string;
  expire_at: number;
};

export type CitizenBindResult = {
  id: number;
  wallet_pubkey?: string;
  wallet_address?: string;
  archive_no?: string;
  sfid_code?: string;
  archive_status?: 'NORMAL' | 'ABNORMAL';
  identity_status: 'NORMAL' | 'ABNORMAL';
  valid_from?: string;
  valid_until?: string;
  status_updated_at?: number;
  province_code?: string;
  city_code?: string;
  bind_status: 'PENDING' | 'BOUND';
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

export async function citizenBindChallenge(
  auth: AdminAuth,
  payload: {
    mode: 'create' | 'replace';
    archive_code_payload: string;
    citizen_id?: number;
  },
): Promise<CitizenBindChallengeResult> {
  return request<CitizenBindChallengeResult>('/api/v1/admin/citizen/bind/challenge', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth),
    },
    body: JSON.stringify(payload),
  });
}

export async function citizenBind(
  auth: AdminAuth,
  payload: {
    challenge_id: string;
    pubkey: string;
    signature: string;
    payload_hash: string;
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
