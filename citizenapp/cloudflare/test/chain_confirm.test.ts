import { afterEach, describe, expect, it, vi } from 'vitest';
import { encodeAddress } from '@polkadot/util-crypto';
import { confirmPublishedPost, deletePostCloudflareData } from '../src/posts/confirm';
import type { Env, MediaAssetRow, PreparedUploadRow, SessionState } from '../src/types';
import {
  decodeSquarePostPublishedEvents,
  u32Le,
  u64Le
} from '../src/chain/square_event';
import {
  scaleString as compactBytes,
  scaleCompact as compactU32,
  bytesToHex as hex
} from '../src/shared/signing_message';
import { fetchChainStorage } from '../src/chain/rpc';

const ownerAccountBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 1));
const ownerAccount = encodeAddress(ownerAccountBytes, 2027);
const postId = 'sqp_test';
const contentHash = `0x${'11'.repeat(32)}`;
const storageReceiptId = 'sqr_test';
const blockHash = `0x${'22'.repeat(32)}`;

describe('square chain confirmation', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

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
      post_category: 'normal',
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
        ready_at: 2,
        archive_state: 'live',
        archived_at: null,
        r2_archive_key: null
      }
    ]);
    const env = {
      DB: db,
      SQUARE_MEDIA: new FakeR2({
        [`square/${ownerAccount}/posts/${postId}/manifest.json`]: JSON.stringify({
          schema: 'citizenapp.square.post.v1',
          owner_account: ownerAccount,
          post_category: 'normal',
          text: '普通动态',
          media_items: [
            {
              media_kind: 'image',
              file_name: 'a.webp',
              content_type: 'image/webp',
              byte_size: 1024,
              sha256: 'aa'.repeat(32)
            }
          ]
        })
      }),
      SQUARE_CACHE: {},
      CHAIN_URL: 'https://chain.test',
      CHAIN_ID: 'worker-rpc.access',
      CHAIN_SECRET: 'test-access-secret'
    } as unknown as Env;
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        Response.json({
          jsonrpc: '2.0',
          id: 1,
          result: buildEventsHex({
            cidNumber: 'CN001-CTZN-000000001-2026',
            postCategory: 'normal'
          })
        })
      )
    );

    const post = await confirmPublishedPost(env, session(), {
      post_id: postId,
      block_hash: blockHash
    });

    expect(post.text).toBe('普通动态');
    expect(post.cid_number).toBe('CN001-CTZN-000000001-2026');
    expect(post.media_items?.[0]).toMatchObject({
      provider: 'cloudflare_images',
      provider_asset_id: 'img_test',
      url: 'https://imagedelivery.net/account/img_test/public',
      asset_state: 'ready'
    });
    expect(db.posts.get(postId)?.post_state).toBe('published');
  });

  it('sends state_getStorage only through the Access-protected HTTPS upstream', async () => {
    const fetchMock = vi.fn(async (url: string, init: RequestInit) => {
      const headers = new Headers(init.headers);
      const body = JSON.parse(init.body as string) as {
        id: number;
        method: string;
        params: string[];
      };
      expect(url).toBe('https://chain.test/');
      expect(headers.get('CF-Access-Client-Id')).toBe('worker-rpc.access');
      expect(headers.get('CF-Access-Client-Secret')).toBe('test-access-secret');
      expect(body.method).toBe('state_getStorage');
      expect(body.params).toEqual(['0x1234', blockHash]);
      return Response.json({ jsonrpc: '2.0', id: body.id, result: '0xabcd' });
    });
    vi.stubGlobal('fetch', fetchMock);

    const result = await fetchChainStorage(chainRpcEnv(), '0x1234', blockHash);

    expect(result).toBe('0xabcd');
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it('rejects non-HTTPS RPC configuration before making a request', async () => {
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      fetchChainStorage(
        chainRpcEnv({ CHAIN_URL: 'http://127.0.0.1:9944' }),
        '0x1234'
      )
    ).rejects.toMatchObject({ code: 'chain_rpc_invalid_config' });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('rejects an oversized RPC response before buffering its body', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response('{}', {
          headers: { 'content-length': String(4 * 1024 * 1024 + 1) }
        })
      )
    );

    await expect(fetchChainStorage(chainRpcEnv(), '0x1234')).rejects.toMatchObject({
      code: 'chain_rpc_response_too_large'
    });
  });

  it('hard-deletes Cloudflare-side post data', async () => {
    const db = new FakeDb();
    const manifestKey = `square/${ownerAccount}/posts/${postId}/manifest.json`;
    const upload = completedUpload(manifestKey);
    db.uploads.set(postId, upload);
    db.mediaAssets.set(upload.upload_id, [imageAsset(upload.upload_id)]);
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
      SQUARE_CACHE: {},
      DEV_UPLOAD_PROXY: '1'
    } as unknown as Env;

    const result = await deletePostCloudflareData(env, session(), postId);

    expect(result).toMatchObject({
      deleted_media_assets: 1,
      deleted_r2_objects: 1
    });
    // 硬删除：帖子行 + 上传行 + 媒体资产 + R2 对象全部清空，无软删残行。
    expect(db.posts.has(postId)).toBe(false);
    expect(db.uploads.has(postId)).toBe(false);
    expect(db.mediaAssets.get(upload.upload_id)).toEqual([]);
    expect(r2.deletedKeys).toEqual([manifestKey]);

    // 再删同一帖子 → 已无残行，报 404，证明是彻底删除而非软删。
    await expect(deletePostCloudflareData(env, session(), postId)).rejects.toMatchObject({
      code: 'post_not_found'
    });
  });
});

function chainRpcEnv(overrides: Partial<Env> = {}): Env {
  return {
    DB: {} as D1Database,
    SQUARE_MEDIA: {} as R2Bucket,
    SQUARE_CACHE: {} as KVNamespace,
    CHAIN_URL: 'https://chain.test',
    CHAIN_ID: 'worker-rpc.access',
    CHAIN_SECRET: 'test-access-secret',
    ...overrides
  };
}

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
    ready_at: 2,
    archive_state: 'live',
    archived_at: null,
    r2_archive_key: null
  };
}

function buildEventsHex(input: {
  cidNumber: string | null;
  postCategory?: 'normal' | 'campaign';
}): string {
  const chunks = [
    Uint8Array.of(0x00),
    u32Le(0),
    Uint8Array.of(36, 0),
    compactBytes(postId),
    ownerAccountBytes,
    input.cidNumber === null
      ? Uint8Array.of(0)
      : concat([Uint8Array.of(1), compactBytes(input.cidNumber)]),
    Uint8Array.of(input.postCategory === 'normal' ? 0 : 1),
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
    if (this.sql.includes('FROM square_memberships')) {
      return {
        owner_account: ownerAccount,
        membership_level: 'democracy',
        expires_at: Date.now() + 60_000,
        updated_at: Date.now(),
        subscription_source: 'stripe',
        stripe_customer_id: 'cus_test',
        stripe_subscription_id: 'sub_test',
        stripe_price_id: 'price_test',
        subscription_status: 'active',
        current_period_start: Date.now(),
        current_period_end: Date.now() + 60_000,
        cancel_at_period_end: 0,
        identity_level: 'visitor',
        identity_checked_at: Date.now(),
        entitlement_lapsed_at: null
      } as T;
    }
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
