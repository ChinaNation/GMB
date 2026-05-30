import { get, post } from '../common/http';
import type { SessionUser } from '../common/types';

export const authLogout = () => post<null>('/api/v1/admin/auth/logout');

export const authMe = () => get<SessionUser>('/api/v1/admin/auth/me');

export const authQrChallenge = () =>
  post<{ challenge_id: string; login_qr_payload: string; session_id: string; expire_at: number }>(
    '/api/v1/admin/auth/qr/challenge',
    { origin: window.location.origin }
  );

export const authQrComplete = (body: {
  challenge_id: string;
  session_id: string;
  admin_pubkey: string;
  signature: string;
}) => post<null>('/api/v1/admin/auth/qr/complete', body);

export const authQrResult = (challenge_id: string, session_id: string) =>
  get<{ status: string; expires_in?: number; user?: SessionUser }>(
    `/api/v1/admin/auth/qr/result?challenge_id=${encodeURIComponent(challenge_id)}&session_id=${encodeURIComponent(session_id)}`
  );
