import type { Env, MediaAssetRow, MediaProvider, MediaUploadMethod } from '../types';
import { HttpError } from '../shared/http';
import { LimitTicket } from '../limits/upload';

export interface ProviderUploadInput {
  ownerAccount: string;
  uploadId: string;
  postId: string;
  mediaIndex: number;
  mediaKind: 'image' | 'video' | 'cover';
  contentType: string;
  byteSize: number;
  maxDurationSeconds: number;
  workerUploadUrl: string;
}

export interface ProviderUploadPlan {
  provider: MediaProvider;
  provider_asset_id: string;
  upload_method: MediaUploadMethod;
  upload_url: string;
  asset_state: MediaAssetRow['asset_state'];
}

interface CloudflareApiResult<T> {
  success?: boolean;
  result?: T;
  errors?: Array<{ message?: string; code?: number | string }>;
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
  if (input.mediaKind === 'video') {
    return createStreamTusUpload(env, input);
  }
  return {
    provider: 'cloudflare_images',
    provider_asset_id: `pending:${input.uploadId}:${input.mediaIndex}`,
    upload_method: 'worker',
    upload_url: input.workerUploadUrl,
    asset_state: 'prepared',
  };
}

export async function refreshProviderAssetState(
  env: Env,
  row: MediaAssetRow
): Promise<Partial<MediaAssetRow>> {
  if (row.asset_state === 'ready' || row.asset_state === 'error') return {};
  if (row.provider === 'cloudflare_stream') {
    return refreshStreamAsset(env, row.provider_asset_id);
  }
  return refreshImageAsset(env, row.provider_asset_id);
}

export async function deleteProviderAsset(
  env: Env,
  row: Pick<MediaAssetRow, 'provider' | 'provider_asset_id'>
): Promise<void> {
  if (row.provider === 'cloudflare_stream') {
    await deleteStreamAsset(env, row.provider_asset_id);
    return;
  }
  await deleteImageAsset(env, row.provider_asset_id);
}

/** 图片先由 Worker 校验真实字节和尺寸，再使用服务端 token 写入 Cloudflare Images。 */
export async function uploadImageAsset(
  env: Env,
  input: ProviderUploadInput,
  bytes: Uint8Array,
  ticket: LimitTicket,
): Promise<{ provider_asset_id: string }> {
  ticket.assertValid();
  if (ticket.byte_size !== bytes.byteLength || !ticket.content_type.startsWith('image/')) {
    throw new HttpError(500, 'image_limit_ticket_mismatch', '图片限制凭证与上传内容不一致');
  }
  const config = requireCloudflareApiConfig(env);
  const form = new FormData();
  form.set(
    'file',
    new Blob([Uint8Array.from(bytes).buffer], { type: ticket.content_type }),
    `${input.postId}-${input.mediaIndex}`,
  );
  form.set('requireSignedURLs', 'true');
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
    `https://api.cloudflare.com/client/v4/accounts/${config.accountId}/images/v1`,
    {
      method: 'POST',
      headers: { authorization: `Bearer ${config.apiToken}` },
      body: form
    }
  );
  const result = await parseCloudflareJson<ImageDetailsResult>(response, 'images_upload_failed');
  if (!result.id) throw new HttpError(502, 'images_upload_incomplete', 'Cloudflare Images 上传响应不完整');
  return { provider_asset_id: result.id };
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
          requiresignedurls: null,
          expiry: new Date(Date.now() + 60 * 60 * 1000).toISOString(),
          name: `${input.postId}-${input.mediaIndex}`,
          filetype: input.contentType
        })
      }
    }
  );
  if (!response.ok) {
    throw new HttpError(response.status, 'stream_tus_upload_failed', 'Cloudflare Stream tus 授权失败');
  }
  const uploadUrl = response.headers.get('location');
  const uid = response.headers.get('stream-media-id');
  if (!uploadUrl || !uid) {
    throw new HttpError(502, 'stream_tus_upload_incomplete', 'Cloudflare Stream tus 上传授权响应不完整');
  }
  return {
    provider: 'cloudflare_stream',
    provider_asset_id: uid,
    upload_method: 'tus',
    upload_url: uploadUrl,
    asset_state: 'prepared'
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
  return {
    asset_state: isError ? 'error' : isReady ? 'ready' : 'processing',
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

function uploadMetadata(values: Record<string, string | null>): string {
  return Object.entries(values)
    .map(([key, value]) => value === null ? key : `${key} ${base64(value)}`)
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
