import { blake2b } from '@noble/hashes/blake2.js';

const SS58_PREFIX = 2027;
const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
const BASE58_INDEX = new Map<string, number>(
  [...BASE58_ALPHABET].map((ch, idx) => [ch, idx])
);

/** 将 32 字节 hex 公钥编码为 CitizenChain SS58 地址（prefix = 2027）。 */
export function hexToSs58(hex: string): string {
  const clean = hex.startsWith('0x') ? hex.slice(2) : hex;
  if (clean.length !== 64) return `0x${clean.slice(0, 8)}...`;
  const pubkey = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    pubkey[i] = parseInt(clean.substring(i * 2, i * 2 + 2), 16);
  }

  const prefixBytes = encodeSs58Prefix(SS58_PREFIX);
  const ss58Pre = new TextEncoder().encode('SS58PRE');
  const payload = new Uint8Array(ss58Pre.length + prefixBytes.length + pubkey.length);
  payload.set(ss58Pre);
  payload.set(prefixBytes, ss58Pre.length);
  payload.set(pubkey, ss58Pre.length + prefixBytes.length);
  const hash: Uint8Array = blake2b(payload, { dkLen: 64 });
  const checksum = hash.slice(0, 2);

  const full = new Uint8Array(prefixBytes.length + pubkey.length + checksum.length);
  full.set(prefixBytes);
  full.set(pubkey, prefixBytes.length);
  full.set(checksum, prefixBytes.length + pubkey.length);
  return encodeBase58(full);
}

/** 校验并规范化用户输入的钱包地址：支持 0x 公钥或 prefix=2027 的 SS58 地址。 */
export function normalizeSs58AccountAddress(input: string, emptyMessage = '请输入钱包地址'): string {
  const value = input.trim();
  if (!value) {
    throw new Error(emptyMessage);
  }
  if (value.startsWith('0x')) {
    const raw = value.slice(2);
    if (!/^[0-9a-fA-F]{64}$/.test(raw)) {
      throw new Error('十六进制钱包地址格式无效，应为 0x + 64 位十六进制');
    }
    return `0x${raw.toLowerCase()}`;
  }

  const data = decodeBase58(value);
  const { prefix, prefixLen } = decodeSs58Prefix(data);
  if (prefix !== SS58_PREFIX) {
    throw new Error('SS58 地址前缀无效，必须为 2027');
  }
  if (data.length < prefixLen + 32 + 2) {
    throw new Error('SS58 地址长度无效');
  }
  const payloadLen = data.length - prefixLen - 2;
  if (payloadLen !== 32) {
    throw new Error('SS58 地址账户长度无效，必须是 32 字节账户地址');
  }

  // 中文注释：按 Substrate SS58 标准校验 Blake2b-512 前两字节校验和。
  const withoutChecksum = data.slice(0, data.length - 2);
  const actualChecksum = data.slice(data.length - 2);
  const ss58Pre = new TextEncoder().encode('SS58PRE');
  const preimage = new Uint8Array(ss58Pre.length + withoutChecksum.length);
  preimage.set(ss58Pre);
  preimage.set(withoutChecksum, ss58Pre.length);
  const hash = blake2b(preimage, { dkLen: 64 });
  if (actualChecksum[0] !== hash[0] || actualChecksum[1] !== hash[1]) {
    throw new Error('SS58 地址校验和无效');
  }

  return value;
}

function encodeSs58Prefix(prefix: number): Uint8Array {
  if (prefix <= 63) {
    return new Uint8Array([prefix]);
  }
  const first = ((prefix >> 2) & 0x3f) | 0x40;
  const second = ((prefix & 0x03) << 6) | ((prefix >> 8) & 0x3f);
  return new Uint8Array([first, second]);
}

function decodeSs58Prefix(data: Uint8Array): { prefix: number; prefixLen: number } {
  if (data.length === 0) {
    throw new Error('SS58 地址为空');
  }
  const first = data[0];
  if (first <= 63) {
    return { prefix: first, prefixLen: 1 };
  }
  if (first <= 127) {
    if (data.length < 2) {
      throw new Error('SS58 地址格式无效');
    }
    const second = data[1];
    const prefix = ((first & 0x3f) << 2) | (second >> 6) | ((second & 0x3f) << 8);
    return { prefix, prefixLen: 2 };
  }
  throw new Error('SS58 地址格式无效');
}

function decodeBase58(input: string): Uint8Array {
  if (!input) {
    throw new Error('SS58 地址为空');
  }
  let leadingZeros = 0;
  while (leadingZeros < input.length && input[leadingZeros] === '1') {
    leadingZeros += 1;
  }

  const bytes: number[] = [0];
  for (const ch of input) {
    const val = BASE58_INDEX.get(ch);
    if (val === undefined) {
      throw new Error('SS58 地址解码失败');
    }
    let carry = val;
    for (let i = 0; i < bytes.length; i += 1) {
      const x = bytes[i] * 58 + carry;
      bytes[i] = x & 0xff;
      carry = x >> 8;
    }
    while (carry > 0) {
      bytes.push(carry & 0xff);
      carry >>= 8;
    }
  }

  const out = new Uint8Array(leadingZeros + bytes.length);
  out.fill(0, 0, leadingZeros);
  for (let i = 0; i < bytes.length; i += 1) {
    out[out.length - 1 - i] = bytes[i];
  }
  return out;
}

function encodeBase58(data: Uint8Array): string {
  let leadingZeros = 0;
  while (leadingZeros < data.length && data[leadingZeros] === 0) leadingZeros++;
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
