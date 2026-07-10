/**
 * QR_V1 冷签信封（官网侧最小实现），逐字节对齐 CitizenApp
 * `lib/qr/envelope.dart` + `lib/qr/bodies/sign_request_body.dart` / `sign_response_body.dart`。
 *
 * 官网无私钥：生成 signRequest 二维码 → CitizenApp「扫一扫」用 owner 主钥签 0x1D →
 * 官网扫回 signResponse → 取 64B sr25519 签名 → 提交 Worker 验签。
 */

const QR_PROTOCOL = 'QR_V1'
const KIND_SIGN_REQUEST = 1
const KIND_SIGN_RESPONSE = 2
/** 广场账户动作（订阅/取消/…）= QrActions.squareAccountAction。 */
const ACTION_SQUARE_ACCOUNT = 9
/** sr25519。 */
const SIG_ALG_SR25519 = 1
/** 二维码扫描窗口（秒）。与 Worker 挑战有效期解耦，仅约束当面扫码时长。 */
const REQUEST_TTL_SECONDS = 300

export interface SignRequestInput {
  challengeId: string
  ownerPubkeyHex: string
  signingPayloadHex: string
  /** 当前 epoch 秒（默认取 Date.now），便于测试固定。 */
  nowEpochSeconds?: number
}

/** 构建 QR_V1 signRequest 信封 JSON，供二维码展示。 */
export function buildSquareActionSignRequest(input: SignRequestInput): string {
  const now = input.nowEpochSeconds ?? Math.floor(Date.now() / 1000)
  return JSON.stringify({
    p: QR_PROTOCOL,
    k: KIND_SIGN_REQUEST,
    i: input.challengeId,
    e: now + REQUEST_TTL_SECONDS,
    b: {
      a: ACTION_SQUARE_ACCOUNT,
      g: SIG_ALG_SR25519,
      u: bytesToBase64Url(hexToBytes(input.ownerPubkeyHex)),
      d: bytesToBase64Url(hexToBytes(input.signingPayloadHex)),
    },
  })
}

/** 解析扫回的 signResponse，取 64B 签名转 `0x` hex；非法一律返回 null。 */
export function parseSignResponseSignature(raw: string): string | null {
  let envelope: unknown
  try {
    envelope = JSON.parse(raw)
  } catch {
    return null
  }
  if (typeof envelope !== 'object' || envelope === null) return null
  const record = envelope as Record<string, unknown>
  if (record.p !== QR_PROTOCOL || record.k !== KIND_SIGN_RESPONSE) return null
  const body = record.b
  if (typeof body !== 'object' || body === null) return null
  const signature = (body as Record<string, unknown>).s
  if (typeof signature !== 'string') return null
  let bytes: Uint8Array
  try {
    bytes = base64UrlToBytes(signature)
  } catch {
    return null
  }
  if (bytes.length !== 64) return null
  return `0x${bytesToHex(bytes)}`
}

export function hexToBytes(hex: string): Uint8Array {
  const clean = hex.startsWith('0x') || hex.startsWith('0X') ? hex.slice(2) : hex
  if (clean.length % 2 !== 0 || /[^0-9a-fA-F]/.test(clean)) {
    throw new Error('invalid hex string')
  }
  const out = new Uint8Array(clean.length / 2)
  for (let i = 0; i < out.length; i += 1) {
    out[i] = Number.parseInt(clean.slice(i * 2, i * 2 + 2), 16)
  }
  return out
}

export function bytesToHex(bytes: Uint8Array): string {
  let hex = ''
  for (const byte of bytes) {
    hex += byte.toString(16).padStart(2, '0')
  }
  return hex
}

export function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

export function base64UrlToBytes(base64Url: string): Uint8Array {
  const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/')
  const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), '=')
  const binary = atob(padded)
  const out = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i += 1) {
    out[i] = binary.charCodeAt(i)
  }
  return out
}
