import { hexToBytes } from './signing_message';

const ACCOUNT_ID_PATTERN = /^0x[0-9a-f]{64}$/;

export function createId(prefix: string): string {
  return `${prefix}_${crypto.randomUUID().replaceAll('-', '')}`;
}

/// AccountId 与 sr25519 签名公钥使用同一组 32 字节；二维码 `u` 只取无前缀 hex。
export function signerPublicKeyHex(accountId: string): string {
  return assertAccountId(accountId).slice(2);
}

export function assertAccountId(value: unknown): string {
  if (typeof value !== 'string' || !ACCOUNT_ID_PATTERN.test(value)) {
    throw new Error('account_id must be lowercase 0x followed by 64 hexadecimal characters');
  }
  return value;
}

/// 仅在链 storage key 或验签库需要原始字节时转换；HTTP、D1、KV、DO、Queue 和 R2
/// 始终保存规范文本 AccountId。
export function accountIdBytes(value: unknown): Uint8Array {
  return hexToBytes(assertAccountId(value));
}
