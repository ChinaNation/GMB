import { describe, expect, it, vi } from 'vitest';
import { encodeAddress } from '@polkadot/util-crypto';
import { confirmPublishedPost } from '../src/posts/confirm';
import type { Env, PreparedUploadRow, SessionState } from '../src/types';
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
      object_keys_json: JSON.stringify([
        `square/${ownerAccount}/posts/${postId}/manifest.json`,
        `square/${ownerAccount}/posts/${postId}/media_001.webp`
      ]),
      status: 'completed',
      created_at: 1,
      completed_at: 2
    };
    db.uploads.set(postId, upload);
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
    expect(post.media_items?.[0].object_key).toContain('media_001.webp');
    expect(db.posts.get(postId)?.post_state).toBe('published');
  });
});

function session(): SessionState {
  return {
    owner_account: ownerAccount,
    created_at: 1,
    expires_at: Date.now() + 100000
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
  posts = new Map<string, Record<string, unknown>>();

  prepare(sql: string) {
    return new FakeStmt(this, sql);
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
    return null;
  }

  async run() {
    if (this.sql.includes('INSERT OR REPLACE INTO square_posts')) {
      this.db.posts.set(this.args[0] as string, {
        post_id: this.args[0],
        owner_account: this.args[1],
        cid_number: this.args[2],
        post_category: this.args[3],
        text: this.args[4],
        content_hash: this.args[5],
        storage_receipt_id: this.args[6],
        chain_block: this.args[7],
        created_at: this.args[8],
        post_state: 'published'
      });
    }
    return { success: true };
  }
}

class FakeR2 {
  constructor(private readonly objects: Record<string, string>) {}

  async get(key: string) {
    const value = this.objects[key];
    if (!value) return null;
    return {
      text: async () => value
    };
  }
}
