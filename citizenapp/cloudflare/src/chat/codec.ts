import { HttpError } from "../shared/http";
import { assertAccountId } from "../shared/ids";

const DEVICE_ID_PATTERN = /^[A-Za-z0-9_.:-]{3,128}$/;
const KEY_PACKAGE_ID_PATTERN = /^[A-Za-z0-9_.:-]{3,160}$/;
const ENVELOPE_ID_PATTERN = /^[A-Za-z0-9_.:-]{3,220}$/;
const HEX_PATTERN = /^[0-9a-f]+$/;
const BASE64URL_PATTERN = /^[A-Za-z0-9_-]*$/;

export function assertChatAccountId(
  value: unknown,
  code = "invalid_chat_account_id",
): string {
  try {
    return assertAccountId(value);
  } catch {
    throw new HttpError(400, code, "聊天钱包账户格式不合法");
  }
}

export function assertDeviceId(value: unknown): string {
  if (typeof value !== "string" || !DEVICE_ID_PATTERN.test(value)) {
    throw new HttpError(400, "invalid_device_id", "Chat 设备编号格式不合法");
  }
  return value;
}

export function assertDevicePublicKeyHex(value: unknown): string {
  if (typeof value !== "string") {
    throw new HttpError(
      400,
      "invalid_device_public_key",
      "Chat 设备公钥格式不合法",
    );
  }
  if (
    value.length < 2 ||
    value.length > 512 ||
    value.length % 2 !== 0
  ) {
    throw new HttpError(
      400,
      "invalid_device_public_key",
      "Chat 设备公钥长度不合法",
    );
  }
  if (!HEX_PATTERN.test(value)) {
    throw new HttpError(
      400,
      "invalid_device_public_key",
      "Chat 设备公钥必须是小写 hex",
    );
  }
  return value;
}

export function assertKeyPackageId(value: unknown): string {
  if (typeof value !== "string" || !KEY_PACKAGE_ID_PATTERN.test(value)) {
    throw new HttpError(
      400,
      "invalid_key_package_id",
      "KeyPackage 编号格式不合法",
    );
  }
  return value;
}

export function assertEnvelopeId(value: unknown): string {
  if (typeof value !== "string" || !ENVELOPE_ID_PATTERN.test(value)) {
    throw new HttpError(
      400,
      "invalid_envelope_id",
      "密文 envelope 编号格式不合法",
    );
  }
  return value;
}

export function assertCipherSuite(value: unknown): string {
  if (typeof value !== "string" || value.length < 3 || value.length > 128) {
    throw new HttpError(
      400,
      "invalid_cipher_suite",
      "MLS cipher suite 格式不合法",
    );
  }
  return value;
}

export function assertBase64Url(
  value: unknown,
  code: string,
  message: string,
): string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > 1_500_000
  ) {
    throw new HttpError(400, code, message);
  }
  if (!BASE64URL_PATTERN.test(value)) {
    throw new HttpError(400, code, message);
  }
  return value;
}

export function assertOptionalBase64Url(
  value: unknown,
  code: string,
  message: string,
): string {
  if (value === undefined || value === null || value === "") {
    return "";
  }
  return assertBase64Url(value, code, message);
}

export function assertPositiveMillis(
  value: unknown,
  code: string,
  message: string,
): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value <= 0) {
    throw new HttpError(400, code, message);
  }
  return value;
}

export function assertLimit(
  value: string | null,
  fallback: number,
  max: number,
): number {
  if (!value) {
    return fallback;
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new HttpError(400, "invalid_limit", "分页数量不合法");
  }
  return Math.min(parsed, max);
}

export function assertMlsMessageKind(value: unknown): string {
  if (
    value === "welcome" ||
    value === "application" ||
    value === "unspecified"
  ) {
    return value;
  }
  throw new HttpError(400, "invalid_mls_message_kind", "MLS 消息类型不合法");
}

export function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary)
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");
}

export function base64UrlToBytes(value: string): Uint8Array {
  const padded = value
    .replace(/-/g, "+")
    .replace(/_/g, "/")
    .padEnd(Math.ceil(value.length / 4) * 4, "=");
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}
