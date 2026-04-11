export type TokenAdminAuth = {
  access_token: string;
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'SHENG_ADMIN' | 'SHI_ADMIN';
  admin_name?: string;
  admin_province?: string | null;
  /// 仅 ShiAdmin 有值：操作员所属的市（用于多签管理页锁定 / 列表过滤）
  admin_city?: string | null;
};

export type AdminAuth = TokenAdminAuth;

export function isTokenAuth(auth: AdminAuth): auth is TokenAdminAuth {
  return 'access_token' in auth;
}

// ── 401 拦截：token 失效时统一回调 ──────────────────────
let _onUnauthorized: (() => void) | null = null;
let _unauthorizedFired = false;

/** AuthProvider 启动时注册回调；卸载时传 null 清除 */
export function setOnUnauthorized(cb: (() => void) | null) {
  _onUnauthorized = cb;
  _unauthorizedFired = false;
}

/** 所有 API 请求使用相对路径，由 Vite(开发) / Nginx(生产) 统一代理到后端 */
// 中文注释：任务卡 3 开放 request + adminHeaders 给 api/ 下其他子文件复用,
// 避免每个新 API 文件都重复一遍 fetch + 错误包装逻辑。
export async function adminRequest<T>(
  path: string,
  auth: AdminAuth,
  init?: RequestInit
): Promise<T> {
  return request<T>(path, {
    ...init,
    headers: {
      ...adminHeaders(auth),
      ...(init?.headers || {}),
    },
  });
}

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

  // ── 401 统一拦截：token 失效 → 触发登出，防抖只触发一次 ──
  if (resp.status === 401 && _onUnauthorized && !_unauthorizedFired) {
    _unauthorizedFired = true;
    _onUnauthorized();
  }

  if (!resp.ok || !body || body.code !== 0) {
    throw new Error(body?.message ?? `request failed (${resp.status})`);
  }
  return body.data as T;
}

export function adminHeaders(auth: AdminAuth): HeadersInit {
  return {
    authorization: `Bearer ${auth.access_token}`
  };
}

export type AdminAuthCheck = {
  ok: boolean;
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'SHENG_ADMIN' | 'SHI_ADMIN';
  admin_name: string;
  admin_province?: string | null;
  admin_city?: string | null;
};

export type AdminIdentifyResult = {
  admin_pubkey: string;
  role: 'KEY_ADMIN' | 'SHENG_ADMIN' | 'SHI_ADMIN';
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
  /** WUMIN_QR_V1 签名请求 JSON，前端直接展示为二维码 */
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
  role: 'SHI_ADMIN';
  status: 'ACTIVE' | 'DISABLED';
  built_in: boolean;
  created_by: string;
  created_by_name?: string;
  created_at: string;
  city: string;
};

// 省级管理员对外行（API 返回结构）。
//
// SFID 业务语义：机构永久存在（43 个省份），省级管理员只是当前替机构发声的人，
// 不存在停用 / 状态切换的概念。被替换即彻底失效。所以**没有 status 字段**。
export type ShengAdminRow = {
  id: number;
  province: string;
  admin_pubkey: string;
  admin_name: string;
  built_in: boolean;
  created_at: string;
  // 最近一次更新时间（含签名密钥 bootstrap），null 表示从未更新
  updated_at?: string | null;
  // 链上签名 pubkey：未首次登录 bootstrap 时为 null/undefined
  signing_pubkey?: string | null;
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

/** 主动登出:通知后端销毁 session。best-effort,不阻塞前端退出流程。 */
export async function adminLogout(auth: AdminAuth): Promise<void> {
  try {
    await request<string>('/api/v1/admin/auth/logout', {
      method: 'POST',
      headers: adminHeaders(auth),
    });
  } catch {
    // 静默:即使后端不可达也不影响前端退出
  }
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
  payload: { admin_pubkey: string; admin_name: string; city: string; created_by?: string }
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
  payload: { admin_pubkey?: string; admin_name?: string; city?: string }
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

export async function listShengAdmins(auth: AdminAuth): Promise<ShengAdminRow[]> {
  return request<ShengAdminRow[]>('/api/v1/admin/sheng-admins', {
    method: 'GET',
    headers: adminHeaders(auth)
  });
}

export async function replaceShengAdmin(
  auth: AdminAuth,
  province: string,
  adminPubkey: string,
  adminName?: string,
): Promise<ShengAdminRow> {
  const payload: Record<string, string> = { admin_pubkey: adminPubkey };
  if (adminName && adminName.trim()) {
    payload.admin_name = adminName.trim();
  }
  return request<ShengAdminRow>(`/api/v1/admin/sheng-admins/${encodeURIComponent(province)}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json',
      ...adminHeaders(auth)
    },
    body: JSON.stringify(payload)
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

// ── 链上余额查询 ──────────────────────────────────────

export type ChainBalanceResult = {
  account_pubkey: string;
  /// u128 字符串（避免 JS 数字溢出），单位为"分"
  balance_min_units: string;
  /// 已格式化的展示文本，例如 "1234.56"
  balance_text: string;
  unit: string;
};

/// 查询链上账户的 free 余额（最小单位 = 分）。
/// 仅在密钥管理页用于展示主账户的链上余额。
export async function getChainBalance(
  auth: AdminAuth,
  accountPubkey: string,
): Promise<ChainBalanceResult> {
  const q = `?account_pubkey=${encodeURIComponent(accountPubkey)}`;
  return request<ChainBalanceResult>(`/api/v1/admin/chain/balance${q}`, {
    method: 'GET',
    headers: adminHeaders(auth),
  });
}

