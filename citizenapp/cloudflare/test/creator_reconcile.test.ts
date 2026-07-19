import { describe, expect, it } from 'vitest';
import {
  reconcileCreatorSubscriptions,
  type ReconcileDeps
} from '../src/membership/reconcile';
import type { ChainSubscriptionState } from '../src/chain/subscription';
import type { Env } from '../src/types';

// 创作者订阅对账：镜像表 square_creator_subscriptions（复合键 subscriber+creator）。
// status、当前 tier/period、最近实扣价和 last_charged_at 全部对齐链上真源。

const NOW = 9_000_000;

interface FakeRow {
  subscriber_account: string;
  creator_account: string;
  tier_id: string;
  period: string;
  price_fen: number;
  status: string;
  last_charged_at: number;
  last_tx_hash: string;
  updated_at: number;
}

function key(subscriber: string, creator: string): string {
  return `${subscriber}|${creator}`;
}

class FakeDb {
  rows = new Map<string, FakeRow>();

  seed(row: Partial<FakeRow> & { subscriber_account: string; creator_account: string }): void {
    this.rows.set(key(row.subscriber_account, row.creator_account), {
      tier_id: 't1',
      period: 'monthly',
      price_fen: 199_900,
      status: 'active',
      last_charged_at: 0,
      last_tx_hash: '0x' + '0'.repeat(64),
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
      this.sql.includes('FROM square_creator_subscriptions') &&
      this.sql.includes('ORDER BY updated_at ASC LIMIT')
    ) {
      const limit = this.args[0] as number;
      const rows = [...this.db.rows.values()]
        .sort((a, b) => a.updated_at - b.updated_at)
        .slice(0, limit)
        .map((row) => ({
          subscriber_account: row.subscriber_account,
          creator_account: row.creator_account
        }));
      return { results: rows as unknown as T[], success: true };
    }
    return { results: [], success: true };
  }

  async run(): Promise<{ success: boolean; meta: { changes: number } }> {
    if (!this.sql.includes('UPDATE square_creator_subscriptions')) {
      return { success: true, meta: { changes: 1 } };
    }
    // Active：binds = [tier, period, price, lastChargedAt, now, subscriber, creator]。
    if (this.sql.includes("status = 'active'") && this.sql.includes('tier_id = ?')) {
      const [tierId, period, priceFen, lastCharged, now, subscriber, creator] = this.args as [
        string,
        string,
        number,
        number,
        number,
        string,
        string
      ];
      const k = key(subscriber, creator);
      const existing = this.db.rows.get(k);
      if (existing) {
        this.db.rows.set(k, {
          ...existing,
          tier_id: tierId,
          period,
          price_fen: priceFen,
          status: 'active',
          last_charged_at: lastCharged,
          updated_at: now
        });
      }
      return { success: true, meta: { changes: existing ? 1 : 0 } };
    }
    // 收紧：binds = [status, now, subscriber, creator]。
    const [status, now, subscriber, creator] = this.args as [string, number, string, string];
    const k = key(subscriber, creator);
    const existing = this.db.rows.get(k);
    if (existing) {
      this.db.rows.set(k, { ...existing, status, updated_at: now });
    }
    return { success: true, meta: { changes: existing ? 1 : 0 } };
  }
}

function fakeEnv(db: FakeDb, overrides: Partial<Env> = {}): Env {
  return {
    DB: db as unknown as D1Database,
    CHAIN_URL: 'https://node.internal/rpc',
    CHAIN_ID: 'access-id',
    CHAIN_SECRET: 'access-secret',
    CREATOR_RECONCILE_ENABLED: '1',
    MEMBERSHIP_RECONCILE_BATCH: '50',
    ...overrides
  } as unknown as Env;
}

/// 注入假链读：按 subscriber|creator 取态；throwPairs 里的键抛错模拟链读失败。
function deps(
  map: Record<string, ChainSubscriptionState | null>,
  throwPairs: Set<string> = new Set()
): ReconcileDeps {
  return {
    now: () => NOW,
    readSubscription: async (_env, subscriber, issuer) => {
      const creator = issuer.kind === 'creator' ? issuer.creatorAccount : '';
      const k = key(subscriber, creator);
      if (throwPairs.has(k)) throw new Error('chain read failed');
      return map[k] ?? null;
    }
  };
}

function creatorActive(lastChargedAt: number): ChainSubscriptionState {
  return {
    plan: { kind: 'creator', tierId: 'gold', billingPeriod: 'yearly' },
    pendingPlan: null,
    startedAt: lastChargedAt,
    lastChargedAt,
    lastChargedPriceFen: 599_900n,
    paidUntil: lastChargedAt + 1_000,
    status: 'active'
  };
}

describe('reconcileCreatorSubscriptions（复合键，fail-closed）', () => {
  it('链上 Active → 当前档位、价格、状态和 last_charged 全量对齐', async () => {
    const db = new FakeDb();
    db.seed({
      subscriber_account: 'S',
      creator_account: 'C',
      status: 'cancelled',
      tier_id: 'gold',
      period: 'yearly',
      updated_at: 100
    });
    await reconcileCreatorSubscriptions(fakeEnv(db), deps({ 'S|C': creatorActive(1000) }));

    const row = db.rows.get(key('S', 'C'))!;
    expect(row.status).toBe('active');
    expect(row.last_charged_at).toBe(1000);
    expect(row.tier_id).toBe('gold');
    expect(row.period).toBe('yearly');
    expect(row.price_fen).toBe(599_900);
    expect(row.updated_at).toBe(NOW);
  });

  it('链上查无 → 收紧 cancelled', async () => {
    const db = new FakeDb();
    db.seed({ subscriber_account: 'S', creator_account: 'C', status: 'active', updated_at: 100 });
    await reconcileCreatorSubscriptions(fakeEnv(db), deps({ 'S|C': null }));
    expect(db.rows.get(key('S', 'C'))!.status).toBe('cancelled');
  });

  it('链上 Terminated → 收紧 terminated', async () => {
    const db = new FakeDb();
    db.seed({ subscriber_account: 'S', creator_account: 'C', status: 'active', updated_at: 100 });
    await reconcileCreatorSubscriptions(
      fakeEnv(db),
      deps({ 'S|C': { ...creatorActive(1000), status: 'terminated' } })
    );
    expect(db.rows.get(key('S', 'C'))!.status).toBe('terminated');
  });

  it('开关关闭时直接返回、零扫描', async () => {
    const db = new FakeDb();
    db.seed({ subscriber_account: 'S', creator_account: 'C', status: 'active', updated_at: 100 });
    const result = await reconcileCreatorSubscriptions(
      fakeEnv(db, { CREATOR_RECONCILE_ENABLED: '0' }),
      deps({ 'S|C': null })
    );
    expect(result).toEqual({ scanned: 0, updated: 0, failed: 0 });
    expect(db.rows.get(key('S', 'C'))!.status).toBe('active');
  });

  it('复合键：同创作者多订阅者各自独立对账', async () => {
    const db = new FakeDb();
    db.seed({ subscriber_account: 'S1', creator_account: 'C', status: 'active', updated_at: 100 });
    db.seed({ subscriber_account: 'S2', creator_account: 'C', status: 'active', updated_at: 200 });

    const result = await reconcileCreatorSubscriptions(
      fakeEnv(db),
      deps({ 'S1|C': creatorActive(1000), 'S2|C': null })
    );

    expect(result.scanned).toBe(2);
    expect(db.rows.get(key('S1', 'C'))!.status).toBe('active');
    expect(db.rows.get(key('S2', 'C'))!.status).toBe('cancelled');
  });

  it('单条链读失败不阻断整批：失败对不动，其它照常', async () => {
    const db = new FakeDb();
    db.seed({ subscriber_account: 'S1', creator_account: 'C', status: 'active', updated_at: 100 });
    db.seed({ subscriber_account: 'S2', creator_account: 'C', status: 'cancelled', updated_at: 200 });

    const result = await reconcileCreatorSubscriptions(
      fakeEnv(db),
      deps({ 'S2|C': creatorActive(1000) }, new Set(['S1|C']))
    );

    expect(result).toEqual({ scanned: 2, updated: 1, failed: 1 });
    expect(db.rows.get(key('S1', 'C'))!.updated_at).toBe(100); // 失败不动
    expect(db.rows.get(key('S2', 'C'))!.status).toBe('active');
  });
});
