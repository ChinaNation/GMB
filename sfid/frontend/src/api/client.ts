export type TokenAdminAuth = {
  access_token: string;
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'INSTITUTION_ADMIN' | 'SYSTEM_ADMIN';
  admin_name?: string;
  admin_province?: string | null;
};

export type AdminAuth = TokenAdminAuth;

export function isTokenAuth(auth: AdminAuth): auth is TokenAdminAuth {
  return 'access_token' in auth;
}

/** 所有 API 请求使用相对路径，由 Vite(开发) / Nginx(生产) 统一代理到后端 */
async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let resp: Response;
  try {
    resp = await fetch(path, init);
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    throw new Error(`无法连接服务器：${msg}`);
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
  role: 'KEY_ADMIN' | 'INSTITUTION_ADMIN' | 'SYSTEM_ADMIN';
  admin_name: string;
  admin_province?: string | null;
};

export type AdminIdentifyResult = {
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'INSTITUTION_ADMIN' | 'SYSTEM_ADMIN';
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

export type GenerateCpmsInstitutionSfidResult = {
  site_sfid: string;
  qr1_payload: string;
};

export type CpmsSiteRow = {
  site_sfid: string;
  install_token_status: 'PENDING' | 'USED' | 'REVOKED';
  status?: 'PENDING' | 'ACTIVE' | 'DISABLED' | 'REVOKED';
  version?: number;
  province_code?: string;
  admin_province?: string;
  city_name?: string;
  institution_code?: string;
  institution_name?: string;
  qr1_payload?: string;
  created_by: string;
  created_by_name?: string;
  created_at: string;
  updated_by?: string | null;
  updated_at?: string | null;
};

export type CpmsRegisterResult = {
  qr3_payload: string;
};

export type CpmsArchiveImportResult = {
  archive_no: string;
  province_code: string;
  status: string;
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
  id: number;
  account_pubkey?: string;
  archive_no?: string;
  sfid_code?: string;
  province_code?: string;
  status: 'UNBOUND' | 'BOUND' | 'UNLINKED';
};

export type CitizenBindChallengeResult = {
  challenge_id: string;
  challenge_text: string;
  /** WUMIN_SIGN_V1.0.0 签名请求 JSON，前端直接展示为二维码 */
  sign_request: string;
  expire_at: number;
};

export type CitizenBindResult = {
  id: number;
  account_pubkey?: string;
  archive_no?: string;
  sfid_code?: string;
  province_code?: string;
  status: 'UNBOUND' | 'BOUND' | 'UNLINKED';
};

export type OperatorRow = {
  id: number;
  admin_pubkey: string;
  admin_name: string;
  role: 'SYSTEM_ADMIN';
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
  admin_name: string;
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

export async function citizenBindChallenge(
  auth: AdminAuth
): Promise<CitizenBindChallengeResult> {
  return request<CitizenBindChallengeResult>('/api/v1/admin/citizen/bind/challenge', {
    method: 'POST',
    headers: adminHeaders(auth)
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
  }
): Promise<CitizenBindResult> {
  return request<CitizenBindResult>('/api/v1/admin/citizen/bind', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function citizenUnbind(
  auth: AdminAuth,
  payload: { citizen_id: number; challenge_id: string; signature: string }
): Promise<CitizenBindResult> {
  return request<CitizenBindResult>('/api/v1/admin/citizen/unbind', {
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

export async function generateCpmsInstitutionSfid(
  auth: AdminAuth,
  payload: { province?: string; city: string; institution: string; institution_name: string }
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

export async function registerCpms(
  auth: AdminAuth,
  payload: { qr_payload: string }
): Promise<CpmsRegisterResult> {
  return request<CpmsRegisterResult>('/api/v1/admin/cpms/register', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function importArchive(
  auth: AdminAuth,
  payload: { qr_payload: string }
): Promise<CpmsArchiveImportResult> {
  return request<CpmsArchiveImportResult>('/api/v1/admin/cpms/archive/import', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function revokeInstallToken(
  auth: AdminAuth,
  siteSfid: string
): Promise<string> {
  return request<string>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/revoke-token`, {
    method: 'POST',
    headers: adminHeaders(auth)
  });
}

export async function reissueInstallToken(
  auth: AdminAuth,
  siteSfid: string
): Promise<GenerateCpmsInstitutionSfidResult> {
  return request<GenerateCpmsInstitutionSfidResult>(`/api/v1/admin/cpms-keys/${encodeURIComponent(siteSfid)}/reissue`, {
    method: 'POST',
    headers: adminHeaders(auth)
  });
}

export async function listCpmsSites(auth: AdminAuth): Promise<CpmsSiteRow[]> {
  const result = await request<{ total: number; limit: number; offset: number; rows: CpmsSiteRow[] }>('/api/v1/admin/cpms-keys', {
    method: 'GET',
    headers: adminHeaders(auth)
  });
  return result.rows ?? [];
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

// ── 多签管理 ──────────────────────────────────────────

export type MultisigSfidRow = {
  site_sfid: string;
  a3: string;
  institution_code: string;
  institution_name: string;
  province: string;
  city: string;
  province_code: string;
  chain_status: 'PENDING' | 'REGISTERED' | 'FAILED';
  chain_tx_hash?: string | null;
  chain_block_number?: number | null;
  created_by: string;
  created_by_name: string;
  created_at: string;
};

export type GenerateMultisigSfidResult = {
  site_sfid: string;
  chain_status: string;
  chain_tx_hash?: string | null;
  chain_block_number?: number | null;
};

export async function generateMultisigSfid(
  auth: AdminAuth,
  payload: {
    a3: string;
    p1?: string;
    province?: string;
    city: string;
    institution: string;
    institution_name: string;
  }
): Promise<GenerateMultisigSfidResult> {
  return request<GenerateMultisigSfidResult>('/api/v1/admin/multisig-sfids/generate', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
  });
}

export async function listMultisigSfids(auth: AdminAuth): Promise<MultisigSfidRow[]> {
  const result = await request<{ total: number; limit: number; offset: number; rows: MultisigSfidRow[] }>(
    '/api/v1/admin/multisig-sfids',
    { method: 'GET', headers: adminHeaders(auth) }
  );
  return result.rows ?? [];
}
