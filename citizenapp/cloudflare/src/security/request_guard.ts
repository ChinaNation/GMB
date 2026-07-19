import type { Env, SessionState } from '../types';
import { verifyP256Signature } from '../auth/device_subkey';
import { HttpError, requireSession } from '../shared/http';
import { sha256Hex } from '../shared/hash';
import {
  OP_SIGN_SQUARE_LOGIN,
  scaleString,
  signingMessage
} from '../shared/signing_message';
import { nowMs } from '../shared/time';
import { assertRequestBodyLimit, readLimitedBytes } from '../limits/request';

const REQUEST_TIME_HEADER = 'x-device-time';
const REQUEST_NONCE_HEADER = 'x-device-nonce';
const REQUEST_SIGNATURE_HEADER = 'x-device-signature';
const REQUEST_MAX_SKEW_MS = 5 * 60 * 1000;
const REQUEST_NONCE_TTL_MS = 48 * 60 * 60 * 1000;
const DEFAULT_WEB_ORIGIN = 'https://www.crcfrcn.com';

interface RateWindowRow {
  request_count: number;
  expires_at: number;
}

/**
 * `/api` 与 `/api-staging` 只是同域部署前缀，业务路由始终使用唯一 `/v1` 契约。
 */
export function normalizeApiPath(pathname: string): string {
  for (const prefix of ['/api-staging', '/api']) {
    if (pathname === prefix) return '/';
    if (pathname.startsWith(`${prefix}/`)) return pathname.slice(prefix.length);
  }
  return pathname;
}

/** 浏览器只允许官网同源；原生 App 没有 Origin，后续由设备证明校验。 */
export function assertAllowedOrigin(request: Request, env: Env): void {
  const origin = request.headers.get('origin');
  if (!origin) return;
  if (!allowedOrigins(env).has(origin)) {
    throw new HttpError(403, 'origin_forbidden', '请求来源不受信任');
  }
}

export function applyCors(request: Request, env: Env, response: Response): Response {
  const origin = request.headers.get('origin');
  if (!origin) return response;
  // 被 guard 拒绝的来源仍需返回原始 403，不能在错误响应阶段再次抛异常变成 500。
  if (!allowedOrigins(env).has(origin)) return response;
  const next = new Response(response.body, response);
  next.headers.set('access-control-allow-origin', origin);
  next.headers.set('access-control-allow-methods', 'GET,POST,PUT,PATCH,DELETE,OPTIONS');
  next.headers.set(
    'access-control-allow-headers',
    'authorization,content-type,x-device-time,x-device-nonce,x-device-signature'
  );
  next.headers.set('access-control-max-age', '600');
  next.headers.append('vary', 'origin');
  return next;
}

function allowedOrigins(env: Env): Set<string> {
  return new Set(
    (env.WEB_ORIGIN ?? DEFAULT_WEB_ORIGIN)
      .split(',')
      .map((value) => value.trim())
      .filter(Boolean)
  );
}

/**
 * 统一入口风控：预登录按 IP 粗限流，登录后按钱包精确限流；写接口和计费型读取
 * 必须提供 P-256 设备证明。Stream webhook 使用各自签名，不重复套设备证明。
 */
export async function guardRequest(request: Request, env: Env, path: string): Promise<void> {
  assertAllowedOrigin(request, env);
  assertRequestBodyLimit(request, path);

  const ipKey = await requestIpKey(request, env);
  if (path === '/v1/square/auth/challenge' || path === '/v1/square/auth/session') {
    await enforceRateLimit(env, `auth:${ipKey}`, 10, 60);
    return;
  }
  if (
    isWebhook(path) ||
    path === '/health' ||
    path === '/v1/chain/bootstrap' ||
    path === '/v1/constitution'
  ) {
    // 宪法公开只读，无会话门禁；重复访问由 KV 短缓存 + 边缘缓存兜住，不做逐请求限流。
    return;
  }
  // relay 已在模块内按 IP 哈希做原子限流，交易本体也已由钱包签名；避免双重计数。
  if (path === '/v1/chain/extrinsics/relay') return;
  // 结算子接口只给本地部署控制台调用，handler 内用 TOPUP_SETTLE_TOKEN 鉴权，
  // 不套 IP 限流（避免控制台批量补发被节流）。
  if (path.startsWith('/v1/square/topup/settlement/')) return;
  // 充值(topup)不挂广场会话：钱包功能独立于广场登录，正确性来自链上真实到账。
  // 仅按 IP 粗限流，防刷 EVM RPC。
  if (path.startsWith('/v1/square/topup/')) {
    await enforceRateLimit(env, `topup:${ipKey}`, 60, 60);
    return;
  }

  const session = await sessionOrNull(request, env);
  const rateKey = session ? `owner:${session.owner_account}` : `ip:${ipKey}`;
  const rate = routeRate(path, request.method);
  await enforceRateLimit(env, `${rate.key}:${rateKey}`, rate.limit, rate.seconds);

  if (session && requiresDeviceProof(path, request.method)) {
    await requireDeviceProof(request, env, path, session);
  }
}

async function sessionOrNull(request: Request, env: Env): Promise<SessionState | null> {
  if (!request.headers.get('authorization')?.startsWith('Bearer ')) return null;
  return requireSession(request, env);
}

function isWebhook(path: string): boolean {
  return path === '/v1/square/uploads/stream/webhook';
}

function requiresDeviceProof(path: string, method: string): boolean {
  if (path.startsWith('/v1/square/auth/')) return false;
  if (path.startsWith('/v1/square/account/delete')) return false;
  // 这些回执对应的链上业务已经由账户签名并 finalized；再次要求设备签名会让同一业务
  // 产生第二次签名。handler 仍强制校验 Bearer 会话、交易哈希和 finalized 链状态。
  if (
    method === 'POST' &&
    (path === '/v1/square/membership/confirm' ||
      path === '/v1/square/creator/subscription/confirm' ||
      path === '/v1/square/creator/plan')
  ) {
    return false;
  }
  // Image.network 只能稳定携带 Bearer header；资料媒体仍由 handler 强制校验钱包
  // session，但不要求它动态生成 P-256 请求签名。
  if (path.startsWith('/v1/square/media/')) return false;
  if (path.startsWith('/v1/chat/')) return true;
  if (path === '/v1/chain/extrinsics/relay') return true;
  return path.startsWith('/v1/square/') && method !== 'OPTIONS';
}

function routeRate(path: string, method: string): { key: string; limit: number; seconds: number } {
  if (path === '/v1/square/uploads/prepare') return { key: 'upload', limit: 30, seconds: 3600 };
  if (path === '/v1/square/contacts' && method === 'GET') {
    return { key: 'contacts_read', limit: 60, seconds: 60 };
  }
  if (path.startsWith('/v1/square/contacts/')) {
    return { key: 'contacts_write', limit: 60, seconds: 60 };
  }
  if (path === '/v1/chat/ws') return { key: 'chat_ws', limit: 12, seconds: 60 };
  if (path.startsWith('/v1/chat/')) return { key: 'chat', limit: 120, seconds: 60 };
  if (method === 'GET') return { key: 'read', limit: 120, seconds: 60 };
  return { key: 'write', limit: 30, seconds: 60 };
}

async function requireDeviceProof(
  request: Request,
  env: Env,
  path: string,
  session: SessionState
): Promise<void> {
  const requestTime = Number.parseInt(request.headers.get(REQUEST_TIME_HEADER) ?? '', 10);
  const nonce = (request.headers.get(REQUEST_NONCE_HEADER) ?? '').toLowerCase();
  const signature = request.headers.get(REQUEST_SIGNATURE_HEADER) ?? '';
  if (!Number.isSafeInteger(requestTime) || Math.abs(nowMs() - requestTime) > REQUEST_MAX_SKEW_MS) {
    throw new HttpError(401, 'device_time_invalid', '设备请求时间已过期');
  }
  if (!/^[a-f0-9]{32}$/.test(nonce)) {
    throw new HttpError(401, 'device_nonce_invalid', '设备请求 nonce 不合法');
  }

  const subkey = await env.DB.prepare(
    'SELECT p256_pubkey FROM square_device_subkeys WHERE owner_account = ?'
  ).bind(session.owner_account).first<{ p256_pubkey: string }>();
  if (!subkey) throw new HttpError(401, 'device_not_registered', '设备子钥未注册');
  const deviceKeyHash = await sha256Hex(subkey.p256_pubkey);
  if (deviceKeyHash !== session.device_key_hash) {
    throw new HttpError(401, 'device_key_changed', '设备密钥已更换，请重新登录');
  }

  const bodyHash = await requestBodyHash(request, path);
  const token = request.headers.get('authorization')!.slice('Bearer '.length).trim();
  const tokenHash = await sha256Hex(token);
  const url = new URL(request.url);
  const canonicalPath = `${path}${url.search}`;
  const canonical = [
    'square_request',
    request.method.toUpperCase(),
    canonicalPath,
    bodyHash,
    String(requestTime),
    nonce,
    tokenHash
  ].join('\n');
  const message = signingMessage(OP_SIGN_SQUARE_LOGIN, scaleString(canonical));
  if (!(await verifyP256Signature(message, signature, subkey.p256_pubkey))) {
    throw new HttpError(401, 'device_signature_invalid', '设备请求签名校验失败');
  }

  const nonceHash = await sha256Hex(`${session.owner_account}:${nonce}`);
  const inserted = await env.DB.prepare(
    `INSERT OR IGNORE INTO square_request_nonces
      (nonce_hash, owner_account, expires_at, created_at) VALUES (?, ?, ?, ?)`
  ).bind(nonceHash, session.owner_account, nowMs() + REQUEST_NONCE_TTL_MS, nowMs()).run();
  if ((inserted.meta?.changes ?? 0) !== 1) {
    throw new HttpError(409, 'device_request_replayed', '设备请求已被使用');
  }
}

async function requestBodyHash(request: Request, path: string): Promise<string> {
  if (request.method === 'GET' || request.method === 'HEAD' || request.method === 'DELETE') {
    return sha256Hex('');
  }
  assertRequestBodyLimit(request, path);
  return sha256Hex(await readLimitedBytes(request.clone()));
}

async function requestIpKey(request: Request, env: Env): Promise<string> {
  const ip = request.headers.get('cf-connecting-ip') ?? 'unknown';
  const secret = env.HASH_KEY ?? 'local-only-rate-key';
  const key = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const digest = await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(ip));
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
    .slice(0, 32);
}

export async function enforceRateLimit(
  env: Env,
  rateKey: string,
  limit: number,
  windowSeconds: number
): Promise<void> {
  const now = nowMs();
  const expiresAt = now + windowSeconds * 1000;
  const row = await env.DB.prepare(
    `INSERT INTO square_rate_windows (rate_key, request_count, expires_at)
      VALUES (?, 1, ?)
      ON CONFLICT(rate_key) DO UPDATE SET
        request_count = CASE WHEN square_rate_windows.expires_at <= ? THEN 1
          ELSE square_rate_windows.request_count + 1 END,
        expires_at = CASE WHEN square_rate_windows.expires_at <= ? THEN excluded.expires_at
          ELSE square_rate_windows.expires_at END
      RETURNING request_count, expires_at`
  ).bind(rateKey, expiresAt, now, now).first<RateWindowRow>();
  if (!row || row.request_count > limit) {
    throw new HttpError(429, 'request_rate_exceeded', '请求过于频繁，请稍后再试');
  }
}

export async function cleanupSecurityState(env: Env): Promise<void> {
  const now = nowMs();
  await env.DB.batch([
    env.DB.prepare('DELETE FROM square_request_nonces WHERE expires_at <= ?').bind(now),
    env.DB.prepare('DELETE FROM square_rate_windows WHERE expires_at <= ?').bind(now)
  ]);
}
