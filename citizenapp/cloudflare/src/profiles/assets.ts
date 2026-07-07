import type { Env } from '../types';
import {
  HttpError,
  jsonResponse,
  parsePositiveInt,
  readJson,
  requireSession
} from '../shared/http';
import { isSha256Hex } from '../shared/hash';
import { createUploadUrl } from '../storage/presigned';
import { normalizeFileExt, profileAssetPrefix } from '../storage/r2_keys';

const ALLOWED_CONTENT_TYPES = ['image/jpeg', 'image/png', 'image/webp'];
const MAX_ASSET_BYTES = 15 * 1024 * 1024;

interface PrepareAssetRequest {
  kind?: unknown;
  content_type?: unknown;
  byte_size?: unknown;
  sha256?: unknown;
}

/// 头像/背景上传授权：object_key 落本人 profile/ 前缀，文件名带 sha256 便于 CDN 缓存失效。
/// 内容不上链（决策 2）；生产返回 R2 预签名 PUT，本地返回 dev-put。
export async function prepareProfileAsset(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<PrepareAssetRequest>(request);

  const kind =
    body.kind === 'banner' ? 'banner' : body.kind === 'avatar' ? 'avatar' : null;
  if (kind === null) {
    throw new HttpError(400, 'invalid_asset_kind', '资源类型必须是 avatar 或 banner');
  }
  if (
    typeof body.content_type !== 'string' ||
    !ALLOWED_CONTENT_TYPES.includes(body.content_type)
  ) {
    throw new HttpError(400, 'invalid_content_type', '头像/背景只支持 jpeg/png/webp');
  }
  if (
    typeof body.byte_size !== 'number' ||
    body.byte_size <= 0 ||
    body.byte_size > MAX_ASSET_BYTES
  ) {
    throw new HttpError(400, 'invalid_byte_size', '文件大小不合法');
  }
  if (!isSha256Hex(body.sha256)) {
    throw new HttpError(400, 'invalid_sha256', 'sha256 必须是 64 位 hex');
  }

  const sha = (body.sha256 as string).toLowerCase();
  const ext = normalizeFileExt(body.content_type);
  const objectKey = `${profileAssetPrefix(session.owner_account)}${kind}_${sha}.${ext}`;
  const expiresSeconds = parsePositiveInt(env.SQUARE_UPLOAD_URL_TTL_SECONDS, 900);
  const uploadUrl = await createUploadUrl(env, {
    object_key: objectKey,
    content_type: body.content_type,
    expires_seconds: expiresSeconds,
    request_url: new URL(request.url),
    upload_id: 'profile',
    dev_upload_path: '/v1/square/profile/assets/dev-put'
  });

  return jsonResponse({
    ok: true,
    object_key: objectKey,
    content_hash: sha,
    upload_url: uploadUrl
  });
}

/// 本地开发上传代理：仅校验对象属本人 profile/ 前缀（无上传行），写入 R2。
export async function devPutProfileAsset(request: Request, env: Env): Promise<Response> {
  if (env.SQUARE_DEV_UPLOAD_PROXY !== '1') {
    throw new HttpError(404, 'dev_upload_proxy_disabled', '开发上传代理未启用');
  }
  const session = await requireSession(request, env);
  const objectKey = new URL(request.url).searchParams.get('object_key');
  if (!objectKey || !objectKey.startsWith(profileAssetPrefix(session.owner_account))) {
    throw new HttpError(403, 'asset_object_forbidden', '无权写入该资源对象');
  }
  const contentType =
    request.headers.get('content-type') ?? 'application/octet-stream';
  const bytes = await request.arrayBuffer();
  await env.SQUARE_MEDIA.put(objectKey, bytes, {
    httpMetadata: { contentType }
  });
  return jsonResponse({
    ok: true,
    object_key: objectKey,
    byte_size: bytes.byteLength
  });
}
