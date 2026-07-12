import { afterEach, describe, expect, it, vi } from 'vitest';
import { scaleCompact as compactU32 } from '../src/shared/signing_message';
import { encodeAddress } from '@polkadot/util-crypto';
import { membershipRoute } from '../src/membership/service';
import { stripeWebhookRoute, verifyStripeSignature } from '../src/membership/webhook';
import { routeRequest } from '../src/routes';
import type { Env, MembershipRow, SessionState } from '../src/types';

const ownerBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 11));
const owner = encodeAddress(ownerBytes, 2027);
const sessionToken = 'session_member';
const stripeSecret = 'whsec_test_secret';

describe('membership route', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('returns effective candidate membership when chain identity is candidate', async () => {
    const env = fakeEnv({
      membership: membershipRow({ membership_level: 'candidate' }),
      storageResponses: [votingIdentityHex(), '0x01']
    });

    const response = await membershipRoute(request('https://w/v1/square/membership'), env);
    const body = (await response.json()) as {
      active: boolean;
      eligible_levels: string[];
      identity: { identity_level: string; has_candidate_identity: boolean };
    };

    expect(body.active).toBe(true);
    expect(body.identity).toMatchObject({
      identity_level: 'candidate',
      has_candidate_identity: true
    });
    // 精确匹配（禁止降档）：candidate 身份只能订 candidate。
    expect(body.eligible_levels).toEqual(['candidate']);
  });

  it('offers only freedom and democracy to a visitor identity', async () => {
    const env = fakeEnv({
      membership: membershipRow({ membership_level: 'freedom' }),
      storageResponses: [null, null]
    });

    const response = await membershipRoute(request('https://w/v1/square/membership'), env);
    const body = (await response.json()) as {
      eligible_levels: string[];
      identity: { identity_level: string };
    };

    expect(body.identity.identity_level).toBe('visitor');
    // 访客身份可订自由与民主两档会员，会员值不再复用身份值 visitor。
    expect(body.eligible_levels).toEqual(['freedom', 'democracy']);
  });

  it('does not activate candidate membership for voting-only identity', async () => {
    const env = fakeEnv({
      membership: membershipRow({ membership_level: 'candidate' }),
      storageResponses: [votingIdentityHex(), null]
    });

    const response = await membershipRoute(request('https://w/v1/square/membership'), env);
    const body = (await response.json()) as {
      active: boolean;
      inactive_code: string;
      eligible_levels: string[];
    };

    expect(body.active).toBe(false);
    expect(body.inactive_code).toBe('membership_identity_required');
    // 精确匹配（禁止降档）：voting 身份只能订 voting。
    expect(body.eligible_levels).toEqual(['voting']);
  });
});

describe('stripe membership webhook', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('upserts a voting membership from a signed subscription event', async () => {
    const db = new FakeDb();
    const env = fakeEnv({
      db,
      membership: null,
      storageResponses: [votingIdentityHex(), null],
      stripeSecret
    });
    const body = stripeEvent({
      membership_level: 'voting',
      status: 'active',
      periodEnd: 1_893_456_000
    });

    // 通过总路由验收 webhook 的唯一正式路径，防止模块重命名后出现重复路径。
    const response = await routeRequest(await signedRequest(body), env);
    const json = (await response.json()) as { action: string; membership_level: string };

    expect(json).toMatchObject({
      action: 'subscription_upserted',
      membership_level: 'voting'
    });
    expect(db.memberships.get(owner)).toMatchObject({
      membership_level: 'voting',
      subscription_status: 'active',
      identity_level: 'voting',
      stripe_subscription_id: 'sub_test'
    });
  });

  it('upserts freedom membership without reading chain identity', async () => {
    const db = new FakeDb();
    const env = fakeEnv({
      db,
      membership: null,
      storageResponses: [],
      stripeSecret
    });
    const body = stripeEvent({
      membership_level: 'freedom',
      status: 'active',
      periodEnd: 1_893_456_000
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string; membership_level: string };

    expect(json).toMatchObject({
      action: 'subscription_upserted',
      membership_level: 'freedom'
    });
    expect(fetch).not.toHaveBeenCalled();
    expect(db.memberships.get(owner)).toMatchObject({
      membership_level: 'freedom',
      subscription_status: 'active',
      identity_level: 'visitor'
    });
  });

  it('upserts democracy membership with its own USD price', async () => {
    const db = new FakeDb();
    const env = fakeEnv({
      db,
      membership: null,
      storageResponses: [],
      stripeSecret
    });
    const body = stripeEvent({
      membership_level: 'democracy',
      status: 'active',
      periodEnd: 1_893_456_000
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string; membership_level: string };

    expect(json).toMatchObject({
      action: 'subscription_upserted',
      membership_level: 'democracy'
    });
    expect(fetch).not.toHaveBeenCalled();
    expect(db.memberships.get(owner)).toMatchObject({
      membership_level: 'democracy',
      subscription_status: 'active',
      identity_level: 'visitor',
      stripe_price_id: 'price_democracy'
    });
  });

  it('records identity_required instead of activating an ineligible candidate subscription', async () => {
    const db = new FakeDb();
    const env = fakeEnv({
      db,
      membership: null,
      storageResponses: [votingIdentityHex(), null],
      stripeSecret
    });
    const body = stripeEvent({
      membership_level: 'candidate',
      status: 'active',
      periodEnd: 1_893_456_000
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string };

    expect(json.action).toBe('identity_rejected');
    expect(db.memberships.get(owner)).toMatchObject({
      membership_level: 'candidate',
      subscription_status: 'identity_required',
      identity_level: 'voting'
    });
  });

  it('rejects a subscription whose Stripe Price is not USD', async () => {
    const db = new FakeDb();
    const env = fakeEnv({
      db,
      membership: null,
      storageResponses: [],
      stripeSecret
    });
    const body = stripeEvent({
      membership_level: 'freedom',
      status: 'active',
      periodEnd: 1_893_456_000,
      priceCurrency: 'hkd',
      priceUnitAmount: 2400
    });

    await expect(stripeWebhookRoute(await signedRequest(body), env)).rejects.toMatchObject({
      code: 'stripe_price_currency_mismatch'
    });
    expect(db.memberships.get(owner)).toBeUndefined();
  });

  it('rejects a subscription whose Stripe Price amount does not match the member level', async () => {
    const db = new FakeDb();
    const env = fakeEnv({
      db,
      membership: null,
      storageResponses: [],
      stripeSecret
    });
    const body = stripeEvent({
      membership_level: 'voting',
      status: 'active',
      periodEnd: 1_893_456_000,
      priceUnitAmount: 299
    });

    await expect(stripeWebhookRoute(await signedRequest(body), env)).rejects.toMatchObject({
      code: 'stripe_price_amount_mismatch'
    });
    expect(db.memberships.get(owner)).toBeUndefined();
  });

  it('rejects a webhook with an invalid signature', async () => {
    await expect(
      verifyStripeSignature('{"id":"evt"}', 't=123,v1=bad', stripeSecret, 123, 300)
    ).rejects.toMatchObject({ code: 'stripe_signature_mismatch' });
  });
});

function fakeEnv(input: {
  db?: FakeDb;
  membership?: MembershipRow | null;
  storageResponses: Array<string | null>;
  stripeSecret?: string;
}): Env {
  const db = input.db ?? new FakeDb();
  if (input.membership !== undefined && input.membership !== null) {
    db.memberships.set(input.membership.owner_account, input.membership);
  }
  const session: SessionState = {
    owner_account: owner,
    device_key_hash: 'a'.repeat(64),
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  const kv = new FakeKv(new Map([[`square_session:${sessionToken}`, session]]));
  const responses = [...input.storageResponses];
  vi.stubGlobal(
    'fetch',
    vi.fn(async () =>
      Response.json({
        jsonrpc: '2.0',
        id: 1,
        result: responses.shift() ?? null
      })
    )
  );

  return {
    DB: db as unknown as D1Database,
    SQUARE_MEDIA: {} as R2Bucket,
    SQUARE_CACHE: kv as unknown as KVNamespace,
    CHAIN_URL: 'https://chain.test',
    CHAIN_ID: 'worker-rpc.access',
    CHAIN_SECRET: 'test-access-secret',
    STRIPE_HOOK_SECRET: input.stripeSecret
  } as unknown as Env;
}

function request(url: string): Request {
  return new Request(url, {
    headers: {
      authorization: `Bearer ${sessionToken}`
    }
  });
}

async function signedRequest(body: string): Promise<Request> {
  const timestamp = Math.floor(Date.now() / 1000);
  const signature = await hmacSha256Hex(stripeSecret, `${timestamp}.${body}`);
  return new Request('https://w/v1/square/membership/webhook', {
    method: 'POST',
    headers: {
      'stripe-signature': `t=${timestamp},v1=${signature}`,
      'content-length': String(new TextEncoder().encode(body).byteLength)
    },
    body
  });
}

function stripeEvent(input: {
  membership_level: string;
  status: string;
  periodEnd: number;
  priceCurrency?: string;
  priceUnitAmount?: number;
}): string {
  const price = stripePriceForMembership(input.membership_level);
  return JSON.stringify({
    id: 'evt_test',
    type: 'customer.subscription.updated',
    data: {
      object: {
        id: 'sub_test',
        customer: 'cus_test',
        status: input.status,
        current_period_start: 1_800_000_000,
        current_period_end: input.periodEnd,
        cancel_at_period_end: false,
        metadata: {
          owner_account: owner,
          membership_level: input.membership_level
        },
        items: {
          data: [
            {
              price: {
                id: price.id,
                currency: input.priceCurrency ?? price.currency,
                unit_amount: input.priceUnitAmount ?? price.unit_amount
              }
            }
          ]
        }
      }
    }
  });
}

function stripePriceForMembership(level: string): {
  id: string;
  currency: 'usd';
  unit_amount: number;
} {
  if (level === 'candidate') {
    return { id: 'price_candidate', currency: 'usd', unit_amount: 9999 };
  }
  if (level === 'voting') {
    return { id: 'price_voting', currency: 'usd', unit_amount: 999 };
  }
  if (level === 'democracy') {
    return { id: 'price_democracy', currency: 'usd', unit_amount: 999 };
  }
  return { id: 'price_freedom', currency: 'usd', unit_amount: 299 };
}

function membershipRow(overrides: Partial<MembershipRow> = {}): MembershipRow {
  const expiresAt = Date.now() + 86_400_000;
  return {
    owner_account: owner,
    membership_level: 'freedom',
    expires_at: expiresAt,
    updated_at: Date.now(),
    subscription_source: 'stripe',
    stripe_customer_id: 'cus_test',
    stripe_subscription_id: 'sub_test',
    stripe_price_id: 'price_test',
    subscription_status: 'active',
    current_period_start: Date.now() - 1000,
    current_period_end: expiresAt,
    cancel_at_period_end: 0,
    identity_level: 'visitor',
    identity_checked_at: Date.now(),
    entitlement_lapsed_at: null,
    ...overrides
  };
}

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

async function hmacSha256Hex(secret: string, payload: string): Promise<string> {
  const encoder = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(payload));
  return [...new Uint8Array(signature)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

class FakeKv {
  constructor(private readonly store: Map<string, unknown>) {}

  async get<T>(key: string): Promise<T | null> {
    return (this.store.get(key) as T) ?? null;
  }
}

class FakeDb {
  memberships = new Map<string, MembershipRow>();

  prepare(sql: string): FakeStmt {
    return new FakeStmt(this, sql);
  }
}

class FakeStmt {
  private args: unknown[] = [];

  constructor(
    private readonly db: FakeDb,
    private readonly sql: string
  ) {}

  bind(...args: unknown[]): FakeStmt {
    this.args = args;
    return this;
  }

  async first<T>(): Promise<T | null> {
    if (this.sql.includes('FROM square_memberships')) {
      return (this.db.memberships.get(this.args[0] as string) ?? null) as T | null;
    }
    return null;
  }

  async all<T>(): Promise<{ results: T[]; success: boolean }> {
    // 本测试不构造已归档视频；实现 D1 all() 契约，确保续订回灌路径真实执行且无告警。
    return { results: [], success: true };
  }

  async run(): Promise<{ success: boolean }> {
    if (this.sql.includes('INSERT INTO square_memberships')) {
      const ownerAccount = this.args[0] as string;
      this.db.memberships.set(ownerAccount, {
        owner_account: ownerAccount,
        membership_level: this.args[1] as string,
        expires_at: this.args[2] as number,
        updated_at: this.args[3] as number,
        subscription_source: 'stripe',
        stripe_customer_id: this.args[4] as string | null,
        stripe_subscription_id: this.args[5] as string,
        stripe_price_id: this.args[6] as string | null,
        subscription_status: this.args[7] as string,
        current_period_start: this.args[8] as number | null,
        current_period_end: this.args[9] as number,
        cancel_at_period_end: this.args[10] as number,
        identity_level: this.args[11] as string,
        identity_checked_at: this.args[12] as number,
        entitlement_lapsed_at: null
      });
    }
    if (this.sql.includes('UPDATE square_memberships') && this.sql.includes('stripe_subscription_id')) {
      const status = this.args[0] as string;
      const subscriptionId = this.args[4] as string;
      for (const [ownerAccount, row] of this.db.memberships) {
        if (row.stripe_subscription_id === subscriptionId) {
          this.db.memberships.set(ownerAccount, {
            ...row,
            subscription_status: status,
            expires_at: this.args[1] as number,
            updated_at: this.args[2] as number,
            entitlement_lapsed_at: row.entitlement_lapsed_at ?? (this.args[3] as number)
          });
        }
      }
    }
    return { success: true };
  }
}
