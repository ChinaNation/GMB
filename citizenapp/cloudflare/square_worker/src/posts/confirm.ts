import type {
  Env,
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
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';
import { sanitizeOwnerAccount } from '../storage/r2_keys';

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

interface SquarePostManifest {
  schema?: string;
  owner_account?: string;
  post_category?: 'normal' | 'campaign';
  text?: string;
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

export async function confirmPublishedPost(
  env: Env,
  session: SessionState,
  body: ConfirmRequest
): Promise<SquarePostFeedItem> {
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
  const mediaItems = manifestMediaItems(manifest, objectKeys);

  await env.DB.prepare(
    `INSERT OR REPLACE INTO square_posts
      (post_id, owner_account, cid_number, post_category, text, content_hash,
        storage_receipt_id, chain_block, created_at, post_state)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'published')`
  )
    .bind(
      upload.post_id,
      upload.owner_account,
      event.cid_number,
      upload.post_category,
      manifest.text ?? '',
      normalizeHash(upload.content_hash),
      upload.storage_receipt_id,
      event.created_block,
      nowMs()
    )
    .run();

  return {
    post_id: upload.post_id,
    owner_account: upload.owner_account,
    cid_number: event.cid_number,
    post_category: upload.post_category,
    text: manifest.text ?? '',
    content_hash: normalizeHash(upload.content_hash),
    storage_receipt_id: upload.storage_receipt_id,
    chain_block: event.created_block,
    created_at: nowMs(),
    post_state: 'published',
    media_items: mediaItems
  };
}

export async function buildFeedPostItem(env: Env, row: SquarePostFeedItem): Promise<SquarePostFeedItem> {
  const objectKeys = await loadObjectKeysForPost(env, row.post_id);
  const manifestObjectKey =
    objectKeys.find((key) => key.endsWith('/manifest.json')) ??
    `square/${sanitizeOwnerAccount(row.owner_account)}/posts/${row.post_id}/manifest.json`;
  const manifest = await readManifest(env, manifestObjectKey).catch(() => null);
  return {
    ...row,
    media_items: manifest ? manifestMediaItems(manifest, objectKeys) : []
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
        storage_receipt_id, estimated_bytes, object_keys_json, status, created_at, completed_at
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

async function loadObjectKeysForPost(env: Env, postId: string): Promise<string[]> {
  const upload = await env.DB.prepare(
    `SELECT upload_id, post_id, owner_account, post_category, manifest_hash, content_hash,
        storage_receipt_id, estimated_bytes, object_keys_json, status, created_at, completed_at
      FROM square_uploads
      WHERE post_id = ?`
  )
    .bind(postId)
    .first<PreparedUploadRow>();
  return upload ? parseObjectKeys(upload) : [];
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

function manifestMediaItems(
  manifest: SquarePostManifest,
  objectKeys: string[]
): SquareFeedMediaItem[] {
  const mediaKeys = objectKeys.filter((key) => !key.endsWith('/manifest.json'));
  const items = Array.isArray(manifest.media_items) ? manifest.media_items : [];
  return items.map((item, index) => ({
    media_kind: item.media_kind === 'video' ? 'video' as const : 'image' as const,
    object_key: mediaKeys[index] ?? '',
    url: mediaKeys[index] ?? '',
    content_type: item.content_type ?? 'application/octet-stream',
    byte_size: item.byte_size ?? 0,
    sha256: item.sha256 ?? ''
  }));
}

function normalizeHash(value: string): string {
  return value.startsWith('0x') ? value.toLowerCase() : `0x${value.toLowerCase()}`;
}
