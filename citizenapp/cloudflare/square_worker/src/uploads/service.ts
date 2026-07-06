import type { Env, PreparedUploadRow, UploadItemInput } from '../types';
import { HttpError, jsonResponse, parsePositiveInt, readJson, requireSession } from '../shared/http';
import { isSha256Hex, sha256Hex } from '../shared/hash';
import { createId } from '../shared/ids';
import { nowMs, secondsFromNow } from '../shared/time';
import { requireActiveMembership } from '../membership/service';
import { buildObjectKeyPlan } from '../storage/r2_keys';
import { createUploadUrl } from '../storage/presigned';
import { assertManifestHash, assertPostCategory, estimateUploadBytes, validateUploadItems } from './validation';

interface PrepareUploadRequest {
  post_category?: unknown;
  manifest_hash?: unknown;
  media_items?: unknown;
}

interface CompleteUploadRequest {
  upload_id?: unknown;
  manifest_hash?: unknown;
  content_hash?: unknown;
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
        storage_receipt_id, estimated_bytes, object_keys_json, status, created_at, completed_at
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
  const manifestHash = assertManifestHash(body.manifest_hash);
  const mediaItems = validateUploadItems(body.media_items);
  const estimatedBytes = estimateUploadBytes(mediaItems);

  // 会员容量在签发上传授权前扣住入口，避免用户绕过 App 直接灌入媒体对象。
  await requireActiveMembership(env, session.owner_account, estimatedBytes);

  const uploadId = createId('squ');
  const postId = createId('sqp');
  const expiresSeconds = parsePositiveInt(env.SQUARE_UPLOAD_URL_TTL_SECONDS, 900);
  const expiresAt = secondsFromNow(expiresSeconds);
  const objectKeyPlan = buildObjectKeyPlan(session.owner_account, postId, mediaItems);
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
  const uploadUrls = await Promise.all(
    objectKeyPlan.media_items.map((item) =>
      createUploadUrl(env, {
        object_key: item.object_key,
        content_type: item.content_type,
        expires_seconds: expiresSeconds,
        request_url: requestUrl,
        upload_id: uploadId
      })
    )
  );

  await env.DB.prepare(
    `INSERT INTO square_uploads
      (upload_id, post_id, owner_account, post_category, manifest_hash, content_hash,
        storage_receipt_id, estimated_bytes, object_keys_json, status, created_at, completed_at)
      VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, 'prepared', ?, NULL)`
  )
    .bind(
      uploadId,
      postId,
      session.owner_account,
      postCategory,
      manifestHash,
      storageReceiptId,
      estimatedBytes,
      JSON.stringify(objectKeyPlan.object_keys),
      nowMs()
    )
    .run();

  return jsonResponse({
    ok: true,
    upload_id: uploadId,
    post_id: postId,
    storage_receipt_id: storageReceiptId,
    expires_at: expiresAt,
    estimated_bytes: estimatedBytes,
    manifest_object_key: objectKeyPlan.manifest_object_key,
    manifest_upload_url: manifestUploadUrl,
    media_items: objectKeyPlan.media_items.map((item, index) => ({
      media_kind: item.media_kind,
      content_type: item.content_type,
      byte_size: item.byte_size,
      object_key: item.object_key,
      upload_url: uploadUrls[index]
    }))
  });
}

export async function devPutUploadObject(request: Request, env: Env): Promise<Response> {
  if (env.SQUARE_DEV_UPLOAD_PROXY !== '1') {
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

  const completedAt = nowMs();

  await env.DB.batch([
    env.DB.prepare(
      `UPDATE square_uploads
        SET content_hash = ?, status = 'completed', completed_at = ?
        WHERE upload_id = ?`
    ).bind(contentHash, completedAt, upload.upload_id),
    env.DB.prepare(
      `UPDATE square_memberships
        SET storage_used_bytes = storage_used_bytes + ?, updated_at = ?
        WHERE owner_account = ?`
    ).bind(upload.estimated_bytes, completedAt, upload.owner_account)
  ]);

  return jsonResponse({
    ok: true,
    upload_id: upload.upload_id,
    post_id: upload.post_id,
    content_hash: contentHash,
    storage_receipt_id: upload.storage_receipt_id,
    storage_state: 'completed'
  });
}

export { validateUploadItems, estimateUploadBytes };
