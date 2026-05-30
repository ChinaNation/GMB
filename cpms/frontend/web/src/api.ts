// CPMS 后端 API 封装

import type {
  ApiResponse, ApiError, AdminUser, Archive,
  CpmsStatusExportFile, InstallStatus,
} from './types';

async function request<T>(url: string, options: RequestInit = {}): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };

  const res = await fetch(url, { ...options, headers, credentials: 'same-origin' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText })) as Partial<ApiError>;
    // 中文注释:登录态由 HttpOnly Cookie 承载；401 时只清理前端用户镜像。
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
  post<{ sfid_number: string }>('/api/v1/install/initialize', { sfid_init_qr_content });
export const bindSuperAdmin = (admin_pubkey: string) =>
  post<AdminUser>('/api/v1/install/super-admin/bind', { admin_pubkey });

// ── 认证 ──
export const authLogout = () => post<null>('/api/v1/admin/auth/logout');
export const authMe = () => get<{ user_id: string; role: string }>('/api/v1/admin/auth/me');
export const authQrChallenge = () => post<{ challenge_id: string; login_qr_payload: string; session_id: string; expire_at: number }>('/api/v1/admin/auth/qr/challenge', {
  origin: window.location.origin,
});
export const authQrComplete = (body: {
  challenge_id: string;
  session_id: string;
  admin_pubkey: string;
  signature: string;
}) => post<null>('/api/v1/admin/auth/qr/complete', body);

export const authQrResult = (challenge_id: string, session_id: string) =>
  get<{ status: string; expires_in?: number; user?: { user_id: string; role: string } }>(
    `/api/v1/admin/auth/qr/result?challenge_id=${encodeURIComponent(challenge_id)}&session_id=${encodeURIComponent(session_id)}`
  );

// ── 超级管理员 ──
export const listOperators = () => get<AdminUser[]>('/api/v1/admin/operators');
export const createOperator = (admin_pubkey: string, admin_name: string) =>
  post<AdminUser>('/api/v1/admin/operators', { admin_pubkey, admin_name });
export const deleteOperator = (id: string) => del<null>(`/api/v1/admin/operators/${id}`);
export const updateCitizenStatus = (archive_id: string, citizen_status: string) =>
  put<{ archive_id: string; citizen_status: string }>(`/api/v1/archives/${archive_id}/citizen-status`, { citizen_status });

// ── 地址管理 ──
export const listTowns = () => get<{ town_code: string; town_name: string }[]>('/api/v1/address/towns');
export const listVillages = (town_code: string) => get<{ village_id: string; town_code: string; village_name: string }[]>(`/api/v1/address/villages?town_code=${encodeURIComponent(town_code)}`);

// ── 操作员 ──
export const createArchive = (body: {
  last_name: string; first_name: string; birth_date: string; gender_code: string; height_cm: number;
  town_code?: string; village_id?: string; address?: string;
  citizen_status?: string; voting_eligible?: boolean;
}) => post<{ archive_id: string; archive_no: string; passport_no: string }>('/api/v1/archives', body);
export const listArchives = (params?: { q?: string; page?: number; page_size?: number }) => {
  const qs = new URLSearchParams();
  if (params?.q) qs.set('q', params.q);
  if (params?.page) qs.set('page', String(params.page));
  if (params?.page_size) qs.set('page_size', String(params.page_size));
  const q = qs.toString();
  return get<{ items: Archive[]; total: number }>(`/api/v1/archives${q ? '?' + q : ''}`);
};
export const getArchive = (id: string) => get<Archive>(`/api/v1/archives/${id}`);
export const updateArchive = (id: string, body: Record<string, unknown>) =>
  put<Archive>(`/api/v1/archives/${id}`, body);
export const bindArchiveWallet = (id: string, wallet_address: string) =>
  post<Archive>(`/api/v1/archives/${id}/wallet`, { wallet_address });
export const generateArchiveQr = (id: string) =>
  post<{ qr_payload: unknown; qr_content: string }>(`/api/v1/archives/${id}/qr/generate`);
export const printArchiveQr = (id: string) =>
  post<{ print_id: string; archive_id: string; archive_no: string; printed_at: number }>(`/api/v1/archives/${id}/qr/print`);
export const createArchiveDeleteChallenge = (id: string) =>
  post<{ challenge_id: string; sign_request: string; expire_at: number }>(`/api/v1/archives/${id}/delete/challenge`);
export const completeArchiveDelete = (id: string, body: {
  challenge_id: string;
  pubkey: string;
  sig_alg: 'sr25519';
  signature: string;
  payload_hash: string;
  signed_at: number;
}) => post<{ archive_id: string; deleted_at: number; deleted_by: string }>(`/api/v1/archives/${id}/delete/complete`, body);
export const exportStatusFile = () =>
  get<{ file_name: string; export_file: CpmsStatusExportFile }>('/api/v1/archives/status-export');

// ── 健康检查 ──
export const health = () => get<{ status: string }>('/api/v1/health');
