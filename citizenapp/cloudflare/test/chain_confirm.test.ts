import { describe, expect, it, vi } from 'vitest';
import { encodeAddress } from '@polkadot/util-crypto';
import { confirmPublishedPost, deletePostCloudflareData } from '../src/posts/confirm';
import type { Env, MediaAssetRow, PreparedUploadRow, SessionState } from '../src/types';
import {
  compactBytes,
  compactU32,
  decodeSquarePostPublishedEvents,
  hex,
  u32Le,
  u64Le
} from '../src/chain/square_event';

const ownerAccountBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 1));
const ownerAccount = encodeAddress(ownerAccountBytes, 2027);
const postId = 'sqp_test';
const contentHash = `0x${'11'.repeat(32)}`;
const storageReceiptId = 'sqr_test';
const blockHash = `0x${'22'.repeat(32)}`;

describe('square chain confirmation', () => {
  it('decodes SquarePostPublished from System.Events bytes', () => {
    const eventsHex = buildEventsHex({
      cidNumber: 'CN001-CTZN-000000001-2026'
    });
    const events = decodeSquarePostPublishedEvents(eventsHex);

    expect(events).toHaveLength(1);
    expect(events[0]).toMatchObject({
      post_id: postId,
      owner_account: ownerAccount,
      cid_number: 'CN001-CTZN-000000001-2026',
      post_category: 'campaign',
      content_hash: contentHash,
      storage_receipt_id: storageReceiptId,
      storage_until: 1800000000000,
      created_block: 88
    });
  });

  it('confirms completed upload and writes published post', async () => {
    const db = new FakeDb();
    const upload: PreparedUploadRow = {
      upload_id: 'squ_test',
      post_id: postId,
      owner_account: ownerAccount,
      post_category: 'campaign',
      manifest_hash: contentHash.slice(2),
      content_hash: contentHash.slice(2),
      storage_receipt_id: storageReceiptId,
      estimated_bytes: 1024,
      object_keys_json: JSON.stringify([`square/${ownerAccount}/posts/${postId}/manifest.json`]),
      status: 'completed',
      created_at: 1,
      completed_at: 2
    };
    db.uploads.set(postId, upload);
    db.mediaAssets.set(upload.upload_id, [
      {
        upload_id: upload.upload_id,
        post_id: postId,
        owner_account: ownerAccount,
        media_index: 0,
        media_kind: 'image',
        provider: 'cloudflare_images',
        provider_asset_id: 'img_test',
        upload_method: 'direct_form',
        content_type: 'image/webp',
        byte_size: 1024,
        asset_state: 'ready',
        delivery_url: 'https://imagedelivery.net/account/img_test/public',
        playback_hls_url: null,
        playback_dash_url: null,
        thumbnail_url: null,
        duration_seconds: null,
        width: 1200,
        height: 800,
        error_code: null,
        created_at: 1,
        updated_at: 2,
        ready_at: 2
      }
    ]);
    const env = {
      DB: db,
      SQUARE_MEDIA: new FakeR2({
        [`square/${ownerAccount}/posts/${postId}/manifest.json`]: JSON.stringify({
          schema: 'citizenapp.square.post.v1',
          owner_account: ownerAccount,
          post_category: 'campaign',
          text: '竞选动态',
          media_items: [
            {
              media_kind: 'image',
              file_name: 'a.webp',
              content_type: 'image/webp',
              byte_size: 1024,
              sha256: 'aa'
            }
          ]
        })
      }),
      FEED_CACHE: {},
      SQUARE_CHAIN_RPC_URL: 'http://chain.test'
    } as unknown as Env;
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        Response.json({
          jsonrpc: '2.0',
          id: 1,
          result: buildEventsHex({
            cidNumber: 'CN001-CTZN-000000001-2026'
          })
        })
      )
    );

    const post = await confirmPublishedPost(env, session(), {
      post_id: postId,
      block_hash: blockHash
    });

    expect(post.text).toBe('竞选动态');
    expect(post.cid_number).toBe('CN001-CTZN-000000001-2026');
    expect(post.media_items?.[0]).toMatchObject({
      provider: 'cloudflare_images',
      provider_asset_id: 'img_test',
      url: 'https://imagedelivery.net/account/img_test/public',
      asset_state: 'ready'
    });
    expect(db.posts.get(postId)?.post_state).toBe('published');
  });

  it('hard-deletes Cloudflare-side post data and reclaims storage', async () => {
    const db = new FakeDb();
    const manifestKey = `square/${ownerAccount}/posts/${postId}/manifest.json`;
    const upload = completedUpload(manifestKey);
    db.uploads.set(postId, upload);
    db.mediaAssets.set(upload.upload_id, [imageAsset(upload.upload_id)]);
    db.membershipStorageUsed.set(ownerAccount, 4096);
    db.posts.set(postId, {
      post_id: postId,
      owner_account: ownerAccount,
      cid_number: 'CN001-CTZN-000000001-2026',
      post_category: 'normal',
      content_format: 'normal',
      title: '旧标题',
      text: '旧动态',
      content_hash: contentHash,
      storage_receipt_id: storageReceiptId,
      chain_block: 88,
      created_at: 1,
      post_state: 'published'
    });
    const r2 = new FakeR2({
      [manifestKey]: JSON.stringify({
        schema: 'citizenapp.square.post.v1',
        owner_account: ownerAccount,
        post_category: 'normal',
        text: '旧动态',
        media_items: []
      })
    });
    const env = {
      DB: db,
      SQUARE_MEDIA: r2,
      FEED_CACHE: {},
      SQUARE_DEV_UPLOAD_PROXY: '1'
    } as unknown as Env;

    const result = await deletePostCloudflareData(env, session(), postId);

    expect(result).toMatchObject({
      deleted_media_assets: 1,
      deleted_r2_objects: 1,
      reclaimed_storage_bytes: 1024
    });
    // 硬删除：帖子行 + 上传行 + 媒体资产 + R2 对象全部清空，无软删残行。
    expect(db.posts.has(postId)).toBe(false);
    expect(db.uploads.has(postId)).toBe(false);
    expect(db.mediaAssets.get(upload.upload_id)).toEqual([]);
    expect(r2.deletedKeys).toEqual([manifestKey]);
    expect(db.membershipStorageUsed.get(ownerAccount)).toBe(3072);

    // 再删同一帖子 → 已无残行，报 404，证明是彻底删除而非软删。
    await expect(deletePostCloudflareData(env, session(), postId)).rejects.toMatchObject({
      code: 'post_not_found'
    });
  });
});

function session(): SessionState {
  return {
    owner_account: ownerAccount,
    created_at: 1,
    expires_at: Date.now() + 100000
  };
}

function completedUpload(manifestKey: string): PreparedUploadRow {
  return {
    upload_id: 'squ_test',
    post_id: postId,
    owner_account: ownerAccount,
    post_category: 'normal',
    manifest_hash: contentHash.slice(2),
    content_hash: contentHash.slice(2),
    storage_receipt_id: storageReceiptId,
    estimated_bytes: 1024,
    object_keys_json: JSON.stringify([manifestKey]),
    status: 'completed',
    created_at: 1,
    completed_at: 2
  };
}

function imageAsset(uploadId: string): MediaAssetRow {
  return {
    upload_id: uploadId,
    post_id: postId,
    owner_account: ownerAccount,
    media_index: 0,
    media_kind: 'image',
    provider: 'cloudflare_images',
    provider_asset_id: 'img_test',
    upload_method: 'direct_form',
    content_type: 'image/webp',
    byte_size: 1024,
    asset_state: 'ready',
    delivery_url: 'https://imagedelivery.net/account/img_test/public',
    playback_hls_url: null,
    playback_dash_url: null,
    thumbnail_url: null,
    duration_seconds: null,
    width: 1200,
    height: 800,
    error_code: null,
    created_at: 1,
    updated_at: 2,
    ready_at: 2
  };
}

function buildEventsHex(input: { cidNumber: string | null }): string {
  const chunks = [
    Uint8Array.of(0x00),
    u32Le(0),
    Uint8Array.of(36, 0),
    compactBytes(postId),
    ownerAccountBytes,
    input.cidNumber === null
      ? Uint8Array.of(0)
      : concat([Uint8Array.of(1), compactBytes(input.cidNumber)]),
    Uint8Array.of(1),
    bytes(contentHash),
    compactBytes(storageReceiptId),
    u64Le(1800000000000),
    u32Le(88),
    compactU32(0)
  ];
  const record = concat(chunks);
  return `0x${hex(concat([compactU32(1), record]))}`;
}

function bytes(input: string): Uint8Array {
  const text = input.startsWith('0x') ? input.slice(2) : input;
  const out = new Uint8Array(text.length / 2);
  for (let i = 0; i < out.length; i += 1) {
    out[i] = Number.parseInt(text.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

function concat(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const out = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.length;
  }
  return out;
}

class FakeDb {
  uploads = new Map<string, PreparedUploadRow>();
  mediaAssets = new Map<string, MediaAssetRow[]>();
  posts = new Map<string, Record<string, unknown>>();
  membershipStorageUsed = new Map<string, number>();

  prepare(sql: string) {
    return new FakeStmt(this, sql);
  }

  async batch(statements: FakeStmt[]) {
    for (const statement of statements) {
      await statement.run();
    }
    return statements.map(() => ({ success: true }));
  }
}

class FakeStmt {
  private args: unknown[] = [];

  constructor(
    private readonly db: FakeDb,
    private readonly sql: string
  ) {}

  bind(...args: unknown[]) {
    this.args = args;
    return this;
  }

  async first<T>() {
    if (this.sql.includes('FROM square_uploads')) {
      return (this.db.uploads.get(this.args[0] as string) ?? null) as T | null;
    }
    if (this.sql.includes('FROM square_posts')) {
      return (this.db.posts.get(this.args[0] as string) ?? null) as T | null;
    }
    return null;
  }

  async run() {
    if (this.sql.includes('INSERT OR REPLACE INTO square_posts')) {
      this.db.posts.set(this.args[0] as string, {
        post_id: this.args[0],
        owner_account: this.args[1],
        cid_number: this.args[2],
        post_category: this.args[3],
        content_format: this.args[4],
        title: this.args[5],
        text: this.args[6],
        content_hash: this.args[7],
        storage_receipt_id: this.args[8],
        chain_block: this.args[9],
        created_at: this.args[10],
        post_state: 'published'
      });
    }
    if (this.sql.includes('DELETE FROM square_posts')) {
      // 硬删除：帖子行整行移除，不保留软删残行。
      this.db.posts.delete(this.args[0] as string);
    }
    if (this.sql.includes('DELETE FROM square_media_assets')) {
      this.db.mediaAssets.set(this.args[0] as string, []);
    }
    if (this.sql.includes('DELETE FROM square_uploads')) {
      // 上传行按 upload_id 删；本假库以 post_id 为键，故按值反查。
      const uploadId = this.args[0] as string;
      for (const [postKey, row] of this.db.uploads) {
        if (row.upload_id === uploadId) {
          this.db.uploads.delete(postKey);
        }
      }
    }
    if (this.sql.includes('storage_used_bytes = MAX')) {
      const bytes = this.args[0] as number;
      const owner = this.args[2] as string;
      const current = this.db.membershipStorageUsed.get(owner) ?? 0;
      this.db.membershipStorageUsed.set(owner, Math.max(0, current - bytes));
    }
    return { success: true };
  }

  async all<T>() {
    if (this.sql.includes('FROM square_media_assets')) {
      return {
        results: (this.db.mediaAssets.get(this.args[0] as string) ?? []) as T[]
      };
    }
    return { results: [] as T[] };
  }
}

class FakeR2 {
  readonly deletedKeys: string[] = [];

  constructor(private readonly objects: Record<string, string>) {}

  async get(key: string) {
    const value = this.objects[key];
    if (!value) return null;
    return {
      text: async () => value
    };
  }

  async delete(key: string) {
    this.deletedKeys.push(key);
    delete this.objects[key];
  }
}
