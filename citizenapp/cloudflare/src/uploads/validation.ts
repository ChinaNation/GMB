import type { PostCategory, UploadItemInput } from '../types';
import { HttpError } from '../shared/http';
import { isSha256Hex } from '../shared/hash';
import { resourceLimit } from '../limits/catalog';

export function assertPostCategory(value: unknown): PostCategory {
  if (value === 'normal' || value === 'campaign') {
    return value;
  }
  throw new HttpError(400, 'invalid_post_category', '动态分类不合法');
}

export function assertManifestHash(value: unknown): string {
  if (isSha256Hex(value)) {
    return value.toLowerCase();
  }
  throw new HttpError(400, 'invalid_manifest_hash', 'manifest_hash 必须是 sha256 hex');
}

export function validateUploadItems(value: unknown): UploadItemInput[] {
  const maxMediaItems = resourceLimit('square_manifest').max_items ?? 0;
  if (!Array.isArray(value) || value.length === 0 || value.length > maxMediaItems) {
    throw new HttpError(400, 'invalid_media_items', '媒体数量必须在 1 到 101 个之间');
  }

  return value.map((raw, index) => {
    if (typeof raw !== 'object' || raw === null) {
      throw new HttpError(400, 'invalid_media_item', `第 ${index + 1} 个媒体不合法`);
    }

    const item = raw as Partial<UploadItemInput>;
    if (item.media_kind !== 'image' && item.media_kind !== 'video' && item.media_kind !== 'cover') {
      throw new HttpError(400, 'invalid_media_kind', `第 ${index + 1} 个媒体类型不合法`);
    }
    if (typeof item.content_type !== 'string') {
      throw new HttpError(400, 'invalid_content_type', `第 ${index + 1} 个媒体 content_type 不合法`);
    }
    const byteSize = item.byte_size;
    if (!Number.isInteger(byteSize) || byteSize === undefined || byteSize <= 0) {
      throw new HttpError(400, 'invalid_byte_size', `第 ${index + 1} 个媒体大小不合法`);
    }

    if (item.media_kind === 'video') {
      if (!['video/mp4', 'video/webm'].includes(item.content_type)) {
        throw new HttpError(400, 'invalid_video_type', '视频只允许 mp4 或 webm');
      }
      if (byteSize > resourceLimit('square_video_candidate').max_bytes) {
        throw new HttpError(400, 'video_too_large', '单个视频体积超出上限');
      }
      if (
        !Number.isInteger(item.duration_seconds) ||
        item.duration_seconds === undefined ||
        item.duration_seconds <= 0
      ) {
        throw new HttpError(400, 'invalid_video_duration', '视频必须提供真实 duration_seconds');
      }
    } else {
      if (!['image/jpeg', 'image/png', 'image/webp'].includes(item.content_type)) {
        throw new HttpError(400, 'invalid_image_type', '图片只允许 jpg、png 或 webp');
      }
      if (byteSize > resourceLimit('square_image_hd').max_bytes) {
        throw new HttpError(400, 'image_too_large', '单张图片超过统一资源上限');
      }
    }

    return {
      media_kind: item.media_kind,
      content_type: item.content_type,
      byte_size: byteSize,
      duration_seconds: item.media_kind === 'video' ? item.duration_seconds : undefined,
      file_ext: typeof item.file_ext === 'string' ? item.file_ext : undefined
    };
  });
}

export function estimateUploadBytes(mediaItems: UploadItemInput[]): number {
  return mediaItems.reduce((sum, item) => sum + item.byte_size, resourceLimit('square_manifest').max_bytes);
}
