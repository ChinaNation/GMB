import type { MembershipLevel } from '../membership/plans';

export type ResourceKey =
  | 'profile_avatar'
  | 'profile_banner'
  | 'profile_doc'
  | 'square_manifest'
  | 'square_image_sd'
  | 'square_image_hd'
  | 'square_cover'
  | 'square_video_sd'
  | 'square_video_hd'
  | 'square_video_candidate'
  | 'chat_device'
  | 'chat_keypackage'
  | 'chat_envelope'
  | 'chat_signal'
  | 'push_wake'
  | 'stripe_webhook'
  | 'stream_webhook'
  | 'chain_extrinsic'
  | 'chain_extrinsic_json'
  | 'chain_rpc_response'
  | 'session_cache'
  | 'session_index'
  | 'identity_cache'
  | 'contact_ciphertext'
  | 'api_json_small'
  | 'api_json';

export interface ResourceLimit {
  max_bytes: number;
  content_types?: readonly string[];
  max_width?: number;
  max_height?: number;
  max_seconds?: number;
  max_count?: number;
  max_items?: number;
  ttl_seconds?: number;
}

const kib = 1024;
const mib = 1024 * kib;
const gib = 1024 * mib;

/**
 * Cloudflare 资源硬上限唯一真源。
 *
 * 环境变量不得放宽这些值；产品权益、路由、存储和第三方上传都必须引用本表。
 */
export const resourceLimits: Readonly<Record<ResourceKey, ResourceLimit>> = {
  profile_avatar: {
    max_bytes: 512 * kib,
    content_types: ['image/jpeg', 'image/png', 'image/webp'],
    max_width: 1024,
    max_height: 1024,
    max_count: 1,
  },
  profile_banner: {
    max_bytes: 1536 * kib,
    content_types: ['image/jpeg', 'image/png', 'image/webp'],
    max_width: 1920,
    max_height: 720,
    max_count: 1,
  },
  profile_doc: { max_bytes: 16 * kib, max_count: 1 },
  square_manifest: {
    max_bytes: 256 * kib,
    content_types: ['application/json'],
    max_count: 1,
    max_items: 101,
  },
  square_image_sd: {
    max_bytes: 1 * mib,
    content_types: ['image/jpeg', 'image/png', 'image/webp'],
    max_width: 1600,
    max_height: 1600,
  },
  square_image_hd: {
    max_bytes: 3 * mib,
    content_types: ['image/jpeg', 'image/png', 'image/webp'],
    max_width: 2560,
    max_height: 2560,
  },
  square_cover: {
    max_bytes: 3 * mib,
    content_types: ['image/jpeg', 'image/png', 'image/webp'],
    max_width: 2560,
    max_height: 2560,
    max_count: 1,
  },
  square_video_sd: {
    max_bytes: 40 * mib,
    content_types: ['video/mp4', 'video/webm'],
    max_seconds: 60,
    max_width: 854,
    max_height: 854,
    max_count: 1,
  },
  square_video_hd: {
    max_bytes: 1536 * mib,
    content_types: ['video/mp4', 'video/webm'],
    max_seconds: 30 * 60,
    max_width: 1920,
    max_height: 1920,
    max_count: 1,
  },
  square_video_candidate: {
    max_bytes: 8 * gib,
    content_types: ['video/mp4', 'video/webm'],
    max_seconds: 3 * 60 * 60,
    max_width: 1920,
    max_height: 1920,
    max_count: 1,
  },
  chat_device: { max_bytes: 16 * kib, max_count: 8 },
  chat_keypackage: { max_bytes: 128 * kib, max_count: 20, ttl_seconds: 7 * 24 * 60 * 60 },
  chat_envelope: { max_bytes: 256 * kib },
  chat_signal: { max_bytes: 64 * kib },
  push_wake: { max_bytes: 1 * kib },
  stripe_webhook: { max_bytes: 256 * kib },
  stream_webhook: { max_bytes: 64 * kib },
  chain_extrinsic: { max_bytes: 64 * kib },
  chain_extrinsic_json: { max_bytes: 132 * kib },
  chain_rpc_response: { max_bytes: 4 * mib },
  session_cache: { max_bytes: 4 * kib, max_count: 1 },
  session_index: { max_bytes: 4 * kib, max_count: 8 },
  identity_cache: { max_bytes: 4 * kib, max_count: 1 },
  // 单条联系人只包含小型端到端密文；限制整个 JSON 请求，防止借同步接口写入大对象。
  contact_ciphertext: { max_bytes: 16 * kib, max_items: 100 },
  api_json_small: { max_bytes: 16 * kib },
  api_json: { max_bytes: 128 * kib },
};

export interface UsageLimit {
  monthly_images: number;
  monthly_video_seconds: number;
  active_uploads: number;
}

export const usageLimits: Readonly<Record<MembershipLevel, UsageLimit>> = {
  freedom: { monthly_images: 300, monthly_video_seconds: 30 * 60, active_uploads: 1 },
  democracy: { monthly_images: 1500, monthly_video_seconds: 180 * 60, active_uploads: 2 },
  voting: { monthly_images: 1500, monthly_video_seconds: 180 * 60, active_uploads: 2 },
  candidate: { monthly_images: 5000, monthly_video_seconds: 1800 * 60, active_uploads: 3 },
};

export function resourceLimit(key: ResourceKey): ResourceLimit {
  return resourceLimits[key];
}

export function imageResource(level: MembershipLevel, cover: boolean): ResourceKey {
  if (cover) return 'square_cover';
  return level === 'freedom' ? 'square_image_sd' : 'square_image_hd';
}

export function videoResource(level: MembershipLevel): ResourceKey {
  if (level === 'candidate') return 'square_video_candidate';
  return level === 'freedom' ? 'square_video_sd' : 'square_video_hd';
}

interface RouteLimit {
  method: string;
  path: RegExp;
  resource_key: ResourceKey;
}

const route = (method: string, path: RegExp, resource_key: ResourceKey = 'api_json_small'): RouteLimit => ({
  method,
  path,
  resource_key,
});

/** 已登记路由是 Worker 进入风控和 D1 前的白名单。 */
const routeLimits: readonly RouteLimit[] = [
  route('GET', /^\/health$/),
  route('GET', /^\/v1\/chain\/bootstrap$/),
  route('GET', /^\/v1\/constitution$/),
  route('GET', /^\/v1\/security\/(turnstile|config)$/),
  route('POST', /^\/v1\/chain\/extrinsics\/relay$/, 'chain_extrinsic_json'),
  route('POST', /^\/v1\/square\/auth\/(challenge|session)$/),
  route('POST', /^\/v1\/square\/auth\/device\/register$/),
  route('GET', /^\/v1\/square\/membership$/),
  route('POST', /^\/v1\/square\/membership\/(subscribe|cancel|prepaid)\/challenge$/),
  route('POST', /^\/v1\/square\/membership\/(subscribe|cancel|prepaid)$/),
  route('POST', /^\/v1\/square\/membership\/prepaid\/change(?:\/challenge)?$/),
  route('POST', /^\/v1\/square\/membership\/webhook$/, 'stripe_webhook'),
  route('POST', /^\/v1\/square\/account\/delete(?:\/challenge)?$/),
  route('GET', /^\/v1\/square\/contacts$/),
  route('PUT', /^\/v1\/square\/contacts\/[^/]+$/, 'contact_ciphertext'),
  route('DELETE', /^\/v1\/square\/contacts\/[^/]+$/),
  route('POST', /^\/v1\/square\/uploads\/prepare$/, 'api_json'),
  route('PUT', /^\/v1\/square\/uploads\/manifest$/, 'square_manifest'),
  route('PUT', /^\/v1\/square\/uploads\/media$/, 'square_image_hd'),
  route('POST', /^\/v1\/square\/uploads\/complete$/),
  route('POST', /^\/v1\/square\/uploads\/stream\/webhook$/, 'stream_webhook'),
  route('POST', /^\/v1\/square\/posts\/confirm$/),
  route('DELETE', /^\/v1\/square\/posts\/[^/]+$/),
  route('GET', /^\/v1\/square\/media\/.+$/),
  route('GET', /^\/v1\/square\/feed\/(recommended|following|campaign)$/),
  route('PUT', /^\/v1\/square\/profile$/),
  route('POST', /^\/v1\/square\/profile\/assets\/prepare$/),
  route('PUT', /^\/v1\/square\/profile\/assets$/, 'profile_banner'),
  route('GET', /^\/v1\/square\/users\/[^/]+(?:\/(posts|follows))?$/),
  route('POST', /^\/v1\/square\/follows$/),
  route('DELETE', /^\/v1\/square\/follows\/[^/]+$/),
  route('POST', /^\/v1\/square\/signals$/),
  route('POST', /^\/v1\/chat\/devices\/register$/, 'chat_device'),
  route('POST', /^\/v1\/chat\/keypackages$/, 'chat_keypackage'),
  route('POST', /^\/v1\/chat\/keypackages\/consume$/),
  route('GET', /^\/v1\/chat\/keypackages\/[^/]+$/),
  route('POST', /^\/v1\/chat\/envelopes$/, 'chat_envelope'),
  route('POST', /^\/v1\/chat\/signals$/, 'chat_signal'),
  route('GET', /^\/v1\/chat\/ws$/),
];

export function routeResource(method: string, path: string): ResourceKey | null {
  const normalizedMethod = method.toUpperCase();
  const match = routeLimits.find((item) =>
    (normalizedMethod === 'OPTIONS' || item.method === normalizedMethod) && item.path.test(path));
  return match?.resource_key ?? null;
}
