// QR_V1 统一协议 TS 类型与解析器。
//
// 唯一事实源:memory/01-architecture/qr/qr-protocol-spec.md
// Golden fixtures:memory/01-architecture/qr/qr-protocol-fixtures/*.json
//
// **本文件是镜像文件,两份必须逐字节相同**:
//   citizenchain/node/frontend/shared/qr/citizenQr.ts
//   citizenchain/onchina/frontend/core/citizenQr.ts
// 由 `citizenchain/crates/qr-protocol/tests/repo_guard.rs` 强制校验。改一份必须改另一份。
//
// body 键全部单字母,与 citizenapp / citizenwallet / onchina Rust 完全一致:
//   a 动作 / g 算法 / u 公钥 / d 载荷 / s 签名 / o 换绑当前账户 / r 换绑当前账户签名
//   c cid_number / n account_id / v 金额 / t 币种 / m 备注 / l 清算行 CID

export const QR_V1 = 'QR_V1' as const;

export type QrKind =
  | 'sign_request'
  | 'sign_response'
  | 'user_contact'
  | 'user_transfer'
  | 'account_id_code';

export const QR_KIND_CODE: Record<QrKind, number> = {
  sign_request: 1,
  sign_response: 2,
  user_contact: 3,
  user_transfer: 4,
  account_id_code: 5,
};

const QR_KIND_BY_CODE = new Map<number, QrKind>(
  Object.entries(QR_KIND_CODE).map(([kind, code]) => [code, kind as QrKind]),
);

// 用户码与账户码都是固定码(无 i/e);收款码与签名请求/响应是临时码。
export const FIXED_KINDS: readonly QrKind[] = ['user_contact', 'account_id_code'];

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
  /** 换绑且当前账户可签名时,与 current_account_signature 成对出现。 */
  current_account_id?: string;
  current_account_signature?: string;
}

/** 用户码 body:身份主键 + 其当前绑定账户。不含昵称与 SS58(见 spec 第 6 节)。 */
export interface UserContactBody {
  cid_number: string;
  account_id: string;
}

/** 收款码 body:收款账户 + 金额币种备注 + 收款方清算行 CID。不含收款人姓名。 */
export interface UserTransferBody {
  account_id: string;
  amount: string;
  symbol: string;
  memo: string;
  bank: string;
}

/** 账户码 body:只声明账户。钱包没有码,账户才有码。 */
export interface AccountIdCodeBody {
  account_id: string;
}

export type QrBodyByKind = {
  sign_request: SignRequestBody;
  sign_response: SignResponseBody;
  user_contact: UserContactBody;
  user_transfer: UserTransferBody;
  account_id_code: AccountIdCodeBody;
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

/** 输入总长度上限:二维码物理容量约 3KB,留足余量后拒绝超大输入。 */
const MAX_PAYLOAD_CHARS = 32768;

/** `account_id` 唯一格式:小写 0x + 64 位十六进制。锚定正则,免疫零宽/同形字。 */
const ACCOUNT_ID_PATTERN = /^0x[0-9a-f]{64}$/;

/** CID / 清算行 CID 字符集白名单:仅 ASCII 字母数字与连字符,挡零宽字符钓鱼。 */
const CID_PATTERN = /^[A-Za-z0-9-]{1,32}$/;

/** base64url 严格字母表:无填充,只允许 A-Z a-z 0-9 - _。 */
const B64URL_PATTERN = /^[A-Za-z0-9_-]+$/;

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
  const missing = allowedKeys.filter((key) => !(key in obj));
  if (missing.length > 0) {
    throw new QrParseError(`${field} 缺少必填字段: ${missing.join(',')}`);
  }
}

function requireAccountId(value: string, field: string): string {
  if (!ACCOUNT_ID_PATTERN.test(value)) {
    throw new QrParseError(`${field} 必须为小写 0x 加 64 位十六进制`);
  }
  return value;
}

function requireCid(value: string, field: string): string {
  if (!CID_PATTERN.test(value)) {
    throw new QrParseError(`${field} 必须为 1 到 32 位字母数字与连字符`);
  }
  return value;
}

/**
 * base64url(无填充)解码为 0x 十六进制。
 *
 * 严格字母表:拒绝 `=` 填充与标准 base64 的 `+`/`/`,并做重编码回环校验,
 * 拒绝非规范末位比特。与 citizenwallet 的实现同口径 —— 松一点就会出现
 * 「同一载荷此端过、彼端拒」的跨端分歧。
 */
function b64ToHex(input: string, field: string, expectedLen?: number): string {
  if (!B64URL_PATTERN.test(input)) {
    throw new QrParseError(`字段 ${field} 必须为无填充 base64url`);
  }
  const padded = input.padEnd(
    input.length + ((4 - (input.length % 4)) % 4),
    '=',
  );
  let binary: string;
  try {
    binary = atob(padded.replace(/-/g, '+').replace(/_/g, '/'));
  } catch {
    throw new QrParseError(`字段 ${field} 必须为 base64url`);
  }
  const bytes = Array.from(binary, (ch) => ch.charCodeAt(0));
  if (bytes.length === 0) {
    throw new QrParseError(`字段 ${field} 不能为空`);
  }
  if (expectedLen !== undefined && bytes.length !== expectedLen) {
    throw new QrParseError(`字段 ${field} 必须解码为 ${expectedLen} 字节`);
  }
  // 回环校验:非规范编码(末位比特非零)重编码后与原串不等,一律拒绝。
  const reencoded = btoa(binary)
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
  if (reencoded !== input) {
    throw new QrParseError(`字段 ${field} 不是规范 base64url 编码`);
  }
  return `0x${bytes.map((b) => b.toString(16).padStart(2, '0')).join('')}`;
}

function parseSignRequestBody(b: Record<string, unknown>): SignRequestBody {
  requireExactKeys(b, ['a', 'g', 'u', 'd'], 'b');
  const action = requireInt(b, 'a');
  const sigAlg = requireInt(b, 'g');
  if (action <= 0) throw new QrParseError('b.a 必须为正整数');
  if (sigAlg !== 1) throw new QrParseError('b.g 必须为 1(sr25519)');
  return {
    action,
    sig_alg: 1,
    signer_public_key: b64ToHex(requireString(b, 'u'), 'u', 32),
    payload_hex: b64ToHex(requireString(b, 'd'), 'd'),
  };
}

function parseSignResponseBody(b: Record<string, unknown>): SignResponseBody {
  // o/r 是换绑场景的可选对:要么都在要么都不在,禁止只出现其中一个。
  const hasO = 'o' in b;
  const hasR = 'r' in b;
  if (hasO !== hasR) {
    throw new QrParseError('b.o 与 b.r 必须成对出现');
  }
  requireExactKeys(b, hasO ? ['u', 's', 'o', 'r'] : ['u', 's'], 'b');
  const out: SignResponseBody = {
    signer_public_key: b64ToHex(requireString(b, 'u'), 'u', 32),
    signature: b64ToHex(requireString(b, 's'), 's', 64),
  };
  if (hasO) {
    out.current_account_id = b64ToHex(requireString(b, 'o'), 'o', 32);
    out.current_account_signature = b64ToHex(requireString(b, 'r'), 'r', 64);
  }
  return out;
}

function parseUserContactBody(b: Record<string, unknown>): UserContactBody {
  requireExactKeys(b, ['c', 'n'], 'b');
  return {
    cid_number: requireCid(requireString(b, 'c'), 'b.c'),
    account_id: requireAccountId(requireString(b, 'n'), 'b.n'),
  };
}

function parseUserTransferBody(b: Record<string, unknown>): UserTransferBody {
  requireExactKeys(b, ['n', 'v', 't', 'm', 'l'], 'b');
  const memo = b['m'];
  if (typeof memo !== 'string') {
    throw new QrParseError('b.m 必须为字符串');
  }
  return {
    account_id: requireAccountId(requireString(b, 'n'), 'b.n'),
    amount: requireString(b, 'v'),
    symbol: requireString(b, 't'),
    memo,
    bank: requireCid(requireString(b, 'l'), 'b.l'),
  };
}

function parseAccountIdCodeBody(
  b: Record<string, unknown>,
): AccountIdCodeBody {
  requireExactKeys(b, ['n'], 'b');
  return { account_id: requireAccountId(requireString(b, 'n'), 'b.n') };
}

export function parseQrEnvelope(
  raw: string | Record<string, unknown>,
): QrEnvelope {
  let data: Record<string, unknown>;
  if (typeof raw === 'string') {
    if (raw.length > MAX_PAYLOAD_CHARS) {
      throw new QrParseError('QR 内容超长');
    }
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

  // k 必须是整数:不接受 "5" 这类等价字符串表示,与其余各端口径一致。
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
    case 'account_id_code':
      body = parseAccountIdCodeBody(b);
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
  if (!ACCOUNT_ID_PATTERN.test(`0x${pp}`)) {
    throw new QrParseError('principal 必须为 32 字节账户标识');
  }
  return `${QR_V1}|${kindCode}|${args.id}|${sys}|${exp}|${pp}`;
}
