import { signatureVerify } from "@polkadot/util-crypto/signature/verify";
import { bytesToBase64Url } from "./codec";
import {
  OP_SIGN_IM_WALLET_BINDING,
  concatBytes,
  scaleString,
  signingMessage,
  u64Le,
} from "../shared/signing_message";

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
  return signingMessage(OP_SIGN_IM_WALLET_BINDING, scalePayload);
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
