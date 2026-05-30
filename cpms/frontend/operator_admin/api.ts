import { get, post, put } from '../common/http';
import type { Archive, CreateArchiveRequest } from './types';

export const createArchive = (body: CreateArchiveRequest) =>
  post<{ archive_id: string; archive_no: string; passport_no: string }>('/api/v1/archives', body);

export const listArchives = (params?: { q?: string; page?: number; page_size?: number }) => {
  const qs = new URLSearchParams();
  if (params?.q) qs.set('q', params.q);
  if (params?.page) qs.set('page', String(params.page));
  if (params?.page_size) qs.set('page_size', String(params.page_size));
  const query = qs.toString();
  return get<{ items: Archive[]; total: number }>(`/api/v1/archives${query ? '?' + query : ''}`);
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
