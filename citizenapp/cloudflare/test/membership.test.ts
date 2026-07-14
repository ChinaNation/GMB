import { afterEach, describe, expect, it, vi } from 'vitest';
import { scaleCompact as compactU32 } from '../src/shared/signing_message';
import { encodeAddress } from '@polkadot/util-crypto';
import { membershipRoute, subscriptionIsActive } from '../src/membership/service';
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
      frozen: boolean;
      inactive_code: string;
      eligible_levels: string[];
    };

    expect(body.active).toBe(false);
    expect(body.frozen).toBe(true);
    expect(body.inactive_code).toBe('membership_frozen_identity_mismatch');
    // 精确匹配（禁止降档）：voting 身份只能订 voting。
    expect(body.eligible_levels).toEqual(['voting']);
  });

  it('freezes a freedom membership held by a voting identity（身份升级未换档，双向冻结）', async () => {
    const env = fakeEnv({
      membership: membershipRow({ membership_level: 'freedom' }),
      storageResponses: [votingIdentityHex(), null]
    });

    const response = await membershipRoute(request('https://w/v1/square/membership'), env);
    const body = (await response.json()) as {
      active: boolean;
      frozen: boolean;
      inactive_code: string;
    };

    // 会员 < 身份 也算不匹配：freedom(访客档) 被 voting 身份持有 → 冻结。
    expect(body.active).toBe(false);
    expect(body.frozen).toBe(true);
    expect(body.inactive_code).toBe('membership_frozen_identity_mismatch');
  });

  it('does not freeze on transient chain read failure（回退上次已知身份，不误冻）', async () => {
    const env = fakeEnv({
      membership: membershipRow({ membership_level: 'candidate', identity_level: 'candidate' }),
      storageResponses: [],
      storageThrows: true
    });

    const response = await membershipRoute(request('https://w/v1/square/membership'), env);
    const body = (await response.json()) as { active: boolean; frozen: boolean };

    // 回链失败 → 回退持久 identity_level='candidate' → 与 candidate 会员匹配 → 不冻。
    expect(body.active).toBe(true);
    expect(body.frozen).toBe(false);
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

  it('upserts from a new-API event with item-level current_period_end（顶层为空也不报错）', async () => {
    const db = new FakeDb();
    const env = fakeEnv({ db, membership: null, storageResponses: [], stripeSecret });
    const body = stripeEvent({
      membership_level: 'freedom',
      status: 'active',
      periodEnd: 1_893_456_000,
      periodOnItem: true
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string };

    // 不再因顶层 current_period_end 为空抛 invalid_stripe_subscription。
    expect(json.action).toBe('subscription_upserted');
    expect(db.memberships.get(owner)).toMatchObject({
      membership_level: 'freedom',
      subscription_status: 'active',
      expires_at: 1_893_456_000 * 1000
    });
  });

  it('迟到的旧 subscription.updated 会回读 Stripe 当前对象，不回滚新档位', async () => {
    const db = new FakeDb();
    const currentEvent = JSON.parse(
      stripeEvent({
        membership_level: 'democracy',
        status: 'active',
        periodEnd: 1_903_456_000
      })
    ) as { data: { object: Record<string, unknown> } };
    const env = fakeEnv({
      db,
      membership: null,
      storageResponses: [],
      stripeSecret,
      stripeDevProxy: false,
      stripeSubscription: currentEvent.data.object
    });
    // 投递快照仍是旧 freedom；真实 Stripe 当前对象已是 democracy。
    const staleBody = stripeEvent({
      membership_level: 'freedom',
      status: 'active',
      periodEnd: 1_893_456_000
    });

    const response = await stripeWebhookRoute(await signedRequest(staleBody), env);
    expect(((await response.json()) as { membership_level: string }).membership_level).toBe(
      'democracy'
    );
    expect(db.memberships.get(owner)).toMatchObject({
      membership_level: 'democracy',
      expires_at: 1_903_456_000 * 1000
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

  it('grants USDC prepaid duration from a paid prepaid checkout.session.completed', async () => {
    const db = new FakeDb();
    const env = fakeEnv({ db, membership: null, storageResponses: [], stripeSecret });
    const body = JSON.stringify({
      id: 'evt_prepaid',
      type: 'checkout.session.completed',
      created: Math.floor(Date.now() / 1000),
      data: {
        object: {
          id: 'cs_prepaid_freedom_quarter',
          payment_status: 'paid',
          payment_method_types: ['crypto'],
          payment_intent: 'pi_test',
          metadata: {
            route: 'usdc_prepaid',
            owner_account: owner,
            membership_level: 'freedom',
            duration: 'quarter'
          }
        }
      }
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string; membership_level: string };

    expect(json).toMatchObject({ action: 'prepaid_granted', membership_level: 'freedom' });
    const row = db.memberships.get(owner);
    expect(row?.subscription_source).toBe('usdc_prepaid');
    expect(row?.membership_level).toBe('freedom');
    // 从 now 起叠 3 个月 → 到期在未来。
    expect((row?.expires_at ?? 0) > Date.now()).toBe(true);
    expect(row?.prepaid_payment_ref).toBe('pi_test');
  });

  it('重复投递同一 Stripe 事件不会重复延长 USDC 时长', async () => {
    const db = new FakeDb();
    const env = fakeEnv({ db, membership: null, storageResponses: [], stripeSecret });
    const body = JSON.stringify({
      id: 'evt_prepaid_replay',
      type: 'checkout.session.completed',
      created: Math.floor(Date.now() / 1000),
      data: {
        object: {
          id: 'cs_prepaid_replay',
          payment_status: 'paid',
          payment_method_types: ['crypto'],
          payment_intent: 'pi_replay',
          metadata: {
            route: 'usdc_prepaid',
            owner_account: owner,
            membership_level: 'freedom',
            duration: 'quarter'
          }
        }
      }
    });

    const first = await stripeWebhookRoute(await signedRequest(body), env);
    expect(((await first.json()) as { action: string }).action).toBe('prepaid_granted');
    const firstExpires = db.memberships.get(owner)?.expires_at;

    const repeated = await stripeWebhookRoute(await signedRequest(body), env);
    expect(((await repeated.json()) as { action: string }).action).toBe(
      'stripe_event_duplicate'
    );
    expect(db.memberships.get(owner)?.expires_at).toBe(firstExpires);
    expect(db.stripePayments.size).toBe(1);
  });

  it('USDC webhook 若不是 Stripe Crypto 支付则拒绝授权', async () => {
    const db = new FakeDb();
    const env = fakeEnv({ db, membership: null, storageResponses: [], stripeSecret });
    const body = JSON.stringify({
      id: 'evt_prepaid_card',
      type: 'checkout.session.completed',
      created: Math.floor(Date.now() / 1000),
      data: {
        object: {
          id: 'cs_prepaid_card',
          payment_status: 'paid',
          payment_method_types: ['card'],
          payment_intent: 'pi_card',
          metadata: {
            route: 'usdc_prepaid',
            owner_account: owner,
            membership_level: 'freedom',
            duration: 'quarter'
          }
        }
      }
    });

    await expect(stripeWebhookRoute(await signedRequest(body), env)).rejects.toMatchObject({
      code: 'stripe_crypto_required'
    });
    expect(db.memberships.get(owner)).toBeUndefined();
    expect(db.stripeWebhookEvents.has('evt_prepaid_card')).toBe(false);
  });

  it('applies USDC upgrade (level only, keeps expires) from a paid usdc_prepaid_upgrade checkout', async () => {
    const db = new FakeDb();
    const env = fakeEnv({
      db,
      membership: membershipRow({
        subscription_source: 'usdc_prepaid',
        membership_level: 'freedom',
        stripe_subscription_id: null,
        expires_at: Date.now() + 30 * 86_400_000
      }),
      storageResponses: [],
      stripeSecret
    });
    const body = JSON.stringify({
      id: 'evt_upgrade',
      type: 'checkout.session.completed',
      created: Math.floor(Date.now() / 1000),
      data: {
        object: {
          id: 'cs_prepaid_upgrade_democracy',
          payment_status: 'paid',
          payment_method_types: ['crypto'],
          payment_intent: 'pi_up',
          metadata: {
            route: 'usdc_prepaid_upgrade',
            owner_account: owner,
            membership_level: 'democracy'
          }
        }
      }
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string; membership_level: string };
    expect(json).toMatchObject({ action: 'prepaid_upgraded', membership_level: 'democracy' });
  });

  it('卡→USDC 切换：预付授权益前设卡到期取消，行转 usdc_prepaid 从卡到期日往后叠', async () => {
    const db = new FakeDb();
    const cardEnd = Date.now() + 20 * 86_400_000;
    const env = fakeEnv({
      db,
      membership: membershipRow({
        subscription_source: 'stripe',
        membership_level: 'freedom',
        stripe_subscription_id: 'sub_card',
        expires_at: cardEnd
      }),
      storageResponses: [],
      stripeSecret
    });
    const body = JSON.stringify({
      id: 'evt_switch',
      type: 'checkout.session.completed',
      created: Math.floor(Date.now() / 1000),
      data: {
        object: {
          id: 'cs_prepaid_freedom_quarter',
          payment_status: 'paid',
          payment_method_types: ['crypto'],
          payment_intent: 'pi_sw',
          metadata: {
            route: 'usdc_prepaid',
            owner_account: owner,
            membership_level: 'freedom',
            duration: 'quarter'
          }
        }
      }
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string };
    expect(json.action).toBe('prepaid_granted');
    const row = db.memberships.get(owner);
    expect(row?.subscription_source).toBe('usdc_prepaid');
    // 从卡到期日往后叠 3 个月 → 到期晚于卡到期。
    expect((row?.expires_at ?? 0) > cardEnd).toBe(true);
  });

  it('异档并存兜底：已有 freedom USDC，candidate 预付按价值折算而非直贴档差', async () => {
    const db = new FakeDb();
    const freedomEnd = Date.now() + 90 * 86_400_000;
    const env = fakeEnv({
      db,
      membership: membershipRow({
        subscription_source: 'usdc_prepaid',
        membership_level: 'freedom',
        stripe_subscription_id: null,
        current_period_start: Date.now(),
        expires_at: freedomEnd
      }),
      storageResponses: [],
      stripeSecret
    });
    const body = JSON.stringify({
      id: 'evt_fold',
      type: 'checkout.session.completed',
      created: Math.floor(Date.now() / 1000),
      data: {
        object: {
          id: 'cs_prepaid_candidate_quarter',
          payment_status: 'paid',
          payment_method_types: ['crypto'],
          payment_intent: 'pi_fold',
          metadata: {
            route: 'usdc_prepaid',
            owner_account: owner,
            membership_level: 'candidate',
            duration: 'quarter'
          }
        }
      }
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    expect(((await response.json()) as { action: string }).action).toBe('prepaid_granted');
    const row = db.memberships.get(owner);
    expect(row?.membership_level).toBe('candidate');
    // 兜底：now+3月 + 折算(90×299/9999≈3天)，绝非朴素的 freedom 到期(now+90d)再叠 3 月(≈180d)。
    const naiveEnd = freedomEnd + 3 * 30 * 86_400_000;
    expect((row?.expires_at ?? 0) < naiveEnd).toBe(true);
    expect((row?.expires_at ?? 0) < Date.now() + 120 * 86_400_000).toBe(true);
    // 至少含买到的 3 个日历月。
    expect((row?.expires_at ?? 0) > Date.now() + 85 * 86_400_000).toBe(true);
  });

  it('does not grant on a card checkout.session.completed (no prepaid route)', async () => {
    const db = new FakeDb();
    const env = fakeEnv({ db, membership: null, storageResponses: [], stripeSecret });
    const body = JSON.stringify({
      id: 'evt_card_checkout',
      type: 'checkout.session.completed',
      created: Math.floor(Date.now() / 1000),
      data: { object: { id: 'cs_card', payment_status: 'paid', metadata: {} } }
    });

    const response = await stripeWebhookRoute(await signedRequest(body), env);
    const json = (await response.json()) as { action: string };
    expect(json.action).toBe('checkout_session_observed');
    expect(db.memberships.get(owner)).toBeUndefined();
  });
});

describe('subscriptionIsActive (USDC 预付)', () => {
  it('usdc_prepaid 只看 expires_at：未到期=有效、过期=无效（不看 status）', () => {
    const future = membershipRow({
      subscription_source: 'usdc_prepaid',
      subscription_status: 'active',
      stripe_subscription_id: null,
      expires_at: Date.now() + 86_400_000
    });
    const past = membershipRow({
      subscription_source: 'usdc_prepaid',
      subscription_status: 'active',
      stripe_subscription_id: null,
      expires_at: Date.now() - 1000
    });
    expect(subscriptionIsActive(future)).toBe(true);
    expect(subscriptionIsActive(past)).toBe(false);
  });
});

function fakeEnv(input: {
  db?: FakeDb;
  membership?: MembershipRow | null;
  storageResponses: Array<string | null>;
  storageThrows?: boolean;
  stripeSecret?: string;
  stripeDevProxy?: boolean;
  stripeSubscription?: Record<string, unknown>;
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
    input.storageThrows
      ? vi.fn(async () => {
          throw new Error('rpc down');
        })
      : vi.fn(async (request: RequestInfo | URL) => {
          if (request.toString().startsWith('https://api.stripe.com/')) {
            return Response.json(input.stripeSubscription ?? {});
          }
          return Response.json({
            jsonrpc: '2.0',
            id: 1,
            result: responses.shift() ?? null
          });
        })
  );

  return {
    DB: db as unknown as D1Database,
    SQUARE_MEDIA: {} as R2Bucket,
    SQUARE_CACHE: kv as unknown as KVNamespace,
    CHAIN_URL: 'https://chain.test',
    CHAIN_ID: 'worker-rpc.access',
    CHAIN_SECRET: 'test-access-secret',
    STRIPE_HOOK_SECRET: input.stripeSecret,
    STRIPE_API_KEY: 'sk_test_secret',
    STRIPE_DEV_PROXY: input.stripeDevProxy === false ? '0' : '1',
    FREEDOM_PRICE_ID: 'price_freedom',
    DEMOCRACY_PRICE_ID: 'price_democracy',
    VOTING_PRICE_ID: 'price_voting',
    CANDIDATE_PRICE_ID: 'price_candidate'
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
  periodOnItem?: boolean;
}): string {
  const price = stripePriceForMembership(input.membership_level);
  const periodStart = 1_800_000_000;
  const item: Record<string, unknown> = {
    price: {
      id: price.id,
      currency: input.priceCurrency ?? price.currency,
      unit_amount: input.priceUnitAmount ?? price.unit_amount
    }
  };
  const object: Record<string, unknown> = {
    id: 'sub_test',
    customer: 'cus_test',
    status: input.status,
    cancel_at_period_end: false,
    metadata: { owner_account: owner, membership_level: input.membership_level }
  };
  // 新版 API：计费周期在 item 上、订阅顶层缺省；旧版：在顶层。
  if (input.periodOnItem) {
    item.current_period_start = periodStart;
    item.current_period_end = input.periodEnd;
  } else {
    object.current_period_start = periodStart;
    object.current_period_end = input.periodEnd;
  }
  object.items = { data: [item] };
  return JSON.stringify({
    id: 'evt_test',
    type: 'customer.subscription.updated',
    created: Math.floor(Date.now() / 1000),
    data: { object }
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
    frozen_at: null,
    collection_paused: 0,
    prepaid_payment_ref: null,
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
  stripeWebhookEvents = new Map<
    string,
    { processed_at: number | null; received_at: number }
  >();
  stripePayments = new Set<string>();

  prepare(sql: string): FakeStmt {
    return new FakeStmt(this, sql);
  }

  async batch(statements: FakeStmt[]): Promise<Array<{ success: boolean }>> {
    const membershipsBefore = new Map(
      [...this.memberships].map(([key, value]) => [key, { ...value }])
    );
    const eventsBefore = new Map(
      [...this.stripeWebhookEvents].map(([key, value]) => [key, { ...value }])
    );
    const paymentsBefore = new Set(this.stripePayments);
    try {
      return await Promise.all(statements.map((statement) => statement.run()));
    } catch (error) {
      this.memberships = membershipsBefore;
      this.stripeWebhookEvents = eventsBefore;
      this.stripePayments = paymentsBefore;
      throw error;
    }
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
    if (this.sql.includes('FROM square_stripe_webhook_events')) {
      return (this.db.stripeWebhookEvents.get(this.args[0] as string) ?? null) as T | null;
    }
    if (this.sql.includes('FROM square_stripe_payments')) {
      const paymentIntentId = this.args[0] as string;
      return (this.db.stripePayments.has(paymentIntentId)
        ? { stripe_payment_intent_id: paymentIntentId }
        : null) as T | null;
    }
    return null;
  }

  async all<T>(): Promise<{ results: T[]; success: boolean }> {
    // 本测试不构造已归档视频；实现 D1 all() 契约，确保续订回灌路径真实执行且无告警。
    return { results: [], success: true };
  }

  async run(): Promise<{ success: boolean; meta: { changes: number } }> {
    if (this.sql.includes('INSERT OR IGNORE INTO square_stripe_webhook_events')) {
      const eventId = this.args[0] as string;
      if (this.db.stripeWebhookEvents.has(eventId)) {
        return { success: true, meta: { changes: 0 } };
      }
      this.db.stripeWebhookEvents.set(eventId, {
        processed_at: null,
        received_at: this.args[4] as number
      });
      return { success: true, meta: { changes: 1 } };
    }
    if (
      this.sql.includes('UPDATE square_stripe_webhook_events') &&
      this.sql.includes('SET processed_at')
    ) {
      const eventId = this.args[1] as string;
      const event = this.db.stripeWebhookEvents.get(eventId);
      if (event) event.processed_at = this.args[0] as number;
      return { success: true, meta: { changes: event ? 1 : 0 } };
    }
    if (
      this.sql.includes('UPDATE square_stripe_webhook_events') &&
      this.sql.includes('SET received_at')
    ) {
      const eventId = this.args[1] as string;
      const event = this.db.stripeWebhookEvents.get(eventId);
      const canReclaim = event != null && event.processed_at == null && event.received_at < (this.args[2] as number);
      if (canReclaim) event.received_at = this.args[0] as number;
      return { success: true, meta: { changes: canReclaim ? 1 : 0 } };
    }
    if (this.sql.includes('DELETE FROM square_stripe_webhook_events')) {
      const eventId = this.args[0] as string;
      const event = this.db.stripeWebhookEvents.get(eventId);
      const deleted = event != null && event.processed_at == null;
      if (deleted) this.db.stripeWebhookEvents.delete(eventId);
      return { success: true, meta: { changes: deleted ? 1 : 0 } };
    }
    if (this.sql.includes('INSERT INTO square_stripe_payments')) {
      const paymentIntentId = this.args[0] as string;
      if (this.db.stripePayments.has(paymentIntentId)) {
        throw new Error('UNIQUE constraint failed: square_stripe_payments.stripe_payment_intent_id');
      }
      this.db.stripePayments.add(paymentIntentId);
      return { success: true, meta: { changes: 1 } };
    }
    if (this.sql.includes('INSERT INTO square_memberships')) {
      const ownerAccount = this.args[0] as string;
      if (this.sql.includes("'usdc_prepaid'")) {
        // USDC 预付 binds: [owner, level, expires, updated, start, end, identity, checked_at, ref]
        this.db.memberships.set(
          ownerAccount,
          membershipRow({
            owner_account: ownerAccount,
            membership_level: this.args[1] as string,
            expires_at: this.args[2] as number,
            subscription_source: 'usdc_prepaid',
            stripe_subscription_id: null,
            subscription_status: 'active',
            current_period_start: this.args[4] as number,
            current_period_end: this.args[5] as number,
            identity_level: this.args[6] as string,
            prepaid_payment_ref: this.args[8] as string | null
          })
        );
      } else {
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
          entitlement_lapsed_at: null,
          frozen_at: null,
          collection_paused: 0,
          prepaid_payment_ref: null
        });
      }
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
    return { success: true, meta: { changes: 1 } };
  }
}
