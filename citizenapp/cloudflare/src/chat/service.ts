import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { sha256Hex } from '../shared/hash';
import { nowMs } from '../shared/time';
import {
  assertBase64Url,
  base64UrlToBytes,
  assertChatAccountId,
  assertChatCidNumber,
  assertCipherSuite,
  assertDeviceId,
  assertDevicePublicKeyHex,
  assertEnvelopeId,
  assertKeyPackageId,
  assertLimit,
  assertPositiveMillis,
} from './codec';
import { buildChatDeviceBindingMessageBase64Url, verifyChatDeviceBinding } from './binding';
import { normalizeP256SignatureHex } from '../auth/device_subkey';
import { relayChatPayload, requireChatRealtimeNamespace } from './realtime';
import { sendChatWake } from './push';
import { resourceLimit } from '../limits/catalog';
import { fetchChainIdentityStateByCid } from '../chain/identity';

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
  cid_number?: unknown;
  device_id?: unknown;
  device_public_key_hex?: unknown;
  key_package_id?: unknown;
  key_package?: unknown;
  cipher_suite?: unknown;
  created_at?: unknown;
  expires_at?: unknown;
}

interface ConsumeKeyPackageRequest {
  cid_number?: unknown;
  key_package_id?: unknown;
}

interface SubmitEnvelopeRequest {
  envelope_id?: unknown;
  sender_device_id?: unknown;
  recipient_cid_number?: unknown;
  recipient_device_id?: unknown;
  envelope?: unknown;
}

interface SubmitSignalRequest {
  sender_device_id?: unknown;
  recipient_cid_number?: unknown;
  recipient_device_id?: unknown;
  signal?: unknown;
}

interface ChatDeviceRow {
  cid_number: string;
  binding_revision: number;
  account_id: string;
  device_id: string;
  device_public_key_hex: string;
  expires_at: number;
}

interface ChatKeyPackageRow {
  cid_number: string;
  binding_revision: number;
  account_id: string;
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
  // 归属身份主键 = 会话 cid_number;account_id = 登记该设备的当前绑定钱包账户(设备所有者/绑定签名主体)。
  const cidNumber = session.cid_number;
  const accountId = assertChatAccountId(session.account_id);
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
    cid_number: cidNumber,
    binding_revision: session.binding_revision,
    account_id: accountId,
    device_id: deviceId,
    device_public_key_hex: devicePublicKeyHex,
    expires_at: expiresAt,
    nonce,
  };
  // 子钥按 (cid_number, device_id) 精确定位当前请求所在设备(device_id == 会话
  // device_key_hash == sha256(p256));同一身份多设备并存时不得用别的设备公钥验签。
  const subkey = await env.DB.prepare(
    `SELECT p256_public_key FROM square_device_subkeys
      WHERE cid_number = ? AND device_id = ?
        AND binding_revision = ? AND account_id = ?`
  )
    .bind(
      cidNumber,
      session.device_key_hash,
      session.binding_revision,
      accountId,
    )
    .first<{ p256_public_key: string }>();
  if (!subkey) throw new HttpError(401, 'missing_device_subkey', '当前身份尚未登记硬件设备子钥');
  // 跨端签名文本须为 `0x`+128hex（ADR-041）；裸/大写/错长与验签失败一律 401。
  const bindingSignatureBare = normalizeP256SignatureHex(body.binding_signature);
  if (
    bindingSignatureBare === null ||
    !(await verifyChatDeviceBinding(input, bindingSignatureBare, subkey.p256_public_key))
  ) {
    throw new HttpError(401, 'invalid_device_binding_signature', 'Chat 设备绑定签名校验失败');
  }

  const createdAt = nowMs();
  const nonceHash = await sha256Hex(nonce);
  await env.DB.prepare(`DELETE FROM chat_device_binding_nonces WHERE expires_at <= ?`).bind(createdAt).run();
  try {
    await env.DB.prepare(
      `INSERT INTO chat_device_binding_nonces (cid_number, nonce_hash, expires_at, created_at)
        VALUES (?, ?, ?, ?)`,
    ).bind(cidNumber, nonceHash, expiresAt, createdAt).run();
  } catch {
    throw new HttpError(409, 'replayed_device_binding', 'Chat 设备绑定凭证已使用');
  }
  // 设备名册按 cid_number 归属;设备数量上限按身份计。account_id 记设备所有者账户。
  const deviceLimit = resourceLimit('chat_device').max_count!;
  const deviceWrite = await env.DB.prepare(
    `INSERT INTO chat_devices
      (cid_number, binding_revision, account_id, device_id, device_public_key_hex, push_provider, push_token, expires_at, created_at)
      SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?
      WHERE EXISTS (SELECT 1 FROM chat_devices WHERE cid_number = ? AND device_id = ?)
        OR (SELECT COUNT(*) FROM chat_devices WHERE cid_number = ? AND expires_at > ?) < ?
      ON CONFLICT(cid_number, device_id) DO UPDATE SET
        binding_revision = excluded.binding_revision,
        account_id = excluded.account_id,
        device_public_key_hex = excluded.device_public_key_hex,
        push_provider = excluded.push_provider,
        push_token = excluded.push_token,
        expires_at = excluded.expires_at,
        created_at = excluded.created_at`,
  ).bind(
    cidNumber, session.binding_revision, accountId, deviceId, devicePublicKeyHex,
    pushProvider, pushToken, expiresAt, createdAt,
    cidNumber, deviceId, cidNumber, createdAt, deviceLimit,
  ).run();
  if ((deviceWrite.meta?.changes ?? 0) !== 1) {
    throw new HttpError(429, 'chat_device_limit_exceeded', 'Chat 设备数量已达到上限');
  }
  return jsonResponse({
    ok: true,
    cid_number: cidNumber,
    binding_revision: session.binding_revision,
    account_id: accountId,
    device_id: deviceId,
    device_public_key_hex: devicePublicKeyHex,
    binding_message: buildChatDeviceBindingMessageBase64Url(input),
    expires_at: expiresAt,
  });
}

export async function publishChatKeyPackage(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<PublishKeyPackageRequest>(request);
  // 发布者身份 = 会话 cid_number(自发布);account_id = 会话当前绑定账户(设备所有者)。
  const cidNumber = assertChatCidNumber(body.cid_number);
  if (cidNumber !== session.cid_number) throw new HttpError(403, 'chat_cid_mismatch', '只能发布当前身份的 KeyPackage');
  const accountId = session.account_id;
  const deviceId = assertDeviceId(body.device_id);
  const publicKey = assertDevicePublicKeyHex(body.device_public_key_hex);
  await requireActiveDevice(
    env,
    cidNumber,
    deviceId,
    session.binding_revision,
    session.account_id,
    publicKey,
  );
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
        (cid_number, binding_revision, account_id, device_id, key_package_id, key_package, cipher_suite, created_at, expires_at)
        SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?
        WHERE (SELECT COUNT(*) FROM chat_keypackages
          WHERE cid_number = ? AND device_id = ? AND binding_revision = ?
            AND expires_at > ?) < ?`,
    ).bind(
      cidNumber, session.binding_revision, accountId, deviceId, keyPackageId,
      keyPackage, cipherSuite, createdAt, expiresAt,
      cidNumber, deviceId, session.binding_revision, nowMs(),
      keyPackageLimit.max_count!,
    ).run();
    if ((inserted.meta?.changes ?? 0) !== 1) {
      throw new HttpError(429, 'key_package_limit_exceeded', 'KeyPackage 数量已达到设备上限');
    }
  } catch (error) {
    if (error instanceof HttpError) throw error;
    throw new HttpError(409, 'key_package_write_rejected', 'KeyPackage 已存在或数量达到上限');
  }
  return jsonResponse({
    ok: true,
    cid_number: cidNumber,
    binding_revision: session.binding_revision,
    device_id: deviceId,
    key_package_id: keyPackageId,
    expires_at: expiresAt,
  });
}

export async function fetchChatKeyPackages(request: Request, env: Env): Promise<Response> {
  await requireSession(request, env);
  const url = new URL(request.url);
  // 路由末段 = 目标身份主键 cid_number;按 cid + device_id JOIN 设备名册。
  const cidNumber = assertChatCidNumber(url.pathname.split('/').pop());
  const binding = await requireCurrentChatBinding(env, cidNumber);
  const limit = assertLimit(url.searchParams.get('limit'), 1, 20);
  const rows = await env.DB.prepare(
    `SELECT kp.cid_number, kp.binding_revision, kp.account_id, kp.device_id,
        d.device_public_key_hex, kp.key_package_id,
        kp.key_package, kp.cipher_suite, kp.created_at, kp.expires_at
      FROM chat_keypackages kp
      JOIN chat_devices d ON d.cid_number = kp.cid_number AND d.device_id = kp.device_id
      WHERE kp.cid_number = ?
        AND kp.binding_revision = ? AND kp.account_id = ?
        AND d.binding_revision = ? AND d.account_id = ?
        AND kp.expires_at > ? AND d.expires_at > ?
      ORDER BY kp.created_at ASC LIMIT ?`,
  ).bind(
    cidNumber,
    binding.binding_revision,
    binding.account_id,
    binding.binding_revision,
    binding.account_id,
    nowMs(),
    nowMs(),
    limit,
  ).all<ChatKeyPackageRow>();
  return jsonResponse({ ok: true, cid_number: cidNumber, key_packages: rows.results ?? [] });
}

/** KeyPackage 是一次性公开材料，成功领取后立即从 D1 硬删除。 */
export async function consumeChatKeyPackage(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ConsumeKeyPackageRequest>(request);
  // 领取目标身份 = body.cid_number;领取者 = 当前会话身份(session.cid_number),无需另传。
  const cidNumber = assertChatCidNumber(body.cid_number);
  const binding = await requireCurrentChatBinding(env, cidNumber);
  const keyPackageId = assertKeyPackageId(body.key_package_id);
  const row = await env.DB.prepare(
    `SELECT kp.cid_number, kp.binding_revision, kp.account_id, kp.device_id,
        d.device_public_key_hex, kp.key_package_id,
        kp.key_package, kp.cipher_suite, kp.created_at, kp.expires_at
      FROM chat_keypackages kp
      JOIN chat_devices d ON d.cid_number = kp.cid_number AND d.device_id = kp.device_id
      WHERE kp.cid_number = ? AND kp.key_package_id = ?
        AND kp.binding_revision = ? AND kp.account_id = ?
        AND d.binding_revision = ? AND d.account_id = ?
        AND kp.expires_at > ? AND d.expires_at > ?`,
  ).bind(
    cidNumber,
    keyPackageId,
    binding.binding_revision,
    binding.account_id,
    binding.binding_revision,
    binding.account_id,
    nowMs(),
    nowMs(),
  ).first<ChatKeyPackageRow>();
  if (!row) throw new HttpError(404, 'key_package_not_available', 'KeyPackage 不存在或已被消费');
  const deleted = await env.DB.prepare(`DELETE FROM chat_keypackages WHERE key_package_id = ? AND cid_number = ?`)
    .bind(keyPackageId, cidNumber).run();
  if ((deleted.meta?.changes ?? 0) !== 1) throw new HttpError(409, 'key_package_already_consumed', 'KeyPackage 已被其他设备消费');
  return jsonResponse({ ok: true, key_package: row });
}

/** 密文只在当前请求中转；接收设备不可达时仅触发无内容唤醒。 */
export async function submitChatEnvelope(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<SubmitEnvelopeRequest>(request);
  // 发件人身份 = 会话 cid_number;收件人按身份主键 recipient_cid_number 寻址。
  const senderCidNumber = session.cid_number;
  const senderDeviceId = assertDeviceId(body.sender_device_id);
  await requireActiveDevice(
    env,
    senderCidNumber,
    senderDeviceId,
    session.binding_revision,
    session.account_id,
  );
  const recipientCidNumber = assertChatCidNumber(body.recipient_cid_number, 'invalid_recipient_cid_number');
  const recipientDeviceId = optionalDeviceId(body.recipient_device_id);
  const envelopeId = assertEnvelopeId(body.envelope_id);
  const envelope = assertBase64Url(body.envelope, 'invalid_envelope', 'Chat 密文必须是 base64url 编码');
  if (base64UrlToBytes(envelope).byteLength > resourceLimit('chat_envelope').max_bytes) {
    throw new HttpError(413, 'chat_envelope_too_large', 'Chat 密文超过服务端上限');
  }
  const sent = await relayChatPayload(env, {
    type: 'gmb_chat_envelope',
    sender_cid_number: senderCidNumber,
    recipient_cid_number: recipientCidNumber,
    recipient_device_id: recipientDeviceId,
    envelope_id: envelopeId,
    envelope,
  });
  const wakeSent = sent === 0 ? await sendChatWake(env, recipientCidNumber, senderCidNumber).catch(() => 0) : 0;
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
  // 发件人身份 = 会话 cid_number;收件人按身份主键 recipient_cid_number 寻址。
  const senderCidNumber = session.cid_number;
  const senderDeviceId = assertDeviceId(body.sender_device_id);
  await requireActiveDevice(
    env,
    senderCidNumber,
    senderDeviceId,
    session.binding_revision,
    session.account_id,
  );
  const recipientCidNumber = assertChatCidNumber(body.recipient_cid_number, 'invalid_recipient_cid_number');
  const signalText = JSON.stringify(body.signal);
  if (!body.signal || new TextEncoder().encode(signalText).byteLength > resourceLimit('chat_signal').max_bytes) {
    throw new HttpError(400, 'invalid_chat_signal', 'Chat 信令格式不合法');
  }
  const sent = await relayChatPayload(env, {
    type: 'gmb_chat_signal',
    sender_cid_number: senderCidNumber,
    recipient_cid_number: recipientCidNumber,
    recipient_device_id: optionalDeviceId(body.recipient_device_id),
    signal: body.signal,
  });
  const wakeSent = sent === 0 ? await sendChatWake(env, recipientCidNumber, senderCidNumber).catch(() => 0) : 0;
  return jsonResponse({ ok: true, delivery_state: sent > 0 ? 'sent' : 'queued', recipient_connections: sent, wake_sent: wakeSent });
}

export async function openChatWebSocket(request: Request, env: Env): Promise<Response> {
  if (request.headers.get('upgrade')?.toLowerCase() !== 'websocket') throw new HttpError(426, 'websocket_required', '请使用 WebSocket 连接');
  const session = await requireSession(request, env);
  const deviceId = assertDeviceId(request.headers.get('x-chat-device'));
  // WS 信箱按身份主键 cid_number 命名(每身份一 DO,换绑后同一 cid 同一信箱)。
  await requireActiveDevice(
    env,
    session.cid_number,
    deviceId,
    session.binding_revision,
    session.account_id,
  );
  const internal = new Request('https://chat.internal/connect', request);
  internal.headers.set('x-chat-cid-number', session.cid_number);
  internal.headers.set(
    'x-chat-binding-revision',
    String(session.binding_revision),
  );
  internal.headers.set('x-chat-account-id', session.account_id);
  internal.headers.set('x-chat-device', deviceId);
  return requireChatRealtimeNamespace(env).getByName(session.cid_number).fetch(internal);
}

async function requireActiveDevice(
  env: Env,
  cidNumber: string,
  deviceId: string,
  bindingRevision: number,
  accountId: string,
  expectedPublicKey?: string,
): Promise<ChatDeviceRow> {
  const row = await env.DB.prepare(
    `SELECT cid_number, binding_revision, account_id, device_id,
        device_public_key_hex, expires_at
      FROM chat_devices
      WHERE cid_number = ? AND device_id = ?
        AND binding_revision = ? AND account_id = ? AND expires_at > ?`,
  ).bind(
    cidNumber,
    deviceId,
    bindingRevision,
    accountId,
    nowMs(),
  ).first<ChatDeviceRow>();
  if (!row) throw new HttpError(403, 'chat_device_not_registered', 'Chat 设备未绑定或已过期');
  if (expectedPublicKey && row.device_public_key_hex !== expectedPublicKey) {
    throw new HttpError(403, 'chat_device_key_mismatch', 'Chat 设备公钥与绑定记录不一致');
  }
  return row;
}

async function requireCurrentChatBinding(env: Env, cidNumber: string) {
  const binding = await fetchChainIdentityStateByCid(env, cidNumber);
  if (
    binding.cid_number !== cidNumber
    || binding.binding_revision <= 0
    || !binding.account_id
  ) {
    throw new HttpError(404, 'chat_cid_not_bound', '目标 CID 当前没有有效绑定账户');
  }
  return binding;
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
