import { afterEach, describe, expect, it, vi } from 'vitest';
import { createLoginChallenge, createSession } from '../src/auth/service';
import { hexToBytes, signingMessage } from '../src/shared/signing_message';
import type { Env } from '../src/types';

const ACCOUNT_ID = '0x1111111111111111111111111111111111111111111111111111111111111111';

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
    } else if (this.sql.includes('UPDATE square_login_challenges SET used_at')) {
      const row = this.db.challenges.get(this.binds[1] as string);
      if (row) row.used_at = this.binds[0] as number;
    }
    return { meta: { changes: 1 } };
  }
  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM square_login_challenges')) {
      return (this.db.challenges.get(this.binds[0] as string) as T) ?? null;
    }
    if (this.sql.includes('FROM square_device_subkeys')) {
      const pubkey = this.db.subkeys.get(this.binds[0] as string);
      return pubkey ? ({ p256_public_key: pubkey } as T) : null;
    }
    return null;
  }
}

class AuthDb {
  readonly challenges = new Map<string, ChallengeRow>();
  readonly subkeys = new Map<string, string>();
  prepare(sql: string): AuthStmt {
    return new AuthStmt(this, sql);
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
    db.subkeys.set(ACCOUNT_ID, pubHex);
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
    return toHex(sig);
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
    // 不再下发任何 GMB_*_V1 字符串域。
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
});
