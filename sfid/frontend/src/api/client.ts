export type TokenAdminAuth = {
  access_token: string;
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'SUPER_ADMIN' | 'OPERATOR_ADMIN' | 'QUERY_ONLY';
  admin_name?: string;
  admin_province?: string | null;
};

export type AdminAuth = TokenAdminAuth;

export function isTokenAuth(auth: AdminAuth): auth is TokenAdminAuth {
  return 'access_token' in auth;
}

function normalizeBaseUrl(raw?: string): string {
  const value = (raw ?? '').trim();
  const allowInsecureHttp =
    String(import.meta.env.VITE_SFID_ALLOW_INSECURE_HTTP || '').toLowerCase() === 'true';
  const isDev = Boolean(import.meta.env.DEV);
  if (!value) {
    if (isDev) {
      return '';
    }
    const isHttpsPage = typeof window !== 'undefined' && window.location.protocol === 'https:';
    if (isHttpsPage) {
      return `https://${window.location.hostname}:8899`;
    }
    if (!allowInsecureHttp && !isDev) {
      throw new Error('VITE_SFID_API_BASE_URL is required unless VITE_SFID_ALLOW_INSECURE_HTTP=true');
    }
    return 'http://127.0.0.1:8899';
  }
  if (value.startsWith('/')) {
    return value.replace(/\/+$/, '');
  }
  if (value.startsWith('http://') || value.startsWith('https://')) {
    if (value.startsWith('http://') && !allowInsecureHttp) {
      throw new Error('Insecure http API URL is blocked. Use https or set VITE_SFID_ALLOW_INSECURE_HTTP=true for local dev.');
    }
    return value.replace(/\/+$/, '');
  }
  const inferred = `https://${value.replace(/\/+$/, '')}`;
  return inferred;
}

const BASE_URL = normalizeBaseUrl(import.meta.env.VITE_SFID_API_BASE_URL);

function fallbackBaseUrl(baseUrl: string): string | null {
  if (baseUrl.includes('127.0.0.1')) {
    return baseUrl.replace('127.0.0.1', 'localhost');
  }
  if (baseUrl.includes('localhost')) {
    return baseUrl.replace('localhost', '127.0.0.1');
  }
  return null;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const targetBase = BASE_URL || (typeof window !== 'undefined' ? window.location.origin : '');
  const buildUrl = (base: string, reqPath: string): string => {
    if (!base) {
      return reqPath;
    }
    if (base === '/api' && reqPath.startsWith('/api/')) {
      return reqPath;
    }
    return `${base}${reqPath}`;
  };
  let resp: Response;
  try {
    resp = await fetch(buildUrl(targetBase, path), init);
  } catch (error) {
    const fallbackBase = fallbackBaseUrl(targetBase);
    if (fallbackBase != null) {
      try {
        resp = await fetch(buildUrl(fallbackBase, path), init);
      } catch (fallbackError) {
        const msg = fallbackError instanceof Error ? fallbackError.message : String(fallbackError);
        throw new Error(`无法连接服务器(${targetBase})：${msg}`);
      }
    } else {
      const msg = error instanceof Error ? error.message : String(error);
      throw new Error(`无法连接服务器(${targetBase})：${msg}`);
    }
  }

  const text = await resp.text();
  let body: any = null;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    const snippet = text.slice(0, 120);
    throw new Error(
      `服务响应格式错误(${resp.status})：${snippet || 'empty body'}，请确认后端已重启到最新版本`
    );
  }

  if (!resp.ok || !body || body.code !== 0) {
    throw new Error(body?.message ?? `request failed (${resp.status})`);
  }
  return body.data as T;
}

function adminHeaders(auth: AdminAuth): HeadersInit {
  return {
    authorization: `Bearer ${auth.access_token}`
  };
}

export type AdminAuthCheck = {
  ok: boolean;
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'SUPER_ADMIN' | 'OPERATOR_ADMIN' | 'QUERY_ONLY';
  admin_name: string;
  admin_province?: string | null;
};

export type AdminIdentifyResult = {
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'SUPER_ADMIN' | 'OPERATOR_ADMIN' | 'QUERY_ONLY';
  status: 'ACTIVE' | 'DISABLED';
  admin_name: string;
  admin_province?: string | null;
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

export type QueryResult = {
  account_pubkey: string;
  found_pending: boolean;
  found_binding: boolean;
  archive_index?: string;
  sfid_code?: string;
};

export type BindConfirmResult = {
  account_pubkey: string;
  archive_index: string;
  sfid_code: string;
  status: string;
  message: string;
};

export type GenerateSfidResult = {
  account_pubkey: string;
  sfid_code: string;
};

export type SfidOptionItem = {
  label: string;
  value: string;
};

export type SfidProvinceItem = {
  name: string;
  code: string;
};

export type SfidCityItem = {
  name: string;
  code: string;
};

export type SfidMetaResult = {
  a3_options: SfidOptionItem[];
  institution_options: SfidOptionItem[];
  provinces: SfidProvinceItem[];
  scoped_province?: string | null;
};

export type BindScanResult = {
  site_sfid: string;
  archive_no: string;
  qr_id: string;
  status: 'NORMAL' | 'ABNORMAL';
  issued_at: number;
  expire_at: number;
};

export type CpmsRegisterResult = {
  site_sfid: string;
  status: string;
  message: string;
};

export type GenerateCpmsInstitutionSfidResult = {
  site_sfid: string;
  issued_at: number;
  expire_at: number;
  qr_payload: string;
};

export type CpmsSiteRow = {
  site_sfid: string;
  pubkey_1: string;
  pubkey_2: string;
  pubkey_3: string;
  status?: 'PENDING' | 'ACTIVE' | 'DISABLED' | 'REVOKED';
  version?: number;
  init_qr_payload?: string | null;
  admin_province?: string;
  created_by: string;
  created_at: string;
  updated_by?: string | null;
  updated_at?: string | null;
};

export type CpmsStatusScanResult = {
  archive_no: string;
  status: 'NORMAL' | 'ABNORMAL';
  message: string;
};

export type KeyringStateResult = {
  version: number;
  main_pubkey: string;
  backup_a_pubkey: string;
  backup_b_pubkey: string;
  updated_at: number;
};

export type KeyringRotateChallengeResult = {
  challenge_id: string;
  keyring_version: number;
  challenge_text: string;
  expire_at: number;
};

export type KeyringRotateCommitResult = {
  old_main_pubkey: string;
  promoted_slot: 'MAIN' | 'BACKUP_A' | 'BACKUP_B';
  chain_tx_hash?: string;
  chain_submit_ok: boolean;
  chain_submit_error?: string | null;
  version: number;
  main_pubkey: string;
  backup_a_pubkey: string;
  backup_b_pubkey: string;
  updated_at: number;
  message: string;
};

export type KeyringRotateVerifyResult = {
  challenge_id: string;
  initiator_pubkey: string;
  keyring_version: number;
  verified: boolean;
  message: string;
};

export type CitizenRow = {
  seq: number;
  account_pubkey: string;
  archive_index?: string;
  sfid_code?: string;
  is_bound: boolean;
};

export type OperatorRow = {
  id: number;
  admin_pubkey: string;
  admin_name: string;
  role: 'OPERATOR_ADMIN';
  status: 'ACTIVE' | 'DISABLED';
  built_in: boolean;
  created_by: string;
  created_by_name?: string;
  created_at: string;
};

export type SuperAdminRow = {
  id: number;
  province: string;
  admin_pubkey: string;
  status: 'ACTIVE' | 'DISABLED';
  built_in: boolean;
  created_at: string;
};

export async function identifyAdmin(identityQr: string): Promise<AdminIdentifyResult> {
  return request<AdminIdentifyResult>('/api/v1/admin/auth/identify', {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify({ identity_qr: identityQr })
  });
}

export async function createAdminChallenge(input: {
  admin_pubkey: string;
  origin: string;
  session_id: string;
}): Promise<AdminChallengeResult> {
  return request<AdminChallengeResult>('/api/v1/admin/auth/challenge', {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(input)
  });
}

export async function createAdminQrChallenge(input: {
  origin: string;
  session_id: string;
}): Promise<AdminQrChallengeResult> {
  return request<AdminQrChallengeResult>('/api/v1/admin/auth/qr/challenge', {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(input)
  });
}

export async function queryAdminQrLoginResult(
  challengeId: string,
  sessionId: string
): Promise<AdminQrLoginStatus> {
  const q = `?challenge_id=${encodeURIComponent(challengeId)}&session_id=${encodeURIComponent(sessionId)}`;
  return request<AdminQrLoginStatus>(`/api/v1/admin/auth/qr/result${q}`, {
    method: 'GET'
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
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(input)
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
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(input)
  });
}

export async function demoSignChallenge(input: {
  challenge_id: string;
  admin_pubkey: string;
}): Promise<AdminDemoSignResult> {
  return request<AdminDemoSignResult>('/api/v1/admin/auth/demo-sign', {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(input)
  });
}

export async function checkAdminAuth(auth: AdminAuth): Promise<AdminAuthCheck> {
  return request<AdminAuthCheck>('/api/v1/admin/auth/check', {
    headers: adminHeaders(auth)
  });
}

export async function listCitizens(auth: AdminAuth, keyword?: string): Promise<CitizenRow[]> {
  const q = keyword ? `?keyword=${encodeURIComponent(keyword)}` : '';
  return request<CitizenRow[]>(`/api/v1/admin/citizens${q}`, {
    headers: adminHeaders(auth)
  });
}

export async function confirmBind(
  auth: AdminAuth,
  payload: { account_pubkey: string; archive_index: string; qr_id: string }
): Promise<BindConfirmResult> {
  return request<BindConfirmResult>('/api/v1/admin/bind/confirm', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function unbind(auth: AdminAuth, accountPubkey: string): Promise<string> {
  return request<string>('/api/v1/admin/bind/unbind', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify({ account_pubkey: accountPubkey })
  });
}

export async function generateSfid(
  auth: AdminAuth,
  payload: {
    account_pubkey: string;
    a3: string;
    p1?: string;
    province: string;
    city: string;
    institution: string;
  }
): Promise<GenerateSfidResult> {
  return request<GenerateSfidResult>('/api/v1/admin/sfid/generate', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function getSfidMeta(auth: AdminAuth): Promise<SfidMetaResult> {
  return request<SfidMetaResult>('/api/v1/admin/sfid/meta', {
    method: 'GET',
    headers: adminHeaders(auth)
  });
}

export async function listSfidCities(auth: AdminAuth, province: string): Promise<SfidCityItem[]> {
  const q = `?province=${encodeURIComponent(province)}`;
  return request<SfidCityItem[]>(`/api/v1/admin/sfid/cities${q}`, {
    method: 'GET',
    headers: adminHeaders(auth)
  });
}

export async function scanBindQr(
  auth: AdminAuth,
  payload: { qr_payload: string }
): Promise<BindScanResult> {
  return request<BindScanResult>('/api/v1/admin/bind/scan', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function registerCpmsKeysScan(
  auth: AdminAuth,
  payload: { qr_payload: string }
): Promise<CpmsRegisterResult> {
  return request<CpmsRegisterResult>('/api/v1/admin/cpms-keys/register-scan', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function generateCpmsInstitutionSfid(
  auth: AdminAuth,
  payload: { province?: string; city: string; institution: string }
): Promise<GenerateCpmsInstitutionSfidResult> {
  return request<GenerateCpmsInstitutionSfidResult>('/api/v1/admin/cpms-keys/sfid/generate', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function scanCpmsStatusQr(
  auth: AdminAuth,
  payload: { qr_payload: string }
): Promise<CpmsStatusScanResult> {
  return request<CpmsStatusScanResult>('/api/v1/admin/cpms-status/scan', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function listCpmsSites(auth: AdminAuth): Promise<CpmsSiteRow[]> {
  return request<CpmsSiteRow[]>('/api/v1/admin/cpms-keys', {
    method: 'GET',
    headers: adminHeaders(auth)
  });
}

export async function updateCpmsKeys(
  auth: AdminAuth,
  siteSfid: string,
  payload: { pubkey_1: string; pubkey_2: string; pubkey_3: string }
): Promise<CpmsSiteRow> {
  return request<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function disableCpmsKeys(
  auth: AdminAuth,
  siteSfid: string,
  reason?: string
): Promise<CpmsSiteRow> {
  return request<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/disable`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify({ reason })
  });
}

export async function revokeCpmsKeys(
  auth: AdminAuth,
  siteSfid: string,
  reason?: string
): Promise<CpmsSiteRow> {
  return request<CpmsSiteRow>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/revoke`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify({ reason })
  });
}

export async function deleteCpmsKeys(auth: AdminAuth, siteSfid: string): Promise<string> {
  return request<string>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}`, {
    method: 'DELETE',
    headers: adminHeaders(auth)
  });
}

export async function listOperators(auth: AdminAuth): Promise<OperatorRow[]> {
  return request<OperatorRow[]>('/api/v1/admin/operators', {
    method: 'GET',
    headers: adminHeaders(auth)
  });
}

export async function createOperator(
  auth: AdminAuth,
  payload: { admin_pubkey: string; admin_name: string }
): Promise<OperatorRow> {
  return request<OperatorRow>('/api/v1/admin/operators', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function updateOperator(
  auth: AdminAuth,
  id: number,
  payload: { admin_pubkey?: string; admin_name?: string }
): Promise<OperatorRow> {
  return request<OperatorRow>(`/api/v1/admin/operators/${id}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function updateOperatorStatus(
  auth: AdminAuth,
  id: number,
  status: 'ACTIVE' | 'DISABLED'
): Promise<OperatorRow> {
  return request<OperatorRow>(`/api/v1/admin/operators/${id}/status`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify({ status })
  });
}

export async function deleteOperator(auth: AdminAuth, id: number): Promise<string> {
  return request<string>(`/api/v1/admin/operators/${id}`, {
    method: 'DELETE',
    headers: adminHeaders(auth)
  });
}

export async function listSuperAdmins(auth: AdminAuth): Promise<SuperAdminRow[]> {
  return request<SuperAdminRow[]>('/api/v1/admin/super-admins', {
    method: 'GET',
    headers: adminHeaders(auth)
  });
}

export async function replaceSuperAdmin(
  auth: AdminAuth,
  province: string,
  adminPubkey: string
): Promise<SuperAdminRow> {
  return request<SuperAdminRow>(`/api/v1/admin/super-admins/${encodeURIComponent(province)}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify({ admin_pubkey: adminPubkey })
  });
}

export async function getAttestorKeyring(auth: AdminAuth): Promise<KeyringStateResult> {
  return request<KeyringStateResult>('/api/v1/admin/attestor/keyring', {
    method: 'GET',
    headers: adminHeaders(auth)
  });
}

export async function createKeyringRotateChallenge(
  auth: AdminAuth,
  payload: { initiator_pubkey: string }
): Promise<KeyringRotateChallengeResult> {
  return request<KeyringRotateChallengeResult>('/api/v1/admin/attestor/rotate/challenge', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function verifyKeyringRotateSignature(
  auth: AdminAuth,
  payload: { challenge_id: string; signature: string }
): Promise<KeyringRotateVerifyResult> {
  return request<KeyringRotateVerifyResult>('/api/v1/admin/attestor/rotate/verify', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function commitKeyringRotate(
  auth: AdminAuth,
  payload: {
    challenge_id: string;
    signature: string;
    new_backup_pubkey: string;
    new_backup_seed_hex?: string;
  }
): Promise<KeyringRotateCommitResult> {
  return request<KeyringRotateCommitResult>('/api/v1/admin/attestor/rotate/commit', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}
