import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Miniflare } from 'miniflare';

import worker from '../src/index';
import { sha256Hex } from '../src/shared/hash';
import {
  OP_SIGN_SQUARE_LOGIN,
  scaleString,
  signingMessage,
} from '../src/shared/signing_message';
import type { Env, SessionState } from '../src/types';

const ACCOUNT_A = `0x${'11'.repeat(32)}`;
const ACCOUNT_B = `0x${'22'.repeat(32)}`;
const CID_A = 'CN220-CTZN2-100000001-2026';
const CID_B = 'CN220-CTZN2-100000002-2026';
vi.mock('../src/chain/identity', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../src/chain/identity')>();
  return {
    ...actual,
    fetchChainIdentityStateByCid: vi.fn(async (_env: unknown, cidNumber: string) => {
      const accountId = cidNumber === CID_A
        ? ACCOUNT_A
        : cidNumber === CID_B
          ? ACCOUNT_B
          : '';
      return {
        account_id: accountId,
        binding_revision: accountId ? 1 : 0,
        identity_level: 'visitor' as const,
        has_voting_identity: false,
        has_candidate_identity: false,
        cid_number: cidNumber,
        checked_at: 0,
      };
    }),
  };
});
vi.mock('../src/auth/wallet_signature', () => ({
  verifyWalletSignature: vi.fn(async () => true),
}));
const SCHEMA_SQL = readFileSync(
  resolve(process.cwd(), 'migrations/0001_square_core.sql'),
  'utf8',
);

interface TestAccount {
  account_id: string;
  cid_number: string;
  token: string;
  private_key: CryptoKey;
}

interface Harness {
  miniflare: Miniflare;
  env: Env;
  accountA: TestAccount;
  accountB: TestAccount;
  nonce: number;
}

let harness: Harness;

describe('本人发布内容本地副本回灌 API', () => {
  beforeEach(async () => {
    harness = await createHarness();
  });

  afterEach(async () => {
    if (harness) await harness.miniflare.dispose();
  });

  it('通过完整 Worker HTTP 入口和真实 D1/R2/KV binding 按 Session CID 隔离回灌', async () => {
    const newest = await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_a_3',
      created_at: 3000,
      text: '本人第三条',
    });
    const middle = await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_a_2',
      created_at: 2000,
      text: '本人第二条',
      content_format: 'article',
    });
    await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_a_1',
      created_at: 1000,
      text: '本人第一条',
    });
    await seedPost(harness.env, harness.accountB, {
      post_id: 'sqp_b_1',
      created_at: 9000,
      text: '另一身份的内容',
    });

    const anonymous = await callWorker(harness, null, '/v1/square/posts/self');
    expect(anonymous.status).toBe(401);
    expect((await responseJson(anonymous)).error_code).toBe('missing_session');

    const firstResponse = await callWorker(
      harness,
      harness.accountA,
      '/v1/square/posts/self?limit=2',
    );
    expect(firstResponse.status).toBe(200);
    const first = await responseJson(firstResponse);
    expect((first.items as Array<{ post_id: string }>).map((item) => item.post_id))
      .toEqual(['sqp_a_3', 'sqp_a_2']);
    expect(first.next_cursor).toEqual(expect.any(String));

    const firstItem = (first.items as Array<Record<string, unknown>>)[0];
    expect(firstItem).toMatchObject({
      post_id: 'sqp_a_3',
      cid_number: CID_A,
      account_id: ACCOUNT_A,
      content_format: 'normal',
      content_hash: newest.content_hash,
      created_at: 3000,
      post_state: 'published',
    });
    expect(decodeBase64(firstItem.manifest_bytes_base64 as string))
      .toEqual(newest.manifest_bytes);
    const secondItem = (first.items as Array<Record<string, unknown>>)[1];
    expect(decodeBase64(secondItem.manifest_bytes_base64 as string))
      .toEqual(middle.manifest_bytes);

    const secondResponse = await callWorker(
      harness,
      harness.accountA,
      `/v1/square/posts/self?limit=2&cursor=${first.next_cursor as string}`,
    );
    expect(secondResponse.status).toBe(200);
    const second = await responseJson(secondResponse);
    expect((second.items as Array<{ post_id: string }>).map((item) => item.post_id))
      .toEqual(['sqp_a_1']);
    expect(second.next_cursor).toBeNull();

    const other = await responseJson(
      await callWorker(harness, harness.accountB, '/v1/square/posts/self'),
    );
    expect((other.items as Array<{ post_id: string }>).map((item) => item.post_id))
      .toEqual(['sqp_b_1']);

    // 本人副本回灌不属于公共 feed 浏览，不写入浏览计数。
    const browseRows = await harness.env.DB.prepare(
      'SELECT COUNT(*) AS count FROM square_browse_days',
    ).first<{ count: number }>();
    expect(browseRows?.count).toBe(0);
  });

  it('通过完整 Worker HTTP 入口和真实 D1 为 finalized 当前账户创建 CID 数据根', async () => {
    const challengeBody = JSON.stringify({
      cid_number: CID_A,
      account_id: ACCOUNT_A,
    });
    const challengeResponse = await worker.fetch(
      new Request(
        'https://worker.test/v1/square/identity/takeover/challenge',
        {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
            'content-length': String(new TextEncoder().encode(challengeBody).byteLength),
          },
          body: challengeBody,
        },
      ),
      harness.env,
    );
    expect(challengeResponse.status).toBe(200);
    const challenge = await responseJson(challengeResponse);

    const takeoverBody = JSON.stringify({
      cid_number: CID_A,
      account_id: ACCOUNT_A,
      binding_revision: challenge.binding_revision,
      challenge_id: challenge.challenge_id,
      signature: '0xcurrent-account-signature',
    });
    const takeoverResponse = await worker.fetch(
      new Request('https://worker.test/v1/square/identity/takeover', {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          'content-length': String(new TextEncoder().encode(takeoverBody).byteLength),
        },
        body: takeoverBody,
      }),
      harness.env,
    );
    expect(takeoverResponse.status).toBe(200);
    const granted = await responseJson(takeoverResponse);
    expect(decodeBase64(granted.cid_data_root_base64 as string)).toHaveLength(32);
    expect(granted).toMatchObject({
      cid_number: CID_A,
      account_id: ACCOUNT_A,
      binding_revision: 1,
    });

    const stored = await harness.env.DB.prepare(
      `SELECT cid_number, active_binding_revision, active_account_id, data_root_hash
        FROM cid_data_roots WHERE cid_number = ?`,
    )
      .bind(CID_A)
      .first<{
        cid_number: string;
        active_binding_revision: number;
        active_account_id: string;
        data_root_hash: string;
      }>();
    expect(stored).toEqual({
      cid_number: CID_A,
      active_binding_revision: 1,
      active_account_id: ACCOUNT_A,
      data_root_hash: granted.data_root_hash,
    });
  });

  it('同毫秒按 post_id 稳定分页且拒绝非法 limit/cursor', async () => {
    await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_same_c',
      created_at: 5000,
      text: 'C',
    });
    await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_same_b',
      created_at: 5000,
      text: 'B',
    });
    await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_same_a',
      created_at: 5000,
      text: 'A',
    });

    const first = await responseJson(
      await callWorker(harness, harness.accountA, '/v1/square/posts/self?limit=2'),
    );
    expect((first.items as Array<{ post_id: string }>).map((item) => item.post_id))
      .toEqual(['sqp_same_c', 'sqp_same_b']);
    const second = await responseJson(
      await callWorker(
        harness,
        harness.accountA,
        `/v1/square/posts/self?limit=2&cursor=${first.next_cursor as string}`,
      ),
    );
    expect((second.items as Array<{ post_id: string }>).map((item) => item.post_id))
      .toEqual(['sqp_same_a']);

    const invalidLimit = await callWorker(
      harness,
      harness.accountA,
      '/v1/square/posts/self?limit=6',
    );
    expect(invalidLimit.status).toBe(400);
    expect((await responseJson(invalidLimit)).error_code).toBe('invalid_limit');

    const invalidCursor = await callWorker(
      harness,
      harness.accountA,
      '/v1/square/posts/self?cursor=not-a-valid-cursor',
    );
    expect(invalidCursor.status).toBe(400);
    expect((await responseJson(invalidCursor)).error_code).toBe('invalid_cursor');
  });

  it('任一 manifest 哈希损坏时整页 fail-closed，不返回部分正确内容', async () => {
    await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_good',
      created_at: 2000,
      text: '正确内容',
    });
    const corrupt = await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_corrupt',
      created_at: 1000,
      text: '将被篡改',
    });
    await harness.env.SQUARE_MEDIA.put(
      corrupt.object_key,
      '{"schema":"citizenapp.square.post.v1","text":"篡改"}',
      { httpMetadata: { contentType: 'application/json' } },
    );

    const response = await callWorker(
      harness,
      harness.accountA,
      '/v1/square/posts/self',
    );
    expect(response.status).toBe(409);
    const body = await responseJson(response);
    expect(body.ok).toBe(false);
    expect(body.error_code).toBe('manifest_hash_mismatch');
    expect(body.items).toBeUndefined();
  });

  it('对象清单与账户/post_id 规范路径不一致时整页 fail-closed', async () => {
    const seeded = await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_no_key',
      created_at: 1000,
      text: '对象仍存在但清单被破坏',
    });
    await harness.env.DB.prepare(
      'UPDATE square_uploads SET object_keys_json = ? WHERE post_id = ?',
    ).bind('[]', seeded.post_id).run();

    expect(await harness.env.SQUARE_MEDIA.head(seeded.object_key)).not.toBeNull();
    const response = await callWorker(
      harness,
      harness.accountA,
      '/v1/square/posts/self',
    );
    expect(response.status).toBe(409);
    expect((await responseJson(response)).error_code).toBe('upload_object_keys_invalid');
  });

  it('帖子与上传归属或链锚不一致时拒绝回灌', async () => {
    const seeded = await seedPost(harness.env, harness.accountA, {
      post_id: 'sqp_owner_conflict',
      created_at: 1000,
      text: '归属冲突',
    });
    await harness.env.DB.prepare(
      'UPDATE square_uploads SET cid_number = ? WHERE post_id = ?',
    ).bind(CID_B, seeded.post_id).run();

    const ownerConflict = await callWorker(
      harness,
      harness.accountA,
      '/v1/square/posts/self',
    );
    expect(ownerConflict.status).toBe(409);
    expect((await responseJson(ownerConflict)).error_code).toBe('post_owner_mismatch');

    await harness.env.DB.prepare(
      'UPDATE square_uploads SET cid_number = ?, manifest_hash = ? WHERE post_id = ?',
    ).bind(CID_A, 'ff'.repeat(32), seeded.post_id).run();
    const hashConflict = await callWorker(
      harness,
      harness.accountA,
      '/v1/square/posts/self',
    );
    expect(hashConflict.status).toBe(409);
    expect((await responseJson(hashConflict)).error_code).toBe('post_upload_hash_mismatch');
  });
});

async function createHarness(): Promise<Harness> {
  const miniflare = new Miniflare({
    modules: true,
    script: 'export default { fetch() { return new Response("test"); } }',
    compatibilityDate: '2026-07-29',
    d1Databases: ['DB'],
    r2Buckets: ['SQUARE_MEDIA'],
    kvNamespaces: ['SQUARE_CACHE'],
    bindings: {
      HASH_KEY: 'post-local-copy-test-rate-key',
      CHAIN_GENESIS_HASH: `0x${'12'.repeat(32)}`,
      CID_DATA_ROOT_MASTER_KEY: `0x${'34'.repeat(32)}`,
    },
  });
  const env = await miniflare.getBindings<Env>();
  // D1 exec 按换行拆语句，无法直接接受含独立注释行的创世 SQL；测试逐条执行同一基线。
  const statements = SCHEMA_SQL
    .split('\n')
    .filter((line) => !line.trimStart().startsWith('--'))
    .join('\n')
    .split(';')
    .map((statement) => statement.trim())
    .filter((statement) => statement.length > 0);
  for (const statement of statements) {
    await env.DB.prepare(statement).run();
  }
  const accountA = await registerAccount(env, ACCOUNT_A, CID_A, 'token-a');
  const accountB = await registerAccount(env, ACCOUNT_B, CID_B, 'token-b');
  return {
    miniflare,
    env,
    accountA,
    accountB,
    nonce: 0,
  };
}

async function registerAccount(
  env: Env,
  accountId: string,
  cidNumber: string,
  token: string,
): Promise<TestAccount> {
  const keyPair = await crypto.subtle.generateKey(
    { name: 'ECDSA', namedCurve: 'P-256' },
    true,
    ['sign', 'verify'],
  );
  const publicKey = toHex(await crypto.subtle.exportKey('raw', keyPair.publicKey));
  const deviceId = await sha256Hex(publicKey);
  const now = Date.now();
  const session: SessionState = {
    cid_number: cidNumber,
    binding_revision: 1,
    account_id: accountId,
    device_key_hash: deviceId,
    created_at: now,
    expires_at: now + 60_000,
  };
  await env.SQUARE_CACHE.put(
    `square_session:${await sha256Hex(token)}`,
    JSON.stringify(session)
  );
  await env.SQUARE_CACHE.put(
    `square_identity:${accountId}`,
    JSON.stringify({
      account_id: accountId,
      identity_level: 'visitor',
      has_voting_identity: false,
      has_candidate_identity: false,
      cid_number: cidNumber,
      binding_revision: 1,
      checked_at: now,
    }),
  );
  await env.DB.prepare(
    `INSERT INTO square_device_subkeys
      (cid_number, device_id, binding_revision, account_id, p256_public_key,
        issued_at, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
  ).bind(cidNumber, deviceId, 1, accountId, publicKey, now, now, now).run();
  return {
    account_id: accountId,
    cid_number: cidNumber,
    token,
    private_key: keyPair.privateKey,
  };
}

async function seedPost(
  env: Env,
  account: TestAccount,
  input: {
    post_id: string;
    created_at: number;
    text: string;
    content_format?: 'normal' | 'article';
  },
): Promise<{
  post_id: string;
  object_key: string;
  manifest_bytes: Uint8Array;
  content_hash: string;
}> {
  const manifest: Record<string, unknown> = {
    schema: 'citizenapp.square.post.v1',
    account_id: account.account_id,
    post_category: 'normal',
    text: input.text,
    media_items: [],
  };
  if (input.content_format === 'article') {
    manifest.content_format = 'article';
    manifest.title = `${input.text}标题`;
    manifest.content_blocks = [{ t: 'text', text: input.text }];
  }
  const manifestBytes = new TextEncoder().encode(JSON.stringify(manifest));
  const contentHash = await sha256Hex(manifestBytes);
  const objectKey =
    `square/${account.account_id.slice(2)}/posts/${input.post_id}/manifest.json`;
  const uploadId = `squ_${input.post_id}`;
  const storageReceiptId = `sqr_${input.post_id}`;
  await env.SQUARE_MEDIA.put(objectKey, manifestBytes, {
    httpMetadata: { contentType: 'application/json' },
  });
  await env.DB.prepare(
    `INSERT INTO square_uploads
      (upload_id, post_id, cid_number, account_id, post_category, manifest_hash,
        content_hash, storage_receipt_id, estimated_bytes, object_keys_json,
        status, expires_at, created_at, completed_at)
      VALUES (?, ?, ?, ?, 'normal', ?, ?, ?, ?, ?, 'completed', ?, ?, ?)`,
  ).bind(
    uploadId,
    input.post_id,
    account.cid_number,
    account.account_id,
    contentHash,
    contentHash,
    storageReceiptId,
    manifestBytes.byteLength,
    JSON.stringify([objectKey]),
    9999999999999,
    input.created_at - 2,
    input.created_at - 1,
  ).run();
  await env.DB.prepare(
    `INSERT INTO square_posts
      (post_id, cid_number, account_id, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state)
      VALUES (?, ?, ?, 'normal', ?, ?, ?, ?, ?, 88, ?, 'published')`,
  ).bind(
    input.post_id,
    account.cid_number,
    account.account_id,
    input.content_format ?? 'normal',
    input.content_format === 'article' ? `${input.text}标题` : null,
    input.text,
    `0x${contentHash}`,
    storageReceiptId,
    input.created_at,
  ).run();
  return {
    post_id: input.post_id,
    object_key: objectKey,
    manifest_bytes: manifestBytes,
    content_hash: contentHash,
  };
}

async function callWorker(
  context: Harness,
  account: TestAccount | null,
  path: string,
): Promise<Response> {
  const headers = new Headers();
  if (account) {
    const requestTime = Date.now();
    const nonce = (++context.nonce).toString(16).padStart(32, '0');
    const canonical = [
      'square_request',
      'GET',
      path,
      await sha256Hex(''),
      String(requestTime),
      nonce,
      await sha256Hex(account.token),
    ].join('\n');
    const signature = await crypto.subtle.sign(
      { name: 'ECDSA', hash: 'SHA-256' },
      account.private_key,
      signingMessage(OP_SIGN_SQUARE_LOGIN, scaleString(canonical)),
    );
    headers.set('authorization', `Bearer ${account.token}`);
    headers.set('x-device-time', String(requestTime));
    headers.set('x-device-nonce', nonce);
    headers.set('x-device-signature', `0x${toHex(signature)}`);
  }
  return worker.fetch(new Request(`https://worker.test${path}`, { headers }), context.env);
}

function decodeBase64(value: string): Uint8Array {
  return Uint8Array.from(atob(value), (character) => character.charCodeAt(0));
}

function toHex(buffer: ArrayBuffer): string {
  return [...new Uint8Array(buffer)]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
}

async function responseJson(response: Response): Promise<Record<string, unknown>> {
  return response.json() as Promise<Record<string, unknown>>;
}
