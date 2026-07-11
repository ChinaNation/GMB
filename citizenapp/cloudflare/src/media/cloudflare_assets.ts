import type { Env, MediaAssetRow, MediaProvider, MediaUploadMethod } from '../types';
import { HttpError } from '../shared/http';

// Cloudflare Stream 200MB 以上必须走 tus；Worker 只签发一次性 URL，不接收视频正文。
const streamTusThresholdBytes = 200 * 1024 * 1024;

export interface ProviderUploadInput {
  ownerAccount: string;
  uploadId: string;
  postId: string;
  mediaIndex: number;
  mediaKind: 'image' | 'video' | 'cover';
  contentType: string;
  byteSize: number;
  maxDurationSeconds: number;
  requestOrigin: string;
}

export interface ProviderUploadPlan {
  provider: MediaProvider;
  provider_asset_id: string;
  upload_method: MediaUploadMethod;
  upload_url: string;
  asset_state: MediaAssetRow['asset_state'];
  delivery_url: string | null;
  playback_hls_url: string | null;
  playback_dash_url: string | null;
  thumbnail_url: string | null;
}

interface CloudflareApiResult<T> {
  success?: boolean;
  result?: T;
  errors?: Array<{ message?: string; code?: number | string }>;
}

interface ImagesDirectUploadResult {
  id?: string;
  uploadURL?: string;
}

interface ImageDetailsResult {
  id?: string;
  draft?: boolean;
  variants?: string[];
}

interface StreamDirectUploadResult {
  uid?: string;
  uploadURL?: string;
}

interface StreamDetailsResult {
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

export function providerForMediaKind(kind: 'image' | 'video' | 'cover'): MediaProvider {
  return kind === 'video' ? 'cloudflare_stream' : 'cloudflare_images';
}

export async function createProviderUpload(
  env: Env,
  input: ProviderUploadInput
): Promise<ProviderUploadPlan> {
  // 本地 Miniflare 没有真实 Images / Stream，使用同源 dev-media 端点验证完整控制流。
  if (env.DEV_UPLOAD_PROXY === '1') {
    return createDevProviderUpload(input);
  }
  if (input.mediaKind === 'video') {
    return createStreamUpload(env, input);
  }
  return createImagesUpload(env, input);
}

export async function refreshProviderAssetState(
  env: Env,
  row: MediaAssetRow
): Promise<Partial<MediaAssetRow>> {
  if (env.DEV_UPLOAD_PROXY === '1') {
    return {};
  }
  if (row.provider === 'cloudflare_stream') {
    return refreshStreamAsset(env, row.provider_asset_id);
  }
  return refreshImageAsset(env, row.provider_asset_id);
}

export async function deleteProviderAsset(
  env: Env,
  row: Pick<MediaAssetRow, 'provider' | 'provider_asset_id'>
): Promise<void> {
  // 本地验收环境没有真实 Images / Stream 资源，删除动作由 D1/R2 清理覆盖。
  if (env.DEV_UPLOAD_PROXY === '1') {
    return;
  }
  if (row.provider === 'cloudflare_stream') {
    await deleteStreamAsset(env, row.provider_asset_id);
    return;
  }
  await deleteImageAsset(env, row.provider_asset_id);
}

export function streamPlaybackUrls(
  env: Env,
  uid: string
): {
  playback_hls_url: string | null;
  playback_dash_url: string | null;
  thumbnail_url: string | null;
} {
  const base = trimTrailingSlash(env.STREAM_URL);
  if (!base) {
    return {
      playback_hls_url: null,
      playback_dash_url: null,
      thumbnail_url: null
    };
  }
  return {
    playback_hls_url: `${base}/${uid}/manifest/video.m3u8`,
    playback_dash_url: `${base}/${uid}/manifest/video.mpd`,
    thumbnail_url: `${base}/${uid}/thumbnails/thumbnail.jpg`
  };
}

export function imageDeliveryUrl(env: Env, imageId: string): string | null {
  const base = trimTrailingSlash(env.IMAGES_URL);
  return base ? `${base}/${imageId}/public` : null;
}

async function createImagesUpload(env: Env, input: ProviderUploadInput): Promise<ProviderUploadPlan> {
  const config = requireCloudflareApiConfig(env);
  const form = new FormData();
  // 广场图片是公开内容，使用公开 variant；访问控制由帖子可见性和 feed 控制承担。
  form.set('requireSignedURLs', 'false');
  form.set(
    'metadata',
    JSON.stringify({
      owner_account: input.ownerAccount,
      upload_id: input.uploadId,
      post_id: input.postId,
      media_index: input.mediaIndex
    })
  );

  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/images/v2/direct_upload`,
    {
      method: 'POST',
      headers: { authorization: `Bearer ${config.apiToken}` },
      body: form
    }
  );
  const result = await parseCloudflareJson<ImagesDirectUploadResult>(response, 'images_direct_upload_failed');
  if (!result.id || !result.uploadURL) {
    throw new HttpError(502, 'images_direct_upload_incomplete', 'Cloudflare Images 上传授权响应不完整');
  }

  return {
    provider: 'cloudflare_images',
    provider_asset_id: result.id,
    upload_method: 'direct_form',
    upload_url: result.uploadURL,
    asset_state: 'prepared',
    delivery_url: imageDeliveryUrl(env, result.id),
    playback_hls_url: null,
    playback_dash_url: null,
    thumbnail_url: null
  };
}

async function createStreamUpload(env: Env, input: ProviderUploadInput): Promise<ProviderUploadPlan> {
  if (input.byteSize > streamTusThresholdBytes) {
    return createStreamTusUpload(env, input);
  }
  // 200MB 以下视频使用 Stream basic direct upload；App 以 multipart form 直传到 Cloudflare。
  const config = requireCloudflareApiConfig(env);
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/stream/direct_upload`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${config.apiToken}`,
        'content-type': 'application/json'
      },
      body: JSON.stringify({
        maxDurationSeconds: input.maxDurationSeconds
      })
    }
  );
  const result = await parseCloudflareJson<StreamDirectUploadResult>(response, 'stream_direct_upload_failed');
  if (!result.uid || !result.uploadURL) {
    throw new HttpError(502, 'stream_direct_upload_incomplete', 'Cloudflare Stream 上传授权响应不完整');
  }
  const playback = streamPlaybackUrls(env, result.uid);

  return {
    provider: 'cloudflare_stream',
    provider_asset_id: result.uid,
    upload_method: 'direct_form',
    upload_url: result.uploadURL,
    asset_state: 'prepared',
    delivery_url: null,
    ...playback
  };
}

async function createStreamTusUpload(env: Env, input: ProviderUploadInput): Promise<ProviderUploadPlan> {
  const config = requireCloudflareApiConfig(env);
  // tus URL 由 Worker 代申请，App 只拿 Location 和 uid，绝不接触 Cloudflare API token。
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/stream?direct_user=true`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${config.apiToken}`,
        'tus-resumable': '1.0.0',
        'upload-length': String(input.byteSize),
        'upload-creator': input.ownerAccount,
        'upload-metadata': uploadMetadata({
          maxDurationSeconds: String(input.maxDurationSeconds),
          name: `${input.postId}-${input.mediaIndex}`,
          filetype: input.contentType
        })
      }
    }
  );
  if (!response.ok) {
    throw new HttpError(response.status, 'stream_tus_upload_failed', await response.text());
  }
  const uploadUrl = response.headers.get('location');
  const uid = response.headers.get('stream-media-id');
  if (!uploadUrl || !uid) {
    throw new HttpError(502, 'stream_tus_upload_incomplete', 'Cloudflare Stream tus 上传授权响应不完整');
  }
  const playback = streamPlaybackUrls(env, uid);

  return {
    provider: 'cloudflare_stream',
    provider_asset_id: uid,
    upload_method: 'tus',
    upload_url: uploadUrl,
    asset_state: 'prepared',
    delivery_url: null,
    ...playback
  };
}

function createDevProviderUpload(input: ProviderUploadInput): ProviderUploadPlan {
  const provider = providerForMediaKind(input.mediaKind);
  const assetId = `${provider === 'cloudflare_stream' ? 'str' : 'img'}_${input.uploadId}_${input.mediaIndex}`;
  const url = new URL('/v1/square/uploads/dev-media', input.requestOrigin);
  url.searchParams.set('upload_id', input.uploadId);
  url.searchParams.set('media_index', String(input.mediaIndex));
  const playback = provider === 'cloudflare_stream'
    ? {
        playback_hls_url: `${input.requestOrigin}/dev-stream/${assetId}/manifest/video.m3u8`,
        playback_dash_url: `${input.requestOrigin}/dev-stream/${assetId}/manifest/video.mpd`,
        thumbnail_url: `${input.requestOrigin}/dev-stream/${assetId}/thumbnails/thumbnail.jpg`
      }
    : {
        playback_hls_url: null,
        playback_dash_url: null,
        thumbnail_url: null
      };

  return {
    provider,
    provider_asset_id: assetId,
    upload_method: provider === 'cloudflare_stream' && input.byteSize > streamTusThresholdBytes
      ? 'tus'
      : 'direct_form',
    upload_url: url.toString(),
    asset_state: 'prepared',
    delivery_url: provider === 'cloudflare_images'
      ? `${input.requestOrigin}/dev-images/${assetId}/public`
      : null,
    ...playback
  };
}

async function refreshImageAsset(
  env: Env,
  imageId: string
): Promise<Partial<MediaAssetRow>> {
  // complete 阶段复查 Images draft 状态，防止客户端未真正上传却直接调用完成接口。
  const config = requireCloudflareApiConfig(env);
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/images/v1/${encodeURIComponent(imageId)}`,
    {
      headers: { authorization: `Bearer ${config.apiToken}` }
    }
  );
  const result = await parseCloudflareJson<ImageDetailsResult>(response, 'images_status_failed');
  return {
    asset_state: result.draft ? 'prepared' : 'ready',
    delivery_url: imageDeliveryUrl(env, result.id ?? imageId) ?? result.variants?.[0] ?? null,
    updated_at: Date.now(),
    ready_at: result.draft ? null : Date.now()
  };
}

async function refreshStreamAsset(
  env: Env,
  uid: string
): Promise<Partial<MediaAssetRow>> {
  // Stream 转码可能滞后；complete 可先返回 processing，webhook 后再更新 ready。
  const config = requireCloudflareApiConfig(env);
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/stream/${encodeURIComponent(uid)}`,
    {
      headers: { authorization: `Bearer ${config.apiToken}` }
    }
  );
  const result = await parseCloudflareJson<StreamDetailsResult>(response, 'stream_status_failed');
  return streamDetailsToAssetUpdate(env, result, uid);
}

async function deleteImageAsset(env: Env, imageId: string): Promise<void> {
  const config = requireCloudflareApiConfig(env);
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/images/v1/${encodeURIComponent(imageId)}`,
    {
      method: 'DELETE',
      headers: { authorization: `Bearer ${config.apiToken}` }
    }
  );
  await assertCloudflareDeleteOk(response, 'images_delete_failed');
}

async function deleteStreamAsset(env: Env, uid: string): Promise<void> {
  const config = requireCloudflareApiConfig(env);
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/stream/${encodeURIComponent(uid)}`,
    {
      method: 'DELETE',
      headers: { authorization: `Bearer ${config.apiToken}` }
    }
  );
  await assertCloudflareDeleteOk(response, 'stream_delete_failed');
}

interface StreamDownloadResult {
  default?: {
    status?: string;
    url?: string;
    percentComplete?: number;
  };
}

/// 冷归档取片：Stream 无冷层，只能用 downloads API 导出编码版 MP4。POST 触发生成（幂等），
/// 就绪则返回可下载 URL；仍在生成返回 null（本轮跳过，下次扫描再归档，避免 Cron 长轮询）。
export async function createStreamDownloadUrl(env: Env, uid: string): Promise<string | null> {
  const config = requireCloudflareApiConfig(env);
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/stream/${encodeURIComponent(uid)}/downloads`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${config.apiToken}`,
        'content-type': 'application/json'
      },
      body: '{}'
    }
  );
  const result = await parseCloudflareJson<StreamDownloadResult>(response, 'stream_download_failed');
  if (result.default?.status === 'ready' && result.default.url) {
    return result.default.url;
  }
  return null;
}

/// 冷归档回灌：从 R2 冷存的短期只读 URL 复制回 Stream，返回新 uid；转码完成信号走 Stream webhook。
export async function copyStreamFromUrl(
  env: Env,
  sourceUrl: string,
  maxDurationSeconds: number
): Promise<string> {
  const config = requireCloudflareApiConfig(env);
  const response = await fetch(
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/stream/copy`,
    {
      method: 'POST',
      headers: {
        authorization: `Bearer ${config.apiToken}`,
        'content-type': 'application/json'
      },
      body: JSON.stringify({ url: sourceUrl, maxDurationSeconds })
    }
  );
  const result = await parseCloudflareJson<StreamDirectUploadResult>(response, 'stream_copy_failed');
  if (!result.uid) {
    throw new HttpError(502, 'stream_copy_incomplete', 'Cloudflare Stream 回灌响应缺少 uid');
  }
  return result.uid;
}

export function streamDetailsToAssetUpdate(
  env: Env,
  result: StreamDetailsResult,
  fallbackUid: string
): Partial<MediaAssetRow> {
  const state = result.status?.state;
  const isReady = result.readyToStream === true && state === 'ready';
  const isError = state === 'error';
  const playback = streamPlaybackUrls(env, result.uid ?? fallbackUid);

  return {
    asset_state: isError ? 'error' : isReady ? 'ready' : 'processing',
    playback_hls_url: result.playback?.hls ?? playback.playback_hls_url,
    playback_dash_url: result.playback?.dash ?? playback.playback_dash_url,
    thumbnail_url: result.thumbnail ?? playback.thumbnail_url,
    duration_seconds: typeof result.duration === 'number' ? result.duration : null,
    width: typeof result.input?.width === 'number' ? result.input.width : null,
    height: typeof result.input?.height === 'number' ? result.input.height : null,
    error_code: result.status?.errorReasonCode ?? result.status?.errReasonCode ?? null,
    updated_at: Date.now(),
    ready_at: isReady ? Date.now() : null
  };
}

async function parseCloudflareJson<T>(response: Response, errorCode: string): Promise<T> {
  const payload = await response.json().catch(() => null) as CloudflareApiResult<T> | null;
  if (!response.ok || payload?.success === false) {
    const message = payload?.errors?.[0]?.message ?? `Cloudflare API 请求失败：${response.status}`;
    throw new HttpError(response.status || 502, errorCode, message);
  }
  if (!payload?.result) {
    throw new HttpError(502, errorCode, 'Cloudflare API 响应缺少 result');
  }
  return payload.result;
}

async function assertCloudflareDeleteOk(response: Response, errorCode: string): Promise<void> {
  const payload = await response.json().catch(() => null) as CloudflareApiResult<unknown> | null;
  if (response.status === 404) {
    return;
  }
  if (!response.ok || payload?.success === false) {
    const message = payload?.errors?.[0]?.message ?? `Cloudflare API 删除失败：${response.status}`;
    throw new HttpError(response.status || 502, errorCode, message);
  }
}

function requireCloudflareApiConfig(env: Env): { accountId: string; apiToken: string } {
  if (!env.CF_ACCOUNT_ID || !env.CF_API_TOKEN) {
    throw new HttpError(503, 'cloudflare_media_api_not_configured', 'Cloudflare Images / Stream API 未配置');
  }
  return {
    accountId: env.CF_ACCOUNT_ID,
    apiToken: env.CF_API_TOKEN
  };
}

function uploadMetadata(values: Record<string, string>): string {
  return Object.entries(values)
    .map(([key, value]) => `${key} ${base64(value)}`)
    .join(',');
}

function base64(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

function trimTrailingSlash(value?: string): string {
  return value?.trim().replace(/\/+$/, '') ?? '';
}
