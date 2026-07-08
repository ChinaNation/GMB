import type { Env } from '../types';
import { HttpError, jsonResponse, parsePositiveInt, readJson } from '../shared/http';
import { createId } from '../shared/ids';
import { nowMs } from '../shared/time';

export const CHAIN_EXTRINSIC_RELAY_PATH = '/v1/chain/extrinsics/relay';

const RELAY_SCHEMA = 'citizenapp.chain.extrinsic_relay.v1';
const DEFAULT_MAX_EXTRINSIC_BYTES = 64 * 1024;
const DEFAULT_MAX_PER_MINUTE = 20;
const RATE_LIMIT_WINDOW_MS = 60_000;
const DEDUPE_WINDOW_MS = 10 * 60_000;

interface RelayRequestBody {
  signed_extrinsic_hex?: unknown;
  [key: string]: unknown;
}

interface JsonRpcResponse<T> {
  result?: T;
  error?: {
    code: number;
    message: string;
  };
}

interface RecentRelayRow {
  relay_id: string;
  tx_hash: string | null;
  created_at: number;
}

/// Worker 只在显式开关开启且服务节点 RPC 已配置时提供广播兜底。
/// 这里不把 RPC URL 写入任何响应，App 只知道固定 path。
export function isChainExtrinsicRelayEnabled(env: Env): boolean {
  return env.CHAIN_EXTRINSIC_RELAY_ENABLED === '1' && Boolean(env.SQUARE_CHAIN_RPC_URL);
}

export async function relaySignedExtrinsicRoute(
  request: Request,
  env: Env
): Promise<Response> {
  if (!isChainExtrinsicRelayEnabled(env)) {
    throw new HttpError(503, 'chain_extrinsic_relay_disabled', '签名交易广播兜底未启用');
  }

  const maxBytes = parsePositiveInt(
    env.CHAIN_EXTRINSIC_RELAY_MAX_BYTES,
    DEFAULT_MAX_EXTRINSIC_BYTES
  );
  assertReasonableContentLength(request, maxBytes);

  const body = await readJson<RelayRequestBody>(request);
  assertNoPrivateMaterial(body);
  assertAllowedKeys(body);

  const signedExtrinsicHex = normalizeSignedExtrinsicHex(
    body.signed_extrinsic_hex,
    maxBytes
  );
  const extrinsicBytes = hexToBytes(signedExtrinsicHex);
  const extrinsicSha256 = await sha256Hex(extrinsicBytes);
  const createdAt = nowMs();
  const requestIpHash = await hashRequestIp(request);

  await enforceRelayRateLimit(env, requestIpHash, createdAt);

  const duplicate = await findRecentBroadcast(env, extrinsicSha256, createdAt);
  if (duplicate?.tx_hash) {
    return jsonResponse({
      ok: true,
      schema: RELAY_SCHEMA,
      relay_id: duplicate.relay_id,
      relay_status: 'broadcast',
      deduplicated: true,
      tx_hash: duplicate.tx_hash,
      accepted_at: duplicate.created_at,
      chain_success_source: 'finalized_runtime_storage_or_events'
    });
  }

  const relayId = createId('cer');
  await insertRelayAttempt(env, {
    relayId,
    extrinsicSha256,
    requestIpHash,
    byteSize: extrinsicBytes.length,
    createdAt
  });

  try {
    const txHash = await submitExtrinsicToRpc(env, signedExtrinsicHex, relayId);
    await markRelayBroadcast(env, relayId, txHash, nowMs());
    return jsonResponse(
      {
        ok: true,
        schema: RELAY_SCHEMA,
        relay_id: relayId,
        relay_status: 'broadcast',
        deduplicated: false,
        tx_hash: txHash,
        accepted_at: createdAt,
        chain_success_source: 'finalized_runtime_storage_or_events'
      },
      { status: 202 }
    );
  } catch (error) {
    const code = error instanceof HttpError ? error.code : 'chain_extrinsic_relay_failed';
    await markRelayFailed(env, relayId, code, nowMs());
    throw error;
  }
}

function assertReasonableContentLength(request: Request, maxBytes: number): void {
  const contentLength = request.headers.get('content-length');
  if (!contentLength) {
    return;
  }
  const parsed = Number.parseInt(contentLength, 10);
  // JSON 会比 extrinsic hex 稍大一些；这里先挡掉明显异常请求。
  if (Number.isFinite(parsed) && parsed > maxBytes * 2 + 512) {
    throw new HttpError(413, 'chain_extrinsic_relay_body_too_large', '签名交易请求体过大');
  }
}

function assertAllowedKeys(body: RelayRequestBody): void {
  const keys = Object.keys(body);
  if (keys.length !== 1 || keys[0] !== 'signed_extrinsic_hex') {
    throw new HttpError(400, 'chain_extrinsic_relay_invalid_fields', '签名交易广播字段不合法');
  }
}

function assertNoPrivateMaterial(value: unknown): void {
  if (Array.isArray(value)) {
    for (const item of value) {
      assertNoPrivateMaterial(item);
    }
    return;
  }
  if (!value || typeof value !== 'object') {
    return;
  }
  for (const [key, nested] of Object.entries(value)) {
    const normalized = key.toLowerCase().replaceAll(/[_-]/g, '');
    if (
      normalized === 'privatekey' ||
      normalized === 'mnemonic' ||
      normalized === 'seed' ||
      normalized === 'secret' ||
      normalized === 'keystore' ||
      normalized === 'password' ||
      normalized === 'recoveryphrase'
    ) {
      throw new HttpError(
        400,
        'chain_extrinsic_relay_private_material_rejected',
        '广播接口不得接收私钥、助记词或密钥材料'
      );
    }
    assertNoPrivateMaterial(nested);
  }
}

export function normalizeSignedExtrinsicHex(value: unknown, maxBytes: number): string {
  if (typeof value !== 'string') {
    throw new HttpError(400, 'chain_extrinsic_relay_invalid_hex', '签名交易必须是 hex 字符串');
  }
  const trimmed = value.trim().toLowerCase();
  if (!/^0x[0-9a-f]+$/.test(trimmed) || trimmed.length % 2 !== 0) {
    throw new HttpError(400, 'chain_extrinsic_relay_invalid_hex', '签名交易 hex 格式不合法');
  }
  const byteSize = (trimmed.length - 2) / 2;
  if (byteSize <= 0 || byteSize > maxBytes) {
    throw new HttpError(413, 'chain_extrinsic_relay_too_large', '签名交易大小超出限制');
  }
  return trimmed;
}

async function submitExtrinsicToRpc(
  env: Env,
  signedExtrinsicHex: string,
  relayId: string
): Promise<string> {
  const rpcUrl = env.SQUARE_CHAIN_RPC_URL;
  if (!rpcUrl) {
    throw new HttpError(503, 'chain_rpc_not_configured', '链服务节点 RPC 未配置');
  }

  const response = await fetch(rpcUrl, {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: relayId,
      method: 'author_submitExtrinsic',
      params: [signedExtrinsicHex]
    })
  });

  if (!response.ok) {
    throw new HttpError(502, 'chain_rpc_http_failed', '签名交易广播请求失败');
  }

  const data = (await response.json()) as JsonRpcResponse<string>;
  if (data.error) {
    throw new HttpError(502, 'chain_rpc_error', data.error.message);
  }
  return normalizeTxHash(data.result);
}

function normalizeTxHash(value: unknown): string {
  if (typeof value !== 'string') {
    throw new HttpError(502, 'chain_rpc_invalid_tx_hash', '链服务节点未返回交易哈希');
  }
  const txHash = value.trim().toLowerCase();
  if (!/^0x[0-9a-f]{64}$/.test(txHash)) {
    throw new HttpError(502, 'chain_rpc_invalid_tx_hash', '链服务节点返回的交易哈希不合法');
  }
  return txHash;
}

async function enforceRelayRateLimit(
  env: Env,
  requestIpHash: string,
  createdAt: number
): Promise<void> {
  const maxPerMinute = parsePositiveInt(
    env.CHAIN_EXTRINSIC_RELAY_MAX_PER_MINUTE,
    DEFAULT_MAX_PER_MINUTE
  );
  const row = await env.DB.prepare(
    `SELECT COUNT(*) AS count
      FROM chain_extrinsic_relays
      WHERE request_ip_hash = ? AND created_at >= ?`
  )
    .bind(requestIpHash, createdAt - RATE_LIMIT_WINDOW_MS)
    .first<{ count: number }>();
  if ((row?.count ?? 0) >= maxPerMinute) {
    throw new HttpError(429, 'chain_extrinsic_relay_rate_limited', '签名交易广播过于频繁');
  }
}

async function findRecentBroadcast(
  env: Env,
  extrinsicSha256: string,
  createdAt: number
): Promise<RecentRelayRow | null> {
  return await env.DB.prepare(
    `SELECT relay_id, tx_hash, created_at
      FROM chain_extrinsic_relays
      WHERE extrinsic_sha256 = ?
        AND relay_status = 'broadcast'
        AND tx_hash IS NOT NULL
        AND created_at >= ?
      ORDER BY created_at DESC
      LIMIT 1`
  )
    .bind(extrinsicSha256, createdAt - DEDUPE_WINDOW_MS)
    .first<RecentRelayRow>();
}

async function insertRelayAttempt(
  env: Env,
  input: {
    relayId: string;
    extrinsicSha256: string;
    requestIpHash: string;
    byteSize: number;
    createdAt: number;
  }
): Promise<void> {
  await env.DB.prepare(
    `INSERT INTO chain_extrinsic_relays
      (relay_id, extrinsic_sha256, tx_hash, request_ip_hash, byte_size,
       relay_status, error_code, created_at, updated_at)
      VALUES (?, ?, NULL, ?, ?, 'received', NULL, ?, ?)`
  )
    .bind(
      input.relayId,
      input.extrinsicSha256,
      input.requestIpHash,
      input.byteSize,
      input.createdAt,
      input.createdAt
    )
    .run();
}

async function markRelayBroadcast(
  env: Env,
  relayId: string,
  txHash: string,
  updatedAt: number
): Promise<void> {
  await env.DB.prepare(
    `UPDATE chain_extrinsic_relays
      SET relay_status = 'broadcast', tx_hash = ?, error_code = NULL, updated_at = ?
      WHERE relay_id = ?`
  )
    .bind(txHash, updatedAt, relayId)
    .run();
}

async function markRelayFailed(
  env: Env,
  relayId: string,
  errorCode: string,
  updatedAt: number
): Promise<void> {
  await env.DB.prepare(
    `UPDATE chain_extrinsic_relays
      SET relay_status = 'failed', error_code = ?, updated_at = ?
      WHERE relay_id = ?`
  )
    .bind(errorCode, updatedAt, relayId)
    .run();
}

function hexToBytes(hex: string): Uint8Array {
  const raw = hex.slice(2);
  const out = new Uint8Array(raw.length / 2);
  for (let i = 0; i < raw.length; i += 2) {
    out[i / 2] = Number.parseInt(raw.slice(i, i + 2), 16);
  }
  return out;
}

async function hashRequestIp(request: Request): Promise<string> {
  const raw =
    request.headers.get('cf-connecting-ip') ??
    request.headers.get('x-forwarded-for')?.split(',')[0]?.trim() ??
    'local';
  return await sha256Hex(new TextEncoder().encode(raw));
}

async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const copy = new Uint8Array(bytes.length);
  copy.set(bytes);
  const digest = await crypto.subtle.digest('SHA-256', copy.buffer);
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}
