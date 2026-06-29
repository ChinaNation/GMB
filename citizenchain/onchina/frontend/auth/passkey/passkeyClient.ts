// WebAuthn passkey 客户端:注册凭证 + 断言换取一次性 assertion 令牌。
//
// 重要操作(Passkey 档)/特殊操作(PasskeyColdSign 档)提交前先调用 assertPasskey 取得
// assertion_id,作为 `X-Passkey-Assertion` 请求头随提交发送。
// 与后端 admins/passkey 一一对应,挑战格式直接来自 webauthn-rs(base64url 字段)。

import { adminRequest } from "../../utils/http";
import type { AdminAuth } from "../types";

interface ServerPublicKeyCredentialDescriptor {
  type: "public-key";
  id: string;
  transports?: string[];
}

interface ServerCreationOptions {
  publicKey: {
    rp: { id?: string; name: string };
    user: { id: string; name: string; displayName: string };
    challenge: string;
    pubKeyCredParams: PublicKeyCredentialParameters[];
    timeout?: number;
    excludeCredentials?: ServerPublicKeyCredentialDescriptor[];
    authenticatorSelection?: AuthenticatorSelectionCriteria;
    attestation?: AttestationConveyancePreference;
  };
}

interface ServerRequestOptions {
  publicKey: {
    challenge: string;
    timeout?: number;
    rpId?: string;
    allowCredentials?: ServerPublicKeyCredentialDescriptor[];
    userVerification?: UserVerificationRequirement;
  };
}

interface PasskeyBeginResponse<T> {
  ceremony_id: string;
  challenge: T;
}

interface PasskeyAssertionResponse {
  assertion_id: string;
  expire_at: number;
}

function b64urlToBuffer(value: string): ArrayBuffer {
  const b64 = value.replace(/-/g, "+").replace(/_/g, "/");
  const pad = b64.length % 4 ? "=".repeat(4 - (b64.length % 4)) : "";
  const binary = atob(b64 + pad);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

function bufferToB64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i += 1) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function toDescriptors(
  list: ServerPublicKeyCredentialDescriptor[] | undefined,
): PublicKeyCredentialDescriptor[] | undefined {
  return list?.map((c) => ({
    type: c.type,
    id: b64urlToBuffer(c.id),
    transports: c.transports as AuthenticatorTransport[] | undefined,
  }));
}

/** 注册一个新的 passkey 到当前管理员账户。 */
export async function registerPasskey(auth: AdminAuth): Promise<void> {
  const begin = await adminRequest<PasskeyBeginResponse<ServerCreationOptions>>(
    "/api/v1/admin/auth/passkey/register/begin",
    auth,
    { method: "POST" },
  );
  const pk = begin.challenge.publicKey;
  const publicKey: PublicKeyCredentialCreationOptions = {
    rp: pk.rp,
    user: {
      id: b64urlToBuffer(pk.user.id),
      name: pk.user.name,
      displayName: pk.user.displayName,
    },
    challenge: b64urlToBuffer(pk.challenge),
    pubKeyCredParams: pk.pubKeyCredParams,
    timeout: pk.timeout,
    excludeCredentials: toDescriptors(pk.excludeCredentials),
    authenticatorSelection: pk.authenticatorSelection,
    attestation: pk.attestation,
  };
  const credential = (await navigator.credentials.create({ publicKey })) as PublicKeyCredential | null;
  if (!credential) {
    throw new Error("passkey registration cancelled");
  }
  const response = credential.response as AuthenticatorAttestationResponse;
  const body = {
    ceremony_id: begin.ceremony_id,
    credential: {
      id: credential.id,
      rawId: bufferToB64url(credential.rawId),
      type: credential.type,
      response: {
        attestationObject: bufferToB64url(response.attestationObject),
        clientDataJSON: bufferToB64url(response.clientDataJSON),
      },
      extensions: credential.getClientExtensionResults(),
    },
  };
  await adminRequest("/api/v1/admin/auth/passkey/register/finish", auth, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
}

/** 完成一次 passkey 断言,返回一次性 assertion_id(作为 X-Passkey-Assertion 头)。 */
export async function assertPasskey(auth: AdminAuth): Promise<string> {
  const begin = await adminRequest<PasskeyBeginResponse<ServerRequestOptions>>(
    "/api/v1/admin/auth/passkey/assert/begin",
    auth,
    { method: "POST" },
  );
  const pk = begin.challenge.publicKey;
  const publicKey: PublicKeyCredentialRequestOptions = {
    challenge: b64urlToBuffer(pk.challenge),
    timeout: pk.timeout,
    rpId: pk.rpId,
    allowCredentials: toDescriptors(pk.allowCredentials),
    userVerification: pk.userVerification,
  };
  const credential = (await navigator.credentials.get({ publicKey })) as PublicKeyCredential | null;
  if (!credential) {
    throw new Error("passkey assertion cancelled");
  }
  const response = credential.response as AuthenticatorAssertionResponse;
  const body = {
    ceremony_id: begin.ceremony_id,
    credential: {
      id: credential.id,
      rawId: bufferToB64url(credential.rawId),
      type: credential.type,
      response: {
        authenticatorData: bufferToB64url(response.authenticatorData),
        clientDataJSON: bufferToB64url(response.clientDataJSON),
        signature: bufferToB64url(response.signature),
        userHandle: response.userHandle ? bufferToB64url(response.userHandle) : null,
      },
      extensions: credential.getClientExtensionResults(),
    },
  };
  const result = await adminRequest<PasskeyAssertionResponse>(
    "/api/v1/admin/auth/passkey/assert/finish",
    auth,
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    },
  );
  return result.assertion_id;
}

interface PasskeyStatusResponse {
  registered: boolean;
}

/** 查询当前管理员是否已注册 passkey(驱动操作列红点 / 登录默认跳转)。 */
export async function getPasskeyStatus(auth: AdminAuth): Promise<boolean> {
  const result = await adminRequest<PasskeyStatusResponse>(
    "/api/v1/admin/auth/passkey/status",
    auth,
    { method: "GET" },
  );
  return result.registered;
}

/** 重要/特殊操作提交头:断言令牌随此头发送给后端消费。 */
export const PASSKEY_ASSERTION_HEADER = "X-Passkey-Assertion";
