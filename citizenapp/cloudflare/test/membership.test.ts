import { describe, expect, it } from 'vitest';
import { encodeAddress } from '@polkadot/util-crypto';
import { platformSubscriptionConfirmRoute } from '../src/membership/citizen_coin';
import {
  batchMemberships,
  getMembership,
  membershipRoute,
  requireActiveMembership,
  subscriptionIsActive
} from '../src/membership/service';
import { membershipPlanList } from '../src/membership/plans';
import type { Env, MembershipRow, SessionState } from '../src/types';

// 会员支付已全部切公民币链上订阅（唯一支付轨）：App 侧热钱包 extrinsic 把订阅/取消上链，
// 本 BFF 只做上链后确认镜像。Stripe / USDC 预付 / 换档折算等旧轨全部下线，这里只覆盖公民币轨。

const ownerBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 11));
const owner = encodeAddress(ownerBytes, 2027);
const otherBytes = Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 71));
const otherOwner = encodeAddress(otherBytes, 2027);
const sessionToken = 'session_member';
const txHash = '0x' + 'a'.repeat(64);
const txHash2 = '0x' + 'b'.repeat(64);

describe('platformSubscriptionConfirmRoute（公民币轨上链后镜像）', () => {
  it('带合法 level 的确认写入 active 镜像', async () => {
    const { env, db } = fakeEnv();

    const response = await platformSubscriptionConfirmRoute(
      confirmRequest({ tx_hash: txHash, level: 'spark' }),
      env
    );
    const body = (await response.json()) as {
      ok: boolean;
      status: string;
      membership_level: string;
    };

    expect(body).toMatchObject({ ok: true, status: 'active', membership_level: 'spark' });
    const row = db.memberships.get(owner);
    expect(row).toMatchObject({
      owner_account: owner,
      membership_level: 'spark',
      subscription_status: 'active',
      entitlement_lapsed_at: null,
      last_tx_hash: txHash
    });
    // 计费周期窗口写入：终点晚于起点，供用量额度与徽章一个稳定窗口。
    expect((row?.current_period_end ?? 0) > (row?.current_period_start ?? 0)).toBe(true);
  });

  it('缺 level 的确认把订阅镜像翻 cancelled 并记权益失效时刻', async () => {
    const { env, db } = fakeEnv();
    db.memberships.set(owner, membershipRow({ subscription_status: 'active' }));

    const response = await platformSubscriptionConfirmRoute(
      confirmRequest({ tx_hash: txHash2 }),
      env
    );
    const body = (await response.json()) as { ok: boolean; status: string };

    expect(body).toEqual({ ok: true, status: 'cancelled' });
    const row = db.memberships.get(owner);
    expect(row?.subscription_status).toBe('cancelled');
    expect(row?.entitlement_lapsed_at).not.toBeNull();
    expect(row?.last_tx_hash).toBe(txHash2);
  });

  it('owner 由 session 派生，不采信 body 里的账户', async () => {
    const { env, db } = fakeEnv();

    await platformSubscriptionConfirmRoute(
      confirmRequest({ tx_hash: txHash, level: 'freedom', owner_account: otherOwner }),
      env
    );

    // 镜像落在 session owner，而非 body 伪造的 otherOwner。
    expect(db.memberships.has(owner)).toBe(true);
    expect(db.memberships.has(otherOwner)).toBe(false);
  });

  it('非法 tx_hash 抛 400 invalid_request', async () => {
    const { env, db } = fakeEnv();

    await expect(
      platformSubscriptionConfirmRoute(
        confirmRequest({ tx_hash: '0xnothex', level: 'freedom' }),
        env
      )
    ).rejects.toMatchObject({ status: 400, code: 'invalid_request' });
    // 大写 hex 同样不合法（要求 0x + 64 位小写 hex）。
    await expect(
      platformSubscriptionConfirmRoute(
        confirmRequest({ tx_hash: '0x' + 'A'.repeat(64), level: 'freedom' }),
        env
      )
    ).rejects.toMatchObject({ status: 400, code: 'invalid_request' });
    expect(db.memberships.size).toBe(0);
  });

  it('重复确认按 owner 主键 upsert，保持幂等只留一行', async () => {
    const { env, db } = fakeEnv();

    await platformSubscriptionConfirmRoute(
      confirmRequest({ tx_hash: txHash, level: 'freedom' }),
      env
    );
    await platformSubscriptionConfirmRoute(
      confirmRequest({ tx_hash: txHash2, level: 'democracy' }),
      env
    );

    expect(db.memberships.size).toBe(1);
    const row = db.memberships.get(owner);
    // 后一次确认覆盖档位与 tx_hash（同 owner 主键 upsert）。
    expect(row?.membership_level).toBe('democracy');
    expect(row?.last_tx_hash).toBe(txHash2);
    expect(row?.subscription_status).toBe('active');
  });

  it('取消后再订阅把 active 复位、entitlement_lapsed_at 清空', async () => {
    const { env, db } = fakeEnv();

    await platformSubscriptionConfirmRoute(
      confirmRequest({ tx_hash: txHash, level: 'freedom' }),
      env
    );
    await platformSubscriptionConfirmRoute(confirmRequest({ tx_hash: txHash2 }), env);
    expect(db.memberships.get(owner)?.subscription_status).toBe('cancelled');
    expect(db.memberships.get(owner)?.entitlement_lapsed_at).not.toBeNull();

    await platformSubscriptionConfirmRoute(
      confirmRequest({ tx_hash: txHash, level: 'spark' }),
      env
    );
    const row = db.memberships.get(owner);
    expect(row?.subscription_status).toBe('active');
    expect(row?.entitlement_lapsed_at).toBeNull();
    expect(row?.membership_level).toBe('spark');
  });
});

describe('getMembership / batchMemberships（读新列）', () => {
  it('getMembership 读回订阅镜像新列', async () => {
    const { env, db } = fakeEnv();
    db.memberships.set(owner, membershipRow({ membership_level: 'democracy' }));

    const row = await getMembership(env, owner);

    expect(row).not.toBeNull();
    expect(row?.membership_level).toBe('democracy');
    expect(row?.subscription_status).toBe('active');
    // 新列均可读：周期窗口、权益失效时刻、最近交易哈希。
    expect(row).toHaveProperty('current_period_start');
    expect(row).toHaveProperty('current_period_end');
    expect(row).toHaveProperty('entitlement_lapsed_at');
    expect(row).toHaveProperty('last_tx_hash');
  });

  it('getMembership 对无镜像账户返回 null', async () => {
    const { env } = fakeEnv();
    expect(await getMembership(env, owner)).toBeNull();
  });

  it('batchMemberships 一次 IN() 查询返回多作者镜像', async () => {
    const { env, db } = fakeEnv();
    db.memberships.set(owner, membershipRow({ membership_level: 'spark' }));
    db.memberships.set(otherOwner, membershipRow({ owner_account: otherOwner, membership_level: 'freedom' }));

    const map = await batchMemberships(env, [owner, otherOwner, owner]);

    expect(map.size).toBe(2);
    expect(map.get(owner)?.membership_level).toBe('spark');
    expect(map.get(otherOwner)?.membership_level).toBe('freedom');
  });

  it('batchMemberships 空入参返回空 Map，不发查询', async () => {
    const { env } = fakeEnv();
    const map = await batchMemberships(env, []);
    expect(map.size).toBe(0);
  });
});

describe('subscriptionIsActive（只看 subscription_status）', () => {
  it('status=active 判有效', () => {
    expect(subscriptionIsActive(membershipRow({ subscription_status: 'active' }))).toBe(true);
  });

  it('status 非 active 一律判无效（cancelled / past_due），不再看 expires_at', () => {
    // expires_at 仍在未来，但状态非 active 即视为无效——按月续扣发生在链上，镜像以状态为准。
    const future = Date.now() + 86_400_000;
    expect(
      subscriptionIsActive(
        membershipRow({ subscription_status: 'cancelled', expires_at: future })
      )
    ).toBe(false);
    expect(
      subscriptionIsActive(
        membershipRow({ subscription_status: 'past_due', expires_at: future })
      )
    ).toBe(false);
  });
});

describe('requireActiveMembership（门禁2）', () => {
  it('订阅有效则放行并返回镜像行', async () => {
    const { env, db } = fakeEnv();
    db.memberships.set(owner, membershipRow({ subscription_status: 'active' }));

    const row = await requireActiveMembership(env, owner);
    expect(row.owner_account).toBe(owner);
  });

  it('无镜像抛 402 membership_required', async () => {
    const { env } = fakeEnv();
    await expect(requireActiveMembership(env, owner)).rejects.toMatchObject({
      status: 402,
      code: 'membership_required'
    });
  });

  it('订阅非 active 抛 402 membership_inactive', async () => {
    const { env, db } = fakeEnv();
    db.memberships.set(owner, membershipRow({ subscription_status: 'cancelled' }));
    await expect(requireActiveMembership(env, owner)).rejects.toMatchObject({
      status: 402,
      code: 'membership_inactive'
    });
  });
});

describe('membershipRoute（返回 plans + membership + active）', () => {
  it('返回有效订阅镜像与全部三档套餐', async () => {
    const { env, db } = fakeEnv();
    db.memberships.set(owner, membershipRow({ membership_level: 'spark' }));

    const response = await membershipRoute(sessionRequest(), env);
    const body = (await response.json()) as {
      ok: boolean;
      active: boolean;
      subscription_active: boolean;
      membership: { membership_level: string } | null;
      plans: Array<{ membership_level: string }>;
    };

    expect(body.ok).toBe(true);
    expect(body.active).toBe(true);
    expect(body.subscription_active).toBe(true);
    expect(body.membership?.membership_level).toBe('spark');
    // 三档解耦：任意身份都拿到同一份三档套餐清单。
    expect(body.plans.map((plan) => plan.membership_level)).toEqual([
      'freedom',
      'democracy',
      'spark'
    ]);
  });

  it('已取消订阅报为 inactive', async () => {
    const { env, db } = fakeEnv();
    db.memberships.set(owner, membershipRow({ subscription_status: 'cancelled' }));

    const response = await membershipRoute(sessionRequest(), env);
    const body = (await response.json()) as { active: boolean; subscription_active: boolean };

    expect(body.active).toBe(false);
    expect(body.subscription_active).toBe(false);
  });

  it('无订阅账户仍返回三档套餐，membership 为 null', async () => {
    const { env } = fakeEnv();

    const response = await membershipRoute(sessionRequest(), env);
    const body = (await response.json()) as {
      active: boolean;
      membership: unknown;
      plans: unknown[];
    };

    expect(body.active).toBe(false);
    expect(body.membership).toBeNull();
    expect(body.plans).toHaveLength(3);
  });
});

describe('membershipPlanList（不含美元字段）', () => {
  it('三档套餐均无任何美元计价字段', () => {
    const plans = membershipPlanList();
    expect(plans.map((plan) => plan.membership_level)).toEqual(['freedom', 'democracy', 'spark']);
    for (const plan of plans) {
      // 计价与扣款是链上 square-post（PlatformPrice + billing keeper）的职责，套餐表只定档位与配额。
      expect(plan).not.toHaveProperty('price_currency');
      expect(plan).not.toHaveProperty('price_usd_cents');
      expect(plan).not.toHaveProperty('price_usd_monthly');
      // 保留的档位字段：展示名、聊天文件上限、动态与文章配额。
      expect(plan).toHaveProperty('display_name');
      expect(plan).toHaveProperty('chat_file_max_bytes');
      expect(plan).toHaveProperty('dynamic');
      expect(plan).toHaveProperty('article');
    }
  });
});

/// 构造带 session 的测试 env 与内存版 square_memberships。
function fakeEnv(): { env: Env; db: FakeDb } {
  const db = new FakeDb();
  const session: SessionState = {
    owner_account: owner,
    device_key_hash: 'a'.repeat(64),
    created_at: 0,
    expires_at: Date.now() + 60_000
  };
  const kv = new FakeKv(new Map([[`square_session:${sessionToken}`, session]]));
  const env = {
    DB: db as unknown as D1Database,
    SQUARE_CACHE: kv as unknown as KVNamespace
  } as unknown as Env;
  return { env, db };
}

/// GET /v1/square/membership 请求（带登录态）。
function sessionRequest(): Request {
  return new Request('https://w/v1/square/membership', {
    headers: { authorization: `Bearer ${sessionToken}` }
  });
}

/// POST /v1/square/membership/confirm 请求（带登录态与 JSON body）。
function confirmRequest(body: Record<string, unknown>): Request {
  return new Request('https://w/v1/square/membership/confirm', {
    method: 'POST',
    headers: {
      authorization: `Bearer ${sessionToken}`,
      'content-type': 'application/json'
    },
    body: JSON.stringify(body)
  });
}

/// 新订阅镜像行（公民币轨，无任何 stripe/prepaid 列）。
function membershipRow(overrides: Partial<MembershipRow> = {}): MembershipRow {
  const now = Date.now();
  const periodEnd = now + 30 * 86_400_000;
  return {
    owner_account: owner,
    membership_level: 'freedom',
    expires_at: periodEnd,
    updated_at: now,
    subscription_status: 'active',
    current_period_start: now,
    current_period_end: periodEnd,
    entitlement_lapsed_at: null,
    last_tx_hash: null,
    ...overrides
  };
}

class FakeKv {
  constructor(private readonly store: Map<string, unknown>) {}

  async get<T>(key: string): Promise<T | null> {
    return (this.store.get(key) as T) ?? null;
  }
}

/// 内存版 square_memberships：按 owner_account 主键存 MembershipRow，
/// 覆盖 citizen_coin 订阅 upsert / 取消 UPDATE 与 service 的 SELECT / IN() 查询。
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
    if (this.sql.includes('FROM square_memberships') && this.sql.includes('owner_account = ?')) {
      return (this.db.memberships.get(this.args[0] as string) ?? null) as T | null;
    }
    return null;
  }

  async all<T>(): Promise<{ results: T[]; success: boolean }> {
    if (this.sql.includes('FROM square_memberships') && this.sql.includes('owner_account IN')) {
      const wanted = new Set(this.args as string[]);
      const rows = [...this.db.memberships.values()].filter((row) =>
        wanted.has(row.owner_account)
      );
      return { results: rows as unknown as T[], success: true };
    }
    return { results: [], success: true };
  }

  async run(): Promise<{ success: boolean; meta: { changes: number } }> {
    // 订阅确认 upsert：binds = [owner, level, expires, updated, periodStart, periodEnd, txHash]。
    if (this.sql.includes('INSERT INTO square_memberships')) {
      const ownerAccount = this.args[0] as string;
      this.db.memberships.set(ownerAccount, {
        owner_account: ownerAccount,
        membership_level: this.args[1] as string,
        expires_at: this.args[2] as number,
        updated_at: this.args[3] as number,
        subscription_status: 'active',
        current_period_start: this.args[4] as number,
        current_period_end: this.args[5] as number,
        entitlement_lapsed_at: null,
        last_tx_hash: this.args[6] as string
      });
      return { success: true, meta: { changes: 1 } };
    }
    // 取消确认 UPDATE：binds = [nowForLapsed, txHash, nowForUpdated, owner]。
    if (this.sql.includes('UPDATE square_memberships') && this.sql.includes("'cancelled'")) {
      const nowForLapsed = this.args[0] as number;
      const txHashArg = this.args[1] as string;
      const nowForUpdated = this.args[2] as number;
      const ownerAccount = this.args[3] as string;
      const existing = this.db.memberships.get(ownerAccount);
      if (!existing) {
        return { success: true, meta: { changes: 0 } };
      }
      this.db.memberships.set(ownerAccount, {
        ...existing,
        subscription_status: 'cancelled',
        entitlement_lapsed_at: existing.entitlement_lapsed_at ?? nowForLapsed,
        last_tx_hash: txHashArg,
        updated_at: nowForUpdated
      });
      return { success: true, meta: { changes: 1 } };
    }
    return { success: true, meta: { changes: 1 } };
  }
}
