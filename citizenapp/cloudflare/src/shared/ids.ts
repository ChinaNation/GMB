import { decodeAddress } from '@polkadot/util-crypto';
import { bytesToHex } from './signing_message';

export function createId(prefix: string): string {
  return `${prefix}_${crypto.randomUUID().replaceAll('-', '')}`;
}

/// ss58 账户 → 32 字节公钥小写 hex（无 0x）。供官网构建 QR_V1 signRequest 的 `u`。
export function ownerPubkeyHex(ownerAccount: string): string {
  return bytesToHex(decodeAddress(ownerAccount));
}

export function assertOwnerAccount(value: unknown): string {
  if (typeof value !== 'string') {
    throw new Error('owner_account must be string');
  }

  const ownerAccount = value.trim();
  if (ownerAccount.length < 16 || ownerAccount.length > 128 || ownerAccount.includes('/')) {
    throw new Error('owner_account is invalid');
  }

  return ownerAccount;
}
