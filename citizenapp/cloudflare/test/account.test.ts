import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('../src/auth/wallet_signature', () => ({
  verifyWalletSignature: vi.fn()
}));

import { verifyWalletSignature } from '../src/auth/wallet_signature';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from '../src/account/action_challenge';
import { purgeAccount } from '../src/account/purge';
import type { Env, MediaAssetRow } from '../src/types';

const mockVerify = verifyWalletSignature as unknown as ReturnType<typeof vi.fn>;

const ACCOUNT_ID = '0x1111111111111111111111111111111111111111111111111111111111111111';

interface ChallengeRecord {
  challenge_id: string;
  account_id: string;
  signing_payload: string;
  expires_at: number;
  used_at: number | null;
}

class ChallengeStmt {
  private binds: unknown[] = [];
  constructor(private readonly db: ChallengeDb, private readonly sql: string) {}
  bind(...args: unknown[]): ChallengeStmt {
    this.binds = args;
    return this;
  }
  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes('INSERT INTO square_login_challenges')) {
      this.db.challenges.set(this.binds[0] as string, {
        challenge_id: this.binds[0] as string,
        account_id: this.binds[1] as string,
        signing_payload: this.binds[2] as string,
        expires_at: this.binds[3] as number,
        used_at: null
      });
    } else if (this.sql.includes('UPDATE square_login_challenges SET used_at = NULL')) {
      const record = this.db.challenges.get(this.binds[0] as string);
      if (record) record.used_at = null;
    } else if (this.sql.includes('UPDATE square_login_challenges SET used_at')) {
      const record = this.db.challenges.get(this.binds[1] as string);
      if (record) record.used_at = this.binds[0] as number;
    }
    return { meta: { changes: 1 } };
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM square_login_challenges')) {
      return (this.db.challenges.get(this.binds[0] as string) as T) ?? null;
    }
    return null;
  }
}

class ChallengeDb {
  readonly challenges = new Map<string, ChallengeRecord>();
  prepare(sql: string): ChallengeStmt {
    return new ChallengeStmt(this, sql);
  }
}

function challengeEnv(): { env: Env; db: ChallengeDb } {
  const db = new ChallengeDb();
  return { env: { DB: db } as unknown as Env, db };
}

describe('consumeActionSignature', () => {
  beforeEach(() => mockVerify.mockReset());

  it('accepts a valid, unused, action-matching wallet signature and marks it used', async () => {
    const { env, db } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(env, ACCOUNT_ID, 'delete_account');

    await expect(
      consumeActionSignature(env, {
        accountId: ACCOUNT_ID,
        action: 'delete_account',
        challengeId: challenge.challengeId,
        signature: 'sig'
      })
    ).resolves.toBeUndefined();
    expect(db.challenges.get(challenge.challengeId)?.used_at).not.toBeNull();
  });

  it('rejects reuse of a consumed challenge', async () => {
    const { env } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(env, ACCOUNT_ID, 'delete_account');
    const input = {
      accountId: ACCOUNT_ID,
      action: 'delete_account' as const,
      challengeId: challenge.challengeId,
      signature: 'sig'
    };
    await consumeActionSignature(env, input);
    await expect(consumeActionSignature(env, input)).rejects.toMatchObject({
      code: 'used_challenge'
    });
  });

  it('rejects a signature issued for a different action context', async () => {
    const { env } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(env, ACCOUNT_ID, 'delete_account', 'context-a');
    await expect(
      consumeActionSignature(env, {
        accountId: ACCOUNT_ID,
        action: 'delete_account',
        challengeId: challenge.challengeId,
        signature: 'sig',
        context: 'context-b'
      })
    ).rejects.toMatchObject({ code: 'action_mismatch' });
    expect(mockVerify).not.toHaveBeenCalled();
  });

  it('rejects a wrong accountId account', async () => {
    const { env } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(env, ACCOUNT_ID, 'delete_account');
    await expect(
      consumeActionSignature(env, {
        accountId: '0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
        action: 'delete_account',
        challengeId: challenge.challengeId,
        signature: 'sig'
      })
    ).rejects.toMatchObject({ code: 'invalid_challenge' });
  });

  it('rejects an expired challenge', async () => {
    const { env, db } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(env, ACCOUNT_ID, 'delete_account');
    db.challenges.get(challenge.challengeId)!.expires_at = 1;
    await expect(
      consumeActionSignature(env, {
        accountId: ACCOUNT_ID,
        action: 'delete_account',
        challengeId: challenge.challengeId,
        signature: 'sig'
      })
    ).rejects.toMatchObject({ code: 'expired_challenge' });
  });

  it('rejects an invalid signature', async () => {
    const { env } = challengeEnv();
    mockVerify.mockResolvedValue(false);
    const challenge = await issueActionChallenge(env, ACCOUNT_ID, 'delete_account');
    await expect(
      consumeActionSignature(env, {
        accountId: ACCOUNT_ID,
        action: 'delete_account',
        challengeId: challenge.challengeId,
        signature: 'bad'
      })
    ).rejects.toMatchObject({ code: 'invalid_signature' });
  });
});

describe('releaseActionChallenge', () => {
  beforeEach(() => mockVerify.mockReset());

  it('resets used_at to null so a consumed challenge can be retried', async () => {
    const { env, db } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(env, ACCOUNT_ID, 'delete_account');
    const input = {
      accountId: ACCOUNT_ID,
      action: 'delete_account' as const,
      challengeId: challenge.challengeId,
      signature: 'sig'
    };
    await consumeActionSignature(env, input);
    expect(db.challenges.get(challenge.challengeId)?.used_at).not.toBeNull();

    await releaseActionChallenge(env, challenge.challengeId);
    expect(db.challenges.get(challenge.challengeId)?.used_at).toBeNull();

    // 释放后可再次消费同一 challenge（下游副作用失败后原地重试）。
    await expect(consumeActionSignature(env, input)).resolves.toBeUndefined();
  });
});

class PurgeStmt {
  binds: unknown[] = [];
  constructor(private readonly db: PurgeDb, readonly sql: string) {}
  bind(...args: unknown[]): PurgeStmt {
    this.binds = args;
    return this;
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM square_memberships') && this.sql.includes('WHERE account_id')) {
      return this.db.membership as T | null;
    }
    return null;
  }
  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes('FROM square_media_assets') && this.sql.includes('provider_asset_id')) {
      return { results: this.db.mediaRows as T[] };
    }
    return { results: [] };
  }
  async run(): Promise<{ meta: { changes: number } }> {
    this.db.deletes.push(this.sql);
    return { meta: { changes: 1 } };
  }
}

class PurgeDb {
  membership: Record<string, unknown> | null = null;
  mediaRows: MediaAssetRow[] = [];
  readonly deletes: string[] = [];
  prepare(sql: string): PurgeStmt {
    return new PurgeStmt(this, sql);
  }
  async batch(statements: PurgeStmt[]): Promise<Array<{ meta: { changes: number } }>> {
    return statements.map((statement) => {
      this.deletes.push(statement.sql);
      return { meta: { changes: 1 } };
    });
  }
}

class FakeR2 {
  deleted: string[] = [];
  constructor(public keys: string[]) {}
  async list(options: { prefix: string }): Promise<{
    objects: Array<{ key: string }>;
    truncated: boolean;
    cursor?: string;
  }> {
    return {
      objects: this.keys.filter((key) => key.startsWith(options.prefix)).map((key) => ({ key })),
      truncated: false
    };
  }
  async delete(keyOrKeys: string | string[]): Promise<void> {
    const keys = Array.isArray(keyOrKeys) ? keyOrKeys : [keyOrKeys];
    this.deleted.push(...keys);
    this.keys = this.keys.filter((key) => !keys.includes(key));
  }
}

class FakeKv {
  store = new Map<string, string>();
  async get(key: string): Promise<string | null> {
    return this.store.get(key) ?? null;
  }
  async put(key: string, value: string): Promise<void> {
    this.store.set(key, value);
  }
  async delete(key: string): Promise<void> {
    this.store.delete(key);
  }
}

describe('purgeAccount', () => {
  afterEach(() => vi.unstubAllGlobals());
  // 会员订阅与注销已解耦（公民币轨）：注销只硬删本地数据，不代签链上退订，
  // 因此不再有 stripe 退订成功/失败分支，purge 也不再抛支付相关错误。
  function buildEnv(): {
    env: Env;
    db: PurgeDb;
    r2: FakeR2;
    kv: FakeKv;
  } {
    const db = new PurgeDb();
    db.membership = { account_id: ACCOUNT_ID };
    db.mediaRows = [{
      upload_id: 'squ_1', post_id: 'sqp_1', account_id: ACCOUNT_ID, media_index: 0,
      media_kind: 'image', provider: 'cloudflare_images', provider_asset_id: 'img_1',
      upload_method: 'worker', resource_key: 'square_image_sd', content_type: 'image/webp',
      byte_size: 1024, asset_state: 'ready', declared_duration_seconds: null,
      duration_seconds: null, width: 100, height: 100, error_code: null,
      created_at: 1, updated_at: 1, ready_at: 1, archive_state: 'live',
      archived_at: null, r2_archive_key: null,
    }];
    const r2 = new FakeR2([
      `profile/${ACCOUNT_ID.slice(2)}/profile.json`,
      `profile/${ACCOUNT_ID.slice(2)}/avatar`,
      `square/${ACCOUNT_ID.slice(2)}/posts/p1/manifest.json`
    ]);
    const kv = new FakeKv();
    kv.store.set(`square_identity:${ACCOUNT_ID}`, '{"identity_level":"voting"}');
    kv.store.set(`square_sessions_by_account_id:${ACCOUNT_ID}`, JSON.stringify(['tok1']));
    kv.store.set('square_session:tok1', '{}');
    const env = {
      DB: db,
      SQUARE_MEDIA: r2,
      SQUARE_CACHE: kv,
      CF_ACCOUNT_ID: 'account',
      CF_API_TOKEN: 'token'
    } as unknown as Env;
    return { env, db, r2, kv };
  }

  it('硬删除全部 A 行与当前 R2 对象、清空会话，并返回删除计数', async () => {
    const { env, db, r2, kv } = buildEnv();
    vi.stubGlobal('fetch', vi.fn(async () => Response.json({ success: true, result: {} })));

    const result = await purgeAccount(env, ACCOUNT_ID);

    // PurgeAccountResult 只返回本地硬删除计数，不触发任何外部订阅副作用。
    expect(result.deleted_media_assets).toBe(1);
    expect(result.deleted_r2_objects).toBe(3);
    expect(result.deleted_rows).toBeGreaterThan(0);

    // A 的 Chat 路由、浏览、关注两端引用和业务表全部进入硬删除清单。
    const joined = db.deletes.join('\n');
    expect(joined).toContain('DELETE FROM square_memberships WHERE account_id = ?');
    expect(joined).toContain('DELETE FROM square_posts WHERE account_id = ?');
    expect(joined).toContain('DELETE FROM square_follows WHERE account_id = ?');
    expect(joined).toContain('DELETE FROM chat_device_binding_nonces WHERE account_id = ?');
    expect(joined).toContain('DELETE FROM chat_devices WHERE account_id = ?');
    expect(joined).toContain('DELETE FROM square_contacts WHERE account_id = ?');
    expect(joined).toContain('DELETE FROM square_browse_days WHERE account_id = ?');

    // R2：只存在并删除 profile / posts 等当前业务对象，Chat 不使用 R2。
    expect(r2.deleted).toContain(`profile/${ACCOUNT_ID.slice(2)}/profile.json`);
    expect(r2.deleted).toContain(`square/${ACCOUNT_ID.slice(2)}/posts/p1/manifest.json`);

    // KV：身份缓存 + 会话都清。
    expect(kv.store.has(`square_identity:${ACCOUNT_ID}`)).toBe(false);
    expect(kv.store.has('square_session:tok1')).toBe(false);
    expect(kv.store.has(`square_sessions_by_account_id:${ACCOUNT_ID}`)).toBe(false);
  });
});
