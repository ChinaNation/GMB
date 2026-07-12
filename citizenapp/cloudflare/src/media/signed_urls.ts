import type { Env, MediaAssetRow } from '../types';
import { HttpError } from '../shared/http';
import { deliveryTtl } from '../limits/delivery';

export interface SignedMediaUrls {
  url: string;
  thumbnail_url: string | null;
}

/** 媒体地址只在组装 Feed 时短期签发，D1 永远不保存可访问 URL。 */
export async function signedMediaUrls(
  env: Env,
  asset: Pick<MediaAssetRow, 'provider' | 'provider_asset_id'>
): Promise<SignedMediaUrls> {
  if (asset.provider === 'cloudflare_stream') {
    return signedStreamUrls(env, asset.provider_asset_id);
  }
  return {
    url: await signedImageUrl(env, asset.provider_asset_id),
    thumbnail_url: null
  };
}

export async function signedImageUrl(env: Env, imageId: string): Promise<string> {
  const base = env.IMAGES_URL?.trim().replace(/\/+$/, '');
  const signingKey = env.IMAGES_SIGNING_KEY;
  if (!base || !signingKey) {
    throw new HttpError(503, 'images_signing_not_configured', 'Cloudflare Images 私有交付未配置');
  }
  const url = new URL(`${base}/${encodeURIComponent(imageId)}/public`);
  const expiry = Math.floor(Date.now() / 1000) + deliveryTtl(env);
  url.searchParams.set('exp', String(expiry));
  const payload = `${url.pathname}?${url.searchParams.toString()}`;
  const key = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(signingKey),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const digest = await crypto.subtle.sign('HMAC', key, new TextEncoder().encode(payload));
  url.searchParams.set('sig', bytesToHex(new Uint8Array(digest)));
  return url.toString();
}

async function signedStreamUrls(env: Env, uid: string): Promise<SignedMediaUrls> {
  const base = env.STREAM_URL?.trim().replace(/\/+$/, '');
  if (!base || !env.STREAM) {
    throw new HttpError(503, 'stream_signing_not_configured', 'Cloudflare Stream 私有播放未配置');
  }
  // Stream binding 在边缘直接签发播放 token，不调用公开 REST token 接口，也不暴露密钥。
  const token = await env.STREAM.video(uid).generateToken();
  return {
    url: `${base}/${token}/manifest/video.m3u8`,
    thumbnail_url: `${base}/${token}/thumbnails/thumbnail.jpg`
  };
}

function bytesToHex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}
