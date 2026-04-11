// WUMIN_QR_V1 统一协议 TS 类型与解析器。
//
// 唯一事实源:memory/05-architecture/qr-protocol-spec.md
// Golden fixtures:memory/05-architecture/qr-protocol-fixtures/*.json
//
// 与 wuminapp/wumin 的 Dart envelope 字段逐字节一致。

export const WUMIN_QR_V1 = 'WUMIN_QR_V1' as const;

export type QrKind =
  | 'login_challenge'
  | 'login_receipt'
  | 'sign_request'
  | 'sign_response'
  | 'user_contact'
  | 'user_transfer'
  | 'user_duoqian';

export const QR_KINDS: readonly QrKind[] = [
  'login_challenge',
  'login_receipt',
  'sign_request',
  'sign_response',
  'user_contact',
  'user_transfer',
  'user_duoqian',
];

export const FIXED_KINDS: readonly QrKind[] = ['user_contact', 'user_duoqian'];

export function isFixedKind(kind: QrKind): boolean {
  return FIXED_KINDS.includes(kind);
}

// ------- body types -------

export interface LoginChallengeBody {
  system: 'sfid' | 'cpms';
  sys_pubkey: string;
  sys_sig: string;
}

export interface LoginReceiptBody {
  system: 'sfid' | 'cpms';
  pubkey: string;
  sig_alg: 'sr25519';
  signature: string;
  payload_hash: string;
  signed_at: number;
}

export interface SignDisplayField {
  label: string;
  value: string;
}

export interface SignDisplay {
  action: string;
  summary: string;
  fields: SignDisplayField[];
}

export interface SignRequestBody {
  address: string;
  pubkey: string;
  sig_alg: 'sr25519';
  payload_hex: string;
  spec_version: number;
  display: SignDisplay;
}

export interface SignResponseBody {
  pubkey: string;
  sig_alg: 'sr25519';
  signature: string;
  payload_hash: string;
  signed_at: number;
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

export interface UserDuoqianBody {
  address: string;
  name: string;
  proposal_id: number;
}

export type QrBodyByKind = {
  login_challenge: LoginChallengeBody;
  login_receipt: LoginReceiptBody;
  sign_request: SignRequestBody;
  sign_response: SignResponseBody;
  user_contact: UserContactBody;
  user_transfer: UserTransferBody;
  user_duoqian: UserDuoqianBody;
};

export interface QrEnvelope<K extends QrKind = QrKind> {
  proto: typeof WUMIN_QR_V1;
  kind: K;
  // 固定码省略以下三项
  id?: string;
  issued_at?: number;
  expires_at?: number;
  body: QrBodyByKind[K];
}

// ------- parser -------

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

function require0xHex(obj: Record<string, unknown>, key: string): string {
  const v = obj[key];
  if (typeof v !== 'string' || !v.startsWith('0x')) {
    throw new QrParseError(`字段 ${key} 必填 0x hex`);
  }
  return v;
}

function parseLoginChallengeBody(b: Record<string, unknown>): LoginChallengeBody {
  const system = requireString(b, 'system');
  if (system !== 'sfid' && system !== 'cpms') {
    throw new QrParseError(`login_challenge.system 非法: ${system}`);
  }
  return {
    system,
    sys_pubkey: require0xHex(b, 'sys_pubkey'),
    sys_sig: require0xHex(b, 'sys_sig'),
  };
}

function parseLoginReceiptBody(b: Record<string, unknown>): LoginReceiptBody {
  const system = requireString(b, 'system');
  if (system !== 'sfid' && system !== 'cpms') {
    throw new QrParseError(`login_receipt.system 非法: ${system}`);
  }
  const sigAlg = requireString(b, 'sig_alg');
  if (sigAlg !== 'sr25519') {
    throw new QrParseError('login_receipt.sig_alg 必须为 sr25519');
  }
  return {
    system,
    pubkey: require0xHex(b, 'pubkey'),
    sig_alg: 'sr25519',
    signature: require0xHex(b, 'signature'),
    payload_hash: require0xHex(b, 'payload_hash'),
    signed_at: requireInt(b, 'signed_at'),
  };
}

function parseSignDisplay(d: Record<string, unknown>): SignDisplay {
  const action = requireString(d, 'action');
  const summary = requireString(d, 'summary');
  const fieldsRaw = d['fields'];
  const fields: SignDisplayField[] = [];
  if (Array.isArray(fieldsRaw)) {
    for (const f of fieldsRaw) {
      if (f && typeof f === 'object') {
        const label = (f as Record<string, unknown>)['label'];
        const value = (f as Record<string, unknown>)['value'];
        if (typeof label === 'string' && typeof value === 'string') {
          fields.push({ label, value });
        }
      }
    }
  }
  return { action, summary, fields };
}

function parseSignRequestBody(b: Record<string, unknown>): SignRequestBody {
  const sigAlg = requireString(b, 'sig_alg');
  if (sigAlg !== 'sr25519') {
    throw new QrParseError('sign_request.sig_alg 必须为 sr25519');
  }
  const display = b['display'];
  if (!display || typeof display !== 'object') {
    throw new QrParseError('sign_request.display 必填对象');
  }
  return {
    address: requireString(b, 'address'),
    pubkey: require0xHex(b, 'pubkey'),
    sig_alg: 'sr25519',
    payload_hex: require0xHex(b, 'payload_hex'),
    spec_version: requireInt(b, 'spec_version'),
    display: parseSignDisplay(display as Record<string, unknown>),
  };
}

function parseSignResponseBody(b: Record<string, unknown>): SignResponseBody {
  const sigAlg = requireString(b, 'sig_alg');
  if (sigAlg !== 'sr25519') {
    throw new QrParseError('sign_response.sig_alg 必须为 sr25519');
  }
  return {
    pubkey: require0xHex(b, 'pubkey'),
    sig_alg: 'sr25519',
    signature: require0xHex(b, 'signature'),
    payload_hash: require0xHex(b, 'payload_hash'),
    signed_at: requireInt(b, 'signed_at'),
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
    throw new QrParseError(
      'user_transfer 的 name/amount/symbol/memo/bank 必须为字符串',
    );
  }
  return { address, name, amount, symbol, memo, bank };
}

function parseUserDuoqianBody(b: Record<string, unknown>): UserDuoqianBody {
  return {
    address: requireString(b, 'address'),
    name: requireString(b, 'name'),
    proposal_id: requireInt(b, 'proposal_id'),
  };
}

/**
 * 解析 WUMIN_QR_V1 envelope。
 * 遇到 proto 不符、kind 未知、字段缺失/类型错,一律抛 QrParseError。
 */
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
  if (data['proto'] !== WUMIN_QR_V1) {
    throw new QrParseError(`proto 必须为 ${WUMIN_QR_V1},实际: ${data['proto']}`);
  }
  const kindRaw = data['kind'];
  if (typeof kindRaw !== 'string' || !QR_KINDS.includes(kindRaw as QrKind)) {
    throw new QrParseError(`未知 kind: ${kindRaw}`);
  }
  const kind = kindRaw as QrKind;

  let id: string | undefined;
  let issuedAt: number | undefined;
  let expiresAt: number | undefined;
  if (isFixedKind(kind)) {
    if ('id' in data || 'issued_at' in data || 'expires_at' in data) {
      throw new QrParseError(`固定码 ${kind} 不应包含 id/issued_at/expires_at`);
    }
  } else {
    id = requireString(data, 'id');
    issuedAt = requireInt(data, 'issued_at');
    expiresAt = requireInt(data, 'expires_at');
  }

  const bodyRaw = data['body'];
  if (!bodyRaw || typeof bodyRaw !== 'object') {
    throw new QrParseError('缺少 body 对象');
  }
  const b = bodyRaw as Record<string, unknown>;

  // 按 kind 派发 body 解析
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let body: any;
  switch (kind) {
    case 'login_challenge':
      body = parseLoginChallengeBody(b);
      break;
    case 'login_receipt':
      body = parseLoginReceiptBody(b);
      break;
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
    case 'user_duoqian':
      body = parseUserDuoqianBody(b);
      break;
  }

  const env: QrEnvelope = {
    proto: WUMIN_QR_V1,
    kind,
    body,
  };
  if (id !== undefined) env.id = id;
  if (issuedAt !== undefined) env.issued_at = issuedAt;
  if (expiresAt !== undefined) env.expires_at = expiresAt;
  return env;
}

/**
 * 构造 envelope 的 JSON 字符串(序列化时固定码不出现时效字段)。
 */
export function serializeQrEnvelope(env: QrEnvelope): string {
  const out: Record<string, unknown> = {
    proto: WUMIN_QR_V1,
    kind: env.kind,
  };
  if (!isFixedKind(env.kind)) {
    if (env.id === undefined || env.issued_at === undefined || env.expires_at === undefined) {
      throw new QrParseError(`临时码 ${env.kind} 必须提供 id/issued_at/expires_at`);
    }
    out['id'] = env.id;
    out['issued_at'] = env.issued_at;
    out['expires_at'] = env.expires_at;
  }
  out['body'] = env.body;
  return JSON.stringify(out);
}

/**
 * 签名原文拼接,与 Dart/Rust 逐字节一致:
 *   WUMIN_QR_V1|<kind>|<id>|<system 或空>|<expires_at 或 0>|<principal>
 */
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
  return `${WUMIN_QR_V1}|${args.kind}|${args.id}|${sys}|${exp}|${pp}`;
}
