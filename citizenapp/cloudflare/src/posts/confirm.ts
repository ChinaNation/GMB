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
import { storedMediaReleaseStatements } from '../limits/usage';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';
import { loadMediaAssets } from '../uploads/service';
import { requireActiveMembership } from '../membership/service';
import { readProfileDoc } from '../profiles/repository';
import { assertMembershipLevel, membershipPlan } from '../membership/plans';
import { assertIdentityCanPublishCategory, assertManifestQuota } from '../uploads/quota';
import { fetchChainIdentityState } from '../chain/identity';
import { uploadObjectKeys } from '../storage/r2_keys';
import {
  manifestObjectKeyFromUpload,
  readVerifiedSquarePostManifest,
  type SquarePostManifest,
} from './manifest';

interface ConfirmRequest {
  post_id?: unknown;
  block_hash?: unknown;
  tx_hash?: unknown;
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
  // 删除自己的动态 = off-chain 操作;归属按身份主键 cid_number(非签名账户)。
  if (post.cid_number !== session.cid_number) {
    throw new HttpError(403, 'post_owner_mismatch', '登录身份与动态作者不一致');
  }

  return deletePostCloudflareDataByCid(
    env,
    session.cid_number,
    postId,
    nowMs(),
  );
}

/**
 * 按身份主键硬删除一个内容项。除已发布帖子外，也允许删除尚未发布但已占用
 * Images/Stream/R2 的上传项，供权益到期清理覆盖完整云端内容。
 */
export async function deletePostCloudflareDataByCid(
  env: Env,
  cidNumber: string,
  postId: string,
  updatedAt: number,
): Promise<{
  deleted_media_assets: number;
  deleted_r2_objects: number;
}> {
  const upload = await loadUploadForPost(env, postId);
  if (upload && upload.cid_number !== cidNumber) {
    throw new HttpError(409, 'content_owner_mismatch', '内容项与身份归属不一致');
  }
  const indexedPost = await loadPostForDeleteOrNull(env, postId);
  if (indexedPost && indexedPost.cid_number !== cidNumber) {
    throw new HttpError(409, 'content_owner_mismatch', '内容项与身份归属不一致');
  }
  if (indexedPost && !upload) {
    throw new HttpError(409, 'post_upload_index_missing', '已发布内容缺少上传对象索引');
  }
  const mediaAssets = upload ? await loadMediaAssets(env, upload.upload_id) : [];
  // 严格清单校验必须早于 provider、R2 和 D1 的任何删除副作用。
  const objectKeys = upload ? uploadObjectKeys(upload) : [];

  for (const asset of mediaAssets) {
    await deleteProviderAsset(env, asset);
  }
  for (const objectKey of objectKeys) {
    await env.SQUARE_MEDIA.delete(objectKey);
  }

  // 外部资源全部删除成功后，再以同一个原子 batch 回收总量并删除 D1 行。
  // 失败时 D1 索引完整保留，下一轮仍能继续定位；链上 content_hash 不受影响。
  const statements = [
    ...storedMediaReleaseStatements(env, mediaAssets, updatedAt),
    env.DB.prepare(
      `DELETE FROM square_posts WHERE post_id = ? AND cid_number = ?`
    ).bind(postId, cidNumber)
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
  const membership = await requireActiveMembership(
    env,
    session.cid_number,
    session.account_id,
  );
  if (typeof body.post_id !== 'string' || body.post_id.trim().length === 0) {
    throw new HttpError(400, 'invalid_post_id', '动态编号不合法');
  }
  if (typeof body.block_hash !== 'string' || !body.block_hash.startsWith('0x')) {
    throw new HttpError(400, 'invalid_block_hash', '区块哈希不合法');
  }

  const upload = await loadCompletedUpload(env, body.post_id.trim());
  // 上传归属按身份主键 cid_number(非签名账户)。
  if (upload.cid_number !== session.cid_number) {
    throw new HttpError(403, 'upload_owner_mismatch', '登录身份与上传记录不一致');
  }
  if (!upload.content_hash || !upload.storage_receipt_id) {
    throw new HttpError(409, 'upload_not_completed', '上传任务尚未完成');
  }

  const eventsHex = await fetchSystemEventsAtBlock(env, body.block_hash);
  const event = findMatchingEvent(decodeSquarePostPublishedEvents(eventsHex), upload);
  if (!event) {
    throw new HttpError(409, 'square_event_not_found', '指定区块没有匹配的广场发布事件');
  }
  // 身份主键 cid_number 在 D1 为 NOT NULL:发布事件必须携带发布者 CID,否则拒绝镜像。
  if (!event.cid_number) {
    throw new HttpError(409, 'square_event_cid_missing', '广场发布事件缺少身份主键 CID');
  }
  const authorCidNumber = event.cid_number;

  const manifestObjectKey = manifestObjectKeyFromUpload(upload);
  const verifiedManifest = await readVerifiedSquarePostManifest(env, manifestObjectKey, {
    account_id: upload.account_id,
    post_category: upload.post_category,
    content_hashes: [upload.manifest_hash, upload.content_hash],
  });
  const manifest = verifiedManifest.manifest;
  const mediaAssets = await loadMediaAssets(env, upload.upload_id);
  const membershipLevel = assertMembershipLevel(membership.membership_level);
  // 发布确认再次按身份档校验分类权限（竞选内容须竞选身份，防上传后身份变化绕过）；
  // 只有竞选帖才读链身份。用量额度另按会员档由 assertManifestQuota 校验。
  if (upload.post_category === 'campaign') {
    const identity = await fetchChainIdentityState(env, session.account_id);
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
  const contentFormat = verifiedManifest.content_format;
  const title = typeof manifest.title === 'string' ? manifest.title : null;
  const createdAt = nowMs();

  await env.DB.prepare(
    `INSERT OR REPLACE INTO square_posts
      (post_id, account_id, cid_number, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'published')`
  )
    .bind(
      upload.post_id,
      upload.account_id,
      authorCidNumber,
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

  // 发帖通知扇出：读作者展示名一次并入队；队列消费者分页跨调用推给全部未静音粉丝。
  // 入队失败只 log、绝不回滚已发布的帖子（链上已 finalized、D1 已镜像）。
  try {
    const authorDoc = await readProfileDoc(env, authorCidNumber);
    await env.SQUARE_NOTIFY_QUEUE?.send({
      author_cid_number: authorCidNumber,
      author_name: authorDoc?.display_name ?? '',
      content_format: contentFormat,
      post_id: upload.post_id,
    });
  } catch (error) {
    console.error(
      `[square-notify] enqueue failed for ${upload.post_id}: ${error instanceof Error ? error.message : error}`,
    );
  }

  return {
    post_id: upload.post_id,
    account_id: upload.account_id,
    cid_number: authorCidNumber,
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
  const manifest = upload ? await readFeedManifest(env, row, upload) : null;
  return {
    ...row,
    // 文章正文图文块随 feed/详情回传（阅读侧按块渲染，内联图 media_index 引用 media_items）。
    content_blocks: manifest?.content_blocks ?? null,
    media_items: manifest && upload
      ? await manifestMediaItems(env, manifest, await loadMediaAssets(env, upload.upload_id))
      : []
  };
}

async function readFeedManifest(
  env: Env,
  row: SquarePostFeedItem,
  upload: PreparedUploadRow,
): Promise<SquarePostManifest | null> {
  try {
    const verified = await readVerifiedSquarePostManifest(
      env,
      manifestObjectKeyFromUpload(upload),
      {
        account_id: row.account_id,
        post_category: row.post_category,
        content_format: row.content_format,
        content_hashes: [
          row.content_hash,
          upload.manifest_hash,
          ...(upload.content_hash ? [upload.content_hash] : []),
        ],
      },
    );
    return verified.manifest;
  } catch {
    // 公共 feed 的 D1 正文仍可展示；损坏 manifest 只禁止文章块/媒体声明进入响应，
    // 且绝不恢复按账户/post_id 猜 R2 路径的旧兜底。
    return null;
  }
}

function findMatchingEvent(
  events: SquarePostPublishedEvent[],
  upload: PreparedUploadRow
): SquarePostPublishedEvent | null {
  return (
    events.find(
      (event) =>
        event.post_id === upload.post_id &&
        event.cid_number === upload.cid_number &&
        event.account_id === upload.account_id &&
        event.post_category === upload.post_category &&
        normalizeHash(event.content_hash) === normalizeHash(upload.content_hash ?? '') &&
        event.storage_receipt_id === upload.storage_receipt_id
    ) ?? null
  );
}

async function loadCompletedUpload(env: Env, postId: string): Promise<PreparedUploadRow> {
  const upload = await env.DB.prepare(
    `SELECT upload_id, post_id, cid_number, account_id, post_category, manifest_hash, content_hash,
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
    `SELECT upload_id, post_id, cid_number, account_id, post_category, manifest_hash, content_hash,
        storage_receipt_id, estimated_bytes, object_keys_json, status, expires_at, created_at, completed_at
      FROM square_uploads
      WHERE post_id = ?`
  )
    .bind(postId)
    .first<PreparedUploadRow>();
}

async function loadPostForDelete(env: Env, postId: string): Promise<SquarePostFeedItem> {
  const post = await loadPostForDeleteOrNull(env, postId);
  if (!post) {
    throw new HttpError(404, 'post_not_found', '动态不存在');
  }
  return post;
}

async function loadPostForDeleteOrNull(
  env: Env,
  postId: string
): Promise<SquarePostFeedItem | null> {
  return env.DB.prepare(
    `SELECT post_id, account_id, cid_number, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state
      FROM square_posts
      WHERE post_id = ?`
  )
    .bind(postId)
    .first<SquarePostFeedItem>();
}

function decodePostId(rawPostId: string): string {
  try {
    return decodeURIComponent(rawPostId).trim();
  } catch {
    throw new HttpError(400, 'invalid_post_id', '动态编号不合法');
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
    const signed = asset && asset.asset_state === 'ready'
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
      height: asset?.height ?? null
    };
  }));
}

function normalizeHash(value: string): string {
  return value.startsWith('0x') ? value.toLowerCase() : `0x${value.toLowerCase()}`;
}
