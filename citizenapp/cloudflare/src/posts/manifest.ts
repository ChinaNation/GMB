import { resourceLimit } from '../limits/catalog';
import { HttpError } from '../shared/http';
import { isSha256Hex, sha256Hex } from '../shared/hash';
import type { Env, PreparedUploadRow } from '../types';
import { uploadObjectKeys } from '../storage/r2_keys';

export interface SquareManifestMediaItem {
  media_kind: 'image' | 'video';
  file_name?: string;
  content_type?: string;
  byte_size?: number;
  sha256?: string;
}

// 文章正文图文块（内联图按 media_index 引用 media_items）；动态无此字段。
export interface SquareManifestContentBlock {
  t: 'text' | 'image';
  text?: string;
  media_index?: number;
}

export interface SquarePostManifest {
  schema: 'citizenapp.square.post';
  account_id: string;
  post_category: 'normal' | 'campaign';
  content_format?: 'normal' | 'article';
  title?: string;
  text: string;
  content_blocks?: SquareManifestContentBlock[];
  media_items: SquareManifestMediaItem[];
}

export interface ManifestAnchors {
  account_id: string;
  post_category: 'normal' | 'campaign';
  content_format?: 'normal' | 'article';
  // 同一份 manifest 可同时受 D1 帖子哈希、上传 content_hash 和 manifest_hash 约束。
  content_hashes?: readonly string[];
}

export interface VerifiedSquarePostManifest {
  manifest: SquarePostManifest;
  bytes: Uint8Array;
  content_hash: string;
  content_format: 'normal' | 'article';
}

/**
 * 从上传记录的服务端对象清单解析唯一 manifest 键。
 *
 * 对象清单必须与 account_id + post_id 生成的唯一规范 manifest 路径逐项完全一致；
 * 缺失、损坏、额外键或错误路径全部 fail-closed。
 */
export function manifestObjectKeyFromUpload(
  upload: Pick<PreparedUploadRow, 'account_id' | 'post_id' | 'object_keys_json'>
): string {
  return uploadObjectKeys(upload)[0];
}

/**
 * 读取并验证 R2 中参与链上 content_hash 的原始 manifest 字节。
 *
 * 返回的 bytes 是 R2 原始字节，不经过 JSON 重编码；客户端本地副本必须保存这份字节，
 * 才能持续复核同一个 SHA-256。manifest 最大 256KiB，因此此处有界缓冲是安全的。
 */
export async function readVerifiedSquarePostManifest(
  env: Env,
  objectKey: string,
  anchors: ManifestAnchors,
): Promise<VerifiedSquarePostManifest> {
  const object = await env.SQUARE_MEDIA.get(objectKey);
  if (!object) {
    throw new HttpError(409, 'manifest_not_found', 'R2 manifest 不存在');
  }
  const maxBytes = resourceLimit('square_manifest').max_bytes;
  if (object.size > maxBytes) {
    await object.body.cancel();
    throw new HttpError(409, 'manifest_stored_too_large', 'R2 manifest 超过服务端上限');
  }

  const bytes = new Uint8Array(await object.arrayBuffer());
  if (bytes.byteLength === 0 || bytes.byteLength > maxBytes) {
    throw new HttpError(409, 'manifest_bytes_invalid', 'R2 manifest 字节长度不合法');
  }
  const contentHash = await sha256Hex(bytes);
  for (const expected of anchors.content_hashes ?? []) {
    if (normalizeSha256(expected) !== contentHash) {
      throw new HttpError(409, 'manifest_hash_mismatch', 'R2 manifest 与发布哈希不一致');
    }
  }

  const manifest = decodeManifest(bytes);
  if (manifest.account_id !== anchors.account_id) {
    throw new HttpError(409, 'manifest_account_mismatch', 'manifest 账户标识不一致');
  }
  if (manifest.post_category !== anchors.post_category) {
    throw new HttpError(409, 'manifest_category_mismatch', 'manifest 动态分类不一致');
  }
  const contentFormat = manifest.content_format ?? 'normal';
  if (anchors.content_format !== undefined && contentFormat !== anchors.content_format) {
    throw new HttpError(409, 'manifest_content_format_mismatch', 'manifest 内容形态不一致');
  }

  return {
    manifest,
    bytes,
    content_hash: contentHash,
    content_format: contentFormat,
  };
}

export function normalizeSha256(value: string): string {
  const normalized = value.startsWith('0x') ? value.slice(2) : value;
  if (!isSha256Hex(normalized)) {
    throw new HttpError(409, 'invalid_content_hash', '发布哈希不是合法 SHA-256');
  }
  return normalized.toLowerCase();
}

function decodeManifest(bytes: Uint8Array): SquarePostManifest {
  let text: string;
  try {
    text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  } catch {
    throw new HttpError(409, 'manifest_utf8_invalid', 'R2 manifest 不是合法 UTF-8');
  }

  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    throw new HttpError(409, 'manifest_json_invalid', 'R2 manifest 不是合法 JSON');
  }
  if (!isRecord(value)) {
    throw new HttpError(409, 'manifest_json_invalid', 'R2 manifest 必须是 JSON 对象');
  }
  if (value.schema !== 'citizenapp.square.post') {
    throw new HttpError(409, 'invalid_manifest_schema', 'R2 manifest schema 不合法');
  }
  if (!isAccountId(value.account_id)) {
    throw new HttpError(409, 'invalid_manifest_account', 'manifest account_id 不合法');
  }
  if (value.post_category !== 'normal' && value.post_category !== 'campaign') {
    throw new HttpError(409, 'invalid_manifest_category', 'manifest post_category 不合法');
  }
  if (
    value.content_format !== undefined &&
    value.content_format !== 'normal' &&
    value.content_format !== 'article'
  ) {
    throw new HttpError(409, 'invalid_manifest_content_format', 'manifest content_format 不合法');
  }
  if (typeof value.text !== 'string' || !Array.isArray(value.media_items)) {
    throw new HttpError(409, 'manifest_content_incomplete', 'manifest 正文或媒体声明不完整');
  }
  if (!value.media_items.every(isMediaItem)) {
    throw new HttpError(409, 'manifest_media_invalid', 'manifest 媒体声明不合法');
  }
  if (value.title !== undefined && typeof value.title !== 'string') {
    throw new HttpError(409, 'manifest_title_invalid', 'manifest 标题不合法');
  }
  if (
    value.content_blocks !== undefined &&
    (!Array.isArray(value.content_blocks) || !value.content_blocks.every(isContentBlock))
  ) {
    throw new HttpError(409, 'manifest_content_blocks_invalid', 'manifest 图文块不合法');
  }

  const manifest: SquarePostManifest = {
    schema: 'citizenapp.square.post',
    account_id: value.account_id,
    post_category: value.post_category,
    text: value.text,
    media_items: value.media_items.filter(isMediaItem),
  };
  if (value.content_format === 'normal' || value.content_format === 'article') {
    manifest.content_format = value.content_format;
  }
  if (typeof value.title === 'string') {
    manifest.title = value.title;
  }
  if (Array.isArray(value.content_blocks)) {
    manifest.content_blocks = value.content_blocks.filter(isContentBlock);
  }
  return manifest;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isAccountId(value: unknown): value is string {
  return typeof value === 'string' && /^0x[0-9a-f]{64}$/.test(value);
}

function isMediaItem(value: unknown): value is SquareManifestMediaItem {
  if (!isRecord(value) || (value.media_kind !== 'image' && value.media_kind !== 'video')) {
    return false;
  }
  return optionalString(value.file_name) &&
    optionalString(value.content_type) &&
    optionalNonNegativeNumber(value.byte_size) &&
    (value.sha256 === undefined || isSha256Hex(value.sha256));
}

function isContentBlock(value: unknown): value is SquareManifestContentBlock {
  if (!isRecord(value) || (value.t !== 'text' && value.t !== 'image')) {
    return false;
  }
  if (value.t === 'text') {
    return typeof value.text === 'string' && value.media_index === undefined;
  }
  return Number.isSafeInteger(value.media_index) &&
    (value.media_index as number) >= 0 &&
    value.text === undefined;
}

function optionalString(value: unknown): boolean {
  return value === undefined || typeof value === 'string';
}

function optionalNonNegativeNumber(value: unknown): boolean {
  return value === undefined || (Number.isSafeInteger(value) && (value as number) >= 0);
}
