import { describe, expect, it } from "vitest";
import { reconcileCreatorSubscriptions, type ReconcileDeps } from "../src/membership/reconcile";
import type { ChainSubscriptionState } from "../src/chain/subscription";
import type { Env } from "../src/types";

const POINT = {
  blockHash: `0x${"b".repeat(64)}`,
  blockNumber: 91,
  chainTimestamp: 9_000,
  observedAt: 10_000,
};

interface Row {
  subscriber_account: string;
  creator_account: string;
  tier_id: string;
  billing_period: string;
  paid_until: number;
  subscription_status: string;
  verified_at: number;
}

const rowKey = (subscriber: string, creator: string) => `${subscriber}|${creator}`;

class FakeDb {
  rows = new Map<string, Row>();
  seed(subscriber: string, creator: string, paidUntil: number): void {
    this.rows.set(rowKey(subscriber, creator), {
      subscriber_account: subscriber,
      creator_account: creator,
      tier_id: "old",
      billing_period: "monthly",
      paid_until: paidUntil,
      subscription_status: "active",
      verified_at: 1,
    });
  }
  prepare(sql: string): FakeStmt { return new FakeStmt(this, sql); }
}

class FakeStmt {
  private args: unknown[] = [];
  constructor(private readonly db: FakeDb, private readonly sql: string) {}
  bind(...args: unknown[]): FakeStmt { this.args = args; return this; }

  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes("SELECT subscriber_account, creator_account")) {
      const [chainTimestamp, limit] = this.args as [number, number];
      const results = [...this.db.rows.values()]
        .filter((row) => row.subscription_status === "active" && row.paid_until <= chainTimestamp)
        .sort((a, b) => a.paid_until - b.paid_until)
        .slice(0, limit)
        .map((row) => ({
          subscriber_account: row.subscriber_account,
          creator_account: row.creator_account,
        }));
      return { results: results as unknown as T[] };
    }
    return { results: [] };
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes("INSERT INTO chain_clock")) return { meta: { changes: 1 } };
    if (this.sql.includes("subscription_status = 'terminated'")) {
      const subscriber = this.args[3] as string;
      const creator = this.args[4] as string;
      const row = this.db.rows.get(rowKey(subscriber, creator));
      if (row) {
        row.subscription_status = "terminated";
        row.verified_at = this.args[2] as number;
      }
      return { meta: { changes: row ? 1 : 0 } };
    }
    if (this.sql.includes("UPDATE square_creator_subscriptions SET tier_id")) {
      const subscriber = this.args[12] as string;
      const creator = this.args[13] as string;
      const row = this.db.rows.get(rowKey(subscriber, creator));
      if (row) {
        row.tier_id = this.args[0] as string;
        row.billing_period = this.args[1] as string;
        row.paid_until = this.args[7] as number;
        row.subscription_status = this.args[8] as string;
        row.verified_at = this.args[11] as number;
      }
      return { meta: { changes: row ? 1 : 0 } };
    }
    return { meta: { changes: 1 } };
  }
}

function env(db: FakeDb, overrides: Partial<Env> = {}): Env {
  return {
    DB: db as unknown as D1Database,
    CHAIN_URL: "https://node.internal/rpc",
    CHAIN_ID: "id",
    CHAIN_SECRET: "secret",
    CREATOR_RECONCILE_ENABLED: "1",
    MEMBERSHIP_RECONCILE_BATCH: "50",
    ...overrides,
  } as Env;
}

function deps(
  states: Record<string, ChainSubscriptionState | null>,
  fail = new Set<string>(),
): ReconcileDeps {
  return {
    finalizedPoint: async () => POINT,
    readSubscriptionAtBlock: async (_env, subscriber, issuer) => {
      const creator = issuer.kind === "creator" ? issuer.creatorAccount : "";
      const key = rowKey(subscriber, creator);
      if (fail.has(key)) throw new Error("chain failed");
      return states[key] ?? null;
    },
  };
}

function active(): ChainSubscriptionState {
  return {
    plan: { kind: "creator", tierId: "gold", billingPeriod: "yearly" },
    pendingPlan: null,
    startedAt: 1_000,
    lastChargedAt: 9_000,
    lastChargedPriceFen: 500n,
    paidUntil: 20_000,
    status: "active",
  };
}

describe("创作者订阅复合主键到期对账", () => {
  it("同一创作者的多个订阅者按复合主键独立更新", async () => {
    const db = new FakeDb();
    db.seed("S1", "C", 7_000);
    db.seed("S2", "C", 8_000);
    await reconcileCreatorSubscriptions(env(db), deps({
      "S1|C": active(),
      "S2|C": null,
    }));
    expect(db.rows.get("S1|C")?.tier_id).toBe("gold");
    expect(db.rows.get("S1|C")?.billing_period).toBe("yearly");
    expect(db.rows.get("S2|C")?.subscription_status).toBe("terminated");
  });

  it("未到期记录不扫描，失败行不阻断同批", async () => {
    const db = new FakeDb();
    db.seed("bad", "C", 7_000);
    db.seed("good", "C", 8_000);
    db.seed("future", "C", 12_000);
    const result = await reconcileCreatorSubscriptions(
      env(db),
      deps({ "good|C": active() }, new Set(["bad|C"])),
    );
    expect(result).toEqual({ scanned: 2, updated: 1, failed: 1 });
    expect(db.rows.get("bad|C")?.verified_at).toBe(1);
    expect(db.rows.get("future|C")?.verified_at).toBe(1);
  });
});
