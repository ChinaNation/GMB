// QR_V1 统一协议 TS 类型与解析器。
//
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// 与 CitizenWallet 的 Dart envelope 字段逐字节一致。

export const QR_V1 = 'QR_V1' as const;

export type QrKind = 'sign_request' | 'sign_response' | 'user_contact' | 'user_transfer';

const KIND_TO_CODE: Record<QrKind, number> = {
  sign_request: 1,
  sign_response: 2,
  user_contact: 3,
  user_transfer: 4,
};

const CODE_TO_KIND: Record<number, QrKind> = {
  1: 'sign_request',
  2: 'sign_response',
  3: 'user_contact',
  4: 'user_transfer',
};

export function isFixedKind(kind: QrKind): boolean {
  return kind === 'user_contact';
}

export interface SignRequestBody {
  action: number;
  sig_alg: 1;
  pubkey: string;
  payload: string;
  payload_hex: string;
}

export interface SignResponseBody {
  pubkey: string;
  signature: string;
}

export interface UserContactBody {
  address: string;
  name: string;
}

export interface UserTransferBody {
  address: string;
  name: string;
  amount: string;
  symbol: string;
  memo: string;
  bank: string;
}

export type QrBodyByKind = {
  sign_request: SignRequestBody;
  sign_response: SignResponseBody;
  user_contact: UserContactBody;
  user_transfer: UserTransferBody;
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

function requireCompactB64(obj: Record<string, unknown>, key: string): string {
  const v = requireString(obj, key);
  if (!/^[A-Za-z0-9_-]+$/.test(v)) throw new QrParseError(`字段 ${key} 必须为 base64url`);
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
  if (g.Buffer) return Uint8Array.from(g.Buffer.from(padded, 'base64'));
  throw new QrParseError('当前环境不支持 base64url 解码');
}

function b64ToHex(value: string, expectedLength: number, field: string): string {
  const bytes = b64ToBytes(value);
  if (bytes.length !== expectedLength) throw new QrParseError(`${field} 长度必须为 ${expectedLength} 字节`);
  return `0x${Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')}`;
}

function b64ToPayloadHex(value: string): string {
  const bytes = b64ToBytes(value);
  if (bytes.length === 0) throw new QrParseError('b.d 不能为空');
  return `0x${Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')}`;
}

function parseSignRequestBody(b: Record<string, unknown>): SignRequestBody {
  const action = requireInt(b, 'a');
  const sigAlg = requireInt(b, 'g');
  if (sigAlg !== 1) throw new QrParseError('b.g 必须为 1(sr25519)');
  const u = requireCompactB64(b, 'u');
  const d = requireCompactB64(b, 'd');
  return {
    action,
    sig_alg: 1,
    pubkey: b64ToHex(u, 32, 'b.u'),
    payload: d,
    payload_hex: b64ToPayloadHex(d),
  };
}

function parseSignResponseBody(b: Record<string, unknown>): SignResponseBody {
  const u = requireCompactB64(b, 'u');
  const s = requireCompactB64(b, 's');
  return {
    pubkey: b64ToHex(u, 32, 'b.u'),
    signature: b64ToHex(s, 64, 'b.s'),
  };
}

function parseUserContactBody(b: Record<string, unknown>): UserContactBody {
  return {
    address: requireString(b, 'address'),
    name: requireString(b, 'name'),
  };
}

function parseUserTransferBody(b: Record<string, unknown>): UserTransferBody {
  const address = requireString(b, 'address');
  const name = b['name'];
  const amount = b['amount'];
  const symbol = b['symbol'];
  const memo = b['memo'];
  const bank = b['bank'];
  if (
    typeof name !== 'string' ||
    typeof amount !== 'string' ||
    typeof symbol !== 'string' ||
    typeof memo !== 'string' ||
    typeof bank !== 'string'
  ) {
    throw new QrParseError('user_transfer 的 name/amount/symbol/memo/bank 必须为字符串');
  }
  return { address, name, amount, symbol, memo, bank };
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
