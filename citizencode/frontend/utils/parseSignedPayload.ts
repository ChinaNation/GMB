// 统一的签名二维码 payload 解析工具。
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// 使用 QR_V1 envelope,不支持字段别名。

import { parseQrEnvelope, QrParseError } from '../core/qr/citizenQr';
import type { SignResponseBody } from '../core/qr/citizenQr';

export type SignedLoginPayload = {
  challenge_id: string;
  session_id?: string;
  admin_account: string;
  signer_pubkey?: string;
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
  if (!challenge_id || !body.pubkey || !body.signature) {
    throw new Error('签名二维码缺少必要字段(id/pubkey/signature)');
  }
  return {
    challenge_id,
    admin_account: body.pubkey,
    signer_pubkey: body.pubkey,
    signature: body.signature,
  };
}

export type SignedReceiptPayload = {
  challenge_id: string;
  signature: string;
  signer_pubkey?: string;
  payload_hash?: string;
};

// 中文注释:解析"挑战签名响应"二维码 payload。
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
  if (!challenge_id || !body.signature || !body.pubkey) {
    throw new Error('签名二维码缺少必要字段(id/pubkey/signature)');
  }
  return { challenge_id, signature: body.signature, signer_pubkey: body.pubkey };
}
