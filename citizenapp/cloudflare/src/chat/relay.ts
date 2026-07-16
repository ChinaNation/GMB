import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { getMembership, subscriptionIsActive } from '../membership/service';

// 大媒体(>100MB)Cloudflare R2 瞬时中转。硬约束:
// - **只有 >100MB(≤5GB)走此路**,其余一切(文本/贴纸/缩略图/≤100MB 媒体)绝不经 R2。
// - **只有薪火会员**能上传(下界 100MB 使非薪火档天然进不来,服务端再验一道)。
// - 存的是**客户端 AES-256-GCM 密文**,服务端拿不到内容密钥,E2E 不破。
// - 拉取后 1:1 一人 ack 即删;群依赖桶生命周期 24h TTL 兜底(外部配置)。

const MiB = 1024 * 1024;
const RELAY_MIN_BYTES = 100 * MiB; // >100MB 下界(≤100MB 一律拒,保证只有大文件进 R2)
const RELAY_MAX_PLAINTEXT = 5120 * MiB; // 薪火档单文件上限 5GB
const RELAY_MAX_CIPHERTEXT = RELAY_MAX_PLAINTEXT + 64 * MiB; // 分块 tag/帧头余量
const RELAY_TTL_MS = 24 * 60 * 60 * 1000;
const OBJECT_PREFIX = 'chat-relay/';

interface InitRelayBody {
  byte_size?: unknown;
  recipient_count?: unknown;
}

/// 收件人 ack 计数键(KV):归零即删对象。1:1 一人 ack 即删,群等全员 ack。
function countKey(objectKey: string): string {
  return `relay:count:${objectKey}`;
}

function requireRelayBucket(env: Env): R2Bucket {
  const bucket = env.CHAT_RELAY;
  if (!bucket) {
    throw new HttpError(503, 'relay_unavailable', '大媒体中转未配置');
  }
  return bucket;
}

/// 仅薪火会员(有效订阅)可上传大文件。
async function requireSpark(env: Env, ownerAccount: string): Promise<void> {
  const membership = await getMembership(env, ownerAccount);
  if (
    !membership ||
    !subscriptionIsActive(membership) ||
    membership.membership_level !== 'spark'
  ) {
    throw new HttpError(403, 'relay_requires_spark', '只有薪火会员可发送大文件');
  }
}

/// 明文尺寸门:必须 >100MB 且 ≤5GB。
function assertRelaySize(byteSize: number): void {
  if (!Number.isFinite(byteSize) || byteSize <= RELAY_MIN_BYTES) {
    throw new HttpError(400, 'relay_size_too_small', '只有大于 100MB 的文件走中转');
  }
  if (byteSize > RELAY_MAX_PLAINTEXT) {
    throw new HttpError(413, 'relay_size_too_large', '文件超过 5GB 上限');
  }
}

function assertObjectKey(raw: string): string {
  const key = decodeURIComponent(raw);
  if (!key.startsWith(OBJECT_PREFIX) || key.includes('..')) {
    throw new HttpError(400, 'invalid_relay_key', '中转对象键非法');
  }
  return key;
}

/// POST /v1/chat/relay/init —— 薪火 + 尺寸门,返回随机对象键。
export async function initChatRelay(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  requireRelayBucket(env);
  await requireSpark(env, session.owner_account);
  const body = await readJson<InitRelayBody>(request);
  const byteSize =
    typeof body.byte_size === 'number' ? body.byte_size : Number(body.byte_size);
  assertRelaySize(byteSize);
  const recipientCount = Math.max(1, Math.floor(Number(body.recipient_count ?? 1)) || 1);
  const objectKey = `${OBJECT_PREFIX}${crypto.randomUUID()}`;
  // 记录待 ack 收件人数;归零(全员拉完)或 24h TTL 先到者删。
  await env.SQUARE_CACHE.put(countKey(objectKey), String(recipientCount), {
    expirationTtl: Math.ceil(RELAY_TTL_MS / 1000),
  });
  return jsonResponse({ ok: true, object_key: objectKey, ttl_millis: RELAY_TTL_MS });
}

/// PUT /v1/chat/relay/:key/blob —— 薪火门 + 密文尺寸上界,流式写 R2(不进内存)。
export async function putChatRelayBlob(
  request: Request,
  env: Env,
  objectKeyRaw: string
): Promise<Response> {
  const session = await requireSession(request, env);
  const bucket = requireRelayBucket(env);
  await requireSpark(env, session.owner_account);
  const objectKey = assertObjectKey(objectKeyRaw);
  if (!request.body) {
    throw new HttpError(400, 'relay_body_missing', '缺少上传内容');
  }
  const contentLength = Number(request.headers.get('content-length') ?? '0');
  if (contentLength > RELAY_MAX_CIPHERTEXT) {
    throw new HttpError(413, 'relay_blob_too_large', '中转密文超过上限');
  }
  await bucket.put(objectKey, request.body, {
    httpMetadata: { contentType: 'application/octet-stream' },
    customMetadata: {
      owner: session.owner_account,
      expires_at: String(Date.now() + RELAY_TTL_MS),
    },
  });
  return jsonResponse({ ok: true });
}

/// GET /v1/chat/relay/:key/blob —— 会话鉴权,流式读 R2(内容为 E2E 密文)。
export async function getChatRelayBlob(
  request: Request,
  env: Env,
  objectKeyRaw: string
): Promise<Response> {
  await requireSession(request, env);
  const bucket = requireRelayBucket(env);
  const objectKey = assertObjectKey(objectKeyRaw);
  const object = await bucket.get(objectKey);
  if (!object) {
    throw new HttpError(404, 'relay_object_gone', '中转文件已过期或被删除');
  }
  return new Response(object.body, {
    status: 200,
    headers: { 'content-type': 'application/octet-stream' },
  });
}

/// POST /v1/chat/relay/:key/ack —— 拉取确认:1:1 一人 ack 即删(群依赖 TTL 兜底)。
export async function ackChatRelay(
  request: Request,
  env: Env,
  objectKeyRaw: string
): Promise<Response> {
  await requireSession(request, env);
  const bucket = requireRelayBucket(env);
  const objectKey = assertObjectKey(objectKeyRaw);
  // 递减待 ack 计数;归零则删 R2 + KV(1:1 一人即删,群等全员)。KV 非原子,
  // 竞态最坏是稍早/稍晚删,24h TTL 兜底。
  const remainingRaw = await env.SQUARE_CACHE.get(countKey(objectKey));
  const remaining = (remainingRaw === null ? 1 : Number(remainingRaw) || 1) - 1;
  if (remaining <= 0) {
    await bucket.delete(objectKey);
    await env.SQUARE_CACHE.delete(countKey(objectKey));
  } else {
    await env.SQUARE_CACHE.put(countKey(objectKey), String(remaining), {
      expirationTtl: Math.ceil(RELAY_TTL_MS / 1000),
    });
  }
  return jsonResponse({ ok: true });
}
