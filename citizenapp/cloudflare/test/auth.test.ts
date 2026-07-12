import { afterEach, describe, expect, it, vi } from 'vitest';
import { createLoginChallenge, createSession } from '../src/auth/service';
import { hexToBytes, signingMessage } from '../src/shared/signing_message';
import type { Env } from '../src/types';

const OWNER = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

/// 构造 `System.Account` 值的 SCALE hex：16 字节前导（nonce/consumers/providers/sufficients）
/// + `data.free`（u128 LE，偏移 16）+ 其余 data 字段（reserved/frozen/flags 置零）。
function accountInfoHex(free: bigint): string {
  const buf = new Uint8Array(80);
  let v = free;
  for (let i = 0; i < 16; i++) {
    buf[16 + i] = Number(v & 0xffn);
    v >>= 8n;
  }
  return '0x' + [...buf].map((b) => b.toString(16).padStart(2, '0')).join('');
}

/// stub 链 RPC：让 `state_getStorage` 返回给定 result（hex 或 null=账户不存在）。
function stubChainResult(result: string | null): void {
  vi.stubGlobal(
    'fetch',
    vi.fn(
      async () =>
        new Response(JSON.stringify({ jsonrpc: '2.0', id: 1, result }), {
          headers: { 'content-type': 'application/json' }
        })
    )
  );
}

/// stub 链 RPC 故障（节点宕机/HTTP 失败）→ 校验 fail-closed。
function stubChainFailure(): void {
  vi.stubGlobal('fetch', vi.fn(async () => new Response('upstream down', { status: 502 })));
}

interface ChallengeRow {
  challenge_id: string;
  owner_account: string;
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
        owner_account: this.binds[1] as string,
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
      return pubkey ? ({ p256_pubkey: pubkey } as T) : null;
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
      SQUARE_CACHE: kv,
      CHAIN_URL: 'https://chain.test',
      CHAIN_ID: 'worker-rpc.access',
      CHAIN_SECRET: 'test-access-secret'
    } as unknown as Env;
    const keyPair = await crypto.subtle.generateKey(
      { name: 'ECDSA', namedCurve: 'P-256' },
      true,
      ['sign', 'verify']
    );
    const pubHex = toHex(await crypto.subtle.exportKey('raw', keyPair.publicKey));
    db.subkeys.set(OWNER, pubHex);
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

  it('signs the op_tag login message with the device subkey and mints a session', async () => {
    const { env, keyPair } = await setup();

    const challenge = await jsonBody(
      await createLoginChallenge(req('/v1/square/auth/challenge', { owner_account: OWNER }), env)
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
    // 链上钱包余额 ≥ ED（111 分）→ 放行。
    stubChainResult(accountInfoHex(1_000n));
    const session = await jsonBody(
      await createSession(
        req('/v1/square/auth/session', {
          owner_account: OWNER,
          challenge_id: challenge.challenge_id,
          signature
        }),
        env
      )
    );
    expect(session.ok).toBe(true);
    expect(typeof session.session_token).toBe('string');
  });

  async function signedSessionRequest(): Promise<{ env: Env; body: unknown }> {
    const { env, keyPair } = await setup();
    const challenge = await jsonBody(
      await createLoginChallenge(req('/v1/square/auth/challenge', { owner_account: OWNER }), env)
    );
    const signature = await signChallenge(
      keyPair,
      challenge.op_tag as number,
      challenge.signing_payload_hex as string
    );
    return {
      env,
      body: {
        owner_account: OWNER,
        challenge_id: challenge.challenge_id,
        signature
      }
    };
  }

  it('rejects a wallet whose on-chain free balance is below the existential deposit', async () => {
    const { env, body } = await signedSessionRequest();
    // free = 110 分 < ED 111 → 非链上钱包。
    stubChainResult(accountInfoHex(110n));
    await expect(
      createSession(req('/v1/square/auth/session', body), env)
    ).rejects.toMatchObject({ code: 'not_onchain_wallet' });
  });

  it('rejects a wallet that has no on-chain account (reaped or never funded)', async () => {
    const { env, body } = await signedSessionRequest();
    // System.Account 不存在 → result null。
    stubChainResult(null);
    await expect(
      createSession(req('/v1/square/auth/session', body), env)
    ).rejects.toMatchObject({ code: 'not_onchain_wallet' });
  });

  it('fails closed when the chain node is unreachable during session issuance', async () => {
    const { env, body } = await signedSessionRequest();
    stubChainFailure();
    await expect(
      createSession(req('/v1/square/auth/session', body), env)
    ).rejects.toMatchObject({ code: 'chain_rpc_http_failed' });
  });

  it('rejects a signature over the wrong message', async () => {
    const { env, keyPair } = await setup();
    const challenge = await jsonBody(
      await createLoginChallenge(req('/v1/square/auth/challenge', { owner_account: OWNER }), env)
    );
    // 对错误 payload 签名 → 摘要不符 → 拒。
    const badSignature = await signChallenge(keyPair, 0x1b, '00'.repeat(8));
    await expect(
      createSession(
        req('/v1/square/auth/session', {
          owner_account: OWNER,
          challenge_id: challenge.challenge_id,
          signature: badSignature
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_signature' });
  });
});
