import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { sha256Hex } from '../shared/hash';
import { nowMs } from '../shared/time';
import {
  assertBase64Url,
  base64UrlToBytes,
  assertChatAccount,
  assertCipherSuite,
  assertDeviceId,
  assertDevicePublicKeyHex,
  assertEnvelopeId,
  assertKeyPackageId,
  assertLimit,
  assertPositiveMillis,
} from './codec';
import { buildChatDeviceBindingMessageBase64Url, verifyChatDeviceBinding } from './binding';
import { relayChatPayload, requireChatRealtimeNamespace } from './realtime';
import { sendChatWake } from './push';
import { resourceLimit } from '../limits/catalog';

type PushProvider = 'apns' | 'fcm';

interface RegisterDeviceRequest {
  device_id?: unknown;
  device_public_key_hex?: unknown;
  push_provider?: unknown;
  push_token?: unknown;
  binding_signature?: unknown;
  expires_at?: unknown;
  nonce?: unknown;
}

interface PublishKeyPackageRequest {
  owner_account?: unknown;
  device_id?: unknown;
  device_public_key_hex?: unknown;
  key_package_id?: unknown;
  key_package?: unknown;
  cipher_suite?: unknown;
  created_at?: unknown;
  expires_at?: unknown;
}

interface ConsumeKeyPackageRequest {
  owner_account?: unknown;
  key_package_id?: unknown;
  requester_account?: unknown;
}

interface SubmitEnvelopeRequest {
  envelope_id?: unknown;
  sender_device_id?: unknown;
  recipient_account?: unknown;
  recipient_device_id?: unknown;
  envelope?: unknown;
}

interface SubmitSignalRequest {
  sender_device_id?: unknown;
  recipient_account?: unknown;
  recipient_device_id?: unknown;
  signal?: unknown;
}

interface ChatDeviceRow {
  owner_account: string;
  device_id: string;
  device_public_key_hex: string;
  expires_at: number;
}

interface ChatKeyPackageRow {
  owner_account: string;
  device_id: string;
  device_public_key_hex: string;
  key_package_id: string;
  key_package: string;
  cipher_suite: string;
  created_at: number;
  expires_at: number;
}

/** 登记当前 Chat 设备和无内容推送 Token；验证后的绑定签名不落库。 */
export async function registerChatDevice(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<RegisterDeviceRequest>(request);
  const ownerAccount = assertChatAccount(session.owner_account);
  const deviceId = assertDeviceId(body.device_id);
  const devicePublicKeyHex = assertDevicePublicKeyHex(body.device_public_key_hex);
  const pushProvider = assertPushProvider(body.push_provider);
  const pushToken = assertPushToken(body.push_token);
  const expiresAt = assertPositiveMillis(body.expires_at, 'invalid_binding_expires_at', 'Chat 设备绑定过期时间不合法');
  if (expiresAt <= nowMs()) throw new HttpError(400, 'expired_device_binding', 'Chat 设备绑定凭证已过期');
  const nonce = assertNonce(body.nonce);
  if (typeof body.binding_signature !== 'string' || body.binding_signature.length === 0) {
    throw new HttpError(400, 'invalid_binding_signature', 'Chat 设备绑定签名不合法');
  }
  const input = {
    owner_account: ownerAccount,
    device_id: deviceId,
    device_public_key_hex: devicePublicKeyHex,
    expires_at: expiresAt,
    nonce,
  };
  const subkey = await env.DB.prepare(`SELECT p256_pubkey FROM square_device_subkeys WHERE owner_account = ?`)
    .bind(ownerAccount)
    .first<{ p256_pubkey: string }>();
  if (!subkey) throw new HttpError(401, 'missing_device_subkey', '当前账户尚未登记硬件设备子钥');
  if (!(await verifyChatDeviceBinding(input, body.binding_signature, subkey.p256_pubkey))) {
    throw new HttpError(401, 'invalid_device_binding_signature', 'Chat 设备绑定签名校验失败');
  }

  const createdAt = nowMs();
  const nonceHash = await sha256Hex(nonce);
  await env.DB.prepare(`DELETE FROM chat_device_binding_nonces WHERE expires_at <= ?`).bind(createdAt).run();
  try {
    await env.DB.prepare(
      `INSERT INTO chat_device_binding_nonces (owner_account, nonce_hash, expires_at, created_at)
        VALUES (?, ?, ?, ?)`,
    ).bind(ownerAccount, nonceHash, expiresAt, createdAt).run();
  } catch {
    throw new HttpError(409, 'replayed_device_binding', 'Chat 设备绑定凭证已使用');
  }
  const deviceLimit = resourceLimit('chat_device').max_count!;
  const deviceWrite = await env.DB.prepare(
    `INSERT INTO chat_devices
      (owner_account, device_id, device_public_key_hex, push_provider, push_token, expires_at, created_at)
      SELECT ?, ?, ?, ?, ?, ?, ?
      WHERE EXISTS (SELECT 1 FROM chat_devices WHERE owner_account = ? AND device_id = ?)
        OR (SELECT COUNT(*) FROM chat_devices WHERE owner_account = ? AND expires_at > ?) < ?
      ON CONFLICT(owner_account, device_id) DO UPDATE SET
        device_public_key_hex = excluded.device_public_key_hex,
        push_provider = excluded.push_provider,
        push_token = excluded.push_token,
        expires_at = excluded.expires_at,
        created_at = excluded.created_at`,
  ).bind(
    ownerAccount, deviceId, devicePublicKeyHex, pushProvider, pushToken, expiresAt, createdAt,
    ownerAccount, deviceId, ownerAccount, createdAt, deviceLimit,
  ).run();
  if ((deviceWrite.meta?.changes ?? 0) !== 1) {
    throw new HttpError(429, 'chat_device_limit_exceeded', 'Chat 设备数量已达到上限');
  }
  return jsonResponse({
    ok: true,
    owner_account: ownerAccount,
    device_id: deviceId,
    device_public_key_hex: devicePublicKeyHex,
    binding_message: buildChatDeviceBindingMessageBase64Url(input),
    expires_at: expiresAt,
  });
}

export async function publishChatKeyPackage(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<PublishKeyPackageRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  if (ownerAccount !== session.owner_account) throw new HttpError(403, 'chat_owner_mismatch', '只能发布当前钱包账户的 KeyPackage');
  const deviceId = assertDeviceId(body.device_id);
  const publicKey = assertDevicePublicKeyHex(body.device_public_key_hex);
  await requireActiveDevice(env, ownerAccount, deviceId, publicKey);
  const keyPackageId = assertKeyPackageId(body.key_package_id);
  const keyPackage = assertBase64Url(body.key_package, 'invalid_key_package', 'KeyPackage 必须是 base64url 编码');
  const cipherSuite = assertCipherSuite(body.cipher_suite);
  const createdAt = assertPositiveMillis(body.created_at, 'invalid_key_package_created_at', 'KeyPackage 创建时间不合法');
  const expiresAt = assertPositiveMillis(body.expires_at, 'invalid_key_package_expires_at', 'KeyPackage 过期时间不合法');
  const keyPackageLimit = resourceLimit('chat_keypackage');
  if (base64UrlToBytes(keyPackage).byteLength > keyPackageLimit.max_bytes) {
    throw new HttpError(413, 'key_package_too_large', 'KeyPackage 超过服务端上限');
  }
  if (expiresAt <= nowMs() || expiresAt <= createdAt ||
      expiresAt - createdAt > keyPackageLimit.ttl_seconds! * 1000) {
    throw new HttpError(400, 'expired_key_package', 'KeyPackage 有效期不合法');
  }
  await env.DB.prepare(`DELETE FROM chat_keypackages WHERE expires_at <= ?`).bind(nowMs()).run();
  try {
    const inserted = await env.DB.prepare(
      `INSERT INTO chat_keypackages
        (owner_account, device_id, key_package_id, key_package, cipher_suite, created_at, expires_at)
        SELECT ?, ?, ?, ?, ?, ?, ?
        WHERE (SELECT COUNT(*) FROM chat_keypackages
          WHERE owner_account = ? AND device_id = ? AND expires_at > ?) < ?`,
    ).bind(
      ownerAccount, deviceId, keyPackageId, keyPackage, cipherSuite, createdAt, expiresAt,
      ownerAccount, deviceId, nowMs(), keyPackageLimit.max_count!,
    ).run();
    if ((inserted.meta?.changes ?? 0) !== 1) {
      throw new HttpError(429, 'key_package_limit_exceeded', 'KeyPackage 数量已达到设备上限');
    }
  } catch (error) {
    if (error instanceof HttpError) throw error;
    throw new HttpError(409, 'key_package_write_rejected', 'KeyPackage 已存在或数量达到上限');
  }
  return jsonResponse({ ok: true, owner_account: ownerAccount, device_id: deviceId, key_package_id: keyPackageId, expires_at: expiresAt });
}

export async function fetchChatKeyPackages(request: Request, env: Env): Promise<Response> {
  await requireSession(request, env);
  const url = new URL(request.url);
  const ownerAccount = assertChatAccount(url.pathname.split('/').pop());
  const limit = assertLimit(url.searchParams.get('limit'), 1, 20);
  const rows = await env.DB.prepare(
    `SELECT kp.owner_account, kp.device_id, d.device_public_key_hex, kp.key_package_id,
        kp.key_package, kp.cipher_suite, kp.created_at, kp.expires_at
      FROM chat_keypackages kp
      JOIN chat_devices d ON d.owner_account = kp.owner_account AND d.device_id = kp.device_id
      WHERE kp.owner_account = ? AND kp.expires_at > ? AND d.expires_at > ?
      ORDER BY kp.created_at ASC LIMIT ?`,
  ).bind(ownerAccount, nowMs(), nowMs(), limit).all<ChatKeyPackageRow>();
  return jsonResponse({ ok: true, owner_account: ownerAccount, key_packages: rows.results ?? [] });
}

/** KeyPackage 是一次性公开材料，成功领取后立即从 D1 硬删除。 */
export async function consumeChatKeyPackage(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ConsumeKeyPackageRequest>(request);
  const ownerAccount = assertChatAccount(body.owner_account);
  const requester = assertChatAccount(body.requester_account, 'invalid_requester_account');
  if (requester !== session.owner_account) throw new HttpError(403, 'requester_mismatch', '只能以当前钱包账户消费 KeyPackage');
  const keyPackageId = assertKeyPackageId(body.key_package_id);
  const row = await env.DB.prepare(
    `SELECT kp.owner_account, kp.device_id, d.device_public_key_hex, kp.key_package_id,
        kp.key_package, kp.cipher_suite, kp.created_at, kp.expires_at
      FROM chat_keypackages kp
      JOIN chat_devices d ON d.owner_account = kp.owner_account AND d.device_id = kp.device_id
      WHERE kp.owner_account = ? AND kp.key_package_id = ? AND kp.expires_at > ? AND d.expires_at > ?`,
  ).bind(ownerAccount, keyPackageId, nowMs(), nowMs()).first<ChatKeyPackageRow>();
  if (!row) throw new HttpError(404, 'key_package_not_available', 'KeyPackage 不存在或已被消费');
  const deleted = await env.DB.prepare(`DELETE FROM chat_keypackages WHERE key_package_id = ? AND owner_account = ?`)
    .bind(keyPackageId, ownerAccount).run();
  if ((deleted.meta?.changes ?? 0) !== 1) throw new HttpError(409, 'key_package_already_consumed', 'KeyPackage 已被其他设备消费');
  return jsonResponse({ ok: true, key_package: row });
}

/** 密文只在当前请求中转；接收设备不可达时仅触发无内容唤醒。 */
export async function submitChatEnvelope(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<SubmitEnvelopeRequest>(request);
  const senderAccount = assertChatAccount(session.owner_account);
  const senderDeviceId = assertDeviceId(body.sender_device_id);
  await requireActiveDevice(env, senderAccount, senderDeviceId);
  const recipientAccount = assertChatAccount(body.recipient_account, 'invalid_recipient_account');
  const recipientDeviceId = optionalDeviceId(body.recipient_device_id);
  const envelopeId = assertEnvelopeId(body.envelope_id);
  const envelope = assertBase64Url(body.envelope, 'invalid_envelope', 'Chat 密文必须是 base64url 编码');
  if (base64UrlToBytes(envelope).byteLength > resourceLimit('chat_envelope').max_bytes) {
    throw new HttpError(413, 'chat_envelope_too_large', 'Chat 密文超过服务端上限');
  }
  const sent = await relayChatPayload(env, {
    type: 'gmb_chat_envelope_v2',
    sender_account: senderAccount,
    recipient_account: recipientAccount,
    recipient_device_id: recipientDeviceId,
    envelope_id: envelopeId,
    envelope,
  });
  const wakeSent = sent === 0 ? await sendChatWake(env, recipientAccount, senderAccount).catch(() => 0) : 0;
  return jsonResponse({
    ok: true,
    envelope_id: envelopeId,
    delivery_state: sent > 0 ? 'sent' : 'queued',
    recipient_connections: sent,
    wake_sent: wakeSent,
  });
}

/** WebRTC SDP/ICE 只做瞬时路由，不写任何 Cloudflare Storage。 */
export async function submitChatSignal(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<SubmitSignalRequest>(request);
  const senderAccount = assertChatAccount(session.owner_account);
  const senderDeviceId = assertDeviceId(body.sender_device_id);
  await requireActiveDevice(env, senderAccount, senderDeviceId);
  const recipientAccount = assertChatAccount(body.recipient_account, 'invalid_recipient_account');
  const signalText = JSON.stringify(body.signal);
  if (!body.signal || new TextEncoder().encode(signalText).byteLength > resourceLimit('chat_signal').max_bytes) {
    throw new HttpError(400, 'invalid_chat_signal', 'Chat 信令格式不合法');
  }
  const sent = await relayChatPayload(env, {
    type: 'gmb_chat_signal_v1',
    sender_account: senderAccount,
    recipient_account: recipientAccount,
    recipient_device_id: optionalDeviceId(body.recipient_device_id),
    signal: body.signal,
  });
  const wakeSent = sent === 0 ? await sendChatWake(env, recipientAccount, senderAccount).catch(() => 0) : 0;
  return jsonResponse({ ok: true, delivery_state: sent > 0 ? 'sent' : 'queued', recipient_connections: sent, wake_sent: wakeSent });
}

export async function openChatWebSocket(request: Request, env: Env): Promise<Response> {
  if (request.headers.get('upgrade')?.toLowerCase() !== 'websocket') throw new HttpError(426, 'websocket_required', '请使用 WebSocket 连接');
  const session = await requireSession(request, env);
  const deviceId = assertDeviceId(request.headers.get('x-chat-device'));
  await requireActiveDevice(env, session.owner_account, deviceId);
  const internal = new Request('https://chat.internal/connect', request);
  internal.headers.set('x-chat-owner', session.owner_account);
  internal.headers.set('x-chat-device', deviceId);
  return requireChatRealtimeNamespace(env).getByName(session.owner_account).fetch(internal);
}

async function requireActiveDevice(
  env: Env,
  ownerAccount: string,
  deviceId: string,
  expectedPublicKey?: string,
): Promise<ChatDeviceRow> {
  const row = await env.DB.prepare(
    `SELECT owner_account, device_id, device_public_key_hex, expires_at
      FROM chat_devices WHERE owner_account = ? AND device_id = ? AND expires_at > ?`,
  ).bind(ownerAccount, deviceId, nowMs()).first<ChatDeviceRow>();
  if (!row) throw new HttpError(403, 'chat_device_not_registered', 'Chat 设备未绑定或已过期');
  if (expectedPublicKey && row.device_public_key_hex !== expectedPublicKey) {
    throw new HttpError(403, 'chat_device_key_mismatch', 'Chat 设备公钥与绑定记录不一致');
  }
  return row;
}

function assertPushProvider(value: unknown): PushProvider {
  if (value === 'apns' || value === 'fcm') return value;
  throw new HttpError(400, 'invalid_push_provider', 'Chat 推送服务不合法');
}

function assertPushToken(value: unknown): string {
  if (typeof value !== 'string' || value.length < 16 || value.length > 4096) {
    throw new HttpError(400, 'invalid_push_token', 'Chat 推送 Token 不合法');
  }
  return value;
}

function assertNonce(value: unknown): string {
  if (typeof value !== 'string' || value.length < 8 || value.length > 128) {
    throw new HttpError(400, 'invalid_binding_nonce', 'Chat 设备绑定 nonce 不合法');
  }
  return value;
}

function optionalDeviceId(value: unknown): string | null {
  return typeof value === 'string' && value.length > 0 ? assertDeviceId(value) : null;
}
