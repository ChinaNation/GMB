import type { Env, MediaAssetRow, PreparedUploadRow, UploadItemInput } from '../types';
import { HttpError, jsonResponse, parsePositiveInt, readJson, requireSession } from '../shared/http';
import { isSha256Hex, sha256Hex } from '../shared/hash';
import { createId } from '../shared/ids';
import { nowMs, secondsFromNow } from '../shared/time';
import { requireActiveMembership } from '../membership/service';
import { membershipPlan, type MembershipLevel } from '../membership/plans';
import {
  createProviderUpload,
  deleteProviderAsset,
  refreshProviderAssetState,
  streamDetailsToAssetUpdate
} from '../media/cloudflare_assets';
import { buildObjectKeyPlan } from '../storage/r2_keys';
import { createUploadUrl } from '../storage/presigned';
import { assertManifestHash, assertPostCategory, estimateUploadBytes, validateUploadItems } from './validation';
import {
  assertContentFormat,
  assertDeclaredContentQuota,
  assertDeclaredLength,
  assertManifestQuota
} from './quota';
import { assertUploadUsage, recordCompletedMedia } from '../security/usage';

interface PrepareUploadRequest {
  post_category?: unknown;
  content_format?: unknown;
  title_length?: unknown;
  text_length?: unknown;
  manifest_hash?: unknown;
  media_items?: unknown;
}

interface CompleteUploadRequest {
  upload_id?: unknown;
  manifest_hash?: unknown;
  content_hash?: unknown;
}

interface StreamWebhookBody {
  uid?: string;
  readyToStream?: boolean;
  thumbnail?: string;
  duration?: number;
  input?: { width?: number; height?: number };
  status?: {
    state?: string;
    errorReasonCode?: string;
    errReasonCode?: string;
  };
  playback?: {
    hls?: string;
    dash?: string;
  };
}

function parseObjectKeys(row: PreparedUploadRow): string[] {
  try {
    const parsed = JSON.parse(row.object_keys_json);
    return Array.isArray(parsed) ? parsed.filter((value) => typeof value === 'string') : [];
  } catch {
    return [];
  }
}

export async function createStorageReceiptId(input: {
  uploadId: string;
  postId: string;
  ownerAccount: string;
  manifestHash: string;
}): Promise<string> {
  // 回执必须在 prepare 阶段固定下来，App 才能先把同一回执写入链上发布索引。
  return `sqr_${await sha256Hex(
    `${input.uploadId}:${input.postId}:${input.ownerAccount}:${input.manifestHash}`
  )}`;
}

async function getPreparedUpload(env: Env, uploadId: string): Promise<PreparedUploadRow> {
  const upload = await env.DB.prepare(
    `SELECT upload_id, post_id, owner_account, post_category, manifest_hash, content_hash,
        storage_receipt_id, estimated_bytes, object_keys_json, status, expires_at, created_at, completed_at
      FROM square_uploads
      WHERE upload_id = ?`
  )
    .bind(uploadId)
    .first<PreparedUploadRow>();

  if (!upload) {
    throw new HttpError(404, 'upload_not_found', '上传任务不存在');
  }

  return upload;
}

export async function prepareUpload(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<PrepareUploadRequest>(request);
  const postCategory = assertPostCategory(body.post_category);
  const contentFormat = assertContentFormat(body.content_format);
  const titleLength = assertDeclaredLength(body.title_length, 'title_length');
  const textLength = assertDeclaredLength(body.text_length, 'text_length');
  const manifestHash = assertManifestHash(body.manifest_hash);
  const mediaItems = validateUploadItems(body.media_items);
  const estimatedBytes = estimateUploadBytes(mediaItems);

  // 会员容量在签发上传授权前扣住入口，避免用户绕过 App 直接灌入媒体对象。
  const membership = await requireActiveMembership(env, session.owner_account);
  const membershipLevel = normalizeMembershipLevel(membership.membership_level);
  const plan = membershipPlan(membershipLevel);
  assertDeclaredContentQuota({
    membershipLevel,
    plan,
    postCategory,
    contentFormat,
    titleLength,
    textLength,
    mediaItems
  });
  const imageCount = mediaItems.filter((item) => item.media_kind !== 'video').length;
  const videoSeconds = mediaItems
    .filter((item) => item.media_kind === 'video')
    .reduce((sum, item) => sum + (item.duration_seconds ?? 0), 0);
  await assertUploadUsage(env, session.owner_account, membershipLevel, imageCount, videoSeconds);

  const uploadId = createId('squ');
  const postId = createId('sqp');
  const hasTusVideo = mediaItems.some(
    (item) => item.media_kind === 'video' && item.byte_size > 200 * 1024 * 1024
  );
  const expiresSeconds = hasTusVideo ? 3600 : parsePositiveInt(env.UPLOAD_TTL_SECONDS, 900);
  const expiresAt = secondsFromNow(expiresSeconds);
  const objectKeyPlan = buildObjectKeyPlan(session.owner_account, postId);
  const requestUrl = new URL(request.url);
  const storageReceiptId = await createStorageReceiptId({
    uploadId,
    postId,
    ownerAccount: session.owner_account,
    manifestHash
  });

  const manifestUploadUrl = await createUploadUrl(env, {
    object_key: objectKeyPlan.manifest_object_key,
    content_type: 'application/json',
    expires_seconds: expiresSeconds,
    request_url: requestUrl,
      upload_id: uploadId
    });
  const mediaUploads = await Promise.all(
    mediaItems.map((item, index) =>
      createProviderUpload(env, {
        ownerAccount: session.owner_account,
        uploadId,
        postId,
        mediaIndex: index,
        mediaKind: item.media_kind,
        contentType: item.content_type,
        byteSize: item.byte_size,
        maxDurationSeconds: item.media_kind === 'video'
          ? Math.min(plan.dynamic.max_video_seconds, (item.duration_seconds ?? 1) + 5)
          : 1,
        requestOrigin: requestUrl.origin
      })
    )
  );

  const createdAt = nowMs();
  await env.DB.batch([
    env.DB.prepare(
      `INSERT INTO square_uploads
        (upload_id, post_id, owner_account, post_category, manifest_hash, content_hash,
          storage_receipt_id, estimated_bytes, object_keys_json, status, expires_at, created_at, completed_at)
        VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, 'prepared', ?, ?, NULL)`
    ).bind(
      uploadId,
      postId,
      session.owner_account,
      postCategory,
      manifestHash,
      storageReceiptId,
      estimatedBytes,
      JSON.stringify(objectKeyPlan.object_keys),
      expiresAt,
      createdAt
    ),
    ...mediaUploads.map((asset, index) => {
      const item = mediaItems[index];
      return env.DB.prepare(
        `INSERT INTO square_media_assets
          (upload_id, post_id, owner_account, media_index, media_kind, provider,
            provider_asset_id, upload_method, content_type, byte_size, asset_state,
            declared_duration_seconds, duration_seconds, width, height, error_code,
            created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, ?, ?, NULL,
            'live', NULL, NULL)`
      ).bind(
        uploadId,
        postId,
        session.owner_account,
        index,
        item.media_kind === 'video' ? 'video' : 'image',
        asset.provider,
        asset.provider_asset_id,
        asset.upload_method,
        item.content_type,
        item.byte_size,
        asset.asset_state,
        item.media_kind === 'video' ? item.duration_seconds ?? null : null,
        createdAt,
        createdAt
      );
    })
  ]);

  return jsonResponse({
    ok: true,
    upload_id: uploadId,
    post_id: postId,
    storage_receipt_id: storageReceiptId,
    expires_at: expiresAt,
    estimated_bytes: estimatedBytes,
    manifest_object_key: objectKeyPlan.manifest_object_key,
    manifest_upload_url: manifestUploadUrl,
    media_items: mediaItems.map((item, index) => ({
      media_kind: item.media_kind === 'video' ? 'video' : 'image',
      content_type: item.content_type,
      byte_size: item.byte_size,
      provider: mediaUploads[index].provider,
      provider_asset_id: mediaUploads[index].provider_asset_id,
      upload_method: mediaUploads[index].upload_method,
      asset_state: mediaUploads[index].asset_state,
      upload_url: mediaUploads[index].upload_url
    }))
  });
}

export async function devPutUploadObject(request: Request, env: Env): Promise<Response> {
  if (env.DEV_UPLOAD_PROXY !== '1') {
    throw new HttpError(404, 'dev_upload_proxy_disabled', '开发上传代理未启用');
  }

  const session = await requireSession(request, env);
  const requestUrl = new URL(request.url);
  const uploadId = requestUrl.searchParams.get('upload_id');
  const objectKey = requestUrl.searchParams.get('object_key');
  if (!uploadId || !objectKey) {
    throw new HttpError(400, 'invalid_dev_put_request', '开发上传请求缺少 upload_id 或 object_key');
  }

  const upload = await getPreparedUpload(env, uploadId);
  const objectKeys = parseObjectKeys(upload);
  if (upload.owner_account !== session.owner_account || !objectKeys.includes(objectKey)) {
    throw new HttpError(403, 'upload_object_forbidden', '无权写入该上传对象');
  }

  const contentType = request.headers.get('content-type') ?? 'application/octet-stream';
  const body = await request.arrayBuffer();
  await env.SQUARE_MEDIA.put(objectKey, body, {
    httpMetadata: {
      contentType
    }
  });

  return jsonResponse({
    ok: true,
    object_key: objectKey,
    byte_size: body.byteLength
  });
}

export async function devUploadMediaAsset(request: Request, env: Env): Promise<Response> {
  if (env.DEV_UPLOAD_PROXY !== '1') {
    throw new HttpError(404, 'dev_upload_proxy_disabled', '开发上传代理未启用');
  }

  const session = await requireSession(request, env);
  const requestUrl = new URL(request.url);
  const uploadId = requestUrl.searchParams.get('upload_id');
  const mediaIndex = Number.parseInt(requestUrl.searchParams.get('media_index') ?? '', 10);
  if (!uploadId || !Number.isInteger(mediaIndex) || mediaIndex < 0) {
    throw new HttpError(400, 'invalid_dev_media_request', '开发媒体上传请求缺少 upload_id 或 media_index');
  }

  const asset = await loadMediaAsset(env, uploadId, mediaIndex);
  if (asset.owner_account !== session.owner_account) {
    throw new HttpError(403, 'upload_media_forbidden', '无权写入该媒体资产');
  }

  // 本地 Miniflare 没有真实 Images / Stream；读取请求体后只更新 D1 状态用于端到端验收。
  await request.arrayBuffer();
  const nextState = asset.provider === 'cloudflare_stream' ? 'processing' : 'ready';
  const updatedAt = nowMs();
  await env.DB.prepare(
    `UPDATE square_media_assets
      SET asset_state = ?, updated_at = ?, ready_at = ?
      WHERE upload_id = ? AND media_index = ?`
  )
    .bind(nextState, updatedAt, nextState === 'ready' ? updatedAt : null, uploadId, mediaIndex)
    .run();

  return jsonResponse({
    ok: true,
    upload_id: uploadId,
    media_index: mediaIndex,
    asset_state: nextState
  });
}

export async function completeUpload(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<CompleteUploadRequest>(request);
  if (typeof body.upload_id !== 'string') {
    throw new HttpError(400, 'invalid_upload_id', '上传任务编号不合法');
  }
  const manifestHash = assertManifestHash(body.manifest_hash);
  if (!isSha256Hex(body.content_hash)) {
    throw new HttpError(400, 'invalid_content_hash', 'content_hash 必须是 sha256 hex');
  }
  const contentHash = body.content_hash.toLowerCase();

  const upload = await getPreparedUpload(env, body.upload_id);
  if (upload.owner_account !== session.owner_account) {
    throw new HttpError(403, 'upload_owner_mismatch', '无权完成该上传任务');
  }
  if (upload.status !== 'prepared') {
    throw new HttpError(409, 'upload_already_completed', '上传任务已完成');
  }
  if (upload.expires_at <= nowMs()) {
    throw new HttpError(410, 'upload_expired', '上传任务已过期');
  }
  if (upload.manifest_hash !== manifestHash) {
    throw new HttpError(409, 'manifest_hash_mismatch', 'manifest_hash 与准备上传时不一致');
  }
  if (contentHash !== manifestHash) {
    throw new HttpError(409, 'content_hash_mismatch', 'content_hash 必须与 manifest_hash 一致');
  }
  if (!upload.storage_receipt_id) {
    throw new HttpError(409, 'storage_receipt_missing', '上传任务缺少预生成存储回执');
  }

  const objectKeys = parseObjectKeys(upload);
  if (objectKeys.length === 0) {
    throw new HttpError(409, 'object_keys_missing', '上传对象列表异常');
  }

  for (const objectKey of objectKeys) {
    const objectMeta = await env.SQUARE_MEDIA.head(objectKey);
    if (!objectMeta) {
      throw new HttpError(409, 'object_missing', `R2 对象未上传：${objectKey}`);
    }
  }

  const mediaAssets = await loadMediaAssets(env, upload.upload_id);
  if (mediaAssets.length === 0) {
    throw new HttpError(409, 'media_assets_missing', '上传记录缺少 Cloudflare Images / Stream 媒体资产');
  }
  const manifestObject = await env.SQUARE_MEDIA.get(objectKeys[0]);
  if (!manifestObject) {
    throw new HttpError(409, 'manifest_missing', 'manifest 对象未上传');
  }
  const manifestText = await manifestObject.text();
  const manifestObjectHash = await sha256Hex(manifestText);
  if (manifestObjectHash !== manifestHash) {
    throw new HttpError(409, 'manifest_object_hash_mismatch', 'R2 manifest 内容与 manifest_hash 不一致');
  }
  const membership = await requireActiveMembership(env, upload.owner_account);
  const membershipLevel = normalizeMembershipLevel(membership.membership_level);
  await assertManifestQuota({
    membershipLevel,
    plan: membershipPlan(membershipLevel),
    upload,
    manifestText,
    mediaAssets
  });

  const refreshedAssets = await Promise.all(mediaAssets.map((asset) => refreshMediaAsset(env, asset)));
  await assertActualVideoDurations(
    env,
    refreshedAssets,
    membershipPlan(membershipLevel).dynamic.max_video_seconds
  );
  const hasProcessingMedia = refreshedAssets.some((asset) => asset.asset_state === 'processing');
  const hasPreparedMedia = refreshedAssets.some((asset) => asset.asset_state === 'prepared');
  const failedMedia = refreshedAssets.find((asset) => asset.asset_state === 'error');
  if (failedMedia) {
    throw new HttpError(409, 'media_asset_error', `媒体资产处理失败：${failedMedia.error_code ?? failedMedia.provider_asset_id}`);
  }
  if (hasPreparedMedia) {
    throw new HttpError(409, 'media_asset_not_uploaded', '媒体资产尚未上传到 Cloudflare Images / Stream');
  }

  const completedAt = nowMs();

  await env.DB.prepare(
    `UPDATE square_uploads
      SET content_hash = ?, status = 'completed', completed_at = ?
      WHERE upload_id = ?`
  )
    .bind(contentHash, completedAt, upload.upload_id)
    .run();
  await recordCompletedMedia(env, upload.owner_account, refreshedAssets);

  return jsonResponse({
    ok: true,
    upload_id: upload.upload_id,
    post_id: upload.post_id,
    content_hash: contentHash,
    storage_receipt_id: upload.storage_receipt_id,
    storage_state: hasProcessingMedia ? 'processing' : 'completed'
  });
}

export async function streamWebhookRoute(request: Request, env: Env): Promise<Response> {
  const secret = env.STREAM_HOOK_SECRET;
  if (!secret) {
    throw new HttpError(503, 'stream_webhook_not_configured', 'Cloudflare Stream webhook secret 未配置');
  }
  const rawBody = await request.text();
  await verifyStreamWebhookSignature(
    rawBody,
    request.headers.get('webhook-signature'),
    secret
  );
  const body = parseStreamWebhookBody(rawBody);
  const uid = typeof body.uid === 'string' ? body.uid : '';
  if (!uid) {
    throw new HttpError(400, 'invalid_stream_webhook', 'Cloudflare Stream webhook 缺少 uid');
  }
  const update = streamDetailsToAssetUpdate(env, body, uid);
  const result = await env.DB.prepare(
    `UPDATE square_media_assets
      SET asset_state = ?, duration_seconds = ?, width = ?, height = ?, error_code = ?,
        updated_at = ?, ready_at = ?
      WHERE provider = 'cloudflare_stream' AND provider_asset_id = ?`
  )
    .bind(
      update.asset_state,
      update.duration_seconds,
      update.width,
      update.height,
      update.error_code,
      update.updated_at ?? nowMs(),
      update.ready_at,
      uid
    )
    .run();

  // 冷归档回灌完成：restoring 资产转码就绪 → 转 live 恢复可播。
  if (update.asset_state === 'ready') {
    await env.DB.prepare(
      `UPDATE square_media_assets SET archive_state = 'live', updated_at = ?
        WHERE provider = 'cloudflare_stream' AND provider_asset_id = ? AND archive_state = 'restoring'`
    )
      .bind(nowMs(), uid)
      .run();
  }

  return jsonResponse({
    ok: true,
    action: result.meta?.changes ? 'stream_asset_updated' : 'stream_asset_ignored',
    provider_asset_id: uid,
    asset_state: update.asset_state
  });
}

export { validateUploadItems, estimateUploadBytes };

function normalizeMembershipLevel(value: string): MembershipLevel {
  // 民主会员使用对齐投票会员的高额度；竞选专属发帖仍只认 candidate 会员。
  if (value === 'candidate' || value === 'voting' || value === 'democracy') {
    return value;
  }
  return 'freedom';
}

async function loadMediaAsset(env: Env, uploadId: string, mediaIndex: number): Promise<MediaAssetRow> {
  const asset = await env.DB.prepare(
    `SELECT upload_id, post_id, owner_account, media_index, media_kind, provider,
        provider_asset_id, upload_method, content_type, byte_size, asset_state,
        declared_duration_seconds, duration_seconds, width, height, error_code,
        created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key
      FROM square_media_assets
      WHERE upload_id = ? AND media_index = ?`
  )
    .bind(uploadId, mediaIndex)
    .first<MediaAssetRow>();
  if (!asset) {
    throw new HttpError(404, 'media_asset_not_found', '媒体资产不存在');
  }
  return asset;
}

export async function loadMediaAssets(env: Env, uploadId: string): Promise<MediaAssetRow[]> {
  const result = await env.DB.prepare(
    `SELECT upload_id, post_id, owner_account, media_index, media_kind, provider,
        provider_asset_id, upload_method, content_type, byte_size, asset_state,
        declared_duration_seconds, duration_seconds, width, height, error_code,
        created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key
      FROM square_media_assets
      WHERE upload_id = ?
      ORDER BY media_index ASC`
  )
    .bind(uploadId)
    .all<MediaAssetRow>();
  return result.results ?? [];
}

async function refreshMediaAsset(env: Env, asset: MediaAssetRow): Promise<MediaAssetRow> {
  const update = await refreshProviderAssetState(env, asset);
  if (Object.keys(update).length === 0) {
    return asset;
  }
  const next: MediaAssetRow = { ...asset, ...update };
  await env.DB.prepare(
    `UPDATE square_media_assets
      SET asset_state = ?, duration_seconds = ?, width = ?, height = ?, error_code = ?,
        updated_at = ?, ready_at = ?
      WHERE upload_id = ? AND media_index = ?`
  )
    .bind(
      next.asset_state,
      next.duration_seconds,
      next.width,
      next.height,
      next.error_code,
      next.updated_at,
      next.ready_at,
      next.upload_id,
      next.media_index
    )
    .run();
  return next;
}

async function assertActualVideoDurations(
  env: Env,
  assets: MediaAssetRow[],
  planMaxSeconds: number
): Promise<void> {
  for (const asset of assets) {
    if (asset.media_kind !== 'video' || asset.duration_seconds === null) continue;
    const declared = asset.declared_duration_seconds ?? 0;
    if (asset.duration_seconds <= planMaxSeconds && asset.duration_seconds <= declared + 5) continue;
    await deleteProviderAsset(env, asset);
    await env.DB.prepare(
      `UPDATE square_media_assets SET asset_state = 'error', error_code = 'video_duration_exceeded',
        updated_at = ? WHERE upload_id = ? AND media_index = ?`
    ).bind(nowMs(), asset.upload_id, asset.media_index).run();
    throw new HttpError(409, 'video_duration_exceeded', '视频真实时长超过申报或会员上限');
  }
}

/** 定时硬删除未完成的上传、R2 manifest 和提供商草稿，释放 Stream 预占分钟。 */
export async function cleanupExpiredUploads(env: Env): Promise<{ deleted: number }> {
  const result = await env.DB.prepare(
    `SELECT upload_id, post_id, owner_account, post_category, manifest_hash, content_hash,
        storage_receipt_id, estimated_bytes, object_keys_json, status, expires_at, created_at, completed_at
      FROM square_uploads WHERE status = 'prepared' AND expires_at <= ? LIMIT 100`
  ).bind(nowMs()).all<PreparedUploadRow>();
  let deleted = 0;
  for (const upload of result.results ?? []) {
    const assets = await loadMediaAssets(env, upload.upload_id);
    for (const asset of assets) await deleteProviderAsset(env, asset);
    const objectKeys = parseObjectKeys(upload);
    if (objectKeys.length > 0) await env.SQUARE_MEDIA.delete(objectKeys);
    await env.DB.batch([
      env.DB.prepare('DELETE FROM square_media_assets WHERE upload_id = ?').bind(upload.upload_id),
      env.DB.prepare("DELETE FROM square_uploads WHERE upload_id = ? AND status = 'prepared'").bind(upload.upload_id)
    ]);
    deleted += 1;
  }
  return { deleted };
}

async function verifyStreamWebhookSignature(
  rawBody: string,
  signatureHeader: string | null,
  secret: string,
  nowSeconds = Math.floor(nowMs() / 1000),
  toleranceSeconds = 300
): Promise<void> {
  if (!signatureHeader) {
    throw new HttpError(400, 'stream_signature_missing', 'Webhook-Signature 缺失');
  }
  const parsed = parseStreamSignatureHeader(signatureHeader);
  if (!parsed.timestamp || !parsed.signature) {
    throw new HttpError(400, 'stream_signature_invalid', 'Webhook-Signature 不合法');
  }
  if (Math.abs(nowSeconds - parsed.timestamp) > toleranceSeconds) {
    throw new HttpError(400, 'stream_signature_expired', 'Webhook-Signature 已过期');
  }
  const expected = await hmacSha256Hex(secret, `${parsed.timestamp}.${rawBody}`);
  if (!timingSafeEqualHex(parsed.signature, expected)) {
    throw new HttpError(400, 'stream_signature_mismatch', 'Webhook-Signature 校验失败');
  }
}

function parseStreamSignatureHeader(header: string): { timestamp: number | null; signature: string | null } {
  let timestamp: number | null = null;
  let signature: string | null = null;
  for (const part of header.split(',')) {
    const [key, value] = part.split('=', 2);
    if (key === 'time') {
      const parsed = Number.parseInt(value ?? '', 10);
      timestamp = Number.isFinite(parsed) ? parsed : null;
    }
    if (key === 'sig1' && value) {
      signature = value;
    }
  }
  return { timestamp, signature };
}

function parseStreamWebhookBody(rawBody: string): StreamWebhookBody {
  try {
    return JSON.parse(rawBody) as StreamWebhookBody;
  } catch {
    throw new HttpError(400, 'invalid_stream_webhook_json', 'Cloudflare Stream webhook JSON 不合法');
  }
}

async function hmacSha256Hex(secret: string, payload: string): Promise<string> {
  const encoder = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(payload));
  return [...new Uint8Array(signature)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

function timingSafeEqualHex(a: string, b: string): boolean {
  if (!/^[a-f0-9]+$/i.test(a) || !/^[a-f0-9]+$/i.test(b) || a.length !== b.length) {
    return false;
  }
  let diff = 0;
  for (let index = 0; index < a.length; index += 1) {
    diff |= a.charCodeAt(index) ^ b.charCodeAt(index);
  }
  return diff === 0;
}
