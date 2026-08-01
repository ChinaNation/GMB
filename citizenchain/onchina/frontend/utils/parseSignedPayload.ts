// 统一的签名二维码 payload 解析工具。
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// 使用 QR_V1 envelope,不支持字段别名。

import { parseQrEnvelope, QrParseError } from '../core/citizenQr';
import type { SignResponseBody } from '../core/citizenQr';

export type SignedLoginPayload = {
  challenge_id: string;
  account_id: string;
  signature: string;
};

export function parseSignedLoginPayload(
  raw: string,
  fallbackChallengeId: string,
): SignedLoginPayload {
  let env;
  try {
    env = parseQrEnvelope(raw);
  } catch (e) {
    if (e instanceof QrParseError) {
      throw new Error(`签名二维码解析失败: ${e.message}`);
    }
    throw e;
  }
  if (env.kind !== 'sign_response') {
    throw new Error(`期望 sign_response,实际: ${env.kind}`);
  }
  const body = env.body as SignResponseBody;
  const challenge_id = env.id || fallbackChallengeId;
  if (!challenge_id || !body.account_id || !body.signature) {
    throw new Error('签名二维码缺少必要字段(id/account_id/signature)');
  }
  return {
    challenge_id,
    account_id: body.account_id,
    signature: body.signature,
  };
}

export type SignedReceiptPayload = {
  challenge_id: string;
  signature: string;
  account_id?: string;
  payload_hash?: string;
  current_account_id?: string;
  current_account_signature?: string;
};

// 解析"挑战签名响应"二维码 payload。
// 只接受 QR_V1 envelope(sign_response)。
// 返回结构供调用方提交后端 verify/commit。
export function parseSignedReceiptPayload(
  raw: string,
  fallbackChallengeId: string,
): SignedReceiptPayload {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error('签名二维码内容为空');
  }
  if (!trimmed.startsWith('{')) {
    throw new Error('签名二维码必须使用 QR_V1 envelope');
  }
  let env;
  try {
    env = parseQrEnvelope(trimmed);
  } catch (e) {
    if (e instanceof QrParseError) {
      throw new Error(`签名二维码解析失败: ${e.message}`);
    }
    throw e;
  }
  if (env.kind !== 'sign_response') {
    throw new Error(`期望 sign_response,实际: ${env.kind}`);
  }
  const challenge_id = env.id || fallbackChallengeId;
  const body = env.body as SignResponseBody;
  if (!challenge_id || !body.signature || !body.account_id) {
    throw new Error('签名二维码缺少必要字段(id/account_id/signature)');
  }
  return {
    challenge_id,
    signature: body.signature,
    account_id: body.account_id,
    current_account_id: body.current_account_id,
    current_account_signature: body.current_account_signature,
  };
}
