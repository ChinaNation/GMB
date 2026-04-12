// CPMS 后端 API 封装

import type {
  ApiResponse, AdminUser, Archive, ChallengeData, VerifyData,
  InstallStatus,
} from './types';

function getToken(): string | null {
  return localStorage.getItem('cpms_token');
}

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(url, { ...options, headers });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    // token 过期自动退出登录
    if (res.status === 401 && token) {
      localStorage.removeItem('cpms_token');
      localStorage.removeItem('cpms_user');
      window.location.href = '/login';
    }
    throw new Error(err.message || `HTTP ${res.status}`);
  }
  return res.json();
}

function get<T>(url: string) { return request<ApiResponse<T>>(url); }
function post<T>(url: string, body?: unknown) {
  return request<ApiResponse<T>>(url, { method: 'POST', body: body ? JSON.stringify(body) : undefined });
}
function put<T>(url: string, body?: unknown) {
  return request<ApiResponse<T>>(url, { method: 'PUT', body: body ? JSON.stringify(body) : undefined });
}
function del<T>(url: string) { return request<ApiResponse<T>>(url, { method: 'DELETE' }); }

// ── 系统初始化 ──
export const installStatus = () => get<InstallStatus>('/api/v1/install/status');
export const installInitialize = (sfid_init_qr_content: string) =>
  post<{ site_sfid: string }>('/api/v1/install/initialize', { sfid_init_qr_content });
export const bindSuperAdmin = (admin_pubkey: string) =>
  post<AdminUser>('/api/v1/install/super-admin/bind', { admin_pubkey });
export const adminGenerateQr2 = () =>
  post<{ qr2_payload: string }>('/api/v1/admin/generate-qr2');
export const adminProcessAnonCert = (sfid_anon_cert_qr_content: string) =>
  post<string>('/api/v1/admin/anon-cert', { sfid_anon_cert_qr_content });

// ── 认证 ──
export const authIdentify = (admin_pubkey: string) =>
  post<{ user_id: string; role: string; status: string }>('/api/v1/admin/auth/identify', { admin_pubkey });
export const authChallenge = (admin_pubkey: string) =>
  post<ChallengeData>('/api/v1/admin/auth/challenge', { admin_pubkey });
export const authVerify = (challenge_id: string, admin_pubkey: string, signature: string) =>
  post<VerifyData>('/api/v1/admin/auth/verify', { challenge_id, admin_pubkey, signature });
export const authLogout = () => post<null>('/api/v1/admin/auth/logout');
export const authQrChallenge = () => post<{ challenge_id: string; login_qr_payload: string; session_id: string; expire_at: number }>('/api/v1/admin/auth/qr/challenge', {
  origin: window.location.origin,
  session_id: `sid-${Date.now()}-${Math.random().toString(16).slice(2)}`,
});
export const authQrComplete = (body: {
  challenge_id: string;
  session_id: string;
  admin_pubkey: string;
  signature: string;
}) => post<null>('/api/v1/admin/auth/qr/complete', body);

export const authQrResult = (challenge_id: string, session_id: string) =>
  get<{ status: string; access_token?: string; expires_in?: number; user?: { user_id: string; role: string } }>(
    `/api/v1/admin/auth/qr/result?challenge_id=${challenge_id}&session_id=${session_id}`
  );

// ── 超级管理员 ──
export const listOperators = () => get<AdminUser[]>('/api/v1/admin/operators');
export const createOperator = (admin_pubkey: string, admin_name: string) =>
  post<AdminUser>('/api/v1/admin/operators', { admin_pubkey, admin_name });
export const updateOperatorStatus = (id: string, status: string) =>
  put<null>(`/api/v1/admin/operators/${id}/status`, { status });
export const deleteOperator = (id: string) => del<null>(`/api/v1/admin/operators/${id}`);
export const updateCitizenStatus = (archive_id: string, citizen_status: string) =>
  put<{ archive_id: string; citizen_status: string }>(`/api/v1/archives/${archive_id}/citizen-status`, { citizen_status });

// ── 地址管理 ──
export const listTowns = () => get<{ town_code: string; town_name: string }[]>('/api/v1/address/towns');
export const listVillages = (town_code: string) => get<{ village_id: string; town_code: string; village_name: string }[]>(`/api/v1/address/villages?town_code=${encodeURIComponent(town_code)}`);
export const createTown = (town_code: string, town_name: string) => post<{ town_code: string; town_name: string }>('/api/v1/address/towns', { town_code, town_name });
export const deleteTown = (code: string) => del<null>(`/api/v1/address/towns/${code}`);
export const createVillage = (town_code: string, village_name: string) => post<{ village_id: string; town_code: string; village_name: string }>('/api/v1/address/villages', { town_code, village_name });
export const deleteVillage = (id: string) => del<null>(`/api/v1/address/villages/${id}`);

// ── 操作员 ──
export const createArchive = (body: {
  province_code: string; city_code: string; full_name: string;
  birth_date: string; gender_code: string; height_cm?: number;
  town_code?: string; village_id?: string; address?: string;
  citizen_status?: string; voting_eligible?: boolean;
}) => post<{ archive_id: string; archive_no: string }>('/api/v1/archives', body);
export const listArchives = (params?: { full_name?: string; page?: number; page_size?: number }) => {
  const qs = new URLSearchParams();
  if (params?.full_name) qs.set('full_name', params.full_name);
  if (params?.page) qs.set('page', String(params.page));
  if (params?.page_size) qs.set('page_size', String(params.page_size));
  const q = qs.toString();
  return get<{ items: Archive[]; total: number }>(`/api/v1/archives${q ? '?' + q : ''}`);
};
export const getArchive = (id: string) => get<Archive>(`/api/v1/archives/${id}`);
export const updateArchive = (id: string, body: Record<string, unknown>) =>
  put<Archive>(`/api/v1/archives/${id}`, body);

// ── 健康检查 ──
export const health = () => get<{ status: string }>('/api/v1/health');
