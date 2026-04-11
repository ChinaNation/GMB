// 统一的签名二维码 payload 解析工具。
// 唯一事实源:memory/05-architecture/qr-protocol-spec.md
// 使用 WUMIN_QR_V1 envelope,不再有任何字段别名兼容。

import { parseQrEnvelope, QrParseError } from '../qr/wuminQr';
import type { LoginReceiptBody } from '../qr/wuminQr';

export type SignedLoginPayload = {
  challenge_id: string;
  session_id?: string;
  admin_pubkey: string;
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
  if (env.kind !== 'login_receipt') {
    throw new Error(`期望 login_receipt,实际: ${env.kind}`);
  }
  const body = env.body as LoginReceiptBody;
  const challenge_id = env.id || fallbackChallengeId;
  if (!challenge_id || !body.pubkey || !body.signature) {
    throw new Error('签名二维码缺少必要字段(id/pubkey/signature)');
  }
  return {
    challenge_id,
    admin_pubkey: body.pubkey,
    signer_pubkey: body.pubkey,
    signature: body.signature,
  };
}

export type KeyringSignedPayload = {
  challenge_id: string;
  signature: string;
};

export function parseKeyringSignedPayload(
  raw: string,
  fallbackChallengeId: string,
): KeyringSignedPayload {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error('签名二维码内容为空');
  }
  if (trimmed.startsWith('{')) {
    let env;
    try {
      env = parseQrEnvelope(trimmed);
    } catch (e) {
      if (e instanceof QrParseError) {
        throw new Error(`签名二维码解析失败: ${e.message}`);
      }
      throw e;
    }
    if (env.kind !== 'login_receipt') {
      throw new Error(`期望 login_receipt,实际: ${env.kind}`);
    }
    const body = env.body as LoginReceiptBody;
    const challenge_id = env.id || fallbackChallengeId;
    if (!challenge_id || !body.signature) {
      throw new Error('签名二维码缺少必要字段(id/signature)');
    }
    return { challenge_id, signature: body.signature };
  }
  return {
    challenge_id: fallbackChallengeId,
    signature: trimmed,
  };
}
