import { afterEach, describe, expect, it, vi } from 'vitest';
import { createLoginChallenge, createSession } from '../src/auth/service';
import { hexToBytes, signingMessage } from '../src/shared/signing_message';
import { sha256Hex } from '../src/shared/hash';
import type { Env } from '../src/types';

const ACCOUNT_ID = '0x1111111111111111111111111111111111111111111111111111111111111111';
// 身份主键 = 该钱包账户链上绑定的 cid_number;登录先解析它(测试里 mock 掉链)。
const TEST_CID = 'CN220-CTZN2-198805200-2026';
vi.mock('../src/chain/identity', () => ({
  fetchChainIdentityStateCached: vi.fn(async (_env: unknown, accountId: string) => ({
    account_id: accountId,
    identity_level: 'visitor',
    has_voting_identity: false,
    has_candidate_identity: false,
    cid_number: 'CN220-CTZN2-198805200-2026',
    checked_at: 0
  }))
}));

interface ChallengeRow {
  challenge_id: string;
  account_id: string;
  signing_payload: string;
  expires_at: number;
  used_at: number | null;
}

class AuthStmt {
  private binds: unknown[] = [];
  constructor(private readonly db: AuthDb, private readonly sql: string) {}
  bind(...args: unknown[]): AuthStmt {
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
      return { meta: { changes: 1 } };
    } else if (this.sql.includes('UPDATE square_login_challenges')) {
      const row = this.db.challenges.get(this.binds[1] as string);
      const accountId = this.binds[2] as string;
      const claimedAt = this.binds[3] as number;
      if (
        row &&
        row.account_id === accountId &&
        row.used_at === null &&
        row.expires_at > claimedAt
      ) {
        row.used_at = this.binds[0] as number;
        return { meta: { changes: 1 } };
      }
      return { meta: { changes: 0 } };
    }
    return { meta: { changes: 0 } };
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM square_login_challenges')) {
      return (this.db.challenges.get(this.binds[0] as string) as T) ?? null;
    }
    return null;
  }
  // 登录按 (cid_number, account_id) 取该身份+账户下的全部设备子钥(可多设备)。
  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes('FROM square_device_subkeys')) {
      const cid = this.binds[0] as string;
      const accountId = this.binds[1] as string;
      const results = [...this.db.subkeys.values()]
        .filter((row) => row.cid_number === cid && row.account_id === accountId)
        .map((row) => ({ p256_public_key: row.p256_public_key }));
      return { results: results as T[] };
    }
    return { results: [] };
  }
}

interface StoredSubkey {
  cid_number: string;
  device_id: string;
  account_id: string;
  p256_public_key: string;
}

class AuthDb {
  readonly challenges = new Map<string, ChallengeRow>();
  readonly subkeys = new Map<string, StoredSubkey>();
  prepare(sql: string): AuthStmt {
    return new AuthStmt(this, sql);
  }
}

class FakeKv {
  store = new Map<string, string>();
  failNextPut = false;
  async get(key: string): Promise<string | null> {
    return this.store.get(key) ?? null;
  }
  async put(key: string, value: string): Promise<void> {
    if (this.failNextPut) {
      this.failNextPut = false;
      throw new Error('kv_write_failed');
    }
    this.store.set(key, value);
  }
  async delete(key: string): Promise<void> {
    this.store.delete(key);
  }
}

function toHex(buf: ArrayBuffer): string {
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

async function jsonBody(request: Response): Promise<Record<string, unknown>> {
  return (await request.json()) as Record<string, unknown>;
}

function req(path: string, body: unknown): Request {
  return new Request(`https://worker.test${path}`, {
    method: 'POST',
    body: JSON.stringify(body)
  });
}

describe('square login (op_tag OP_SIGN_SQUARE_LOGIN)', () => {
  afterEach(() => vi.unstubAllGlobals());

  async function setup() {
    const db = new AuthDb();
    const kv = new FakeKv();
    const env = {
      DB: db,
      SQUARE_CACHE: kv
    } as unknown as Env;
    const keyPair = await crypto.subtle.generateKey(
      { name: 'ECDSA', namedCurve: 'P-256' },
      true,
      ['sign', 'verify']
    );
    const pubHex = toHex(await crypto.subtle.exportKey('raw', keyPair.publicKey));
    const deviceId = await sha256Hex(pubHex);
    db.subkeys.set(`${TEST_CID}:${deviceId}`, {
      cid_number: TEST_CID,
      device_id: deviceId,
      account_id: ACCOUNT_ID,
      p256_public_key: pubHex
    });
    return { db, kv, env, keyPair };
  }

  async function signChallenge(
    keyPair: CryptoKeyPair,
    opTag: number,
    payloadHex: string
  ): Promise<string> {
    const message = signingMessage(opTag, hexToBytes(payloadHex));
    const sig = await crypto.subtle.sign(
      { name: 'ECDSA', hash: 'SHA-256' },
      keyPair.privateKey,
      message
    );
    // 跨端签名文本统一带 `0x`（ADR-041）；后端入口 normalize 为裸后验签。
    return `0x${toHex(sig)}`;
  }

  it('只验证设备子钥并签发 Session，全程不访问链账户、余额或 RPC', async () => {
    const { env, keyPair } = await setup();
    const fetchSpy = vi.fn(async () => {
      throw new Error('Session 不得访问链 RPC');
    });
    vi.stubGlobal('fetch', fetchSpy);

    const challenge = await jsonBody(
      await createLoginChallenge(req('/v1/square/auth/challenge', { account_id: ACCOUNT_ID }), env)
    );
    expect(challenge.op_tag).toBe(0x1b);
    expect(typeof challenge.signing_payload_hex).toBe('string');
    // 只下发 32 字节摘要，不下发任何字符串域。
    expect(challenge.signing_payload_hex).not.toContain('GMB');

    const signature = await signChallenge(
      keyPair,
      challenge.op_tag as number,
      challenge.signing_payload_hex as string
    );
    const session = await jsonBody(
      await createSession(
        req('/v1/square/auth/session', {
          account_id: ACCOUNT_ID,
          challenge_id: challenge.challenge_id,
          signature
        }),
        env
      )
    );
    expect(session.ok).toBe(true);
    expect(typeof session.session_token).toBe('string');
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('rejects a signature over the wrong message', async () => {
    const { env, keyPair } = await setup();
    const challenge = await jsonBody(
      await createLoginChallenge(req('/v1/square/auth/challenge', { account_id: ACCOUNT_ID }), env)
    );
    // 对错误 payload 签名 → 摘要不符 → 拒。
    const badSignature = await signChallenge(keyPair, 0x1b, '00'.repeat(8));
    await expect(
      createSession(
        req('/v1/square/auth/session', {
          account_id: ACCOUNT_ID,
          challenge_id: challenge.challenge_id,
          signature: badSignature
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_signature' });
  });

  it('并发提交同一挑战时只签发一个 Session', async () => {
    const { env, db, kv, keyPair } = await setup();
    const challenge = await jsonBody(
      await createLoginChallenge(req('/v1/square/auth/challenge', { account_id: ACCOUNT_ID }), env)
    );
    const signature = await signChallenge(
      keyPair,
      challenge.op_tag as number,
      challenge.signing_payload_hex as string
    );
    const requestBody = {
      account_id: ACCOUNT_ID,
      challenge_id: challenge.challenge_id,
      signature
    };

    const results = await Promise.allSettled([
      createSession(req('/v1/square/auth/session', requestBody), env),
      createSession(req('/v1/square/auth/session', requestBody), env)
    ]);
    expect(results.filter((result) => result.status === 'fulfilled')).toHaveLength(1);
    const rejected = results.find((result) => result.status === 'rejected');
    expect(rejected).toMatchObject({ reason: { code: 'used_challenge' } });
    expect(db.challenges.get(challenge.challenge_id as string)?.used_at).not.toBeNull();
    expect([...kv.store.keys()].filter((key) => key.startsWith('square_session:'))).toHaveLength(1);
  });

  it('KV 写入失败后仍烧毁挑战且不留下孤立 Session', async () => {
    const { env, db, kv, keyPair } = await setup();
    const challenge = await jsonBody(
      await createLoginChallenge(req('/v1/square/auth/challenge', { account_id: ACCOUNT_ID }), env)
    );
    const signature = await signChallenge(
      keyPair,
      challenge.op_tag as number,
      challenge.signing_payload_hex as string
    );
    kv.failNextPut = true;

    await expect(
      createSession(
        req('/v1/square/auth/session', {
          account_id: ACCOUNT_ID,
          challenge_id: challenge.challenge_id,
          signature
        }),
        env
      )
    ).rejects.toThrow('kv_write_failed');
    expect(db.challenges.get(challenge.challenge_id as string)?.used_at).not.toBeNull();
    expect([...kv.store.keys()].filter((key) => key.startsWith('square_session:'))).toHaveLength(0);
    await expect(
      createSession(
        req('/v1/square/auth/session', {
          account_id: ACCOUNT_ID,
          challenge_id: challenge.challenge_id,
          signature
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'used_challenge' });
  });
});
