import type {
  Env,
  MediaAssetRow,
  PreparedUploadRow,
  SessionState,
  SquareFeedMediaItem,
  SquarePostFeedItem
} from '../types';
import { fetchSystemEventsAtBlock } from '../chain/rpc';
import {
  decodeSquarePostPublishedEvents,
  type SquarePostPublishedEvent
} from '../chain/square_event';
import { deleteProviderAsset } from '../media/cloudflare_assets';
import { signedMediaUrls } from '../media/signed_urls';
import { releaseStoredMedia } from '../limits/usage';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';
import { sanitizeOwnerAccount } from '../storage/r2_keys';
import { loadMediaAssets } from '../uploads/service';
import { requireActiveMembership } from '../membership/service';
import { assertMembershipLevel, membershipPlan } from '../membership/plans';
import { assertIdentityCanPublishCategory, assertManifestQuota } from '../uploads/quota';
import { fetchChainIdentityState } from '../chain/identity';

interface ConfirmRequest {
  post_id?: unknown;
  block_hash?: unknown;
  tx_hash?: unknown;
}

interface SquareManifestMediaItem {
  media_kind: 'image' | 'video';
  file_name?: string;
  content_type?: string;
  byte_size?: number;
  sha256?: string;
}

// 文章正文图文块（内联图按 media_index 引用 media_items）；动态无此字段。
interface SquareManifestContentBlock {
  t: 'text' | 'image';
  text?: string;
  media_index?: number;
}

interface SquarePostManifest {
  schema?: string;
  owner_account?: string;
  post_category?: 'normal' | 'campaign';
  content_format?: 'normal' | 'article';
  title?: string;
  text?: string;
  content_blocks?: SquareManifestContentBlock[];
  media_items?: SquareManifestMediaItem[];
}

export async function confirmPostRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ConfirmRequest>(request);
  const result = await confirmPublishedPost(env, session, body);
  return jsonResponse({
    ok: true,
    post: result
  });
}

export async function deletePostRoute(request: Request, env: Env, rawPostId: string): Promise<Response> {
  const session = await requireSession(request, env);
  const postId = decodePostId(rawPostId);
  const result = await deletePostCloudflareData(env, session, postId);
  return jsonResponse({
    ok: true,
    post_id: postId,
    post_state: 'deleted',
    cleanup: result
  });
}

export async function deletePostCloudflareData(
  env: Env,
  session: SessionState,
  postId: string
): Promise<{
  deleted_media_assets: number;
  deleted_r2_objects: number;
}> {
  if (postId.length === 0) {
    throw new HttpError(400, 'invalid_post_id', '动态编号不合法');
  }

  const post = await loadPostForDelete(env, postId);
  if (post.owner_account !== session.owner_account) {
    throw new HttpError(403, 'post_owner_mismatch', '登录钱包与动态作者不一致');
  }

  const upload = await loadUploadForPost(env, postId);
  const mediaAssets = upload ? await loadMediaAssets(env, upload.upload_id) : [];
  const objectKeys = upload ? parseObjectKeys(upload) : [];

  for (const asset of mediaAssets) {
    await deleteProviderAsset(env, asset);
  }
  await releaseStoredMedia(env, mediaAssets);
  for (const objectKey of objectKeys) {
    await env.SQUARE_MEDIA.delete(objectKey);
  }

  // 硬删除：彻底删掉帖子行本身，不留软删残行；链上仅存 content_hash 不受影响。
  const statements = [
    env.DB.prepare(
      `DELETE FROM square_posts WHERE post_id = ? AND owner_account = ?`
    ).bind(postId, session.owner_account)
  ];

  if (upload) {
    statements.push(
      env.DB.prepare('DELETE FROM square_media_assets WHERE upload_id = ?').bind(upload.upload_id)
    );
    // 一并删上传任务行，避免其 R2 对象已删后 D1 仍残留悬挂元数据。
    statements.push(
      env.DB.prepare('DELETE FROM square_uploads WHERE upload_id = ?').bind(upload.upload_id)
    );
  }
  await env.DB.batch(statements);

  return {
    deleted_media_assets: mediaAssets.length,
    deleted_r2_objects: objectKeys.length
  };
}

export async function confirmPublishedPost(
  env: Env,
  session: SessionState,
  body: ConfirmRequest
): Promise<SquarePostFeedItem> {
  // 发布确认是最后一道服务端闸门；会员在上传后失效也不得把链上事件投影为广场内容。
  const membership = await requireActiveMembership(env, session.owner_account);
  if (typeof body.post_id !== 'string' || body.post_id.trim().length === 0) {
    throw new HttpError(400, 'invalid_post_id', '动态编号不合法');
  }
  if (typeof body.block_hash !== 'string' || !body.block_hash.startsWith('0x')) {
    throw new HttpError(400, 'invalid_block_hash', '区块哈希不合法');
  }

  const upload = await loadCompletedUpload(env, body.post_id.trim());
  if (upload.owner_account !== session.owner_account) {
    throw new HttpError(403, 'upload_owner_mismatch', '登录钱包与上传记录不一致');
  }
  if (!upload.content_hash || !upload.storage_receipt_id) {
    throw new HttpError(409, 'upload_not_completed', '上传任务尚未完成');
  }

  const eventsHex = await fetchSystemEventsAtBlock(env, body.block_hash);
  const event = findMatchingEvent(decodeSquarePostPublishedEvents(eventsHex), upload);
  if (!event) {
    throw new HttpError(409, 'square_event_not_found', '指定区块没有匹配的广场发布事件');
  }

  const objectKeys = parseObjectKeys(upload);
  const manifestObjectKey = objectKeys.find((key) => key.endsWith('/manifest.json'));
  if (!manifestObjectKey) {
    throw new HttpError(409, 'manifest_object_missing', '上传记录缺少 manifest 对象');
  }
  const manifest = await readManifest(env, manifestObjectKey);
  validateManifest(manifest, upload);
  const mediaAssets = await loadMediaAssets(env, upload.upload_id);
  const membershipLevel = assertMembershipLevel(membership.membership_level);
  // 发布确认再次按身份档校验分类权限（竞选内容须竞选身份，防上传后身份变化绕过）；
  // 只有竞选帖才读链身份。用量额度另按会员档由 assertManifestQuota 校验。
  if (upload.post_category === 'campaign') {
    const identity = await fetchChainIdentityState(env, session.owner_account);
    assertIdentityCanPublishCategory(identity.identity_level, 'campaign');
  }
  await assertManifestQuota({
    membershipLevel,
    plan: membershipPlan(membershipLevel),
    upload,
    manifestText: JSON.stringify(manifest),
    mediaAssets
  });
  const mediaItems = await manifestMediaItems(env, manifest, mediaAssets);
  const contentFormat = manifest.content_format === 'article' ? 'article' : 'normal';
  const title = typeof manifest.title === 'string' ? manifest.title : null;
  const createdAt = nowMs();

  await env.DB.prepare(
    `INSERT OR REPLACE INTO square_posts
      (post_id, owner_account, cid_number, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'published')`
  )
    .bind(
      upload.post_id,
      upload.owner_account,
      event.cid_number,
      upload.post_category,
      contentFormat,
      title,
      manifest.text ?? '',
      normalizeHash(upload.content_hash),
      upload.storage_receipt_id,
      event.created_block,
      createdAt
    )
    .run();

  return {
    post_id: upload.post_id,
    owner_account: upload.owner_account,
    cid_number: event.cid_number,
    post_category: upload.post_category,
    content_format: contentFormat,
    title,
    text: manifest.text ?? '',
    content_blocks: manifest.content_blocks ?? null,
    content_hash: normalizeHash(upload.content_hash),
    storage_receipt_id: upload.storage_receipt_id,
    chain_block: event.created_block,
    created_at: createdAt,
    post_state: 'published',
    media_items: mediaItems
  };
}

export async function buildFeedPostItem(env: Env, row: SquarePostFeedItem): Promise<SquarePostFeedItem> {
  const upload = await loadUploadForPost(env, row.post_id);
  const objectKeys = upload ? parseObjectKeys(upload) : [];
  const manifestObjectKey =
    objectKeys.find((key) => key.endsWith('/manifest.json')) ??
    `square/${sanitizeOwnerAccount(row.owner_account)}/posts/${row.post_id}/manifest.json`;
  const manifest = await readManifest(env, manifestObjectKey).catch(() => null);
  return {
    ...row,
    // 文章正文图文块随 feed/详情回传（阅读侧按块渲染，内联图 media_index 引用 media_items）。
    content_blocks: manifest?.content_blocks ?? null,
    media_items: manifest && upload
      ? await manifestMediaItems(env, manifest, await loadMediaAssets(env, upload.upload_id))
      : []
  };
}

function findMatchingEvent(
  events: SquarePostPublishedEvent[],
  upload: PreparedUploadRow
): SquarePostPublishedEvent | null {
  return (
    events.find(
      (event) =>
        event.post_id === upload.post_id &&
        event.owner_account === upload.owner_account &&
        event.post_category === upload.post_category &&
        normalizeHash(event.content_hash) === normalizeHash(upload.content_hash ?? '') &&
        event.storage_receipt_id === upload.storage_receipt_id
    ) ?? null
  );
}

async function loadCompletedUpload(env: Env, postId: string): Promise<PreparedUploadRow> {
  const upload = await env.DB.prepare(
    `SELECT upload_id, post_id, owner_account, post_category, manifest_hash, content_hash,
        storage_receipt_id, estimated_bytes, object_keys_json, status, expires_at, created_at, completed_at
      FROM square_uploads
      WHERE post_id = ?`
  )
    .bind(postId)
    .first<PreparedUploadRow>();
  if (!upload) {
    throw new HttpError(404, 'upload_not_found', '上传记录不存在');
  }
  if (upload.status !== 'completed') {
    throw new HttpError(409, 'upload_not_completed', '上传任务尚未完成');
  }
  return upload;
}

async function loadUploadForPost(env: Env, postId: string): Promise<PreparedUploadRow | null> {
  return env.DB.prepare(
    `SELECT upload_id, post_id, owner_account, post_category, manifest_hash, content_hash,
        storage_receipt_id, estimated_bytes, object_keys_json, status, expires_at, created_at, completed_at
      FROM square_uploads
      WHERE post_id = ?`
  )
    .bind(postId)
    .first<PreparedUploadRow>();
}

async function loadPostForDelete(env: Env, postId: string): Promise<SquarePostFeedItem> {
  const post = await env.DB.prepare(
    `SELECT post_id, owner_account, cid_number, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state
      FROM square_posts
      WHERE post_id = ?`
  )
    .bind(postId)
    .first<SquarePostFeedItem>();
  if (!post) {
    throw new HttpError(404, 'post_not_found', '动态不存在');
  }
  return post;
}

function decodePostId(rawPostId: string): string {
  try {
    return decodeURIComponent(rawPostId).trim();
  } catch {
    throw new HttpError(400, 'invalid_post_id', '动态编号不合法');
  }
}

function parseObjectKeys(row: PreparedUploadRow): string[] {
  try {
    const parsed = JSON.parse(row.object_keys_json);
    return Array.isArray(parsed) ? parsed.filter((value) => typeof value === 'string') : [];
  } catch {
    return [];
  }
}

async function readManifest(env: Env, objectKey: string): Promise<SquarePostManifest> {
  const object = await env.SQUARE_MEDIA.get(objectKey);
  if (!object) {
    throw new HttpError(409, 'manifest_not_found', 'R2 manifest 不存在');
  }
  const data = JSON.parse(await object.text()) as SquarePostManifest;
  if (data.schema !== 'citizenapp.square.post.v1') {
    throw new HttpError(409, 'invalid_manifest_schema', 'R2 manifest schema 不合法');
  }
  return data;
}

function validateManifest(manifest: SquarePostManifest, upload: PreparedUploadRow): void {
  if (manifest.owner_account !== upload.owner_account) {
    throw new HttpError(409, 'manifest_owner_mismatch', 'manifest 钱包账户不一致');
  }
  if (manifest.post_category !== upload.post_category) {
    throw new HttpError(409, 'manifest_category_mismatch', 'manifest 动态分类不一致');
  }
}

async function manifestMediaItems(
  env: Env,
  manifest: SquarePostManifest,
  mediaAssets: MediaAssetRow[]
): Promise<SquareFeedMediaItem[]> {
  const items = Array.isArray(manifest.media_items) ? manifest.media_items : [];
  return Promise.all(items.map(async (item, index) => {
    const asset = mediaAssets[index];
    const mediaKind = item.media_kind === 'video' ? 'video' as const : 'image' as const;
    const signed = asset && asset.asset_state === 'ready' && asset.archive_state === 'live'
      ? await signedMediaUrls(env, asset)
      : { url: '', thumbnail_url: null };
    return {
      media_kind: mediaKind,
      object_key: asset?.provider_asset_id ?? '',
      url: signed.url,
      provider: asset?.provider ?? (mediaKind === 'video' ? 'cloudflare_stream' : 'cloudflare_images'),
      provider_asset_id: asset?.provider_asset_id ?? '',
      asset_state: asset?.asset_state ?? 'prepared',
      thumbnail_url: signed.thumbnail_url,
      content_type: item.content_type ?? asset?.content_type ?? 'application/octet-stream',
      byte_size: item.byte_size ?? asset?.byte_size ?? 0,
      sha256: item.sha256 ?? '',
      duration_seconds: asset?.duration_seconds ?? null,
      width: asset?.width ?? null,
      height: asset?.height ?? null,
      archive_state: asset?.archive_state ?? 'live'
    };
  }));
}

function normalizeHash(value: string): string {
  return value.startsWith('0x') ? value.toLowerCase() : `0x${value.toLowerCase()}`;
}
