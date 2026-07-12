import { AwsClient } from 'aws4fetch';
import type { Env } from '../types';

function encodeObjectPath(objectKey: string): string {
  return objectKey
    .split('/')
    .map((part) => encodeURIComponent(part))
    .join('/');
}

export function canCreateRealPresignedUrl(env: Env): boolean {
  return Boolean(env.CF_ACCOUNT_ID && env.R2_ACCESS_ID && env.R2_SECRET_KEY);
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
  const bucketName = env.R2_BUCKET ?? 'citizenapp-square-media';
  const endpoint = new URL(
    `https://${env.CF_ACCOUNT_ID}.r2.cloudflarestorage.com/${bucketName}/${encodeObjectPath(objectKey)}`
  );
  endpoint.searchParams.set('X-Amz-Expires', String(expiresSeconds));
  const awsClient = new AwsClient({
    accessKeyId: env.R2_ACCESS_ID!,
    secretAccessKey: env.R2_SECRET_KEY!,
    service: 's3',
    region: 'auto'
  });
  const signedRequest = await awsClient.sign(endpoint, {
    method: 'GET',
    aws: { signQuery: true }
  });
  return signedRequest.url;
}
