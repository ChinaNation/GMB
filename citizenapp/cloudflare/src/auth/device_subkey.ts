import { HttpError } from '../shared/http';
import {
  OP_SIGN_SQUARE_DEVICE_BIND,
  concatBytes,
  scaleString,
  signingMessage,
  u64Le, hexToBytes} from '../shared/signing_message';

// P-256 设备子钥：后台握手用硬件 P-256 子钥（Keystore/SE）静默 ECDSA 签名，
// 私钥永不出硬件，替代原先「静默读 sr25519 seed 签登录挑战」。
//
// 格式约定（与客户端逐字节一致）：
// - pubkey = 裸未压缩点 65B（0x04 || X(32) || Y(32)）hex。
// - signature = 裸 r||s 64B hex（客户端把平台 DER 签名转 raw）。
// - 验签走 Workers Web Crypto ES256（ECDSA over SHA-256），message = 32B 摘要。

const P256_PUBKEY_BYTES = 65; // 0x04 || X(32) || Y(32)
const P256_SIG_BYTES = 64; // r(32) || s(32)

export interface DeviceBindingInput {
  account_id: string;
  p256_public_key: string;
  issued_at: number;
}

/// 设备绑定证明消息：sr25519 主钥对 `signing_message(OP_SIGN_SQUARE_DEVICE_BIND)`
/// 签名，证明该 P-256 子钥属于此钱包。SCALE 拼接顺序须与公民端逐字节一致。
export function buildDeviceBindingSigningMessage(input: DeviceBindingInput): Uint8Array {
  const scalePayload = concatBytes(
    scaleString(input.account_id),
    scaleString(input.p256_public_key),
    u64Le(input.issued_at),
  );
  return signingMessage(OP_SIGN_SQUARE_DEVICE_BIND, scalePayload);
}

/// 严格校验 P-256 公钥 hex（65 字节未压缩点，以字节 04 开头，不带 `0x`）。
export function assertP256PublicKeyHex(value: unknown): string {
  if (typeof value !== 'string' || !/^04[0-9a-f]{128}$/.test(value)) {
    throw new HttpError(400, 'invalid_device_pubkey', '设备子钥公钥格式不合法');
  }
  return value;
}

/// Web Crypto ES256 验签：pubkey 裸点 65B、signature 裸 r||s 64B。
/// [message] 为 `signing_message(op_tag)` 32 字节摘要（ECDSA 内部再 SHA-256）。
export async function verifyP256Signature(
  message: Uint8Array<ArrayBuffer>,
  signatureHex: string,
  pubkeyHex: string,
): Promise<boolean> {
  if (!/^[0-9a-f]{128}$/.test(signatureHex) || !/^04[0-9a-f]{128}$/.test(pubkeyHex)) {
    return false;
  }
  const sig = hexToBytes(signatureHex);
  const pub = hexToBytes(pubkeyHex);
  if (sig.length !== P256_SIG_BYTES || pub.length !== P256_PUBKEY_BYTES) {
    return false;
  }
  let key: CryptoKey;
  try {
    key = await crypto.subtle.importKey(
      'raw',
      pub,
      { name: 'ECDSA', namedCurve: 'P-256' },
      false,
      ['verify'],
    );
  } catch {
    return false;
  }
  try {
    return await crypto.subtle.verify(
      { name: 'ECDSA', hash: 'SHA-256' },
      key,
      sig,
      message,
    );
  } catch {
    return false;
  }
}
