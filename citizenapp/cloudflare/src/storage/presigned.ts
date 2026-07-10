import { AwsClient } from 'aws4fetch';
import type { Env } from '../types';
import { HttpError } from '../shared/http';

export interface UploadUrlInput {
  object_key: string;
  content_type: string;
  expires_seconds: number;
  request_url: URL;
  upload_id: string;
  dev_upload_path?: string;
}

export interface DownloadUrlInput {
  object_key: string;
  expires_seconds: number;
  request_url: URL;
  access_id: string;
  dev_download_path?: string;
  dev_query?: Record<string, string>;
}

function encodeObjectPath(objectKey: string): string {
  return objectKey
    .split('/')
    .map((part) => encodeURIComponent(part))
    .join('/');
}

export function canCreateRealPresignedUrl(env: Env): boolean {
  return Boolean(env.R2_ACCOUNT_ID && env.R2_ACCESS_KEY_ID && env.R2_SECRET_ACCESS_KEY);
}

export async function createUploadUrl(env: Env, input: UploadUrlInput): Promise<string> {
  if (canCreateRealPresignedUrl(env)) {
    const bucketName = env.R2_BUCKET_NAME ?? 'citizenapp-square-media';
    // R2 兼容 S3 预签名 URL；签名发生在 Worker，CitizenApp 只拿短期 PUT URL。
    const endpoint = new URL(`https://${env.R2_ACCOUNT_ID}.r2.cloudflarestorage.com/${bucketName}/${encodeObjectPath(
      input.object_key
    )}`);
    endpoint.searchParams.set('X-Amz-Expires', String(input.expires_seconds));
    const awsClient = new AwsClient({
      accessKeyId: env.R2_ACCESS_KEY_ID!,
      secretAccessKey: env.R2_SECRET_ACCESS_KEY!,
      service: 's3',
      region: 'auto'
    });
    const signedRequest = await awsClient.sign(endpoint, {
      method: 'PUT',
      headers: {
        'content-type': input.content_type
      },
      aws: {
        signQuery: true
      }
    });

    return signedRequest.url;
  }

  // 本地 Miniflare 不生成真实 R2 S3 签名，开发代理只允许本地验证使用。
  if (env.SQUARE_DEV_UPLOAD_PROXY === '1') {
    const devUrl = new URL(input.dev_upload_path ?? '/v1/square/uploads/dev-put', input.request_url.origin);
    devUrl.searchParams.set('upload_id', input.upload_id);
    devUrl.searchParams.set('object_key', input.object_key);
    return devUrl.toString();
  }

  throw new HttpError(503, 'r2_presign_unavailable', 'R2 上传授权未配置');
}

export async function createDownloadUrl(env: Env, input: DownloadUrlInput): Promise<string> {
  if (canCreateRealPresignedUrl(env)) {
    const bucketName = env.R2_BUCKET_NAME ?? 'citizenapp-square-media';
    // 下载同样只签短期 R2 URL；Worker 不代理或解密聊天附件内容。
    const endpoint = new URL(`https://${env.R2_ACCOUNT_ID}.r2.cloudflarestorage.com/${bucketName}/${encodeObjectPath(
      input.object_key
    )}`);
    endpoint.searchParams.set('X-Amz-Expires', String(input.expires_seconds));
    const awsClient = new AwsClient({
      accessKeyId: env.R2_ACCESS_KEY_ID!,
      secretAccessKey: env.R2_SECRET_ACCESS_KEY!,
      service: 's3',
      region: 'auto'
    });
    const signedRequest = await awsClient.sign(endpoint, {
      method: 'GET',
      aws: {
        signQuery: true
      }
    });

    return signedRequest.url;
  }

  if (env.SQUARE_DEV_UPLOAD_PROXY === '1') {
    const devUrl = new URL(input.dev_download_path ?? '/v1/square/uploads/dev-get', input.request_url.origin);
    devUrl.searchParams.set('access_id', input.access_id);
    devUrl.searchParams.set('object_key', input.object_key);
    for (const [key, value] of Object.entries(input.dev_query ?? {})) {
      devUrl.searchParams.set(key, value);
    }
    return devUrl.toString();
  }

  throw new HttpError(503, 'r2_presign_unavailable', 'R2 下载授权未配置');
}

/// 冷归档回灌用：对 R2 冷存对象签发短期只读 URL，供 Cloudflare Stream copy-from-URL 拉取。
/// 无 R2 S3 凭证时返回 null（调用方据此判定回灌不可用）。
export async function signR2GetUrl(
  env: Env,
  objectKey: string,
  expiresSeconds: number
): Promise<string | null> {
  if (!canCreateRealPresignedUrl(env)) {
    return null;
  }
  const bucketName = env.R2_BUCKET_NAME ?? 'citizenapp-square-media';
  const endpoint = new URL(
    `https://${env.R2_ACCOUNT_ID}.r2.cloudflarestorage.com/${bucketName}/${encodeObjectPath(objectKey)}`
  );
  endpoint.searchParams.set('X-Amz-Expires', String(expiresSeconds));
  const awsClient = new AwsClient({
    accessKeyId: env.R2_ACCESS_KEY_ID!,
    secretAccessKey: env.R2_SECRET_ACCESS_KEY!,
    service: 's3',
    region: 'auto'
  });
  const signedRequest = await awsClient.sign(endpoint, {
    method: 'GET',
    aws: { signQuery: true }
  });
  return signedRequest.url;
}
