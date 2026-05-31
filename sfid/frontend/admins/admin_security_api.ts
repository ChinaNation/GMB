// 中文注释:管理员安全动作 API。
// 一般写操作走 Passkey grant;重要写操作叠加 WUMIN_QR_V1/sign_request 冷钱包签名。

import type { AdminAuth } from '../auth/types';
import { adminRequest } from '../utils/http';

export type AdminActionType =
  | 'CREATE_OPERATOR'
  | 'UPDATE_OPERATOR'
  | 'DELETE_OPERATOR'
  | 'CREATE_SHENG_ADMIN'
  | 'UPDATE_SHENG_ADMIN'
  | 'DELETE_SHENG_ADMIN'
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

export type AdminSecurityLevel = 'GENERAL' | 'IMPORTANT';

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
  security_level: AdminSecurityLevel;
  expires_at: number;
};

export type AdminSecurityGrantOutput = {
  grant_id: string;
  action_type: AdminActionType;
  security_level: AdminSecurityLevel;
  target: string;
  expires_at: number;
};

export async function startPasskeyRegistration(
  auth: AdminAuth,
  label = '管理员 Passkey',
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

export async function createGeneralSecurityGrant(
  auth: AdminAuth,
  actionType: AdminActionType,
  payload: unknown,
): Promise<AdminSecurityGrantOutput> {
  const prepared = await prepareAdminAction(auth, actionType, payload);
  if (prepared.security_level !== 'GENERAL') {
    throw new Error('该操作需要冷钱包签名确认');
  }
  const passkeyAssertion = await getPasskeyAssertion(prepared.webauthn_options);
  return commitAdminAction<AdminSecurityGrantOutput>(auth, {
    action_id: prepared.action_id,
    passkey_assertion: passkeyAssertion,
  });
}

export async function createPasskeyCredential(options: any): Promise<unknown> {
  const publicKey = toCreationOptions(options.publicKey);
  const credential = await navigator.credentials.create({ publicKey });
  if (!credential) throw new Error('Passkey 创建已取消');
  return credentialToJSON(credential as PublicKeyCredential);
}

export async function getPasskeyAssertion(options: any): Promise<unknown> {
  const publicKey = toRequestOptions(options.publicKey);
  const credential = await navigator.credentials.get({ publicKey });
  if (!credential) throw new Error('Passkey 验证已取消');
  return credentialToJSON(credential as PublicKeyCredential);
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
