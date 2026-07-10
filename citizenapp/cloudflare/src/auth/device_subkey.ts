import { HttpError } from '../shared/http';
import {
  OP_SIGN_SQUARE_DEVICE_BIND,
  concatBytes,
  scaleString,
  signingMessage,
  u64Le,
} from '../shared/signing_message';

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
  owner_account: string;
  p256_pubkey: string;
  issued_at: number;
}

/// 设备绑定证明消息：sr25519 主钥对 `signing_message(OP_SIGN_SQUARE_DEVICE_BIND)`
/// 签名，证明该 P-256 子钥属于此钱包。SCALE 拼接顺序须与公民端逐字节一致。
export function buildDeviceBindingSigningMessage(input: DeviceBindingInput): Uint8Array {
  const scalePayload = concatBytes(
    scaleString(input.owner_account),
    scaleString(input.p256_pubkey),
    u64Le(input.issued_at),
  );
  return signingMessage(OP_SIGN_SQUARE_DEVICE_BIND, scalePayload);
}

/// 校验并归一化 P-256 公钥 hex（65 字节未压缩点，0x04 前缀）。
export function assertP256PublicKeyHex(value: unknown): string {
  if (typeof value !== 'string') {
    throw new HttpError(400, 'invalid_device_pubkey', '设备子钥公钥格式不合法');
  }
  const hex = value.trim().toLowerCase().replace(/^0x/, '');
  if (hex.length !== P256_PUBKEY_BYTES * 2 || !/^04[0-9a-f]+$/.test(hex)) {
    throw new HttpError(400, 'invalid_device_pubkey', '设备子钥公钥必须是 65 字节未压缩点');
  }
  return hex;
}

function hexToBytes(hex: string): Uint8Array<ArrayBuffer> {
  const clean = hex.trim().toLowerCase().replace(/^0x/, '');
  const out = new Uint8Array(new ArrayBuffer(clean.length / 2));
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(clean.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

/// Web Crypto ES256 验签：pubkey 裸点 65B、signature 裸 r||s 64B。
/// [message] 为 `signing_message(op_tag)` 32 字节摘要（ECDSA 内部再 SHA-256）。
export async function verifyP256Signature(
  message: Uint8Array<ArrayBuffer>,
  signatureHex: string,
  pubkeyHex: string,
): Promise<boolean> {
  const sigClean = signatureHex.trim().toLowerCase().replace(/^0x/, '');
  const pubClean = pubkeyHex.trim().toLowerCase().replace(/^0x/, '');
  if (
    !/^[0-9a-f]{128}$/.test(sigClean) ||
    !/^04[0-9a-f]{128}$/.test(pubClean)
  ) {
    return false;
  }
  const sig = hexToBytes(sigClean);
  const pub = hexToBytes(pubClean);
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
