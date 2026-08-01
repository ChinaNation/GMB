import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

vi.mock('../src/auth/wallet_signature', () => ({
  verifyWalletSignature: vi.fn()
}));

const finalizedBinding = vi.hoisted(() => ({
  accountId: '0x1111111111111111111111111111111111111111111111111111111111111111',
  revision: 1
}));

// 注销按身份主键 cid_number 删 off-chain 表：mock 让 fetchChainIdentityStateCached
// 返回带 cid_number 的身份态，使 purge 的 cid-keyed 删除分支（follows/browse/
// user_signals/notify_reads/request_nonces/rate_windows）真正执行。其余导出保留真实实现。
vi.mock('../src/chain/identity', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../src/chain/identity')>();
  return {
    ...actual,
    fetchChainIdentityStateCached: vi.fn(async (_env: unknown, accountId: string) => ({
      account_id: accountId,
      identity_level: 'voting' as const,
      has_voting_identity: true,
      has_candidate_identity: false,
      cid_number: 'CN220-CTZN2-198805200-2026',
      binding_revision: finalizedBinding.revision,
      checked_at: 0
    })),
    fetchChainIdentityStateByCid: vi.fn(async (_env: unknown, cidNumber: string) => ({
      account_id: finalizedBinding.accountId,
      identity_level: 'voting' as const,
      has_voting_identity: true,
      has_candidate_identity: false,
      cid_number: cidNumber,
      binding_revision: finalizedBinding.revision,
      checked_at: 0
    }))
  };
});

import { verifyWalletSignature } from '../src/auth/wallet_signature';
import {
  consumeActionSignature,
  issueActionChallenge,
  releaseActionChallenge
} from '../src/account/action_challenge';
import {
  purgeFinalizedOldAccountCredentials,
  purgeIdentity
} from '../src/account/purge';
import { deleteAccountChallengeRoute } from '../src/account/service';
import { routeRequest } from '../src/routes';
import type { Env, MediaAssetRow } from '../src/types';

const mockVerify = verifyWalletSignature as unknown as ReturnType<typeof vi.fn>;

const ACCOUNT_ID = '0x1111111111111111111111111111111111111111111111111111111111111111';
const NEW_ACCOUNT_ID = '0x2222222222222222222222222222222222222222222222222222222222222222';
// 唯一身份主键 CID：注销按 cid_number 删 off-chain 表，须与上方 identity mock 一致。
const STANDARD_CID = 'CN220-CTZN2-198805200-2026';
const OLD_SESSION_TOKEN = `${STANDARD_CID}.old-token`;
const PURGE_CURRENT_SESSION_HASH =
  'a90ed5c348b900e30228ac7217a09522dd841c05ab06ace750e5e14802715957';
const PURGE_OLD_SESSION_HASH =
  '065cb6f72e61c32ae129ad1aa939b1ba3b6d8b5c75f0c66665a38b5633b97f91';
const OLD_SESSION_HASH =
  '7d3911076a691bc7e94304f8dfcdeba33679c38254f49871de9eb4b76625b143';
const NEW_SESSION_HASH =
  '7a416ea32777f5cc8a9c801db1aab6005a8680fc12efd037b0291cc3929815fb';
const CURRENT_SESSION_HASH =
  'ef6036bfacfc26e4d8f0ea4199e6c1a4571376f5e9949854a07ef59530d5d50b';

interface ChallengeRecord {
  challenge_id: string;
  cid_number: string;
  binding_revision: number;
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
        cid_number: this.binds[1] as string,
        binding_revision: this.binds[2] as number,
        account_id: this.binds[3] as string,
        signing_payload: this.binds[4] as string,
        expires_at: this.binds[5] as number,
        used_at: null
      });
    } else if (this.sql.includes('UPDATE square_login_challenges SET used_at = NULL')) {
      const record = this.db.challenges.get(this.binds[0] as string);
      if (record) record.used_at = null;
    } else if (
      this.sql.includes('UPDATE square_login_challenges') &&
      this.sql.includes('SET used_at = ?')
    ) {
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
  async all<T>(): Promise<{ results: T[] }> {
    return { results: [] };
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

class TakeoverStmt {
  private binds: unknown[] = [];
  constructor(private readonly db: TakeoverDb, private readonly sql: string) {}
  bind(...args: unknown[]): TakeoverStmt {
    this.binds = args;
    return this;
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM square_login_challenges')) {
      return (this.db.challenges.get(this.binds[0] as string) as T) ?? null;
    }
    return null;
  }
  async all<T>(): Promise<{ results: T[] }> {
    return { results: [] };
  }
  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes('INSERT INTO square_login_challenges')) {
      this.db.challenges.set(this.binds[0] as string, {
        challenge_id: this.binds[0] as string,
        cid_number: this.binds[1] as string,
        binding_revision: this.binds[2] as number,
        account_id: this.binds[3] as string,
        signing_payload: this.binds[4] as string,
        expires_at: this.binds[5] as number,
        used_at: null
      });
      return { meta: { changes: 1 } };
    }
    if (this.sql.includes('SET used_at = NULL')) {
      const row = this.db.challenges.get(this.binds[0] as string);
      if (row) row.used_at = null;
      return { meta: { changes: row ? 1 : 0 } };
    }
    if (
      this.sql.includes('UPDATE square_login_challenges') &&
      this.sql.includes('SET used_at = ?')
    ) {
      const row = this.db.challenges.get(this.binds[1] as string);
      const canConsume =
        row !== undefined &&
        row.cid_number === this.binds[2] &&
        row.binding_revision === this.binds[3] &&
        row.account_id === this.binds[4] &&
        row.used_at === null &&
        row.expires_at > (this.binds[5] as number);
      if (canConsume) row!.used_at = this.binds[0] as number;
      return { meta: { changes: canConsume ? 1 : 0 } };
    }
    return { meta: { changes: 0 } };
  }
}

class TakeoverDb {
  readonly challenges = new Map<string, ChallengeRecord>();
  prepare(sql: string): TakeoverStmt {
    return new TakeoverStmt(this, sql);
  }
  async batch(statements: TakeoverStmt[]): Promise<unknown[]> {
    return Promise.all(statements.map((statement) => statement.run()));
  }
}

function takeoverEnv(): { env: Env; db: TakeoverDb } {
  const db = new TakeoverDb();
  const env = {
    DB: db,
    SQUARE_CACHE: { delete: async () => undefined },
    CHAIN_GENESIS_HASH: `0x${'12'.repeat(32)}`
  } as unknown as Env;
  return { env, db };
}

function jsonPost(body: Record<string, unknown>): Request {
  return new Request('http://worker.test/v1/square/identity/takeover', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body)
  });
}

describe('consumeActionSignature', () => {
  beforeEach(() => mockVerify.mockReset());

  it('accepts a valid, unused, action-matching wallet signature and marks it used', async () => {
    const { env, db } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(
      env,
      STANDARD_CID,
      1,
      ACCOUNT_ID,
      'delete_account'
    );

    await expect(
      consumeActionSignature(env, {
        cidNumber: STANDARD_CID,
        bindingRevision: 1,
        accountId: ACCOUNT_ID,
        action: 'delete_account',
        challengeId: challenge.challengeId,
        signature: 'sig'
      })
    ).resolves.toBeUndefined();
    expect(db.challenges.get(challenge.challengeId)?.cid_number).toBe(STANDARD_CID);
    expect(db.challenges.get(challenge.challengeId)?.used_at).not.toBeNull();
  });

  it('rejects reuse of a consumed challenge', async () => {
    const { env } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(
      env,
      STANDARD_CID,
      1,
      ACCOUNT_ID,
      'delete_account'
    );
    const input = {
      cidNumber: STANDARD_CID,
      bindingRevision: 1,
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

  it('rejects a challenge owned by a different CID', async () => {
    const { env } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(
      env,
      STANDARD_CID,
      1,
      ACCOUNT_ID,
      'delete_account'
    );
    await expect(
      consumeActionSignature(env, {
        cidNumber: 'CN220-CTZN2-199001010-2026',
        bindingRevision: 1,
        accountId: ACCOUNT_ID,
        action: 'delete_account',
        challengeId: challenge.challengeId,
        signature: 'sig'
      })
    ).rejects.toMatchObject({ code: 'invalid_challenge' });
    expect(mockVerify).not.toHaveBeenCalled();
  });

  it('rejects a wrong accountId account', async () => {
    const { env } = challengeEnv();
    mockVerify.mockResolvedValue(true);
    const challenge = await issueActionChallenge(
      env,
      STANDARD_CID,
      1,
      ACCOUNT_ID,
      'delete_account'
    );
    await expect(
      consumeActionSignature(env, {
        cidNumber: STANDARD_CID,
        bindingRevision: 1,
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
    const challenge = await issueActionChallenge(
      env,
      STANDARD_CID,
      1,
      ACCOUNT_ID,
      'delete_account'
    );
    db.challenges.get(challenge.challengeId)!.expires_at = 1;
    await expect(
      consumeActionSignature(env, {
        cidNumber: STANDARD_CID,
        bindingRevision: 1,
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
    const challenge = await issueActionChallenge(
      env,
      STANDARD_CID,
      1,
      ACCOUNT_ID,
      'delete_account'
    );
    await expect(
      consumeActionSignature(env, {
        cidNumber: STANDARD_CID,
        bindingRevision: 1,
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
    const challenge = await issueActionChallenge(
      env,
      STANDARD_CID,
      1,
      ACCOUNT_ID,
      'delete_account'
    );
    const input = {
      cidNumber: STANDARD_CID,
      bindingRevision: 1,
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
    if (this.sql.includes('LEFT JOIN square_uploads')) {
      return this.db.postWithoutUpload as T | null;
    }
    return null;
  }
  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes('FROM square_media_assets') && this.sql.includes('provider_asset_id')) {
      return { results: this.db.mediaRows as T[] };
    }
    if (this.sql.includes('FROM square_uploads WHERE cid_number')) {
      return { results: this.db.uploadRows as T[] };
    }
    if (this.sql.includes('FROM square_sessions WHERE cid_number')) {
      const cidNumber = this.binds[0] as string;
      const accountId = this.sql.includes('AND account_id = ?')
        ? this.binds[1] as string
        : null;
      return {
        results: this.db.sessionRows
          .filter((row) =>
            row.cid_number === cidNumber &&
            (accountId === null || row.account_id === accountId)
          )
          .map((row) => ({ session_token_hash: row.session_token_hash })) as T[]
      };
    }
    return { results: [] };
  }
  async run(): Promise<{ meta: { changes: number } }> {
    return this.db.execute(this);
  }
}

class PurgeDb {
  mediaRows: MediaAssetRow[] = [];
  uploadRows: Array<{
    upload_id: string;
    post_id: string;
    cid_number: string;
    account_id: string;
    object_keys_json: string;
  }> = [];
  sessionRows: Array<{
    session_token_hash: string;
    cid_number: string;
    account_id: string;
  }> = [];
  chatDeviceRows: Array<{ cid_number: string; account_id: string }> = [];
  chatKeyPackageRows: Array<{ cid_number: string; account_id: string }> = [];
  deviceSubkeyRows: Array<{ cid_number: string; account_id: string }> = [];
  postWithoutUpload: { post_id: string } | null = null;
  readonly deletes: string[] = [];
  readonly batches: string[][] = [];
  prepare(sql: string): PurgeStmt {
    return new PurgeStmt(this, sql);
  }
  async batch(statements: PurgeStmt[]): Promise<Array<{ meta: { changes: number } }>> {
    this.batches.push(statements.map((statement) => statement.sql));
    return statements.map((statement) => this.execute(statement));
  }
  execute(statement: PurgeStmt): { meta: { changes: number } } {
    this.deletes.push(statement.sql);
    const deleteCredentialRows = (
      rows: Array<{ cid_number: string; account_id: string }>
    ): {
      kept: Array<{ cid_number: string; account_id: string }>;
      changes: number;
    } => {
      const cidNumber = statement.binds[0] as string;
      const accountId = statement.binds[1] as string;
      const kept = rows.filter(
        (row) =>
          row.cid_number !== cidNumber || row.account_id !== accountId
      );
      return { kept, changes: rows.length - kept.length };
    };
    if (
      statement.sql.includes(
        'DELETE FROM chat_devices WHERE cid_number = ? AND account_id = ?'
      )
    ) {
      const result = deleteCredentialRows(this.chatDeviceRows);
      this.chatDeviceRows = result.kept;
      return { meta: { changes: result.changes } };
    }
    if (
      statement.sql.includes(
        'DELETE FROM chat_keypackages WHERE cid_number = ? AND account_id = ?'
      )
    ) {
      const result = deleteCredentialRows(this.chatKeyPackageRows);
      this.chatKeyPackageRows = result.kept;
      return { meta: { changes: result.changes } };
    }
    if (
      statement.sql.includes(
        'DELETE FROM square_device_subkeys WHERE cid_number = ? AND account_id = ?'
      )
    ) {
      const result = deleteCredentialRows(this.deviceSubkeyRows);
      this.deviceSubkeyRows = result.kept;
      return { meta: { changes: result.changes } };
    }
    if (statement.sql.includes('DELETE FROM square_sessions WHERE cid_number = ?')) {
      const cidNumber = statement.binds[0] as string;
      const accountId = statement.sql.includes('AND account_id = ?')
        ? statement.binds[1] as string
        : null;
      const before = this.sessionRows.length;
      this.sessionRows = this.sessionRows.filter((row) =>
        row.cid_number !== cidNumber ||
        (accountId !== null && row.account_id !== accountId)
      );
      return { meta: { changes: before - this.sessionRows.length } };
    }
    return { meta: { changes: 1 } };
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
  failDeleteKey: string | null = null;
  async get<T>(key: string, type?: 'json'): Promise<T | string | null> {
    const value = this.store.get(key);
    if (value === undefined) return null;
    return type === 'json' ? JSON.parse(value) as T : value;
  }
  async put(key: string, value: string): Promise<void> {
    this.store.set(key, value);
  }
  async delete(key: string): Promise<void> {
    if (key === this.failDeleteKey) throw new Error('kv_delete_failed');
    this.store.delete(key);
  }
}

describe('purgeIdentity', () => {
  afterEach(() => vi.unstubAllGlobals());
  // 会员订阅与注销已解耦（公民币轨）：注销只硬删本地数据，不代签链上退订，
  // 因此不再有外部支付退订成功/失败分支，purge 也不再抛支付相关错误。
  function buildEnv(): {
    env: Env;
    db: PurgeDb;
    r2: FakeR2;
    kv: FakeKv;
  } {
    const db = new PurgeDb();
    db.mediaRows = [{
      upload_id: 'squ_1', post_id: 'sqp_1', cid_number: STANDARD_CID, account_id: ACCOUNT_ID, media_index: 0,
      media_kind: 'image', provider: 'cloudflare_images', provider_asset_id: 'img_1',
      upload_method: 'worker', resource_key: 'square_image_sd', content_type: 'image/webp',
      byte_size: 1024, asset_state: 'ready', declared_duration_seconds: null,
      duration_seconds: null, width: 100, height: 100, error_code: null,
      created_at: 1, updated_at: 1, ready_at: 1,
    }];
    const oldAccountId =
      '0x3333333333333333333333333333333333333333333333333333333333333333';
    db.uploadRows = [
      {
        upload_id: 'squ_1',
        post_id: 'sqp_1',
        cid_number: STANDARD_CID,
        account_id: ACCOUNT_ID,
        object_keys_json: JSON.stringify([
          `square/${ACCOUNT_ID.slice(2)}/posts/sqp_1/manifest.json`
        ])
      },
      {
        upload_id: 'squ_old',
        post_id: 'sqp_old',
        cid_number: STANDARD_CID,
        account_id: oldAccountId,
        object_keys_json: JSON.stringify([
          `square/${oldAccountId.slice(2)}/posts/sqp_old/manifest.json`
        ])
      }
    ];
    db.sessionRows = [
      {
        session_token_hash: PURGE_CURRENT_SESSION_HASH,
        cid_number: STANDARD_CID,
        account_id: ACCOUNT_ID
      },
      {
        session_token_hash: PURGE_OLD_SESSION_HASH,
        cid_number: STANDARD_CID,
        account_id: oldAccountId
      }
    ];
    const r2 = new FakeR2([
      `profile/${STANDARD_CID}/profile.json`,
      `profile/${STANDARD_CID}/avatar`,
      `square/${ACCOUNT_ID.slice(2)}/posts/sqp_1/manifest.json`,
      `square/${oldAccountId.slice(2)}/posts/sqp_old/manifest.json`
    ]);
    const kv = new FakeKv();
    kv.store.set(`square_identity:${ACCOUNT_ID}`, '{"identity_level":"voting"}');
    kv.store.set(`square_identity_cid:${STANDARD_CID}`, '{"identity_level":"voting"}');
    kv.store.set(
      `square_session:${PURGE_CURRENT_SESSION_HASH}`,
      JSON.stringify({ cid_number: STANDARD_CID, account_id: ACCOUNT_ID })
    );
    kv.store.set(
      `square_session:${PURGE_OLD_SESSION_HASH}`,
      JSON.stringify({ cid_number: STANDARD_CID, account_id: oldAccountId })
    );
    const env = {
      DB: db,
      SQUARE_MEDIA: r2,
      SQUARE_CACHE: kv,
      CF_ACCOUNT_ID: 'account',
      CF_API_TOKEN: 'token'
    } as unknown as Env;
    return { env, db, r2, kv };
  }

  it('按 CID 硬删除跨换绑数据与 R2 对象、清空全部身份会话', async () => {
    const { env, db, r2, kv } = buildEnv();
    vi.stubGlobal('fetch', vi.fn(async () => Response.json({ success: true, result: {} })));

    const result = await purgeIdentity(env, STANDARD_CID, ACCOUNT_ID);

    // PurgeIdentityResult 只返回本地硬删除计数，不触发任何外部订阅副作用。
    expect(result.deleted_media_assets).toBe(1);
    expect(result.deleted_r2_objects).toBe(4);
    expect(result.deleted_rows).toBeGreaterThan(0);

    // CID 的 Chat 路由、浏览、关注两端引用和业务表全部进入硬删除清单。
    const joined = db.deletes.join('\n');
    // R6:注销=删身份,身份内容(帖子/上传/媒体)按 cid_number 删该身份跨换绑账户全部内容。
    expect(joined).toContain('DELETE FROM square_posts WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_uploads WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_media_assets WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_device_subkeys WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_sessions WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_login_challenges WHERE cid_number = ?');
    // R5 起 chat_device_binding_nonces PK 改为 (cid_number, nonce_hash)，绑定 nonce 按身份主键 cid 硬删。
    expect(joined).toContain('DELETE FROM chat_device_binding_nonces WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM chat_devices WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM chat_keypackages WHERE cid_number = ?');
    // R3 起 square_memberships 随身份主键 cid_number 走（从 account_id 分支移出），
    // R4 起通讯录密文亦按 cid 归属，与订阅/预留/用量/创作者档一同按 cid 硬删（关注两端引用一并清）。
    expect(joined).toContain('DELETE FROM square_contacts WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_memberships WHERE cid_number = ?');
    expect(joined).toContain(
      'DELETE FROM chain_transaction_confirmations WHERE cid_number = ?'
    );
    expect(joined).toContain('DELETE FROM topup_orders WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM resource_reservations WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM resource_usage WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_creator_tiers WHERE creator_cid_number = ?');
    expect(joined).toContain(
      'DELETE FROM square_creator_subscriptions WHERE subscriber_cid_number = ? OR creator_cid_number = ?'
    );
    expect(joined).toContain(
      'DELETE FROM square_follows WHERE follower_cid_number = ? OR followed_cid_number = ?'
    );
    expect(joined).toContain('DELETE FROM square_browse_days WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_user_signals WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_notify_reads WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_request_nonces WHERE cid_number = ?');
    expect(joined).toContain('DELETE FROM square_rate_windows WHERE rate_key LIKE ?');

    // R2：资料按 CID；帖子按 D1 精确清单覆盖当前及换绑前发布账户。
    expect(r2.deleted).toContain(`profile/${STANDARD_CID}/profile.json`);
    expect(r2.deleted).toContain(
      `square/${ACCOUNT_ID.slice(2)}/posts/sqp_1/manifest.json`
    );
    expect(r2.deleted).toContain(
      `square/${'33'.repeat(32)}/posts/sqp_old/manifest.json`
    );

    // KV：CID 身份缓存及跨换绑会话全部清除。
    expect(kv.store.has(`square_identity:${ACCOUNT_ID}`)).toBe(false);
    expect(kv.store.has(`square_identity_cid:${STANDARD_CID}`)).toBe(false);
    expect(kv.store.has(`square_session:${PURGE_CURRENT_SESSION_HASH}`)).toBe(false);
    expect(kv.store.has(`square_session:${PURGE_OLD_SESSION_HASH}`)).toBe(false);
    expect(db.sessionRows).toEqual([]);

    // 存储总量释放与对应媒体行删除必须位于同一个 D1 原子 batch；禁止恢复先释放、
    // 后删内容的双阶段路径，否则注销重试会重复扣减全局容量。
    const contentDeleteBatch = db.batches.find((batch) =>
      batch.some((sql) => sql.includes('DELETE FROM square_media_assets WHERE cid_number = ?'))
    );
    expect(contentDeleteBatch).toBeDefined();
    expect(
      contentDeleteBatch!.filter((sql) => sql.includes('UPDATE resource_totals'))
    ).toHaveLength(2);
  });

  it('对象清单损坏时保留内容 D1 索引并拒绝继续 R2 清理', async () => {
    const { env, db } = buildEnv();
    db.uploadRows[0]!.object_keys_json = '[]';
    await expect(
      purgeIdentity(env, STANDARD_CID, ACCOUNT_ID)
    ).rejects.toMatchObject({ code: 'upload_object_keys_invalid' });
    expect(db.deletes.join('\n')).not.toContain(
      'DELETE FROM square_uploads WHERE cid_number = ?'
    );
  });

  it('跨存储清理中途失败不先释放总量，重试成功仅在内容删除原子批次释放一次', async () => {
    const { env, db, kv } = buildEnv();
    vi.stubGlobal('fetch', vi.fn(async () => Response.json({ success: true, result: {} })));
    kv.failDeleteKey = `square_identity_cid:${STANDARD_CID}`;

    await expect(
      purgeIdentity(env, STANDARD_CID, ACCOUNT_ID)
    ).rejects.toThrow('kv_delete_failed');
    expect(
      db.deletes.filter((sql) => sql.includes('UPDATE resource_totals'))
    ).toHaveLength(0);
    expect(
      db.deletes.some((sql) => sql.includes('DELETE FROM square_media_assets WHERE cid_number = ?'))
    ).toBe(false);

    kv.failDeleteKey = null;
    await expect(
      purgeIdentity(env, STANDARD_CID, ACCOUNT_ID)
    ).resolves.toMatchObject({ deleted_media_assets: 1 });

    const releaseBatches = db.batches.filter((batch) =>
      batch.some((sql) => sql.includes('UPDATE resource_totals'))
    );
    expect(releaseBatches).toHaveLength(1);
    expect(
      releaseBatches[0]!.filter((sql) => sql.includes('UPDATE resource_totals'))
    ).toHaveLength(2);
    expect(
      releaseBatches[0]!.some((sql) =>
        sql.includes('DELETE FROM square_media_assets WHERE cid_number = ?')
      )
    ).toBe(true);
  });
});

describe('注销入口默认拒（不再匿名对任意账户发起挑战）', () => {
  it('account/delete/challenge 无会话 → guard 默认拒 missing_session', async () => {
    const { env } = challengeEnv();
    const body = JSON.stringify({ account_id: ACCOUNT_ID });
    await expect(
      routeRequest(
        new Request('https://worker.test/v1/square/account/delete/challenge', {
          method: 'POST',
          headers: {
            'content-type': 'application/json',
            'content-length': String(new TextEncoder().encode(body).length)
          },
          body
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'missing_session' });
  });

  it('handler 拒绝用当前 CID 会话为其它账户申请注销挑战', async () => {
    const db = new ChallengeDb();
    const kv = new CredentialKv();
    kv.store.set(`square_session:${CURRENT_SESSION_HASH}`, {
      cid_number: STANDARD_CID,
      account_id: ACCOUNT_ID,
      device_key_hash: 'a'.repeat(64),
      created_at: 1,
      expires_at: Date.now() + 60_000
    });
    const env = { DB: db, SQUARE_CACHE: kv } as unknown as Env;
    const request = new Request(
      'https://worker.test/v1/square/account/delete/challenge',
      {
        method: 'POST',
        headers: {
          authorization: 'Bearer current-token',
          'content-type': 'application/json'
        },
        body: JSON.stringify({ account_id: NEW_ACCOUNT_ID })
      }
    );
    await expect(
      deleteAccountChallengeRoute(request, env)
    ).rejects.toMatchObject({ code: 'delete_account_mismatch' });
  });
});

describe('purgeFinalizedOldAccountCredentials', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('只删旧账户鉴权敏感数据(Chat/设备子钥/挑战/会话),不碰随身份迁移数据(通讯录/动态/会员/关注)', async () => {
    const db = new PurgeDb();
    const kv = new FakeKv();
    db.sessionRows = [
      {
        session_token_hash: OLD_SESSION_HASH,
        cid_number: STANDARD_CID,
        account_id: ACCOUNT_ID
      },
      {
        session_token_hash: NEW_SESSION_HASH,
        cid_number: STANDARD_CID,
        account_id: NEW_ACCOUNT_ID
      }
    ];
    kv.store.set(`square_identity:${ACCOUNT_ID}`, '{"identity_level":"voting"}');
    kv.store.set(
      `square_session:${OLD_SESSION_HASH}`,
      JSON.stringify({ cid_number: STANDARD_CID, account_id: ACCOUNT_ID })
    );
    kv.store.set(
      `square_session:${NEW_SESSION_HASH}`,
      JSON.stringify({ cid_number: STANDARD_CID, account_id: NEW_ACCOUNT_ID })
    );
    const env = { DB: db, SQUARE_CACHE: kv } as unknown as Env;

    const result = await purgeFinalizedOldAccountCredentials(
      env,
      STANDARD_CID,
      ACCOUNT_ID
    );

    const joined = db.deletes.join('\n');
    // 账户级鉴权敏感数据全删(Chat 端到端材料/登录挑战/设备子钥)。
    expect(joined).toContain(
      'DELETE FROM chat_keypackages WHERE cid_number = ? AND account_id = ?'
    );
    expect(joined).toContain(
      'DELETE FROM chat_devices WHERE cid_number = ? AND account_id = ?'
    );
    expect(joined).toContain(
      'DELETE FROM square_login_challenges WHERE cid_number = ? AND account_id = ?'
    );
    expect(joined).toContain(
      'DELETE FROM square_device_subkeys WHERE cid_number = ? AND account_id = ?'
    );
    // 换绑后这些 CID 级状态由新账户继续使用，绝不能随旧账户吊销。
    expect(joined).not.toContain('chat_device_binding_nonces');
    // 随 CID 迁到新账户的数据(永不丢失)绝不删——R4 起通讯录密文亦按 cid 归属,一并保留。
    expect(joined).not.toContain('square_contacts');
    expect(joined).not.toContain('square_posts');
    expect(joined).not.toContain('square_memberships');
    expect(joined).not.toContain('square_follows');
    expect(joined).not.toContain('square_media_assets');
    // 身份缓存清除 + 会话清空(旧私钥泄漏也无法重建旧会话)。
    expect(kv.store.has(`square_identity:${ACCOUNT_ID}`)).toBe(false);
    expect(kv.store.has(`square_session:${OLD_SESSION_HASH}`)).toBe(false);
    expect(kv.store.has(`square_session:${NEW_SESSION_HASH}`)).toBe(true);
    expect(db.sessionRows).toEqual([
      {
        session_token_hash: NEW_SESSION_HASH,
        cid_number: STANDARD_CID,
        account_id: NEW_ACCOUNT_ID
      }
    ]);
    expect(result.deleted_rows).toBeGreaterThan(0);
  });

  it('延迟旧 CID 事件不删除已复用同一账户的新 CID 凭证', async () => {
    const reusedCid = 'CN220-CTZN2-199001010-2026';
    const db = new PurgeDb();
    db.chatDeviceRows = [
      { cid_number: STANDARD_CID, account_id: ACCOUNT_ID },
      { cid_number: reusedCid, account_id: ACCOUNT_ID },
      { cid_number: STANDARD_CID, account_id: NEW_ACCOUNT_ID }
    ];
    db.chatKeyPackageRows = [...db.chatDeviceRows];
    db.deviceSubkeyRows = [...db.chatDeviceRows];
    db.sessionRows = [
      {
        session_token_hash: OLD_SESSION_HASH,
        cid_number: STANDARD_CID,
        account_id: ACCOUNT_ID
      },
      {
        session_token_hash: NEW_SESSION_HASH,
        cid_number: reusedCid,
        account_id: ACCOUNT_ID
      }
    ];
    const kv = new FakeKv();
    kv.store.set(
      `square_session:${OLD_SESSION_HASH}`,
      JSON.stringify({ cid_number: STANDARD_CID, account_id: ACCOUNT_ID })
    );
    kv.store.set(
      `square_session:${NEW_SESSION_HASH}`,
      JSON.stringify({ cid_number: reusedCid, account_id: ACCOUNT_ID })
    );
    const env = { DB: db, SQUARE_CACHE: kv } as unknown as Env;

    await purgeFinalizedOldAccountCredentials(
      env,
      STANDARD_CID,
      ACCOUNT_ID
    );

    const expectedRows = [
      { cid_number: reusedCid, account_id: ACCOUNT_ID },
      { cid_number: STANDARD_CID, account_id: NEW_ACCOUNT_ID }
    ];
    expect(db.chatDeviceRows).toEqual(expectedRows);
    expect(db.chatKeyPackageRows).toEqual(expectedRows);
    expect(db.deviceSubkeyRows).toEqual(expectedRows);
    expect(db.sessionRows).toEqual([
      {
        session_token_hash: NEW_SESSION_HASH,
        cid_number: reusedCid,
        account_id: ACCOUNT_ID
      }
    ]);
    expect(kv.store.has(`square_session:${NEW_SESSION_HASH}`)).toBe(true);
  });
});

class CredentialKv {
  readonly store = new Map<string, unknown>();

  async get<T>(key: string, type?: 'json'): Promise<T | string | null> {
    const value = this.store.get(key);
    if (value === undefined) return null;
    if (type === 'json') return value as T;
    return typeof value === 'string' ? value : JSON.stringify(value);
  }

  async delete(key: string): Promise<void> {
    this.store.delete(key);
  }

  async put(key: string, value: string): Promise<void> {
    this.store.set(key, value);
  }

}
