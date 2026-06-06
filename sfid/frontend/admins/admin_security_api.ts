// 中文注释:管理员安全动作 API。
// 管理端权限统一为 LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE 三类。

import type { AdminAuth } from '../auth/types';
import { ApiError, adminRequest } from '../utils/http';

export type AdminActionType =
  | 'CREATE_OPERATOR'
  | 'DELETE_OPERATOR'
  | 'CREATE_FEDERAL_ADMIN'
  | 'DELETE_FEDERAL_ADMIN'
  | 'INSTITUTION_CREATE'
  | 'INSTITUTION_UPDATE'
  | 'INSTITUTION_CREATE_ACCOUNT'
  | 'INSTITUTION_DELETE_ACCOUNT'
  | 'INSTITUTION_UPLOAD_DOCUMENT'
  | 'INSTITUTION_DELETE_DOCUMENT'
  | 'PUBLIC_SECURITY_RECONCILE'
  | 'CITIZEN_BIND_COMMIT'
  | 'CPMS_STATUS_IMPORT_CONFIRM'
  | 'CPMS_ISSUE_INSTALL_CODE'
  | 'CPMS_REVOKE_INSTALL_TOKEN'
  | 'CPMS_REISSUE_INSTALL_TOKEN'
  | 'CPMS_DISABLE_KEYS'
  | 'CPMS_ENABLE_KEYS'
  | 'CPMS_REVOKE_KEYS'
  | 'CPMS_DELETE_KEYS';

export type AdminOperationAuth = 'LOGIN_STATE' | 'PASSKEY' | 'PASSKEY_CHALLENGE';
export type AdminRoleTarget = 'FEDERAL_ADMIN' | 'SHI_ADMIN';

export type SignDisplayField = { key?: string; label: string; value: string };

export type PasskeyStartOutput = {
  registration_id: string;
  request_id: string;
  sign_request: string;
  payload_hash: string;
  expires_at: number;
};

export type PasskeyConfirmOutput = {
  registration_id: string;
  public_key_options: any;
  expires_at: number;
};

export type PrepareAdminActionOutput = {
  action_id: string;
  action_type: AdminActionType;
  webauthn_options: any;
  sign_request?: string | null;
  payload_hash: string;
  auth_type: AdminOperationAuth;
  expires_at: number;
};

export type AdminSecurityGrantOutput = {
  grant_id: string;
  action_type: AdminActionType;
  auth_type: AdminOperationAuth;
  target: string;
  expires_at: number;
};

export function formatAdminCreateError(error: unknown, targetRole: AdminRoleTarget, fallback: string): string {
  if (!(error instanceof ApiError)) {
    return error instanceof Error ? error.message : fallback;
  }
  // 中文注释:管理员新增失败统一按稳定 error_code 显示,不解析后端 message。
  if (error.errorCode === 'SFID_ADMIN_PUBKEY_EXISTS_AS_FEDERAL_ADMIN') {
    return targetRole === 'FEDERAL_ADMIN'
      ? '该账户已是联邦管理员，不能重复新增'
      : '该账户已是联邦管理员，不能新增为市级管理员';
  }
  if (error.errorCode === 'SFID_ADMIN_PUBKEY_EXISTS_AS_SHI_ADMIN') {
    return targetRole === 'FEDERAL_ADMIN'
      ? '该账户已是市级管理员，不能新增为联邦管理员'
      : '该账户已是市级管理员，不能重复新增';
  }
  if (error.errorCode === 'SFID_ADMIN_FEDERAL_ADMIN_PROVINCE_LIMIT_REACHED') {
    return '联邦管理员已满 5 人，不能继续新增';
  }
  if (error.errorCode === 'SFID_ADMIN_SHI_ADMIN_CITY_LIMIT_REACHED') {
    return '本市市级管理员已满 30 人，不能继续新增';
  }
  return error.message || fallback;
}

export async function startPasskeyRegistration(
  auth: AdminAuth,
  label = '管理员通行密钥',
): Promise<PasskeyStartOutput> {
  return adminRequest<PasskeyStartOutput>('/api/v1/admin/passkeys/register/start', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ label }),
  });
}

export async function confirmPasskeyRegistration(
  auth: AdminAuth,
  input: {
    registration_id: string;
    signer_pubkey: string;
    signature: string;
    payload_hash: string;
  },
): Promise<PasskeyConfirmOutput> {
  return adminRequest<PasskeyConfirmOutput>('/api/v1/admin/passkeys/register/confirm', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export async function completePasskeyRegistration(
  auth: AdminAuth,
  input: {
    registration_id: string;
    credential: unknown;
  },
): Promise<{ credential_id: string; passkey_count: number }> {
  return adminRequest('/api/v1/admin/passkeys/register/complete', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export async function prepareAdminAction(
  auth: AdminAuth,
  actionType: AdminActionType,
  payload: unknown,
): Promise<PrepareAdminActionOutput> {
  return adminRequest<PrepareAdminActionOutput>('/api/v1/admin/actions/prepare', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ action_type: actionType, payload }),
  });
}

export async function commitAdminAction<T>(
  auth: AdminAuth,
  input: {
    action_id: string;
    passkey_assertion: unknown;
    signer_pubkey?: string;
    signature?: string;
    payload_hash?: string;
  },
): Promise<T> {
  return adminRequest<T>('/api/v1/admin/actions/commit', auth, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export async function createPasskeySecurityGrant(
  auth: AdminAuth,
  actionType: AdminActionType,
  payload: unknown,
): Promise<AdminSecurityGrantOutput> {
  const prepared = await prepareAdminAction(auth, actionType, payload);
  if (prepared.auth_type !== 'PASSKEY') {
    throw new Error('该操作需要冷钱包签名确认');
  }
  const passkeyAssertion = await getPasskeyAssertion(prepared.webauthn_options);
  return commitAdminAction<AdminSecurityGrantOutput>(auth, {
    action_id: prepared.action_id,
    passkey_assertion: passkeyAssertion,
  });
}

export async function createPasskeyCredential(options: any): Promise<unknown> {
  if (!navigator.credentials?.create) {
    throw new Error('当前浏览器不支持通行密钥');
  }
  const publicKey = toCreationOptions(options.publicKey);
  let credential: Credential | null;
  try {
    credential = await navigator.credentials.create({ publicKey });
  } catch (error) {
    throw normalizeWebAuthnError(error, 'create');
  }
  if (!credential) throw new Error('已取消创建通行密钥');
  return credentialToJSON(credential as PublicKeyCredential);
}

export async function getPasskeyAssertion(options: any): Promise<unknown> {
  if (!navigator.credentials?.get) {
    throw new Error('当前浏览器不支持通行密钥');
  }
  const publicKey = toRequestOptions(options.publicKey);
  let credential: Credential | null;
  try {
    credential = await navigator.credentials.get({ publicKey });
  } catch (error) {
    throw normalizeWebAuthnError(error, 'assert');
  }
  if (!credential) throw new Error('已取消通行密钥验证');
  return credentialToJSON(credential as PublicKeyCredential);
}

function normalizeWebAuthnError(error: unknown, mode: 'create' | 'assert'): Error {
  const cancelText = mode === 'create' ? '已取消创建通行密钥' : '已取消通行密钥验证';
  if (error instanceof DOMException) {
    if (error.name === 'NotAllowedError' || error.name === 'AbortError') {
      return new Error(cancelText);
    }
    if (error.name === 'NotSupportedError') {
      return new Error('当前浏览器不支持通行密钥');
    }
    if (error.name === 'SecurityError') {
      return new Error('当前页面不允许使用通行密钥');
    }
    if (error.name === 'InvalidStateError') {
      return new Error('当前设备已存在该通行密钥');
    }
  }
  const raw = error instanceof Error ? error.message : String(error);
  const lower = raw.toLowerCase();
  if (lower.includes('the operation either timed out or was not allowed') || lower.includes('notallowederror')) {
    return new Error(cancelText);
  }
  return new Error(raw || '通行密钥操作失败');
}

function toCreationOptions(publicKey: any): PublicKeyCredentialCreationOptions {
  return {
    ...publicKey,
    challenge: base64UrlToBuffer(publicKey.challenge),
    user: {
      ...publicKey.user,
      id: base64UrlToBuffer(publicKey.user.id),
    },
    excludeCredentials: publicKey.excludeCredentials?.map((item: any) => ({
      ...item,
      id: base64UrlToBuffer(item.id),
    })),
  };
}

function toRequestOptions(publicKey: any): PublicKeyCredentialRequestOptions {
  return {
    ...publicKey,
    challenge: base64UrlToBuffer(publicKey.challenge),
    allowCredentials: publicKey.allowCredentials?.map((item: any) => ({
      ...item,
      id: base64UrlToBuffer(item.id),
    })),
  };
}

function credentialToJSON(credential: PublicKeyCredential): Record<string, unknown> {
  const response = credential.response as any;
  const out: Record<string, unknown> = {
    id: credential.id,
    rawId: bufferToBase64Url(credential.rawId),
    type: credential.type,
    extensions: credential.getClientExtensionResults(),
  };
  if (response.attestationObject) {
    out.response = {
      attestationObject: bufferToBase64Url(response.attestationObject),
      clientDataJSON: bufferToBase64Url(response.clientDataJSON),
      transports: typeof response.getTransports === 'function' ? response.getTransports() : undefined,
    };
  } else {
    out.response = {
      authenticatorData: bufferToBase64Url(response.authenticatorData),
      clientDataJSON: bufferToBase64Url(response.clientDataJSON),
      signature: bufferToBase64Url(response.signature),
      userHandle: response.userHandle ? bufferToBase64Url(response.userHandle) : null,
    };
  }
  return out;
}

function base64UrlToBuffer(value: string): ArrayBuffer {
  const normalized = value.replace(/-/g, '+').replace(/_/g, '/');
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '=');
  const raw = atob(padded);
  const bytes = new Uint8Array(raw.length);
  for (let i = 0; i < raw.length; i += 1) bytes[i] = raw.charCodeAt(i);
  return bytes.buffer;
}

function bufferToBase64Url(value: ArrayBuffer): string {
  const bytes = new Uint8Array(value);
  let raw = '';
  for (const byte of bytes) raw += String.fromCharCode(byte);
  return btoa(raw).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
}
