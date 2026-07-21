import { describe, expect, it } from "vitest";
import { reconcileMemberships, type ReconcileDeps } from "../src/membership/reconcile";
import type { ChainSubscriptionState } from "../src/chain/subscription";
import type { Env } from "../src/types";

const POINT = {
  blockHash: `0x${"a".repeat(64)}`,
  blockNumber: 90,
  chainTimestamp: 9_000,
  observedAt: 10_000,
};

interface Row {
  owner_account: string;
  membership_level: string;
  paid_until: number;
  subscription_status: string;
  verified_at: number;
  entitlement_lapsed_at: number | null;
}

class FakeDb {
  rows = new Map<string, Row>();

  seed(owner: string, paidUntil: number, status = "active"): void {
    this.rows.set(owner, {
      owner_account: owner,
      membership_level: "freedom",
      paid_until: paidUntil,
      subscription_status: status,
      verified_at: 1,
      entitlement_lapsed_at: null,
    });
  }

  prepare(sql: string): FakeStmt {
    return new FakeStmt(this, sql);
  }
}

class FakeStmt {
  private args: unknown[] = [];
  constructor(private readonly db: FakeDb, private readonly sql: string) {}
  bind(...args: unknown[]): FakeStmt { this.args = args; return this; }

  async all<T>(): Promise<{ results: T[] }> {
    if (this.sql.includes("SELECT owner_account FROM square_memberships")) {
      const [chainTimestamp, limit] = this.args as [number, number];
      const results = [...this.db.rows.values()]
        .filter((row) => row.subscription_status === "active" && row.paid_until <= chainTimestamp)
        .sort((a, b) => a.paid_until - b.paid_until)
        .slice(0, limit)
        .map((row) => ({ owner_account: row.owner_account }));
      return { results: results as T[] };
    }
    return { results: [] };
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes("INSERT INTO chain_clock")) return { meta: { changes: 1 } };
    if (this.sql.includes("subscription_status = 'terminated'")) {
      const owner = this.args[3] as string;
      const row = this.db.rows.get(owner);
      if (row) {
        row.subscription_status = "terminated";
        row.entitlement_lapsed_at = row.paid_until;
        row.verified_at = this.args[2] as number;
      }
      return { meta: { changes: row ? 1 : 0 } };
    }
    if (this.sql.includes("UPDATE square_memberships SET membership_level")) {
      const owner = this.args[10] as string;
      const row = this.db.rows.get(owner);
      if (row) {
        row.membership_level = this.args[0] as string;
        row.paid_until = this.args[4] as number;
        row.subscription_status = this.args[5] as string;
        row.verified_at = this.args[8] as number;
        row.entitlement_lapsed_at = this.args[9] as number | null;
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
    MEMBERSHIP_RECONCILE_ENABLED: "1",
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
    readSubscriptionAtBlock: async (_env, subscriber) => {
      if (fail.has(subscriber)) throw new Error("chain failed");
      return states[subscriber] ?? null;
    },
  };
}

function active(level: "freedom" | "democracy" | "spark"): ChainSubscriptionState {
  return {
    plan: { kind: "platform", membershipLevel: level },
    startedAt: 1_000,
    lastChargedAt: 9_000,
    lastChargedPriceFen: 200n,
    paidUntil: 20_000,
    status: "active",
    authorizedPriceFen: 200n,
    suspendReason: null,
  };
}

describe("平台订阅低资源到期对账", () => {
  it("只扫描已到期 Active，未到期记录不读链", async () => {
    const db = new FakeDb();
    db.seed("due", 8_000);
    db.seed("future", 12_000);
    const result = await reconcileMemberships(env(db), deps({ due: active("democracy") }));
    expect(result).toEqual({ scanned: 1, updated: 1, failed: 0 });
    expect(db.rows.get("due")?.membership_level).toBe("democracy");
    expect(db.rows.get("due")?.paid_until).toBe(20_000);
    expect(db.rows.get("future")?.paid_until).toBe(12_000);
  });

  it("链上查无时 fail-closed 为 terminated", async () => {
    const db = new FakeDb();
    db.seed("due", 8_000);
    await reconcileMemberships(env(db), deps({ due: null }));
    expect(db.rows.get("due")?.subscription_status).toBe("terminated");
  });

  it("单条链读失败不阻断同批其它记录", async () => {
    const db = new FakeDb();
    db.seed("bad", 7_000);
    db.seed("good", 8_000);
    const result = await reconcileMemberships(
      env(db),
      deps({ good: active("spark") }, new Set(["bad"])),
    );
    expect(result).toEqual({ scanned: 2, updated: 1, failed: 1 });
    expect(db.rows.get("bad")?.verified_at).toBe(1);
    expect(db.rows.get("good")?.membership_level).toBe("spark");
  });

  it("关闭开关或链 RPC 未配置时零扫描", async () => {
    const db = new FakeDb();
    db.seed("due", 8_000);
    await expect(reconcileMemberships(
      env(db, { MEMBERSHIP_RECONCILE_ENABLED: "0" }),
      deps({ due: null }),
    )).resolves.toEqual({ scanned: 0, updated: 0, failed: 0 });
    await expect(reconcileMemberships(
      env(db, { CHAIN_URL: undefined }),
      deps({ due: null }),
    )).resolves.toEqual({ scanned: 0, updated: 0, failed: 0 });
  });
});
