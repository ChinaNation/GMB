import type { Env } from '../types';
import { HttpError } from '../shared/http';

const MEDIA_PREFIX = '/v1/square/media/';

/// 公开媒体读取通道：把 R2 中的广场媒体 / 头像 / 背景对象按 object_key 直出，
/// 供 App `Image.network` 与 CDN 缓存使用。只允许 square/ 与 profile/ 前缀，杜绝任意读。
export async function mediaRoute(
  request: Request,
  env: Env,
  path: string
): Promise<Response> {
  const objectKey = path
    .slice(MEDIA_PREFIX.length)
    .split('/')
    .map((segment) => decodeURIComponent(segment))
    .join('/');

  if (
    !objectKey ||
    objectKey.includes('..') ||
    (!objectKey.startsWith('square/') && !objectKey.startsWith('profile/'))
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
