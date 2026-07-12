import type { Env } from '../types';
import { HttpError, requireSession } from '../shared/http';
import { resourceLimit } from '../limits/catalog';

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

  const match = /^profile\/[^/]+\/(avatar|banner)$/.exec(objectKey);
  if (!match) {
    throw new HttpError(400, 'invalid_media_key', '媒体对象路径不合法');
  }

  const object = await env.SQUARE_MEDIA.get(objectKey);
  if (!object) {
    throw new HttpError(404, 'media_not_found', '媒体对象不存在');
  }
  const resourceKey = match[1] === 'avatar' ? 'profile_avatar' : 'profile_banner';
  if (object.size > resourceLimit(resourceKey).max_bytes) {
    await object.body.cancel();
    throw new HttpError(500, 'stored_media_limit_exceeded', '已存资料资源超过服务端上限');
  }

  const headers = new Headers();
  headers.set(
    'content-type',
    object.httpMetadata?.contentType ?? 'application/octet-stream'
  );
  // 资料对象也必须经过钱包会话；禁止共享缓存把已授权响应泄露给其他访问者。
  // 固定对象键会被覆盖，禁止缓存旧头像或背景。
  headers.set('cache-control', 'private, no-store');
  if (object.httpEtag) {
    headers.set('etag', object.httpEtag);
  }
  return new Response(object.body, { headers });
}
