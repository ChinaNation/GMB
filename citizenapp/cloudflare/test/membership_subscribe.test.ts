import { afterEach, describe, expect, it, vi } from 'vitest';
import { encodeAddress } from '@polkadot/util-crypto';

vi.mock('../src/auth/wallet_signature', () => ({
  verifyWalletSignature: vi.fn()
}));

import { subscribeChallengeRoute, subscribeConfirmRoute } from '../src/membership/subscribe';
import { changeStripeSubscriptionTier } from '../src/membership/stripe_api';
import {
  prepaidChallengeRoute,
  prepaidChangeChallengeRoute,
  prepaidChangeConfirmRoute,
  prepaidConfirmRoute
} from '../src/membership/prepaid';
import {
  cancelMembershipChallengeRoute,
  cancelMembershipRoute
} from '../src/account/service';
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
    } else if (this.sql.includes('UPDATE square_login_challenges SET used_at = NULL')) {
      const row = this.db.challenges.get(this.binds[0] as string);
      if (row) row.used_at = null;
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
    if (this.sql.includes('FROM square_memberships')) {
      return (this.db.memberships.get(this.binds[0] as string) as T) ?? null;
    }
    return null;
  }
}

class ChallengeDb {
  readonly challenges = new Map<string, ChallengeRow>();
  readonly memberships = new Map<string, Record<string, unknown>>();
  prepare(sql: string): ChallengeStmt {
    return new ChallengeStmt(this, sql);
  }
}

/// 播种一条活跃会员行，供「换档 / 续订」分支测试（只含分派器读到的字段）。
function seedMembership(
  db: ChallengeDb,
  level: string,
  opts?: { cancelAtPeriodEnd?: boolean; status?: string; subId?: string }
): void {
  db.memberships.set(owner, {
    owner_account: owner,
    membership_level: level,
    subscription_status: opts?.status ?? 'active',
    stripe_subscription_id: opts?.subId ?? 'sub_1',
    cancel_at_period_end: opts?.cancelAtPeriodEnd ? 1 : 0,
    expires_at: Number.MAX_SAFE_INTEGER
  });
}

/// 播种一条有效 USDC 预付会员行（供换档测试）。
function seedPrepaid(db: ChallengeDb, level: string, opts?: { expiresInDays?: number }): void {
  const end = Date.now() + (opts?.expiresInDays ?? 30) * 86_400_000;
  db.memberships.set(owner, {
    owner_account: owner,
    membership_level: level,
    subscription_source: 'usdc_prepaid',
    subscription_status: 'active',
    stripe_subscription_id: null,
    cancel_at_period_end: 0,
    current_period_start: Date.now(),
    current_period_end: end,
    expires_at: end
  });
}

function fakeEnv(input: {
  db: ChallengeDb;
  storageResponses?: Array<string | null>;
  stripeResponse?: unknown;
  stripeStatus?: number;
  stripeDevProxy?: boolean;
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
    DEMOCRACY_PRICE_ID: 'price_democracy',
    SPARK_PRICE_ID: 'price_spark',
    CHECKOUT_SUCCESS_URL: 'https://example.com/membership?checkout=success',
    CHECKOUT_CANCEL_URL: 'https://example.com/membership?checkout=cancel',
    ...(input.stripeDevProxy ? { STRIPE_DEV_PROXY: '1' } : {})
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

  it('issues a spark challenge for any identity (ADR-036 解耦，无身份预检)', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db });
    const res = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: 'spark'
      }),
      env
    );
    expect(((await res.json()) as { membership_level: string }).membership_level).toBe('spark');
  });

  it('issues a democracy challenge for any identity', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db });
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
          membership_level: 'democracy',
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

  it('Stripe 建单失败 → 释放挑战，同一 challenge 重试成功', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    const failingEnv = fakeEnv({
      db,
      stripeResponse: { error: { message: 'boom' } },
      stripeStatus: 502
    });
    const challengeId = await issue(failingEnv, 'freedom');

    await expect(
      subscribeConfirmRoute(
        req('/v1/square/membership/subscribe', {
          owner_account: owner,
          membership_level: 'freedom',
          challenge_id: challengeId,
          signature: '0xSIG'
        }),
        failingEnv
      )
    ).rejects.toMatchObject({ code: 'stripe_checkout_failed' });
    // 挑战已释放：used_at 回到 null，未被烧掉。
    expect(db.challenges.get(challengeId)?.used_at).toBeNull();

    // Stripe 恢复正常 → 同一 challenge 重试直接成功，无需重新扫码签名。
    const okEnv = fakeEnv({
      db,
      stripeResponse: {
        id: 'cs_freedom',
        url: 'https://checkout.stripe.com/c/pay/cs_freedom'
      }
    });
    const res = await subscribeConfirmRoute(
      req('/v1/square/membership/subscribe', {
        owner_account: owner,
        membership_level: 'freedom',
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      okEnv
    );
    expect(((await res.json()) as Record<string, unknown>).checkout_url).toBe(
      'https://checkout.stripe.com/c/pay/cs_freedom'
    );
  });
});

describe('subscribe confirm — 换档 / 续订（一钱包一订阅）', () => {
  async function issueSubscribe(env: Env, level: string): Promise<string> {
    const res = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: level
      }),
      env
    );
    return ((await res.json()) as { challenge_id: string }).challenge_id;
  }

  async function confirm(env: Env, level: string, challengeId: string): Promise<Record<string, unknown>> {
    const res = await subscribeConfirmRoute(
      req('/v1/square/membership/subscribe', {
        owner_account: owner,
        membership_level: level,
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );
    return (await res.json()) as Record<string, unknown>;
  }

  it('已有 freedom → 订阅 democracy = 升档，且不新建 checkout（防重订）', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedMembership(db, 'freedom');
    const env = fakeEnv({ db, stripeDevProxy: true });
    const json = await confirm(env, 'democracy', await issueSubscribe(env, 'democracy'));
    expect(json.action).toBe('upgraded');
    expect(json.checkout_url).toBeUndefined();
  });

  it('已有 democracy → 订阅 freedom = 降档', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedMembership(db, 'democracy');
    const env = fakeEnv({ db, stripeDevProxy: true });
    const json = await confirm(env, 'freedom', await issueSubscribe(env, 'freedom'));
    expect(json.action).toBe('downgraded');
    expect(json.checkout_url).toBeUndefined();
  });

  it('已有 freedom（待取消）→ 订阅 freedom = 续订 resumed', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedMembership(db, 'freedom', { cancelAtPeriodEnd: true });
    const env = fakeEnv({ db, stripeDevProxy: true });
    const json = await confirm(env, 'freedom', await issueSubscribe(env, 'freedom'));
    expect(json.action).toBe('resumed');
  });

  it('已有 freedom（未取消）→ 订阅 freedom = 无操作 already_subscribed', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedMembership(db, 'freedom');
    const env = fakeEnv({ db, stripeDevProxy: true });
    const json = await confirm(env, 'freedom', await issueSubscribe(env, 'freedom'));
    expect(json.action).toBe('already_subscribed');
  });

  it('挑战响应带当前订阅态 current', async () => {
    const db = new ChallengeDb();
    seedMembership(db, 'freedom', { cancelAtPeriodEnd: true });
    const env = fakeEnv({ db, stripeDevProxy: true });
    const res = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: 'democracy'
      }),
      env
    );
    const json = (await res.json()) as { current: { membership_level: string; cancel_at_period_end: boolean } | null };
    expect(json.current).toEqual({ membership_level: 'freedom', cancel_at_period_end: true });
  });
});

describe('changeStripeSubscriptionTier — 升档无支付方式', () => {
  it('无可扣款方式升档被 Stripe 400 → membership_upgrade_needs_payment（非裸 502）', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db });
    // GET 订阅 → 带 item id；POST 更新 → 400「无支付方式」。
    vi.stubGlobal(
      'fetch',
      vi.fn(async (_url: RequestInfo | URL, init?: RequestInit) => {
        if (init?.method === 'POST') {
          return Response.json(
            {
              error: {
                message:
                  'This customer has no attached payment source or default payment method.'
              }
            },
            { status: 400 }
          );
        }
        return Response.json({ items: { data: [{ id: 'si_x' }] } });
      })
    );
    await expect(
      changeStripeSubscriptionTier(env, {
        subscriptionId: 'sub_x',
        newPriceId: 'price_x',
        isUpgrade: true
      })
    ).rejects.toMatchObject({ code: 'membership_upgrade_needs_payment' });
  });
});

describe('prepaid — USDC 预付购买', () => {
  it('challenge 返回 op_tag 0x1D + duration + months', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db, stripeDevProxy: true });
    const res = await prepaidChallengeRoute(
      req('/v1/square/membership/prepaid/challenge', {
        owner_account: owner,
        membership_level: 'freedom',
        duration: 'year'
      }),
      env
    );
    const json = (await res.json()) as Record<string, unknown>;
    expect(json.op_tag).toBe(0x1d);
    expect(json.duration).toBe('year');
    expect(json.months).toBe(12);
    expect(typeof json.challenge_id).toBe('string');
  });

  it('confirm 验签后建一次性 Checkout（dev 短路返回 checkout_url）', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    const env = fakeEnv({ db, stripeDevProxy: true });
    const challengeRes = await prepaidChallengeRoute(
      req('/v1/square/membership/prepaid/challenge', {
        owner_account: owner,
        membership_level: 'freedom',
        duration: 'quarter'
      }),
      env
    );
    const challengeId = ((await challengeRes.json()) as { challenge_id: string }).challenge_id;

    const res = await prepaidConfirmRoute(
      req('/v1/square/membership/prepaid', {
        owner_account: owner,
        membership_level: 'freedom',
        duration: 'quarter',
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );
    const json = (await res.json()) as Record<string, unknown>;
    expect(typeof json.checkout_url).toBe('string');
    expect(json.duration).toBe('quarter');
  });

  it('真实建单参数只允许 Stripe Crypto，不允许银行卡回落', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    let capturedBody = '';
    const env = fakeEnv({
      db,
      stripeResponse: { id: 'cs_crypto', url: 'https://checkout.stripe.com/c/pay/cs_crypto' },
      onStripeBody: (body) => {
        capturedBody = body;
      }
    });
    const challengeRes = await prepaidChallengeRoute(
      req('/v1/square/membership/prepaid/challenge', {
        owner_account: owner,
        membership_level: 'freedom',
        duration: 'quarter'
      }),
      env
    );
    const challengeId = ((await challengeRes.json()) as { challenge_id: string }).challenge_id;

    await prepaidConfirmRoute(
      req('/v1/square/membership/prepaid', {
        owner_account: owner,
        membership_level: 'freedom',
        duration: 'quarter',
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );

    const form = new URLSearchParams(capturedBody);
    expect(form.get('mode')).toBe('payment');
    expect(form.get('payment_method_types[0]')).toBe('crypto');
  });

  it('时长不合法 → invalid_prepaid_duration', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db, stripeDevProxy: true });
    await expect(
      prepaidChallengeRoute(
        req('/v1/square/membership/prepaid/challenge', {
          owner_account: owner,
          membership_level: 'freedom',
          duration: 'month'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'invalid_prepaid_duration' });
  });

  it('已有活跃 USDC 且选异档 → prepaid_tier_change_required（引导走换档）', async () => {
    const db = new ChallengeDb();
    seedPrepaid(db, 'freedom', { expiresInDays: 30 });
    const env = fakeEnv({ db, stripeDevProxy: true });
    await expect(
      prepaidChallengeRoute(
        req('/v1/square/membership/prepaid/challenge', {
          owner_account: owner,
          membership_level: 'democracy',
          duration: 'quarter'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'prepaid_tier_change_required' });
  });

  it('已有活跃 USDC 同档续费 → 放行（叠加时长）', async () => {
    const db = new ChallengeDb();
    seedPrepaid(db, 'freedom', { expiresInDays: 30 });
    const env = fakeEnv({ db, stripeDevProxy: true });
    const res = await prepaidChallengeRoute(
      req('/v1/square/membership/prepaid/challenge', {
        owner_account: owner,
        membership_level: 'freedom',
        duration: 'year'
      }),
      env
    );
    expect(((await res.json()) as { months: number }).months).toBe(12);
  });
});

describe('切换支付 USDC→卡', () => {
  it('有有效 USDC 预付时订卡：卡订阅带 trial_end=USDC 到期日', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedPrepaid(db, 'freedom', { expiresInDays: 30 });
    const usdcEnd = (db.memberships.get(owner) as { expires_at: number }).expires_at;
    let capturedBody = '';
    const env = fakeEnv({
      db,
      stripeResponse: { id: 'cs_switch', url: 'https://checkout.stripe.com/c/pay/cs_switch' },
      onStripeBody: (b) => {
        capturedBody = b;
      }
    });
    const challengeRes = await subscribeChallengeRoute(
      req('/v1/square/membership/subscribe/challenge', {
        owner_account: owner,
        membership_level: 'freedom'
      }),
      env
    );
    const challengeId = ((await challengeRes.json()) as { challenge_id: string }).challenge_id;
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
    expect(json.checkout_url).toBe('https://checkout.stripe.com/c/pay/cs_switch');
    // 卡订阅试用期到 USDC 到期日（秒），首次扣费顺延。
    expect(new URLSearchParams(capturedBody).get('subscription_data[trial_end]')).toBe(
      String(Math.floor(usdcEnd / 1000))
    );
    expect(
      new URLSearchParams(capturedBody).get(
        'subscription_data[metadata][payment_switch]'
      )
    ).toBe('usdc_to_stripe');
  });
});

describe('prepaid change — USDC 同路线换挡', () => {
  async function changeChallengeId(env: Env, target: string): Promise<string> {
    const res = await prepaidChangeChallengeRoute(
      req('/v1/square/membership/prepaid/change/challenge', {
        owner_account: owner,
        membership_level: target
      }),
      env
    );
    return ((await res.json()) as { challenge_id: string }).challenge_id;
  }

  it('降档 democracy→freedom：即时切档 + 折算到期变长', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedPrepaid(db, 'democracy', { expiresInDays: 30 });
    const env = fakeEnv({ db, stripeDevProxy: true });
    const challengeId = await changeChallengeId(env, 'freedom');
    const res = await prepaidChangeConfirmRoute(
      req('/v1/square/membership/prepaid/change', {
        owner_account: owner,
        membership_level: 'freedom',
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );
    const json = (await res.json()) as Record<string, unknown>;
    expect(json.action).toBe('downgraded');
    expect(json.membership_level).toBe('freedom');
    // 剩 30 天 democracy($9.99) 折成 freedom($2.99) ≈ 100 天 → 比原到期更久。
    expect((json.expires_at as number) > Date.now() + 30 * 86_400_000).toBe(true);
  });

  it('升档 freedom→democracy：建差价 Checkout（dev 短路）', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedPrepaid(db, 'freedom', { expiresInDays: 30 });
    const env = fakeEnv({ db, stripeDevProxy: true });
    const challengeId = await changeChallengeId(env, 'democracy');
    const res = await prepaidChangeConfirmRoute(
      req('/v1/square/membership/prepaid/change', {
        owner_account: owner,
        membership_level: 'democracy',
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );
    const json = (await res.json()) as Record<string, unknown>;
    expect(json.action).toBe('upgrade_pending');
    expect(typeof json.checkout_url).toBe('string');
    expect((json.amount_cents as number) > 0).toBe(true);
  });

  it('无有效 USDC 预付会员 → no_active_prepaid', async () => {
    const db = new ChallengeDb();
    const env = fakeEnv({ db, stripeDevProxy: true });
    await expect(
      prepaidChangeChallengeRoute(
        req('/v1/square/membership/prepaid/change/challenge', {
          owner_account: owner,
          membership_level: 'freedom'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'no_active_prepaid' });
  });
});

describe('cancel — 按支付方式识别（ADR-034 段4）', () => {
  async function cancelChallengeId(env: Env): Promise<string> {
    const res = await cancelMembershipChallengeRoute(
      req('/v1/square/membership/cancel/challenge', { owner_account: owner }),
      env
    );
    return ((await res.json()) as { challenge_id: string }).challenge_id;
  }

  it('USDC 预付取消 → cancel_kind=usdc_prepaid + 到期日（不动订阅）', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedPrepaid(db, 'freedom', { expiresInDays: 30 });
    const usdcEnd = (db.memberships.get(owner) as { expires_at: number }).expires_at;
    const env = fakeEnv({ db, stripeDevProxy: true });
    const challengeId = await cancelChallengeId(env);
    const res = await cancelMembershipRoute(
      req('/v1/square/membership/cancel', {
        owner_account: owner,
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );
    const json = (await res.json()) as Record<string, unknown>;
    expect(json.cancel_kind).toBe('usdc_prepaid');
    expect(json.expires_at).toBe(usdcEnd);
  });

  it('卡连续订阅取消 → cancel_kind=stripe（到期取消）', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    seedMembership(db, 'democracy', { subId: 'sub_card' });
    const env = fakeEnv({ db, stripeDevProxy: true });
    const challengeId = await cancelChallengeId(env);
    const res = await cancelMembershipRoute(
      req('/v1/square/membership/cancel', {
        owner_account: owner,
        challenge_id: challengeId,
        signature: '0xSIG'
      }),
      env
    );
    expect(((await res.json()) as { cancel_kind: string }).cancel_kind).toBe('stripe');
  });

  it('无活跃订阅取消 → no_active_subscription', async () => {
    const db = new ChallengeDb();
    mockVerify.mockResolvedValue(true);
    const env = fakeEnv({ db, stripeDevProxy: true });
    const challengeId = await cancelChallengeId(env);
    await expect(
      cancelMembershipRoute(
        req('/v1/square/membership/cancel', {
          owner_account: owner,
          challenge_id: challengeId,
          signature: '0xSIG'
        }),
        env
      )
    ).rejects.toMatchObject({ code: 'no_active_subscription' });
  });
});
