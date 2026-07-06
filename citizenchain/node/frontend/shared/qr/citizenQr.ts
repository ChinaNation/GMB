// QR_V1 统一协议 TS 类型与解析器。
//
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// Golden fixtures:memory/01-architecture/qr/qr-protocol-fixtures/*.json

export const QR_V1 = 'QR_V1' as const;

export type QrKind =
  | 'sign_request'
  | 'sign_response'
  | 'user_contact'
  | 'user_transfer';

export const QR_KIND_CODE: Record<QrKind, number> = {
  sign_request: 1,
  sign_response: 2,
  user_contact: 3,
  user_transfer: 4,
};

const QR_KIND_BY_CODE = new Map<number, QrKind>(
  Object.entries(QR_KIND_CODE).map(([kind, code]) => [code, kind as QrKind]),
);

export const FIXED_KINDS: readonly QrKind[] = ['user_contact'];

export function isFixedKind(kind: QrKind): boolean {
  return FIXED_KINDS.includes(kind);
}

export interface SignRequestBody {
  action: number;
  sig_alg: 1;
  pubkey: string;
  payload_hex: string;
}

export interface SignResponseBody {
  pubkey: string;
  signature: string;
}

export interface UserContactBody {
  address: string;
  contactName: string;
}

export interface UserTransferBody {
  address: string;
  recipientName: string;
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
  p: typeof QR_V1;
  k: number;
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

function normalizeB64(input: string): string {
  return input
    .replace(/-/g, '+')
    .replace(/_/g, '/')
    .padEnd(input.length + ((4 - (input.length % 4)) % 4), '=');
}

function b64ToHex(input: string, field: string, expectedLen?: number): string {
  let binary: string;
  try {
    binary = atob(normalizeB64(input));
  } catch {
    throw new QrParseError(`字段 ${field} 必须为 base64url`);
  }
  const bytes = Array.from(binary, (ch) => ch.charCodeAt(0));
  if (expectedLen !== undefined && bytes.length !== expectedLen) {
    throw new QrParseError(`字段 ${field} 必须解码为 ${expectedLen} 字节`);
  }
  if (bytes.length === 0) {
    throw new QrParseError(`字段 ${field} 不能为空`);
  }
  return `0x${bytes.map((b) => b.toString(16).padStart(2, '0')).join('')}`;
}

function parseSignRequestBody(b: Record<string, unknown>): SignRequestBody {
  const action = requireInt(b, 'a');
  const sigAlg = requireInt(b, 'g');
  if (action <= 0) throw new QrParseError('b.a 必须为正整数');
  if (sigAlg !== 1) throw new QrParseError('b.g 必须为 1(sr25519)');
  const u = requireString(b, 'u');
  const d = requireString(b, 'd');
  return {
    action,
    sig_alg: 1,
    pubkey: b64ToHex(u, 'u', 32),
    payload_hex: b64ToHex(d, 'd'),
  };
}

function parseSignResponseBody(b: Record<string, unknown>): SignResponseBody {
  const u = requireString(b, 'u');
  const s = requireString(b, 's');
  return {
    pubkey: b64ToHex(u, 'u', 32),
    signature: b64ToHex(s, 's', 64),
  };
}

function parseUserContactBody(b: Record<string, unknown>): UserContactBody {
  return {
    address: requireString(b, 'address'),
    contactName: requireString(b, 'contact_name'),
  };
}

function parseUserTransferBody(b: Record<string, unknown>): UserTransferBody {
  const address = requireString(b, 'address');
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
    throw new QrParseError(
      'user_transfer 的 recipient_name/amount/symbol/memo/bank 必须为字符串',
    );
  }
  return { address, recipientName, amount, symbol, memo, bank };
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
  if (!data || typeof data !== 'object') {
    throw new QrParseError('QR 内容不是对象');
  }
  if (data['p'] !== QR_V1) {
    throw new QrParseError(`p 必须为 ${QR_V1},实际: ${data['p']}`);
  }

  const kindCode = requireInt(data, 'k');
  const kind = QR_KIND_BY_CODE.get(kindCode);
  if (!kind) {
    throw new QrParseError(`未知 k: ${kindCode}`);
  }

  let id: string | undefined;
  let expiresAt: number | undefined;
  if (isFixedKind(kind)) {
    if ('i' in data || 'e' in data) {
      throw new QrParseError(`固定码 ${kindCode} 不应包含 i/e`);
    }
  } else {
    id = requireString(data, 'i');
    expiresAt = requireInt(data, 'e');
  }

  const bodyRaw = data['b'];
  if (!bodyRaw || typeof bodyRaw !== 'object') {
    throw new QrParseError('缺少 b 对象');
  }
  const b = bodyRaw as Record<string, unknown>;

  let body: QrBodyByKind[QrKind];
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

  const env: QrEnvelope = {
    p: QR_V1,
    k: kindCode,
    kind,
    body,
  };
  if (id !== undefined) env.id = id;
  if (expiresAt !== undefined) env.expires_at = expiresAt;
  return env;
}

export function serializeQrEnvelope(env: QrEnvelope): string {
  const out: Record<string, unknown> = {
    p: QR_V1,
    k: QR_KIND_CODE[env.kind],
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
  kind: QrKind | number;
  id: string;
  system?: string | null;
  expiresAt?: number | null;
  principal: string;
}): string {
  const sys = args.system ?? '';
  const exp = args.expiresAt ?? 0;
  const kindCode =
    typeof args.kind === 'number' ? args.kind : QR_KIND_CODE[args.kind];
  let pp = args.principal;
  if (pp.startsWith('0x') || pp.startsWith('0X')) pp = pp.slice(2);
  pp = pp.toLowerCase();
  return `${QR_V1}|${kindCode}|${args.id}|${sys}|${exp}|${pp}`;
}
