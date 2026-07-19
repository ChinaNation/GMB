import { describe, expect, it } from 'vitest';
import { reconcileMemberships, type ReconcileDeps } from '../src/membership/reconcile';
import type { ChainSubscriptionState } from '../src/chain/subscription';
import type { Env } from '../src/types';

// 对账器把 D1 会员镜像对齐链上订阅真态。链读用注入的假实现替换（不打真链）；
// isChainRpcConfigured 只看 env 三件套，配齐即可通过软跳过闸门。

const NOW = 9_000_000;

interface FakeRow {
  owner_account: string;
  membership_level: string;
  subscription_status: string;
  current_period_start: number | null;
  current_period_end: number | null;
  expires_at: number;
  entitlement_lapsed_at: number | null;
  updated_at: number;
}

/// 内存版 square_memberships：覆盖对账器的 SELECT(order/limit) 与两种 UPDATE。
class FakeDb {
  rows = new Map<string, FakeRow>();

  seed(row: Partial<FakeRow> & { owner_account: string }): void {
    this.rows.set(row.owner_account, {
      membership_level: 'freedom',
      subscription_status: 'active',
      current_period_start: 0,
      current_period_end: 0,
      expires_at: 0,
      entitlement_lapsed_at: null,
      updated_at: 0,
      ...row
    });
  }

  prepare(sql: string): FakeStmt {
    return new FakeStmt(this, sql);
  }
}

class FakeStmt {
  private args: unknown[] = [];
  constructor(private readonly db: FakeDb, private readonly sql: string) {}

  bind(...args: unknown[]): FakeStmt {
    this.args = args;
    return this;
  }

  async all<T>(): Promise<{ results: T[]; success: boolean }> {
    if (
      this.sql.includes('SELECT owner_account FROM square_memberships') &&
      this.sql.includes('ORDER BY updated_at ASC LIMIT')
    ) {
      const limit = this.args[0] as number;
      const owners = [...this.db.rows.values()]
        .sort((a, b) => a.updated_at - b.updated_at)
        .slice(0, limit)
        .map((row) => ({ owner_account: row.owner_account }));
      return { results: owners as unknown as T[], success: true };
    }
    return { results: [], success: true };
  }

  async run(): Promise<{ success: boolean; meta: { changes: number } }> {
    // Active 刷新：binds = [level, periodStart, periodEnd, expires, now, owner]。
    if (this.sql.includes('UPDATE square_memberships') && this.sql.includes("subscription_status = 'active'")) {
      const owner = this.args[5] as string;
      const existing = this.db.rows.get(owner);
      if (existing) {
        this.db.rows.set(owner, {
          ...existing,
          membership_level: this.args[0] as string,
          subscription_status: 'active',
          current_period_start: this.args[1] as number,
          current_period_end: this.args[2] as number,
          expires_at: this.args[3] as number,
          entitlement_lapsed_at: null,
          updated_at: this.args[4] as number
        });
      }
      return { success: true, meta: { changes: existing ? 1 : 0 } };
    }
    // 收紧（terminated/cancelled）：binds = [status, nowForLapsed, nowForUpdated, owner]。
    if (this.sql.includes('UPDATE square_memberships') && this.sql.includes('COALESCE(entitlement_lapsed_at')) {
      const owner = this.args[3] as string;
      const existing = this.db.rows.get(owner);
      if (existing) {
        this.db.rows.set(owner, {
          ...existing,
          subscription_status: this.args[0] as string,
          entitlement_lapsed_at: existing.entitlement_lapsed_at ?? (this.args[1] as number),
          updated_at: this.args[2] as number
        });
      }
      return { success: true, meta: { changes: existing ? 1 : 0 } };
    }
    return { success: true, meta: { changes: 1 } };
  }
}

/// 配齐链 RPC env（isChainRpcConfigured 通过）+ 对账开关。
function fakeEnv(db: FakeDb, overrides: Partial<Env> = {}): Env {
  return {
    DB: db as unknown as D1Database,
    CHAIN_URL: 'https://node.internal/rpc',
    CHAIN_ID: 'access-id',
    CHAIN_SECRET: 'access-secret',
    MEMBERSHIP_RECONCILE_ENABLED: '1',
    MEMBERSHIP_RECONCILE_BATCH: '50',
    ...overrides
  } as unknown as Env;
}

/// 注入假链读：subscriber→态映射；throwOwners 里的账户抛错模拟链读失败。
function deps(
  map: Record<string, ChainSubscriptionState | null>,
  throwOwners: Set<string> = new Set()
): ReconcileDeps {
  return {
    now: () => NOW,
    readSubscription: async (_env, subscriber, _issuer) => {
      if (throwOwners.has(subscriber)) throw new Error('chain read failed');
      return map[subscriber] ?? null;
    }
  };
}

/// 内存版 KV：仅覆盖 get，供开关测试。
class FakeKv {
  constructor(private readonly store: Map<string, string>) {}
  async get(key: string): Promise<string | null> {
    return this.store.get(key) ?? null;
  }
}

function active(level: 'freedom' | 'democracy' | 'spark', lastChargedAt: number): ChainSubscriptionState {
  return {
    plan: { kind: 'platform', membershipLevel: level },
    pendingPlan: null,
    startedAt: lastChargedAt,
    lastChargedAt,
    lastChargedPriceFen: 199_900n,
    paidUntil: lastChargedAt + 1_000,
    status: 'active'
  };
}

describe('reconcileMemberships（状态对账，fail-closed）', () => {
  it('链上 Active → 镜像刷新为 active、档位取链、周期锚 last_charged、清权益失效时刻', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'cancelled', entitlement_lapsed_at: 123, updated_at: 100 });
    const result = await reconcileMemberships(fakeEnv(db), deps({ A: active('democracy', 1000) }));

    expect(result).toEqual({ scanned: 1, updated: 1, failed: 0 });
    const row = db.rows.get('A')!;
    expect(row.subscription_status).toBe('active');
    expect(row.membership_level).toBe('democracy');
    expect(row.entitlement_lapsed_at).toBeNull();
    expect(row.current_period_start).toBe(1000);
    expect((row.current_period_end ?? 0) > (row.current_period_start ?? 0)).toBe(true);
    expect(row.updated_at).toBe(NOW);
  });

  it('链上查无订阅 → 镜像收紧 cancelled 并记权益失效时刻', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'active', entitlement_lapsed_at: null, updated_at: 100 });
    await reconcileMemberships(fakeEnv(db), deps({ A: null }));

    const row = db.rows.get('A')!;
    expect(row.subscription_status).toBe('cancelled');
    expect(row.entitlement_lapsed_at).toBe(NOW);
  });

  it('链上 Terminated → 镜像收紧 terminated 并记权益失效时刻', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'active', updated_at: 100 });
    await reconcileMemberships(
      fakeEnv(db),
      deps({ A: { ...active('freedom', 1000), status: 'terminated' } })
    );

    const row = db.rows.get('A')!;
    expect(row.subscription_status).toBe('terminated');
    expect(row.entitlement_lapsed_at).toBe(NOW);
  });

  it('开关关闭时直接返回、零扫描', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'active', updated_at: 100 });
    const result = await reconcileMemberships(
      fakeEnv(db, { MEMBERSHIP_RECONCILE_ENABLED: '0' }),
      deps({ A: null })
    );
    expect(result).toEqual({ scanned: 0, updated: 0, failed: 0 });
    expect(db.rows.get('A')!.subscription_status).toBe('active');
  });

  it('链 RPC 未配置时软跳过、零扫描', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'active', updated_at: 100 });
    const result = await reconcileMemberships(
      fakeEnv(db, { CHAIN_URL: undefined }),
      deps({ A: null })
    );
    expect(result).toEqual({ scanned: 0, updated: 0, failed: 0 });
    expect(db.rows.get('A')!.subscription_status).toBe('active');
  });

  it('KV 开关 = 0 覆盖 wrangler var = 1 → 停', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'active', updated_at: 100 });
    const env = fakeEnv(db, {
      MEMBERSHIP_RECONCILE_ENABLED: '1',
      SQUARE_CACHE: new FakeKv(
        new Map([['flag:membership_reconcile', '0']])
      ) as unknown as KVNamespace
    });
    const result = await reconcileMemberships(env, deps({ A: null }));
    expect(result).toEqual({ scanned: 0, updated: 0, failed: 0 });
    expect(db.rows.get('A')!.subscription_status).toBe('active');
  });

  it('KV 开关 = 1 覆盖 wrangler var = 0 → 跑', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'active', updated_at: 100 });
    const env = fakeEnv(db, {
      MEMBERSHIP_RECONCILE_ENABLED: '0',
      SQUARE_CACHE: new FakeKv(
        new Map([['flag:membership_reconcile', '1']])
      ) as unknown as KVNamespace
    });
    const result = await reconcileMemberships(env, deps({ A: null }));
    expect(result.scanned).toBe(1);
    expect(db.rows.get('A')!.subscription_status).toBe('cancelled');
  });

  it('限流分批：按 updated_at 最旧优先，只取 batch 条', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'old1', updated_at: 100, subscription_status: 'active' });
    db.seed({ owner_account: 'old2', updated_at: 200, subscription_status: 'active' });
    db.seed({ owner_account: 'newer', updated_at: 300, subscription_status: 'active' });

    const result = await reconcileMemberships(
      fakeEnv(db, { MEMBERSHIP_RECONCILE_BATCH: '2' }),
      deps({ old1: null, old2: null, newer: null })
    );

    expect(result.scanned).toBe(2);
    // 最旧两条被处理（updated_at → NOW、状态收紧）；最新一条本轮未动。
    expect(db.rows.get('old1')!.updated_at).toBe(NOW);
    expect(db.rows.get('old2')!.updated_at).toBe(NOW);
    expect(db.rows.get('newer')!.updated_at).toBe(300);
    expect(db.rows.get('newer')!.subscription_status).toBe('active');
  });

  it('单条链读失败不阻断整批：失败行不动，其它行照常对账', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'X', updated_at: 100, subscription_status: 'active' });
    db.seed({ owner_account: 'Y', updated_at: 200, subscription_status: 'cancelled' });

    const result = await reconcileMemberships(
      fakeEnv(db),
      deps({ Y: active('spark', 1000) }, new Set(['X']))
    );

    expect(result).toEqual({ scanned: 2, updated: 1, failed: 1 });
    // X 链读失败：不动 updated_at（下轮重试）。
    expect(db.rows.get('X')!.updated_at).toBe(100);
    // Y 正常对账为 active。
    expect(db.rows.get('Y')!.subscription_status).toBe('active');
    expect(db.rows.get('Y')!.membership_level).toBe('spark');
  });

  it('幂等：同态跑两轮结果一致', async () => {
    const db = new FakeDb();
    db.seed({ owner_account: 'A', subscription_status: 'cancelled', updated_at: 100 });
    const d = deps({ A: active('freedom', 1000) });
    await reconcileMemberships(fakeEnv(db), d);
    const first = { ...db.rows.get('A')! };
    await reconcileMemberships(fakeEnv(db), d);
    const second = db.rows.get('A')!;
    expect(second).toEqual(first);
  });
});
