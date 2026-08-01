import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Miniflare } from 'miniflare';

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
import { purgeIdentity } from '../src/account/purge';
import {
  cidTakeoverChallengeRoute,
  cidTakeoverRoute
} from '../src/account/cid_data_root';
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
const CURRENT_SESSION_HASH =
  'ef6036bfacfc26e4d8f0ea4199e6c1a4571376f5e9949854a07ef59530d5d50b';
const SCHEMA_SQL = readFileSync(
  resolve(process.cwd(), 'schema/citizenapp.sql'),
  'utf8'
);

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

interface DataRootRecord {
  cid_number: string;
  recovery_ciphertext: string;
  recovery_nonce: string;
  recovery_key_version: number;
  data_root_hash: string;
  active_binding_revision: number;
  active_account_id: string;
  created_at: number;
  updated_at: number;
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
    if (this.sql.includes('FROM cid_data_roots')) {
      return (this.db.roots.get(this.binds[0] as string) as T) ?? null;
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
    if (this.sql.includes('INSERT INTO cid_data_roots')) {
      const cidNumber = this.binds[0] as string;
      if (!this.db.roots.has(cidNumber)) {
        this.db.roots.set(cidNumber, {
          cid_number: cidNumber,
          recovery_ciphertext: this.binds[1] as string,
          recovery_nonce: this.binds[2] as string,
          recovery_key_version: this.binds[3] as number,
          data_root_hash: this.binds[4] as string,
          active_binding_revision: this.binds[5] as number,
          active_account_id: this.binds[6] as string,
          created_at: this.binds[7] as number,
          updated_at: this.binds[8] as number
        });
        return { meta: { changes: 1 } };
      }
      return { meta: { changes: 0 } };
    }
    if (this.sql.includes('UPDATE cid_data_roots')) {
      const row = this.db.roots.get(this.binds[3] as string);
      const canAdvance =
        row !== undefined &&
        row.active_binding_revision <= (this.binds[4] as number);
      if (canAdvance) {
        row!.active_binding_revision = this.binds[0] as number;
        row!.active_account_id = this.binds[1] as string;
        row!.updated_at = this.binds[2] as number;
      }
      return { meta: { changes: canAdvance ? 1 : 0 } };
    }
    return { meta: { changes: 0 } };
  }
}

class TakeoverDb {
  readonly challenges = new Map<string, ChallengeRecord>();
  readonly roots = new Map<string, DataRootRecord>();
  prepare(sql: string): TakeoverStmt {
    return new TakeoverStmt(this, sql);
  }
}

function takeoverEnv(): { env: Env; db: TakeoverDb } {
  const db = new TakeoverDb();
  const env = {
    DB: db,
    CHAIN_GENESIS_HASH: `0x${'12'.repeat(32)}`,
    CID_DATA_ROOT_RECOVERY_KEY: `0x${'34'.repeat(32)}`
  } as unknown as Env;
  return { env, db };
}

function jsonPost(body: Record<string, unknown>): Request {
  const encoded = JSON.stringify(body);
  return new Request('http://worker.test/v1/square/identity/takeover', {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'content-length': String(new TextEncoder().encode(encoded).length)
    },
    body: encoded
  });
}

function routeJsonPost(path: string, body: Record<string, unknown>): Request {
  const encoded = JSON.stringify(body);
  return new Request(`http://worker.test${path}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'content-length': String(new TextEncoder().encode(encoded).length),
      'cf-connecting-ip': '127.0.0.1'
    },
    body: encoded
  });
}

async function applySchema(db: D1Database): Promise<void> {
  const statements = SCHEMA_SQL
    .split('\n')
    .filter((line) => !line.trimStart().startsWith('--'))
    .join('\n')
    .split(';')
    .map((statement) => statement.trim())
    .filter((statement) => statement.length > 0);
  for (const statement of statements) await db.prepare(statement).run();
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function base64ToBytes(value: unknown): Uint8Array {
  if (typeof value !== 'string') throw new Error('base64 expected');
  const binary = atob(value);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function arrayBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(
    bytes.byteOffset,
    bytes.byteOffset + bytes.byteLength,
  ) as ArrayBuffer;
}

function hex(bytes: Uint8Array): string {
  return `0x${Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('')}`;
}

async function decryptTakeoverGrant(
  env: Env,
  recipientKeyPair: CryptoKeyPair,
  recipientPublicKey: string,
  challengeId: string,
  granted: Record<string, unknown>,
): Promise<Uint8Array> {
  const senderPublicKey = granted.recovery_sender_public_key;
  const dataRootHash = granted.data_root_hash;
  if (typeof senderPublicKey !== 'string' || typeof dataRootHash !== 'string') {
    throw new Error('grant fields missing');
  }
  const sender = await crypto.subtle.importKey(
    'raw',
    Uint8Array.from(senderPublicKey.slice(2).match(/../g)!, (part) => Number.parseInt(part, 16)),
    { name: 'X25519' },
    false,
    []
  );
  const shared = new Uint8Array(await crypto.subtle.deriveBits(
    { name: 'X25519', public: sender },
    recipientKeyPair.privateKey,
    256
  ));
  try {
    const baseKey = await crypto.subtle.importKey('raw', shared, 'HKDF', false, ['deriveKey']);
    const salt = new TextEncoder().encode(
      `citizenapp.cid-data-root/recovery-grant/salt|${env.CHAIN_GENESIS_HASH}|`
      + `${STANDARD_CID}|${finalizedBinding.revision}|${finalizedBinding.accountId}|${challengeId}`
    );
    const key = await crypto.subtle.deriveKey(
      { name: 'HKDF', hash: 'SHA-256', salt, info: new TextEncoder().encode('citizenapp.cid-data-root/recovery-grant') },
      baseKey,
      { name: 'AES-GCM', length: 256 },
      false,
      ['decrypt']
    );
    const aad = new TextEncoder().encode(
      `citizenapp.cid-data-root/recovery-grant|${env.CHAIN_GENESIS_HASH}|${STANDARD_CID}|`
      + `${finalizedBinding.revision}|${finalizedBinding.accountId}|${challengeId}|`
      + `${recipientPublicKey}|${senderPublicKey}|${dataRootHash}`
    );
    const decrypted = await crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: arrayBuffer(base64ToBytes(granted.recovery_nonce_base64)),
        additionalData: arrayBuffer(aad),
        tagLength: 128,
      },
      key,
      arrayBuffer(base64ToBytes(granted.encrypted_cid_data_root_base64))
    );
    return new Uint8Array(decrypted);
  } finally {
    shared.fill(0);
  }
}

describe('CID finalized 新账户接管', () => {
  beforeEach(() => {
    mockVerify.mockReset();
    mockVerify.mockResolvedValue(true);
    finalizedBinding.accountId = ACCOUNT_ID;
    finalizedBinding.revision = 1;
  });

  async function issueAndTakeover(env: Env, cidNumber = STANDARD_CID) {
    const generated = await crypto.subtle.generateKey(
      { name: 'X25519' },
      true,
      ['deriveBits']
    );
    if (!('privateKey' in generated)) throw new Error('X25519 key pair expected');
    const recipientPublicKey = hex(new Uint8Array(
      await crypto.subtle.exportKey('raw', generated.publicKey)
    ));
    const challengeResponse = await cidTakeoverChallengeRoute(
      jsonPost({
        cid_number: cidNumber,
        account_id: finalizedBinding.accountId,
        recovery_public_key: recipientPublicKey
      }),
      env
    );
    const challenge = await challengeResponse.json<Record<string, unknown>>();
    const takeoverResponse = await cidTakeoverRoute(
      jsonPost({
        cid_number: cidNumber,
        binding_revision: finalizedBinding.revision,
        account_id: finalizedBinding.accountId,
        recovery_public_key: recipientPublicKey,
        challenge_id: challenge.challenge_id,
        signature: '0xsignature'
      }),
      env
    );
    const granted = await takeoverResponse.json<Record<string, unknown>>();
    return {
      challenge,
      granted,
      recipientPublicKey,
      dataRoot: await decryptTakeoverGrant(
        env,
        generated,
        recipientPublicKey,
        challenge.challenge_id as string,
        granted
      )
    };
  }

  it('挑战完整绑定且数据根只以 X25519 加密信封返回，并只消费一次', async () => {
    const { env } = takeoverEnv();
    const { challenge, granted, dataRoot } = await issueAndTakeover(env);
    expect(challenge.cid_number).toBe(STANDARD_CID);
    expect(challenge.binding_revision).toBe(1);
    expect(challenge.account_id).toBe(ACCOUNT_ID);
    expect(granted).not.toHaveProperty('cid_data_root_base64');
    expect(base64ToBytes(granted.encrypted_cid_data_root_base64)).toHaveLength(48);
    expect(dataRoot).toHaveLength(32);
    expect(granted.data_root_hash).toMatch(/^[0-9a-f]{64}$/);
    expect(mockVerify).toHaveBeenCalledTimes(1);

    await expect(
      cidTakeoverRoute(
        jsonPost({
          cid_number: STANDARD_CID,
          binding_revision: 1,
          account_id: ACCOUNT_ID,
          recovery_public_key: challenge.recovery_public_key,
          challenge_id: challenge.challenge_id,
          signature: '0xsignature'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'used_challenge' });
  });

  it('通过完整 Worker 路由与真实本地 D1/KV binding 完成加密接管', async () => {
    const miniflare = new Miniflare({
      modules: true,
      script: 'export default { fetch() { return new Response("test"); } }',
      // 测试随仓库锁定的 workerd 最高只支持到 2026-07-29；生产配置仍为 2026-07-23。
      compatibilityDate: '2026-07-29',
      d1Databases: ['DB'],
      kvNamespaces: ['SQUARE_CACHE'],
      bindings: {
        HASH_KEY: 'cid-data-root-rate-limit-test-key',
        CHAIN_GENESIS_HASH: `0x${'12'.repeat(32)}`,
        CID_DATA_ROOT_RECOVERY_KEY: `0x${'34'.repeat(32)}`
      }
    });
    try {
      const env = await miniflare.getBindings<Env>();
      await applySchema(env.DB);
      const generated = await crypto.subtle.generateKey(
        { name: 'X25519' },
        true,
        ['deriveBits']
      );
      if (!('privateKey' in generated)) throw new Error('X25519 key pair expected');
      const recipientPublicKey = hex(new Uint8Array(
        await crypto.subtle.exportKey('raw', generated.publicKey)
      ));
      const challengeResponse = await routeRequest(
        routeJsonPost('/v1/square/identity/takeover/challenge', {
          cid_number: STANDARD_CID,
          account_id: ACCOUNT_ID,
          recovery_public_key: recipientPublicKey
        }),
        env
      );
      expect(challengeResponse.status).toBe(200);
      const challenge = await challengeResponse.json<Record<string, unknown>>();
      const takeoverResponse = await routeRequest(
        routeJsonPost('/v1/square/identity/takeover', {
          cid_number: STANDARD_CID,
          binding_revision: 1,
          account_id: ACCOUNT_ID,
          recovery_public_key: recipientPublicKey,
          challenge_id: challenge.challenge_id,
          signature: '0xsignature'
        }),
        env
      );
      expect(takeoverResponse.status).toBe(200);
      const granted = await takeoverResponse.json<Record<string, unknown>>();
      expect(await decryptTakeoverGrant(
        env,
        generated,
        recipientPublicKey,
        challenge.challenge_id as string,
        granted
      )).toHaveLength(32);
      const row = await env.DB.prepare(
        `SELECT recovery_ciphertext, recovery_nonce, data_root_hash
          FROM cid_data_roots WHERE cid_number = ?`
      ).bind(STANDARD_CID).first<{
        recovery_ciphertext: string;
        recovery_nonce: string;
        data_root_hash: string;
      }>();
      expect(row?.recovery_ciphertext).toMatch(/^[A-Za-z0-9+/]+={0,2}$/);
      expect(row?.recovery_nonce).toMatch(/^[A-Za-z0-9+/]+={0,2}$/);
      expect(row?.data_root_hash).toBe(granted.data_root_hash);
      expect(row).not.toHaveProperty('cid_data_root');
    } finally {
      await miniflare.dispose();
    }
  });

  it('旧钱包与旧设备完全不参与，revision 推进后新账户取得同一数据根', async () => {
    const { env, db } = takeoverEnv();
    const first = await issueAndTakeover(env);
    finalizedBinding.accountId = NEW_ACCOUNT_ID;
    finalizedBinding.revision = 2;
    const second = await issueAndTakeover(env);

    expect(second.dataRoot).toEqual(first.dataRoot);
    expect(second.granted.data_root_hash).toBe(first.granted.data_root_hash);
    expect(db.roots.get(STANDARD_CID)?.active_binding_revision).toBe(2);
    expect(db.roots.get(STANDARD_CID)?.active_account_id).toBe(NEW_ACCOUNT_ID);
    expect(mockVerify).toHaveBeenLastCalledWith(
      expect.any(Uint8Array),
      '0xsignature',
      NEW_ACCOUNT_ID
    );
  });

  it('替换临时接收公钥、跨创世重放或 finalized 绑定变化全部失败关闭', async () => {
    const { env } = takeoverEnv();
    const generated = await crypto.subtle.generateKey({ name: 'X25519' }, true, ['deriveBits']);
    if (!('privateKey' in generated)) throw new Error('X25519 key pair expected');
    const recipientPublicKey = hex(new Uint8Array(await crypto.subtle.exportKey('raw', generated.publicKey)));
    const response = await cidTakeoverChallengeRoute(
      jsonPost({ cid_number: STANDARD_CID, account_id: ACCOUNT_ID, recovery_public_key: recipientPublicKey }),
      env
    );
    const challenge = await response.json<Record<string, unknown>>();

    await expect(cidTakeoverRoute(jsonPost({
      cid_number: STANDARD_CID,
      binding_revision: 1,
      account_id: ACCOUNT_ID,
      recovery_public_key: `0x${'56'.repeat(32)}`,
      challenge_id: challenge.challenge_id,
      signature: '0xsignature'
    }), env)).rejects.toMatchObject({ code: 'takeover_payload_mismatch' });

    env.CHAIN_GENESIS_HASH = `0x${'99'.repeat(32)}`;
    await expect(cidTakeoverRoute(jsonPost({
      cid_number: STANDARD_CID,
      binding_revision: 1,
      account_id: ACCOUNT_ID,
      recovery_public_key: recipientPublicKey,
      challenge_id: challenge.challenge_id,
      signature: '0xsignature'
    }), env)).rejects.toMatchObject({ code: 'takeover_payload_mismatch' });

    env.CHAIN_GENESIS_HASH = `0x${'12'.repeat(32)}`;
    finalizedBinding.accountId = NEW_ACCOUNT_ID;
    finalizedBinding.revision = 2;
    await expect(cidTakeoverRoute(jsonPost({
      cid_number: STANDARD_CID,
      binding_revision: 1,
      account_id: ACCOUNT_ID,
      recovery_public_key: recipientPublicKey,
      challenge_id: challenge.challenge_id,
      signature: '0xsignature'
    }), env)).rejects.toMatchObject({ code: 'cid_binding_changed' });
  });

  it('过期挑战在验签前拒绝', async () => {
    const { env, db } = takeoverEnv();
    const generated = await crypto.subtle.generateKey({ name: 'X25519' }, true, ['deriveBits']);
    if (!('privateKey' in generated)) throw new Error('X25519 key pair expected');
    const recipientPublicKey = hex(new Uint8Array(await crypto.subtle.exportKey('raw', generated.publicKey)));
    const response = await cidTakeoverChallengeRoute(
      jsonPost({ cid_number: STANDARD_CID, account_id: ACCOUNT_ID, recovery_public_key: recipientPublicKey }),
      env
    );
    const challenge = await response.json<Record<string, unknown>>();
    db.challenges.get(challenge.challenge_id as string)!.expires_at = 1;
    await expect(cidTakeoverRoute(jsonPost({
      cid_number: STANDARD_CID,
      binding_revision: 1,
      account_id: ACCOUNT_ID,
      recovery_public_key: recipientPublicKey,
      challenge_id: challenge.challenge_id,
      signature: '0xsignature'
    }), env)).rejects.toMatchObject({ code: 'expired_challenge' });
    expect(mockVerify).not.toHaveBeenCalled();
  });
});

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
