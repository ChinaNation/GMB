import type { Env } from '../types';
import { HttpError, requireSession } from '../shared/http';

const MEDIA_PREFIX = '/v1/square/media/';

/// 钱包用户资料媒体读取通道：只把 R2 中的头像 / 背景对象按 object_key 直出。
/// 广场主媒体已经迁移到 Cloudflare Images / Stream，manifest 也不作为公开媒体暴露。
export async function mediaRoute(
  request: Request,
  env: Env,
  path: string
): Promise<Response> {
  await requireSession(request, env);
  const objectKey = path
    .slice(MEDIA_PREFIX.length)
    .split('/')
    .map((segment) => decodeURIComponent(segment))
    .join('/');

  if (
    !objectKey ||
    objectKey.includes('..') ||
    !objectKey.startsWith('profile/')
  ) {
    throw new HttpError(400, 'invalid_media_key', '媒体对象路径不合法');
  }

  const object = await env.SQUARE_MEDIA.get(objectKey);
  if (!object) {
    throw new HttpError(404, 'media_not_found', '媒体对象不存在');
  }

  const headers = new Headers();
  headers.set(
    'content-type',
    object.httpMetadata?.contentType ?? 'application/octet-stream'
  );
  headers.set('cache-control', 'public, max-age=31536000, immutable');
  headers.set('access-control-allow-origin', '*');
  if (object.httpEtag) {
    headers.set('etag', object.httpEtag);
  }
  return new Response(object.body, { headers });
}
