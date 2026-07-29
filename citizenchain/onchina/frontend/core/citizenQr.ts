// QR_V1 统一协议 TS 类型与解析器。
//
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// 与 CitizenApp / CitizenWallet 的 Dart envelope 字段逐字节一致。

import { decodeSs58 } from '../utils/ss58';

export const QR_V1 = 'QR_V1' as const;

export type QrKind = 'sign_request' | 'sign_response' | 'user_contact' | 'user_transfer'
  | 'wallet_code';

const KIND_TO_CODE: Record<QrKind, number> = {
  sign_request: 1,
  sign_response: 2,
  user_contact: 3,
  user_transfer: 4,
  wallet_code: 5,
};

const CODE_TO_KIND: Record<number, QrKind> = {
  1: 'sign_request',
  2: 'sign_response',
  3: 'user_contact',
  4: 'user_transfer',
  5: 'wallet_code',
};

export function isFixedKind(kind: QrKind): boolean {
  // 用户码与钱包码都是固定码（无 i/e）；收款码与签名请求/响应是临时码。
  return kind === 'user_contact' || kind === 'wallet_code';
}

export interface SignRequestBody {
  action: number;
  sig_alg: 1;
  account_id: string;
  payload: string;
  payload_hex: string;
}

export interface SignResponseBody {
  account_id: string;
  signature: string;
}

export interface UserContactBody {
  cid_number: string;
  ss58_address: string;
  display_name: string;
}

export interface UserTransferBody {
  ss58_address: string;
  recipient_name: string;
  amount: string;
  symbol: string;
  memo: string;
  bank: string;
}

/** 钱包码 body：只声明账户，不含任何身份字段。 */
export interface WalletCodeBody {
  account_id: string;
}

export type QrBodyByKind = {
  sign_request: SignRequestBody;
  sign_response: SignResponseBody;
  user_contact: UserContactBody;
  user_transfer: UserTransferBody;
  wallet_code: WalletCodeBody;
};

export interface QrEnvelope<K extends QrKind = QrKind> {
  proto: typeof QR_V1;
  kind: K;
  id?: string;
  expires_at?: number;
  body: QrBodyByKind[K];
}

export class QrParseError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'QrParseError';
  }
}

function requireString(obj: Record<string, unknown>, key: string): string {
  const v = obj[key];
  if (typeof v !== 'string' || v.length === 0) {
    throw new QrParseError(`字段 ${key} 必填非空字符串`);
  }
  return v;
}

function requireInt(obj: Record<string, unknown>, key: string): number {
  const v = obj[key];
  if (typeof v !== 'number' || !Number.isInteger(v)) {
    throw new QrParseError(`字段 ${key} 必填整数`);
  }
  return v;
}

function requireExactKeys(
  obj: Record<string, unknown>,
  allowedKeys: readonly string[],
  field: string,
): void {
  const allowed = new Set(allowedKeys);
  const unknown = Object.keys(obj).filter((key) => !allowed.has(key));
  if (unknown.length > 0) {
    throw new QrParseError(`${field} 包含未知字段: ${unknown.join(',')}`);
  }
}

function requireCompactB64(obj: Record<string, unknown>, key: string): string {
  const v = requireString(obj, key);
  if (!/^[A-Za-z0-9_-]+$/.test(v)) {
    throw new QrParseError(`字段 ${key} 必须为 base64url`);
  }
  return v;
}

function b64ToBytes(value: string): Uint8Array {
  const normalized = value.replace(/-/g, '+').replace(/_/g, '/');
  const padded = normalized + '='.repeat((4 - (normalized.length % 4)) % 4);
  const g = globalThis as typeof globalThis & {
    atob?: (input: string) => string;
    Buffer?: { from(input: string, encoding: string): Uint8Array };
  };
  if (typeof g.atob === 'function') {
    const binary = g.atob(padded);
    return Uint8Array.from(binary, (ch) => ch.charCodeAt(0));
  }
  if (g.Buffer) {
    return Uint8Array.from(g.Buffer.from(padded, 'base64'));
  }
  throw new QrParseError('当前环境不支持 base64url 解码');
}

function b64ToHex(value: string, expectedLength: number, field: string): string {
  const bytes = b64ToBytes(value);
  if (bytes.length !== expectedLength) {
    throw new QrParseError(`${field} 长度必须为 ${expectedLength} 字节`);
  }
  return `0x${Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')}`;
}

function b64ToPayloadHex(value: string): string {
  const bytes = b64ToBytes(value);
  if (bytes.length === 0) throw new QrParseError('b.d 不能为空');
  return `0x${Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')}`;
}

function parseSignRequestBody(b: Record<string, unknown>): SignRequestBody {
  requireExactKeys(b, ['a', 'g', 'u', 'd'], 'b');
  const action = requireInt(b, 'a');
  const sigAlg = requireInt(b, 'g');
  if (sigAlg !== 1) throw new QrParseError('b.g 必须为 1(sr25519)');
  const u = requireCompactB64(b, 'u');
  const d = requireCompactB64(b, 'd');
  return {
    action,
    sig_alg: 1,
    account_id: b64ToHex(u, 32, 'b.u'),
    payload: d,
    payload_hex: b64ToPayloadHex(d),
  };
}

function parseSignResponseBody(b: Record<string, unknown>): SignResponseBody {
  requireExactKeys(b, ['u', 's'], 'b');
  const u = requireCompactB64(b, 'u');
  const s = requireCompactB64(b, 's');
  return {
    account_id: b64ToHex(u, 32, 'b.u'),
    signature: b64ToHex(s, 64, 'b.s'),
  };
}

function parseUserContactBody(b: Record<string, unknown>): UserContactBody {
  requireExactKeys(b, ['cid_number', 'ss58_address', 'display_name'], 'b');
  const cidNumber = requireString(b, 'cid_number');
  if (cidNumber !== cidNumber.trim() || new TextEncoder().encode(cidNumber).length > 32) {
    throw new QrParseError('b.cid_number 必须为无首尾空格的 1 到 32 字节字符串');
  }
  const ss58Address = requireString(b, 'ss58_address');
  if (ss58Address !== ss58Address.trim()) {
    throw new QrParseError('b.ss58_address 不得包含首尾空格');
  }
  try {
    decodeSs58(ss58Address);
  } catch (error) {
    throw new QrParseError(
      `b.ss58_address 必须为本链规范 SS58 地址: ${
        error instanceof Error ? error.message : '校验失败'
      }`,
    );
  }
  const displayName = requireString(b, 'display_name');
  if (displayName !== displayName.trim() || [...displayName].length > 40) {
    throw new QrParseError('b.display_name 必须为无首尾空格的 1 到 40 字符串');
  }
  return {
    cid_number: cidNumber,
    ss58_address: ss58Address,
    display_name: displayName,
  };
}

function parseUserTransferBody(b: Record<string, unknown>): UserTransferBody {
  requireExactKeys(
    b,
    ['ss58_address', 'recipient_name', 'amount', 'symbol', 'memo', 'bank'],
    'b',
  );
  const ss58Address = requireString(b, 'ss58_address');
  const recipientName = b['recipient_name'];
  const amount = b['amount'];
  const symbol = b['symbol'];
  const memo = b['memo'];
  const bank = b['bank'];
  if (
    typeof recipientName !== 'string' ||
    typeof amount !== 'string' ||
    typeof symbol !== 'string' ||
    typeof memo !== 'string' ||
    typeof bank !== 'string'
  ) {
    throw new QrParseError('user_transfer 的 recipient_name/amount/symbol/memo/bank 必须为字符串');
  }
  return {
    ss58_address: ss58Address,
    recipient_name: recipientName,
    amount,
    symbol,
    memo,
    bank,
  };
}

function parseWalletCodeBody(b: Record<string, unknown>): WalletCodeBody {
  requireExactKeys(b, ['account_id'], 'b');
  const accountId = requireString(b, 'account_id');
  if (!/^0x[0-9a-f]{64}$/.test(accountId)) {
    throw new QrParseError('wallet_code.account_id 必须为小写 0x 加 64 位十六进制');
  }
  return { account_id: accountId };
}

export function parseQrEnvelope(raw: string | Record<string, unknown>): QrEnvelope {
  let data: Record<string, unknown>;
  if (typeof raw === 'string') {
    try {
      data = JSON.parse(raw) as Record<string, unknown>;
    } catch (e) {
      throw new QrParseError(`QR 内容非合法 JSON: ${(e as Error).message}`);
    }
  } else {
    data = raw;
  }
  if (!data || typeof data !== 'object') throw new QrParseError('QR 内容不是对象');
  if (data['p'] !== QR_V1) throw new QrParseError(`p 必须为 ${QR_V1},实际: ${data['p']}`);
  const code = requireInt(data, 'k');
  const kind = CODE_TO_KIND[code];
  if (!kind) throw new QrParseError(`未知 k: ${code}`);
  requireExactKeys(data, isFixedKind(kind) ? ['p', 'k', 'b'] : ['p', 'k', 'i', 'e', 'b'], 'QR');

  let id: string | undefined;
  let expiresAt: number | undefined;
  if (isFixedKind(kind)) {
    if ('i' in data || 'e' in data) throw new QrParseError(`固定码 ${kind} 不应包含 i/e`);
  } else {
    id = requireString(data, 'i');
    expiresAt = requireInt(data, 'e');
  }

  const bodyRaw = data['b'];
  if (!bodyRaw || typeof bodyRaw !== 'object') throw new QrParseError('缺少 b 对象');
  const b = bodyRaw as Record<string, unknown>;

  let body: QrBodyByKind[typeof kind];
  switch (kind) {
    case 'sign_request':
      body = parseSignRequestBody(b);
      break;
    case 'sign_response':
      body = parseSignResponseBody(b);
      break;
    case 'user_contact':
      body = parseUserContactBody(b);
      break;
    case 'user_transfer':
      body = parseUserTransferBody(b);
      break;
    case 'wallet_code':
      body = parseWalletCodeBody(b);
      break;
  }

  const env: QrEnvelope = { proto: QR_V1, kind, body };
  if (id !== undefined) env.id = id;
  if (expiresAt !== undefined) env.expires_at = expiresAt;
  return env;
}

export function serializeQrEnvelope(env: QrEnvelope): string {
  const out: Record<string, unknown> = {
    p: QR_V1,
    k: KIND_TO_CODE[env.kind],
  };
  if (!isFixedKind(env.kind)) {
    if (env.id === undefined || env.expires_at === undefined) {
      throw new QrParseError(`临时码 ${env.kind} 必须提供 id/expires_at`);
    }
    out['i'] = env.id;
    out['e'] = env.expires_at;
  }
  out['b'] = env.body;
  return JSON.stringify(out);
}

export function buildSignatureMessage(args: {
  kind: QrKind;
  id: string;
  system?: string | null;
  expiresAt?: number | null;
  principal: string;
}): string {
  const sys = args.system ?? '';
  const exp = args.expiresAt ?? 0;
  let pp = args.principal;
  if (pp.startsWith('0x') || pp.startsWith('0X')) pp = pp.slice(2);
  pp = pp.toLowerCase();
  return `${QR_V1}|${KIND_TO_CODE[args.kind]}|${args.id}|${sys}|${exp}|${pp}`;
}
