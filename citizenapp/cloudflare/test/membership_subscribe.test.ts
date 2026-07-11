import { afterEach, describe, expect, it, vi } from 'vitest';
import { scaleCompact as compactU32 } from '../src/shared/signing_message';
import { encodeAddress } from '@polkadot/util-crypto';

vi.mock('../src/auth/wallet_signature', () => ({
  verifyWalletSignature: vi.fn()
}));

import { subscribeChallengeRoute, subscribeConfirmRoute } from '../src/membership/subscribe';
import { verifyWalletSignature } from '../src/auth/wallet_signature';
import type { Env } from '../src/types';

const mockVerify = verifyWalletSignature as unknown as ReturnType<typeof vi.fn>;

const ownerBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 41));
const owner = encodeAddress(ownerBytes, 2027);

interface ChallengeRow {
  challenge_id: string;
  owner_account: string;
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
    return null;
  }
}

class ChallengeDb {
  readonly challenges = new Map<string, ChallengeRow>();
  prepare(sql: string): ChallengeStmt {
    return new ChallengeStmt(this, sql);
  }
}

function fakeEnv(input: {
  db: ChallengeDb;
  storageResponses?: Array<string | null>;
  stripeResponse?: unknown;
  stripeStatus?: number;
  onStripeBody?: (body: string) => void;
}): Env {
  const responses = [...(input.storageResponses ?? [])];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (request: RequestInfo | URL, init?: RequestInit) => {
      const url = request.toString();
      if (url.startsWith('https://api.stripe.com/')) {
        input.onStripeBody?.(init?.body?.toString() ?? '');
        return Response.json(input.stripeResponse ?? {}, { status: input.stripeStatus ?? 200 });
      }
      return Response.json({ jsonrpc: '2.0', id: 1, result: responses.shift() ?? null });
    })
  );
  return {
    DB: input.db,
    CHAIN_URL: 'https://chain.test',
    CHAIN_ID: 'worker-rpc.access',
    CHAIN_SECRET: 'test-access-secret',
    STRIPE_API_KEY: 'sk_test_secret',
    FREEDOM_PRICE_ID: 'price_freedom',
    VOTING_PRICE_ID: 'price_voting',
    CANDIDATE_PRICE_ID: 'price_candidate',
    CHECKOUT_SUCCESS_URL: 'https://example.com/membership?checkout=success',
    CHECKOUT_CANCEL_URL: 'https://example.com/membership?checkout=cancel'
  } as unknown as Env;
}

function req(path: string, body: unknown): Request {
  return new Request(`https://w${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body)
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
  mockVerify.mockReset();
});

describe('subscribe challenge (signed, level-bound)', () => {
  it('freedom challenge returns op_tag 0x1D + owner_pubkey_hex + level', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db });
    const res = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: 'freedom'
      }),
      env
    );
    const json = (await res.json()) as Record<string, unknown>;
    expect(json.op_tag).toBe(0x1d);
    expect(json.membership_level).toBe('freedom');
    expect(typeof json.owner_pubkey_hex).toBe('string');
    expect((json.owner_pubkey_hex as string).length).toBe(64);
    expect(typeof json.signing_payload_hex).toBe('string');
    expect(db.challenges.size).toBe(1);
  });

  it('candidate challenge issued only when chain identity is candidate', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db, storageResponses: [votingIdentityHex(), '0x01'] });
    const res = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: 'candidate'
      }),
      env
    );
    expect(((await res.json()) as { membership_level: string }).membership_level).toBe('candidate');
  });

  it('rejects candidate challenge when owner only has voting identity', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db, storageResponses: [votingIdentityHex(), null] });
    await expect(
      subscribeChallengeRoute(
        req('/v1/square/membership/subscribe/challenge', {
          owner_account: owner,
          membership_level: 'candidate'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'membership_identity_mismatch' });
  });

  it('rejects a downgrade: voting identity cannot subscribe freedom', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db, storageResponses: [votingIdentityHex(), null] });
    await expect(
      subscribeChallengeRoute(
        req('/v1/square/membership/subscribe/challenge', {
          owner_account: owner,
          membership_level: 'freedom'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'membership_identity_mismatch' });
  });

  it('issues a democracy (白金) challenge for a visitor identity', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db, storageResponses: [null, null] });
    const res = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: 'democracy'
      }),
      env
    );
    expect(((await res.json()) as { membership_level: string }).membership_level).toBe(
      'democracy'
    );
  });
});

describe('subscribe confirm (signed)', () => {
  async function issue(env: Env, level: string): Promise<string> {
    const res = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: level
      }),
      env
    );
    return ((await res.json()) as { challenge_id: string }).challenge_id;
  }

  it('valid signature (level matches) → creates Stripe checkout', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    let capturedBody = '';
    const env = fakeEnv({
      db,
      stripeResponse: { id: 'cs_freedom', url: 'https://checkout.stripe.com/c/pay/cs_freedom' },
      onStripeBody: (body) => {
        capturedBody = body;
      }
    });
    const challengeId = await issue(env, 'freedom');

    const res = await subscribeConfirmRoute(
      req('/v1/square/membership/subscribe', {
        owner_account: owner,
        membership_level: 'freedom',
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );
    const json = (await res.json()) as Record<string, unknown>;
    expect(json).toMatchObject({
      checkout_url: 'https://checkout.stripe.com/c/pay/cs_freedom',
      membership_level: 'freedom'
    });
    expect(new URLSearchParams(capturedBody).get('mode')).toBe('subscription');
  });

  it('level tampering (confirm level ≠ challenge level) → action_mismatch', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    const env = fakeEnv({ db });
    const challengeId = await issue(env, 'freedom');

    await expect(
      subscribeConfirmRoute(
        req('/v1/square/membership/subscribe', {
          owner_account: owner,
          membership_level: 'voting',
          challenge_id: challengeId,
          signature: '0xSIG'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'action_mismatch' });
  });

  it('invalid signature → invalid_signature, no Stripe call', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(false);
    const env = fakeEnv({ db });
    const challengeId = await issue(env, 'freedom');

    await expect(
      subscribeConfirmRoute(
        req('/v1/square/membership/subscribe', {
          owner_account: owner,
          membership_level: 'freedom',
          challenge_id: challengeId,
          signature: '0xBAD'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_signature' });
  });
});

function votingIdentityHex(): string {
  const cid = new TextEncoder().encode('CN001-CTZN-000000001-2026');
  const body = concat([
    compactU32(cid.length),
    cid,
    u32Le(20200101),
    u32Le(20991231),
    Uint8Array.of(0)
  ]);
  return `0x${hex(body)}`;
}

function u32Le(value: number): Uint8Array {
  const out = new Uint8Array(4);
  new DataView(out.buffer).setUint32(0, value, true);
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

function hex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}
