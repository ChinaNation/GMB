import type { MediaAssetRow, PostCategory, PostContentFormat, UploadItemInput } from '../types';
import { HttpError } from '../shared/http';
import { isSha256Hex } from '../shared/hash';
import type { MembershipLevel, MembershipPlan } from '../membership/plans';

interface DeclaredQuotaInput {
  membershipLevel: MembershipLevel;
  plan: MembershipPlan;
  postCategory: PostCategory;
  contentFormat: PostContentFormat;
  titleLength: number;
  textLength: number;
  mediaItems: UploadItemInput[];
}

interface ManifestQuotaInput {
  membershipLevel: MembershipLevel;
  plan: MembershipPlan;
  upload: {
    owner_account: string;
    post_category: PostCategory;
  };
  manifestText: string;
  mediaAssets: MediaAssetRow[];
}

interface SquareManifest {
  schema?: unknown;
  owner_account?: unknown;
  post_category?: unknown;
  content_format?: unknown;
  title?: unknown;
  text?: unknown;
  media_items?: unknown;
}

interface SquareManifestMediaItem {
  media_kind?: unknown;
  content_type?: unknown;
  byte_size?: unknown;
  sha256?: unknown;
  duration_seconds?: unknown;
}

export function assertContentFormat(value: unknown): PostContentFormat {
  if (value === undefined || value === null || value === 'normal') {
    return 'normal';
  }
  if (value === 'article') {
    return 'article';
  }
  throw new HttpError(400, 'invalid_content_format', 'content_format 必须是 normal 或 article');
}

export function assertDeclaredLength(value: unknown, fieldName: 'title_length' | 'text_length'): number {
  if (Number.isInteger(value) && typeof value === 'number' && value >= 0) {
    return value;
  }
  throw new HttpError(400, `invalid_${fieldName}`, `${fieldName} 必须是非负整数`);
}

export function assertDeclaredContentQuota(input: DeclaredQuotaInput): void {
  assertMembershipCanPublishCategory(input.membershipLevel, input.postCategory);
  assertContentQuota(input);
}

export async function assertManifestQuota(input: ManifestQuotaInput): Promise<void> {
  const manifest = parseManifest(input.manifestText);
  if (manifest.schema !== 'citizenapp.square.post.v1') {
    throw new HttpError(400, 'invalid_manifest_schema', 'manifest schema 不合法');
  }
  if (manifest.owner_account !== input.upload.owner_account) {
    throw new HttpError(409, 'manifest_owner_mismatch', 'manifest owner_account 与上传任务不一致');
  }
  if (manifest.post_category !== input.upload.post_category) {
    throw new HttpError(409, 'manifest_post_category_mismatch', 'manifest post_category 与上传任务不一致');
  }

  const contentFormat = assertContentFormat(manifest.content_format);
  const text = typeof manifest.text === 'string' ? manifest.text.trim() : null;
  if (text === null) {
    throw new HttpError(400, 'invalid_manifest_text', 'manifest text 必须是字符串');
  }
  const title = manifest.title === undefined || manifest.title === null ? '' : manifest.title;
  if (typeof title !== 'string') {
    throw new HttpError(400, 'invalid_manifest_title', 'manifest title 必须是字符串');
  }
  const mediaItems = parseManifestMediaItems(manifest.media_items);
  assertManifestMatchesAssets(mediaItems, input.mediaAssets);
  assertDeclaredContentQuota({
    membershipLevel: input.membershipLevel,
    plan: input.plan,
    postCategory: input.upload.post_category,
    contentFormat,
    titleLength: title.trim().length,
    textLength: text.length,
    mediaItems
  });
}

function assertMembershipCanPublishCategory(
  membershipLevel: MembershipLevel,
  postCategory: PostCategory
): void {
  if (postCategory === 'campaign' && membershipLevel !== 'candidate') {
    throw new HttpError(403, 'campaign_membership_required', '只有竞选公民会员可以发布竞选内容');
  }
}

function assertContentQuota(input: DeclaredQuotaInput): void {
  if (input.contentFormat === 'article') {
    assertArticleQuota(input);
    return;
  }
  assertDynamicQuota(input);
}

function assertDynamicQuota(input: DeclaredQuotaInput): void {
  if (input.textLength > input.plan.dynamic.text_max_chars) {
    throw new HttpError(
      400,
      'dynamic_text_too_long',
      `动态文字不能超过 ${input.plan.dynamic.text_max_chars} 字`
    );
  }
  const imageCount = countMedia(input.mediaItems, 'image');
  const videoCount = countMedia(input.mediaItems, 'video');
  if (imageCount > input.plan.dynamic.max_images) {
    throw new HttpError(
      400,
      'dynamic_image_count_exceeded',
      `动态图片不能超过 ${input.plan.dynamic.max_images} 张`
    );
  }
  if (videoCount > input.plan.dynamic.max_videos) {
    throw new HttpError(
      400,
      'dynamic_video_count_exceeded',
      `动态视频不能超过 ${input.plan.dynamic.max_videos} 个`
    );
  }
  // 单视频体积直接引用会员套餐；套餐值由统一资源限制表生成，避免两套上限漂移。
  for (const item of input.mediaItems) {
    if (item.media_kind === 'video' && item.byte_size > input.plan.dynamic.max_video_bytes) {
      throw new HttpError(
        400,
        'dynamic_video_too_large',
        `单个视频不能超过 ${formatMaxVideoBytes(input.plan.dynamic.max_video_bytes)}`
      );
    }
    if (
      item.media_kind === 'video' &&
      (item.duration_seconds ?? 0) > input.plan.dynamic.max_video_seconds
    ) {
      throw new HttpError(
        400,
        'dynamic_video_too_long',
        `单个视频不能超过 ${input.plan.dynamic.max_video_seconds} 秒`
      );
    }
  }
}

function formatMaxVideoBytes(bytes: number): string {
  const gib = 1024 * 1024 * 1024;
  const mib = 1024 * 1024;
  return bytes % gib === 0 ? `${bytes / gib}GB` : `${Math.round(bytes / mib)}MB`;
}

function assertArticleQuota(input: DeclaredQuotaInput): void {
  if (
    input.titleLength < input.plan.article.title_min_chars ||
    input.titleLength > input.plan.article.title_max_chars
  ) {
    throw new HttpError(
      400,
      'article_title_invalid',
      `文章标题必须是 ${input.plan.article.title_min_chars}-${input.plan.article.title_max_chars} 字`
    );
  }
  if (input.textLength === 0) {
    throw new HttpError(400, 'article_body_required', '文章正文不能为空');
  }
  if (input.textLength > input.plan.article.body_max_chars) {
    throw new HttpError(
      400,
      'article_body_too_long',
      `文章正文不能超过 ${input.plan.article.body_max_chars} 字`
    );
  }
  const videoCount = countMedia(input.mediaItems, 'video');
  if (videoCount > 0) {
    throw new HttpError(400, 'article_video_not_allowed', '文章不能上传视频');
  }
  if (!isImageLike(input.mediaItems[0])) {
    throw new HttpError(400, 'article_cover_required', '文章必须上传 1 张首图');
  }
  const bodyImageCount = input.mediaItems.slice(1).filter(isImageLike).length;
  if (bodyImageCount > input.plan.article.max_images) {
    throw new HttpError(
      400,
      'article_image_count_exceeded',
      `文章正文图片不能超过 ${input.plan.article.max_images} 张`
    );
  }
}

function parseManifest(value: string): SquareManifest {
  try {
    const parsed = JSON.parse(value);
    if (typeof parsed === 'object' && parsed !== null) {
      return parsed as SquareManifest;
    }
  } catch {
    // 统一落到下面的结构错误，避免泄露解析细节。
  }
  throw new HttpError(400, 'invalid_manifest_json', 'manifest 不是合法 JSON 对象');
}

function parseManifestMediaItems(value: unknown): UploadItemInput[] {
  if (!Array.isArray(value) || value.length === 0) {
    throw new HttpError(400, 'invalid_manifest_media_items', 'manifest media_items 不能为空');
  }
  return value.map((raw, index) => {
    if (typeof raw !== 'object' || raw === null) {
      throw new HttpError(400, 'invalid_manifest_media_item', `第 ${index + 1} 个 manifest 媒体不合法`);
    }
    const item = raw as SquareManifestMediaItem;
    if (item.media_kind !== 'image' && item.media_kind !== 'video' && item.media_kind !== 'cover') {
      throw new HttpError(400, 'invalid_manifest_media_kind', `第 ${index + 1} 个 manifest 媒体类型不合法`);
    }
    if (typeof item.content_type !== 'string') {
      throw new HttpError(400, 'invalid_manifest_content_type', `第 ${index + 1} 个媒体 content_type 不合法`);
    }
    if (!Number.isInteger(item.byte_size) || typeof item.byte_size !== 'number' || item.byte_size <= 0) {
      throw new HttpError(400, 'invalid_manifest_byte_size', `第 ${index + 1} 个媒体大小不合法`);
    }
    if (!isSha256Hex(item.sha256)) {
      throw new HttpError(400, 'invalid_manifest_media_hash', `第 ${index + 1} 个媒体 sha256 不合法`);
    }
    return {
      media_kind: item.media_kind,
      content_type: item.content_type,
      byte_size: item.byte_size,
      duration_seconds: item.media_kind === 'video' && typeof item.duration_seconds === 'number'
        ? item.duration_seconds
        : undefined
    };
  });
}

function assertManifestMatchesAssets(mediaItems: UploadItemInput[], mediaAssets: MediaAssetRow[]): void {
  if (mediaItems.length !== mediaAssets.length) {
    throw new HttpError(409, 'manifest_media_count_mismatch', 'manifest 媒体数量与上传授权不一致');
  }
  for (const [index, item] of mediaItems.entries()) {
    const asset = mediaAssets[index];
    if (!asset) {
      throw new HttpError(409, 'manifest_media_asset_missing', `第 ${index + 1} 个媒体缺少上传资产`);
    }
    const manifestKind = item.media_kind === 'video' ? 'video' : 'image';
    if (
      asset.media_kind !== manifestKind ||
      asset.content_type !== item.content_type ||
      asset.byte_size !== item.byte_size ||
      (manifestKind === 'video' &&
        asset.declared_duration_seconds !== item.duration_seconds)
    ) {
      throw new HttpError(409, 'manifest_media_mismatch', `第 ${index + 1} 个媒体与上传授权不一致`);
    }
  }
}

function countMedia(mediaItems: UploadItemInput[], mediaKind: 'image' | 'video'): number {
  return mediaItems.filter((item) => (mediaKind === 'image' ? isImageLike(item) : item.media_kind === 'video')).length;
}

function isImageLike(item: UploadItemInput | undefined): boolean {
  return item?.media_kind === 'image' || item?.media_kind === 'cover';
}
