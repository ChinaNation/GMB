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
  streamDetailsToAssetUpdate,
  uploadImageAsset,
} from '../media/cloudflare_assets';
import { buildObjectKeyPlan } from '../storage/r2_keys';
import { assertManifestHash, assertPostCategory, estimateUploadBytes, validateUploadItems } from './validation';
import {
  assertContentFormat,
  assertDeclaredContentQuota,
  assertDeclaredLength,
  assertIdentityCanPublishCategory,
  assertManifestQuota
} from './quota';
import { fetchChainIdentityState } from '../chain/identity';
import { imageResource, resourceLimit, videoResource, type ResourceKey } from '../limits/catalog';
import { apiRouteUrl, readLimitedBytes, readLimitedText } from '../limits/request';
import { assertDeclaredResource, validateUploadBytes } from '../limits/upload';
import { putR2Object } from '../limits/storage';
import {
  consumeUploadUsage,
  releaseUploadReservation,
  reserveUploadUsage,
} from '../limits/usage';

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
  accountId: string;
  manifestHash: string;
}): Promise<string> {
  // 回执必须在 prepare 阶段固定下来，App 才能先把同一回执写入链上发布索引。
  return `sqr_${await sha256Hex(
    `${input.uploadId}:${input.postId}:${input.accountId}:${input.manifestHash}`
  )}`;
}

async function getPreparedUpload(env: Env, uploadId: string): Promise<PreparedUploadRow> {
  const upload = await env.DB.prepare(
    `SELECT upload_id, post_id, account_id, post_category, manifest_hash, content_hash,
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

  // 发帖分类权限按身份档（竞选内容须竞选身份）；只有竞选帖才读链身份，普通帖免 RPC。
  if (postCategory === 'campaign') {
    const identity = await fetchChainIdentityState(env, session.account_id);
    assertIdentityCanPublishCategory(identity.identity_level, postCategory);
  }
  // 会员权益和统一资源表共同约束声明；客户端提供的数据只用于申请，不是最终凭据。
  const membership = await requireActiveMembership(env, session.account_id);
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
  const uploadId = createId('squ');
  const postId = createId('sqp');
  const hasVideo = mediaItems.some((item) => item.media_kind === 'video');
  const expiresSeconds = hasVideo ? 3600 : parsePositiveInt(env.UPLOAD_TTL_SECONDS, 900);
  const expiresAt = secondsFromNow(expiresSeconds);
  const objectKeyPlan = buildObjectKeyPlan(session.account_id, postId);
  const storageReceiptId = await createStorageReceiptId({
    uploadId,
    postId,
    accountId: session.account_id,
    manifestHash
  });

  const manifestUploadUrl = apiRouteUrl(request, '/v1/square/uploads/manifest', { upload_id: uploadId });
  const mediaResources = mediaItems.map((item, index) => mediaResource(
    membershipLevel,
    contentFormat,
    item,
    index,
  ));
  mediaItems.forEach((item, index) => assertDeclaredResource({
    resource_key: mediaResources[index]!,
    byte_size: item.byte_size,
    content_type: item.content_type,
    duration_seconds: item.duration_seconds,
  }));

  await reserveUploadUsage({
    env,
    upload_id: uploadId,
    account_id: session.account_id,
    membership_level: membershipLevel,
    membership,
    byte_size: estimatedBytes,
    image_count: imageCount,
    video_seconds: videoSeconds,
    expires_at: expiresAt,
  });

  let mediaUploads: Awaited<ReturnType<typeof createProviderUpload>>[] = [];
  try {
    mediaUploads = await Promise.all(mediaItems.map((item, index) => createProviderUpload(env, {
      accountId: session.account_id,
      uploadId,
      postId,
      mediaIndex: index,
      mediaKind: item.media_kind,
      contentType: item.content_type,
      byteSize: item.byte_size,
      maxDurationSeconds: item.media_kind === 'video'
        ? Math.min(plan.dynamic.max_video_seconds, (item.duration_seconds ?? 1) + 5)
        : 1,
      workerUploadUrl: apiRouteUrl(request, '/v1/square/uploads/media', {
        upload_id: uploadId,
        media_index: String(index),
      }),
    })));

    const createdAt = nowMs();
    await env.DB.batch([
    env.DB.prepare(
      `INSERT INTO square_uploads
        (upload_id, post_id, account_id, post_category, manifest_hash, content_hash,
          storage_receipt_id, estimated_bytes, object_keys_json, status, expires_at, created_at, completed_at)
        VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, 'prepared', ?, ?, NULL)`
    ).bind(
      uploadId,
      postId,
      session.account_id,
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
          (upload_id, post_id, account_id, media_index, media_kind, provider,
            provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
            declared_duration_seconds, duration_seconds, width, height, error_code,
            created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, ?, ?, NULL,
            'live', NULL, NULL)`
      ).bind(
        uploadId,
        postId,
        session.account_id,
        index,
        item.media_kind === 'video' ? 'video' : 'image',
        asset.provider,
        asset.provider_asset_id,
        asset.upload_method,
        mediaResources[index],
        item.content_type,
        item.byte_size,
        asset.asset_state,
        item.media_kind === 'video' ? item.duration_seconds ?? null : null,
        createdAt,
        createdAt
      );
      })
    ]);
  } catch (error) {
    for (const asset of mediaUploads.filter((item) => item.provider === 'cloudflare_stream')) {
      await deleteProviderAsset(env, asset).catch(() => undefined);
    }
    await releaseUploadReservation(env, uploadId);
    throw error;
  }

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
      resource_key: mediaResources[index],
      asset_state: mediaUploads[index].asset_state,
      upload_url: mediaUploads[index].upload_url
    }))
  });
}

/** manifest 只通过 Worker 写 R2，真实字节与 prepare 哈希必须完全一致。 */
export async function putManifest(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const requestUrl = new URL(request.url);
  const uploadId = requestUrl.searchParams.get('upload_id');
  if (!uploadId) throw new HttpError(400, 'invalid_upload_id', '上传任务编号不合法');

  const upload = await getPreparedUpload(env, uploadId);
  const objectKeys = parseObjectKeys(upload);
  const objectKey = objectKeys.find((key) => key.endsWith('/manifest.json'));
  if (upload.account_id !== session.account_id || !objectKey) {
    throw new HttpError(403, 'upload_object_forbidden', '无权写入该上传对象');
  }
  const bytes = await readLimitedBytes(request, 'square_manifest', true);
  const ticket = await validateUploadBytes({
    resource_key: 'square_manifest',
    bytes,
    content_type: request.headers.get('content-type') ?? '',
    expected_hash: upload.manifest_hash,
  });
  await putR2Object(env, objectKey, bytes, ticket);
  return jsonResponse({
    ok: true,
    object_key: objectKey,
    byte_size: ticket.byte_size,
  });
}

/** 图片正文经 Worker 校验后写 Images；视频正文只允许走 prepare 签发的 Stream tus。 */
export async function putMediaAsset(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const requestUrl = new URL(request.url);
  const uploadId = requestUrl.searchParams.get('upload_id');
  const mediaIndex = Number.parseInt(requestUrl.searchParams.get('media_index') ?? '', 10);
  if (!uploadId || !Number.isInteger(mediaIndex) || mediaIndex < 0) {
    throw new HttpError(400, 'invalid_media_upload', '媒体上传请求缺少 upload_id 或 media_index');
  }

  const asset = await loadMediaAsset(env, uploadId, mediaIndex);
  if (asset.account_id !== session.account_id) {
    throw new HttpError(403, 'upload_media_forbidden', '无权写入该媒体资产');
  }
  if (asset.media_kind !== 'image' || asset.upload_method !== 'worker') {
    throw new HttpError(405, 'video_worker_upload_forbidden', '视频必须使用 Stream tus 直传');
  }
  const bytes = await readLimitedBytes(request, asset.resource_key as ResourceKey, true);
  const ticket = await validateUploadBytes({
    resource_key: asset.resource_key as ResourceKey,
    bytes,
    content_type: request.headers.get('content-type') ?? '',
    expected_bytes: asset.byte_size,
  });
  const updatedAt = nowMs();
  const locked = await env.DB.prepare(
    `UPDATE square_media_assets SET asset_state = 'uploaded', updated_at = ?
      WHERE upload_id = ? AND media_index = ? AND asset_state = 'prepared'`
  ).bind(updatedAt, uploadId, mediaIndex).run();
  if ((locked.meta?.changes ?? 0) !== 1) {
    throw new HttpError(409, 'media_upload_already_started', '媒体上传已开始或已完成');
  }
  let uploaded: { provider_asset_id: string };
  try {
    uploaded = await uploadImageAsset(env, {
      accountId: asset.account_id,
      uploadId: asset.upload_id,
      postId: asset.post_id,
      mediaIndex: asset.media_index,
      mediaKind: 'image',
      contentType: asset.content_type,
      byteSize: asset.byte_size,
      maxDurationSeconds: 1,
      workerUploadUrl: request.url,
    }, bytes, ticket);
  } catch (error) {
    await env.DB.prepare(
      `UPDATE square_media_assets SET asset_state = 'prepared', updated_at = ?
        WHERE upload_id = ? AND media_index = ? AND asset_state = 'uploaded'`
    ).bind(nowMs(), uploadId, mediaIndex).run();
    throw error;
  }
  await env.DB.prepare(
    `UPDATE square_media_assets
      SET provider_asset_id = ?, asset_state = 'ready', width = ?, height = ?,
        updated_at = ?, ready_at = ? WHERE upload_id = ? AND media_index = ? AND asset_state = 'uploaded'`
  )
    .bind(uploaded.provider_asset_id, ticket.width, ticket.height, updatedAt, updatedAt, uploadId, mediaIndex)
    .run();

  return jsonResponse({
    ok: true,
    upload_id: uploadId,
    media_index: mediaIndex,
    provider_asset_id: uploaded.provider_asset_id,
    asset_state: 'ready',
    byte_size: ticket.byte_size,
    width: ticket.width,
    height: ticket.height,
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
  if (upload.account_id !== session.account_id) {
    throw new HttpError(403, 'upload_account_mismatch', '无权完成该上传任务');
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
  if (mediaAssets.some((asset) => asset.asset_state === 'uploaded')) {
    throw new HttpError(409, 'media_upload_in_progress', '媒体资产仍在上传');
  }
  const manifestObject = await env.SQUARE_MEDIA.get(objectKeys[0]);
  if (!manifestObject) {
    throw new HttpError(409, 'manifest_missing', 'manifest 对象未上传');
  }
  if (manifestObject.size > resourceLimit('square_manifest').max_bytes) {
    await manifestObject.body.cancel();
    throw new HttpError(409, 'manifest_stored_too_large', 'R2 manifest 超过服务端上限');
  }
  const manifestText = await manifestObject.text();
  const manifestObjectHash = await sha256Hex(manifestText);
  if (manifestObjectHash !== manifestHash) {
    throw new HttpError(409, 'manifest_object_hash_mismatch', 'R2 manifest 内容与 manifest_hash 不一致');
  }
  const membership = await requireActiveMembership(env, upload.account_id);
  const membershipLevel = normalizeMembershipLevel(membership.membership_level);
  await assertManifestQuota({
    membershipLevel,
    plan: membershipPlan(membershipLevel),
    upload,
    manifestText,
    mediaAssets
  });

  const refreshedAssets = await Promise.all(mediaAssets.map((asset) => refreshMediaAsset(env, asset)));
  await assertActualVideoLimits(env, refreshedAssets);
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

  await consumeUploadUsage(env, upload.upload_id, refreshedAssets, contentHash, completedAt);

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
  const rawBody = await readLimitedText(request, 'stream_webhook');
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
  const asset = await loadMediaAssetByProvider(env, uid);
  let update = streamDetailsToAssetUpdate(env, body, uid);
  if (asset && update.asset_state === 'ready') {
    const errorCode = videoLimitError({ ...asset, ...update });
    if (errorCode) {
      await deleteProviderAsset(env, asset);
      update = { ...update, asset_state: 'error', error_code: errorCode, ready_at: null };
    }
  }
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
  // 会员三档（ADR-036，与身份解耦）：spark/democracy 原样，其余归 freedom。
  if (value === 'spark' || value === 'democracy') {
    return value;
  }
  return 'freedom';
}

function mediaResource(
  level: MembershipLevel,
  contentFormat: 'normal' | 'article',
  item: UploadItemInput,
  index: number,
): ResourceKey {
  if (item.media_kind === 'video') return videoResource(level);
  return imageResource(level, contentFormat === 'article' && index === 0);
}

async function loadMediaAsset(env: Env, uploadId: string, mediaIndex: number): Promise<MediaAssetRow> {
  const asset = await env.DB.prepare(
    `SELECT upload_id, post_id, account_id, media_index, media_kind, provider,
        provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
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
    `SELECT upload_id, post_id, account_id, media_index, media_kind, provider,
        provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
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

async function loadMediaAssetByProvider(env: Env, providerAssetId: string): Promise<MediaAssetRow | null> {
  return env.DB.prepare(
    `SELECT upload_id, post_id, account_id, media_index, media_kind, provider,
      provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
      declared_duration_seconds, duration_seconds, width, height, error_code,
      created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key
      FROM square_media_assets WHERE provider = 'cloudflare_stream' AND provider_asset_id = ?`
  ).bind(providerAssetId).first<MediaAssetRow>();
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

async function assertActualVideoLimits(
  env: Env,
  assets: MediaAssetRow[],
): Promise<void> {
  for (const asset of assets) {
    if (asset.media_kind !== 'video' || asset.duration_seconds === null) continue;
    const errorCode = videoLimitError(asset);
    if (!errorCode) continue;
    await deleteProviderAsset(env, asset);
    await env.DB.prepare(
      `UPDATE square_media_assets SET asset_state = 'error', error_code = ?,
        updated_at = ? WHERE upload_id = ? AND media_index = ?`
    ).bind(errorCode, nowMs(), asset.upload_id, asset.media_index).run();
    throw new HttpError(
      409,
      errorCode,
      errorCode === 'video_dimensions_exceeded'
        ? '视频真实分辨率超过会员清晰度上限'
        : '视频真实时长超过申报或会员上限',
    );
  }
}

function videoLimitError(asset: MediaAssetRow): string | null {
  const limit = resourceLimit(asset.resource_key as ResourceKey);
  const declared = asset.declared_duration_seconds ?? 0;
  if (asset.duration_seconds === null || asset.duration_seconds > (limit.max_seconds ?? 0) ||
      asset.duration_seconds > declared + 5) {
    return 'video_duration_exceeded';
  }
  if (asset.width === null || asset.height === null ||
      asset.width > (limit.max_width ?? 0) || asset.height > (limit.max_height ?? 0)) {
    return 'video_dimensions_exceeded';
  }
  return null;
}

/** 定时硬删除未完成的上传、R2 manifest 和提供商草稿，释放 Stream 预占分钟。 */
export async function cleanupExpiredUploads(env: Env): Promise<{ deleted: number }> {
  const result = await env.DB.prepare(
    `SELECT upload_id, post_id, account_id, post_category, manifest_hash, content_hash,
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
      env.DB.prepare("DELETE FROM square_uploads WHERE upload_id = ? AND status = 'prepared'").bind(upload.upload_id),
      env.DB.prepare(
        "DELETE FROM resource_reservations WHERE reservation_id = ? AND reservation_state = 'reserved'"
      ).bind(upload.upload_id),
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
