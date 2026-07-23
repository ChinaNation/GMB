import { bytesToBase64Url } from "./codec";
import {
  OP_SIGN_CHAT_DEVICE_BIND,
  concatBytes,
  scaleString,
  signingMessage,
  u64Le,
} from "../shared/signing_message";
import { verifyP256Signature } from "../auth/device_subkey";

export interface ChatDeviceBindingInput {
  account_id: string;
  device_id: string;
  device_public_key_hex: string;
  expires_at: number;
  nonce: string;
}

export function buildChatDeviceBindingMessage(
  input: ChatDeviceBindingInput,
): Uint8Array<ArrayBuffer> {
  // 必须与 CitizenApp 的 Chat 设备绑定 SCALE 字段顺序逐字节一致。
  const scalePayload = concatBytes(
    scaleString(input.account_id),
    scaleString(input.device_id),
    scaleString(input.device_public_key_hex),
    u64Le(input.expires_at),
    scaleString(input.nonce),
  );
  return signingMessage(OP_SIGN_CHAT_DEVICE_BIND, scalePayload);
}

export function buildChatDeviceBindingMessageBase64Url(
  input: ChatDeviceBindingInput,
): string {
  return bytesToBase64Url(buildChatDeviceBindingMessage(input));
}

export async function verifyChatDeviceBinding(
  input: ChatDeviceBindingInput,
  signature: string,
  p256PublicKeyHex: string,
): Promise<boolean> {
  // Chat 初始化只使用已登记的硬件设备子钥，禁止读取或验签钱包 seed。
  return verifyP256Signature(
    buildChatDeviceBindingMessage(input),
    signature,
    p256PublicKeyHex,
  );
}
