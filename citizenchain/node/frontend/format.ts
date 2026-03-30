import { blake2b } from '@noble/hashes/blake2.js';

/**
 * 金额千分位格式化工具。
 *
 * @example
 * formatAmount("1234567.89") // "1,234,567.89"
 * formatAmount("100")        // "100"
 * formatAmount(null)         // null
 */
/**
 * 将链上余额（分）格式化为带千分位的元显示。
 * 1 元 = 100 分。
 *
 * @example
 * formatBalance("123456") // "1,234.56 元"
 * formatBalance("50")     // "0.50 元"
 */
export function formatBalance(fenStr: string): string {
  const fen = BigInt(fenStr);
  const negative = fen < 0n;
  const abs = negative ? -fen : fen;
  const yuan = abs / 100n;
  const remainder = abs % 100n;
  const yuanFormatted = yuan.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  const decimal = remainder.toString().padStart(2, '0');
  return `${negative ? '-' : ''}${yuanFormatted}.${decimal} 元`;
}

/**
 * 将 32 字节 hex 公钥编码为 SS58 地址（prefix = 2027）。
 * 输入：64 位 hex 字符串（可带 0x 前缀）。
 */
export function hexToSs58(hex: string): string {
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  if (clean.length !== 64) return `0x${clean.slice(0, 8)}…`;
  const pubkey = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    pubkey[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }
  // SS58 prefix 2027: two-byte encoding
  // first = ((2027 & 0xFC) >> 2) | 0x40 = (2024 >> 2) | 64 = 506 | 64 = 570...
  // Actually: for prefix > 63, encode as two bytes:
  // first  = ((prefix >> 2) & 0x3F) | 0x40
  // second = ((prefix & 0x03) << 6) | ((prefix >> 8) & 0x3F)
  const prefix = 2027;
  const first = ((prefix >> 2) & 0x3F) | 0x40;
  const second = ((prefix & 0x03) << 6) | ((prefix >> 8) & 0x3F);
  // Checksum: Blake2b-512 of SS58PRE + prefix_bytes + pubkey, take first 2 bytes
  const ss58Pre = new TextEncoder().encode('SS58PRE');
  const payload = new Uint8Array(ss58Pre.length + 2 + 32);
  payload.set(ss58Pre);
  payload[ss58Pre.length] = first;
  payload[ss58Pre.length + 1] = second;
  payload.set(pubkey, ss58Pre.length + 2);
  const hash: Uint8Array = blake2b(payload, { dkLen: 64 });
  const checksum = hash.slice(0, 2);
  // Full bytes: prefix(2) + pubkey(32) + checksum(2) = 36 bytes
  const full = new Uint8Array(36);
  full[0] = first;
  full[1] = second;
  full.set(pubkey, 2);
  full.set(checksum, 34);
  return encodeBase58(full);
}

const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

function encodeBase58(data: Uint8Array): string {
  let leadingZeros = 0;
  while (leadingZeros < data.length && data[leadingZeros] === 0) leadingZeros++;
  // Convert to big integer manually
  const digits: number[] = [0];
  for (let i = leadingZeros; i < data.length; i++) {
    let carry = data[i];
    for (let j = 0; j < digits.length; j++) {
      const x = digits[j] * 256 + carry;
      digits[j] = x % 58;
      carry = Math.floor(x / 58);
    }
    while (carry > 0) {
      digits.push(carry % 58);
      carry = Math.floor(carry / 58);
    }
  }
  let result = '1'.repeat(leadingZeros);
  for (let i = digits.length - 1; i >= 0; i--) {
    result += BASE58_ALPHABET[digits[i]];
  }
  return result;
}

export function formatAmount(value: string | null | undefined): string | null {
  if (value == null) return null;
  const trimmed = value.trim();
  if (!trimmed) return null;

  const match = trimmed.match(/^(-?[\d.]+)(.*)$/);
  if (!match) return trimmed;

  const [, numPart, suffix] = match;
  const [intPart, decimal] = numPart.split('.');
  const formatted = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  const decimalStr = decimal != null ? `.${decimal}` : '';
  return `${formatted}${decimalStr}${suffix}`;
}
