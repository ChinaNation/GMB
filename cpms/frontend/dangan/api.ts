import { del, get, post, put } from '../common/http';
import type { ApiError, ApiResponse } from '../common/types';
import type { Archive, ArchiveMaterial, CreateArchiveRequest } from './types';

export const createArchive = (body: CreateArchiveRequest) =>
  post<{ archive_id: string; archive_no: string; passport_no: string }>('/api/v1/archives', body);

export interface ArchiveListParams {
  limit?: number;
  cursor?: string;
  search?: string;
  birth_date?: string;
  town_code?: string;
  village_id?: string;
  citizen_status?: string;
}

export interface ArchiveListResponse {
  items: Archive[];
  limit: number;
  next_cursor: string | null;
  has_next: boolean;
  total_active: number;
}

export const listArchives = (params?: ArchiveListParams) => {
  const qs = new URLSearchParams();
  if (params?.limit) qs.set('limit', String(params.limit));
  if (params?.cursor) qs.set('cursor', params.cursor);
  if (params?.search) qs.set('search', params.search);
  if (params?.birth_date) qs.set('birth_date', params.birth_date);
  if (params?.town_code) qs.set('town_code', params.town_code);
  if (params?.village_id) qs.set('village_id', params.village_id);
  if (params?.citizen_status) qs.set('citizen_status', params.citizen_status);
  const query = qs.toString();
  return get<ArchiveListResponse>(`/api/v1/archives${query ? '?' + query : ''}`);
};

export const getArchive = (id: string) => get<Archive>(`/api/v1/archives/${id}`);

export const updateArchive = (id: string, body: Record<string, unknown>) =>
  put<Archive>(`/api/v1/archives/${id}`, body);

export const bindArchiveWallet = (id: string, wallet_address: string) =>
  post<Archive>(`/api/v1/archives/${id}/wallet`, { wallet_address });

export const generateArchiveQr = (id: string) =>
  post<{ qr_payload: unknown; qr_content: string }>(`/api/v1/archives/${id}/qr/generate`);

export const printArchiveQr = (id: string) =>
  post<{ print_id: string; archive_id: string; archive_no: string; printed_at: number }>(
    `/api/v1/archives/${id}/qr/print`
  );

export const createArchiveDeleteChallenge = (id: string) =>
  post<{ challenge_id: string; sign_request: string; expire_at: number }>(
    `/api/v1/archives/${id}/delete/challenge`
  );

export const completeArchiveDelete = (id: string, body: {
  challenge_id: string;
  pubkey: string;
  sig_alg: 'sr25519';
  signature: string;
  payload_hash: string;
  signed_at: number;
}) =>
  post<{ archive_id: string; deleted_at: number; deleted_by: string }>(
    `/api/v1/archives/${id}/delete/complete`,
    body
  );

export const listArchiveMaterials = (id: string) =>
  get<{ items: ArchiveMaterial[] }>(`/api/v1/archives/${id}/materials`);

export const uploadArchiveMaterial = async (id: string, body: FormData) => {
  // 中文注释：资料上传使用浏览器自动生成 multipart boundary，不能走 JSON HTTP 封装。
  const res = await fetch(`/api/v1/archives/${id}/materials`, {
    method: 'POST',
    body,
    credentials: 'same-origin',
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText })) as Partial<ApiError>;
    if (res.status === 401) {
      sessionStorage.removeItem('cpms_user');
      window.dispatchEvent(new Event('cpms-auth-expired'));
    }
    throw new Error(err.message || `HTTP ${res.status}`);
  }
  return res.json() as Promise<ApiResponse<{ item: ArchiveMaterial }>>;
};

export const deleteArchiveMaterial = (archiveId: string, materialId: string) =>
  del<{ deleted_at: number }>(`/api/v1/archives/${archiveId}/materials/${materialId}`);

export const archiveMaterialDownloadUrl = (archiveId: string, materialId: string) =>
  `/api/v1/archives/${archiveId}/materials/${materialId}/download`;
