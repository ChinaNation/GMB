// QR_V1 统一协议 TS 类型与解析器。
//
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// Golden fixtures:memory/01-architecture/qr/qr-protocol-fixtures/*.json

import { normalizeSs58Address } from '../ss58';

export const QR_V1 = 'QR_V1' as const;

export type QrKind =
  | 'sign_request'
  | 'sign_response'
  | 'user_contact'
  | 'user_transfer'
  | 'wallet_code';

export const QR_KIND_CODE: Record<QrKind, number> = {
  sign_request: 1,
  sign_response: 2,
  user_contact: 3,
  user_transfer: 4,
  wallet_code: 5,
};

const QR_KIND_BY_CODE = new Map<number, QrKind>(
  Object.entries(QR_KIND_CODE).map(([kind, code]) => [code, kind as QrKind]),
);

// 用户码与钱包码都是固定码（无 i/e）；收款码与签名请求/响应是临时码。
export const FIXED_KINDS: readonly QrKind[] = ['user_contact', 'wallet_code'];

export function isFixedKind(kind: QrKind): boolean {
  return FIXED_KINDS.includes(kind);
}

export interface SignRequestBody {
  action: number;
  sig_alg: 1;
  signer_public_key: string;
  payload_hex: string;
}

export interface SignResponseBody {
  signer_public_key: string;
  signature: string;
}

export interface UserContactBody {
  cid_number: string;
  ss58_address: string;
  display_name: string;
}

export interface UserTransferBody {
  ss58_address: string;
  recipientName: string;
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

/** 严格字段闸:body 出现未知字段直接拒,防止旧协议字段混入。 */
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
    signer_public_key: b64ToHex(u, 'u', 32),
    payload_hex: b64ToHex(d, 'd'),
  };
}

function parseSignResponseBody(b: Record<string, unknown>): SignResponseBody {
  const u = requireString(b, 'u');
  const s = requireString(b, 's');
  return {
    signer_public_key: b64ToHex(u, 'u', 32),
    signature: b64ToHex(s, 's', 64),
  };
}

/// 身份主键是 `cid_number`;`ss58_address` 只作展示与边界输入输出。
/// 按规范必须拒绝旧 `contact_name`、缺失字段、未知字段、非 2027 SS58 及非规范 CID/昵称。
function parseUserContactBody(b: Record<string, unknown>): UserContactBody {
  requireExactKeys(b, ['cid_number', 'ss58_address', 'display_name'], 'b');
  const cidNumber = requireString(b, 'cid_number');
  if (
    cidNumber !== cidNumber.trim() ||
    new TextEncoder().encode(cidNumber).length > 32
  ) {
    throw new QrParseError('b.cid_number 必须为无首尾空格的 1 到 32 字节字符串');
  }
  const ss58Address = requireString(b, 'ss58_address');
  if (ss58Address !== ss58Address.trim()) {
    throw new QrParseError('b.ss58_address 不得包含首尾空格');
  }
  try {
    normalizeSs58Address(ss58Address);
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
  const ss58_address = requireString(b, 'ss58_address');
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
  return { ss58_address, recipientName, amount, symbol, memo, bank };
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
    case 'wallet_code':
      body = parseWalletCodeBody(b);
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
