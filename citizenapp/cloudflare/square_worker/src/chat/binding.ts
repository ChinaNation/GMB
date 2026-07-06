import { blake2AsU8a } from "@polkadot/util-crypto/blake2";
import { signatureVerify } from "@polkadot/util-crypto/signature/verify";
import { bytesToBase64Url } from "./codec";

const GMB_SIGN_DOMAIN = [0x47, 0x4d, 0x42];
const OP_SIGN_IM_WALLET_BINDING = 0x1a;

export interface DeviceBindingPayloadInput {
  wallet_account: string;
  im_device_id: string;
  im_device_pubkey: string;
  expires_at_millis: number;
  nonce: string;
}

export function buildDeviceBindingSigningMessage(
  input: DeviceBindingPayloadInput,
): Uint8Array {
  // 必须与公民端 OP_SIGN_IM_WALLET_BINDING 的 SCALE 拼接顺序保持一致。
  const scalePayload = concatBytes(
    scaleString(input.wallet_account),
    scaleString(input.im_device_id),
    scaleString(input.im_device_pubkey),
    u64Le(input.expires_at_millis),
    scaleString(input.nonce),
  );
  return blake2AsU8a(
    new Uint8Array([
      ...GMB_SIGN_DOMAIN,
      OP_SIGN_IM_WALLET_BINDING,
      ...scalePayload,
    ]),
    256,
  );
}

export function buildDeviceBindingSigningMessageBase64Url(
  input: DeviceBindingPayloadInput,
): string {
  return bytesToBase64Url(buildDeviceBindingSigningMessage(input));
}

export async function verifyDeviceBindingSignature(
  input: DeviceBindingPayloadInput,
  signature: string,
): Promise<boolean> {
  try {
    // Worker 只验证“钱包账户授权此 IM 设备”，不接触 IM 消息明文。
    const result = signatureVerify(
      buildDeviceBindingSigningMessage(input),
      signature,
      input.wallet_account,
    );
    return result.isValid;
  } catch {
    return false;
  }
}

function scaleString(value: string): Uint8Array {
  const bytes = new TextEncoder().encode(value);
  return concatBytes(scaleCompact(bytes.length), bytes);
}

function scaleCompact(value: number): Uint8Array {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new RangeError(
      "SCALE compact value must be a non-negative safe integer",
    );
  }
  if (value < 1 << 6) {
    return new Uint8Array([value << 2]);
  }
  if (value < 1 << 14) {
    const encoded = (value << 2) | 0x01;
    return new Uint8Array([encoded & 0xff, (encoded >> 8) & 0xff]);
  }
  if (value < 1 << 30) {
    const encoded = (value << 2) | 0x02;
    return new Uint8Array([
      encoded & 0xff,
      (encoded >> 8) & 0xff,
      (encoded >> 16) & 0xff,
      (encoded >> 24) & 0xff,
    ]);
  }
  throw new RangeError(
    "SCALE compact value is too large for IM binding payload",
  );
}

function u64Le(value: number): Uint8Array {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new RangeError("u64 value must be a non-negative safe integer");
  }
  let current = BigInt(value);
  const out = new Uint8Array(8);
  for (let index = 0; index < out.length; index += 1) {
    out[index] = Number(current & 0xffn);
    current >>= 8n;
  }
  return out;
}

function concatBytes(...items: Uint8Array[]): Uint8Array {
  const total = items.reduce((sum, item) => sum + item.length, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const item of items) {
    out.set(item, offset);
    offset += item.length;
  }
  return out;
}
