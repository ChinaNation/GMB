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

// 身份主键 cid 与其当前绑定账户 account_id 刻意取不同值，验证「按 cid 迭代镜像、按 account_id 回链读」的拆分。
const CID_DUE = "CN220-CTZN2-198805200-2026";
const CID_FUTURE = "CN220-CTZN2-197001010-2026";
const CID_BAD = "CN220-CTZN2-199001010-2026";
const CID_GOOD = "CN220-CTZN2-199512120-2026";

interface Row {
  cid_number: string;
  account_id: string;
  membership_level: string;
  paid_until: number;
  subscription_status: string;
  verified_at: number;
  entitlement_lapsed_at: number | null;
}

class FakeDb {
  // PK = cid_number；account_id 仅为当前绑定账户列。
  rows = new Map<string, Row>();

  seed(cidNumber: string, accountId: string, paidUntil: number, status = "active"): void {
    this.rows.set(cidNumber, {
      cid_number: cidNumber,
      account_id: accountId,
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
    if (this.sql.includes("SELECT cid_number, account_id FROM square_memberships")) {
      const [chainTimestamp, limit] = this.args as [number, number];
      const results = [...this.db.rows.values()]
        .filter((row) => row.subscription_status === "active" && row.paid_until <= chainTimestamp)
        .sort((a, b) => a.paid_until - b.paid_until)
        .slice(0, limit)
        .map((row) => ({ cid_number: row.cid_number, account_id: row.account_id }));
      return { results: results as T[] };
    }
    return { results: [] };
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes("INSERT INTO chain_clock")) return { meta: { changes: 1 } };
    if (this.sql.includes("subscription_status = 'terminated'")) {
      const cidNumber = this.args[3] as string;
      const row = this.db.rows.get(cidNumber);
      if (row) {
        row.subscription_status = "terminated";
        row.entitlement_lapsed_at = row.paid_until;
        row.verified_at = this.args[2] as number;
      }
      return { meta: { changes: row ? 1 : 0 } };
    }
    if (this.sql.includes("UPDATE square_memberships SET membership_level")) {
      const cidNumber = this.args[10] as string;
      const row = this.db.rows.get(cidNumber);
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
    // 回链读订阅按当前绑定账户 account_id 入参，states/fail 均以 account_id 为键。
    readSubscriptionAtBlock: async (_env, accountId) => {
      if (fail.has(accountId)) throw new Error("chain failed");
      return states[accountId] ?? null;
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
    db.seed(CID_DUE, "acct-due", 8_000);
    db.seed(CID_FUTURE, "acct-future", 12_000);
    const result = await reconcileMemberships(env(db), deps({ "acct-due": active("democracy") }));
    expect(result).toEqual({ scanned: 1, updated: 1, failed: 0 });
    expect(db.rows.get(CID_DUE)?.membership_level).toBe("democracy");
    expect(db.rows.get(CID_DUE)?.paid_until).toBe(20_000);
    expect(db.rows.get(CID_FUTURE)?.paid_until).toBe(12_000);
  });

  it("链上查无时 fail-closed 为 terminated", async () => {
    const db = new FakeDb();
    db.seed(CID_DUE, "acct-due", 8_000);
    await reconcileMemberships(env(db), deps({ "acct-due": null }));
    expect(db.rows.get(CID_DUE)?.subscription_status).toBe("terminated");
  });

  it("单条链读失败不阻断同批其它记录", async () => {
    const db = new FakeDb();
    db.seed(CID_BAD, "acct-bad", 7_000);
    db.seed(CID_GOOD, "acct-good", 8_000);
    const result = await reconcileMemberships(
      env(db),
      deps({ "acct-good": active("spark") }, new Set(["acct-bad"])),
    );
    expect(result).toEqual({ scanned: 2, updated: 1, failed: 1 });
    expect(db.rows.get(CID_BAD)?.verified_at).toBe(1);
    expect(db.rows.get(CID_GOOD)?.membership_level).toBe("spark");
  });

  it("关闭开关或链 RPC 未配置时零扫描", async () => {
    const db = new FakeDb();
    db.seed(CID_DUE, "acct-due", 8_000);
    await expect(reconcileMemberships(
      env(db, { MEMBERSHIP_RECONCILE_ENABLED: "0" }),
      deps({ "acct-due": null }),
    )).resolves.toEqual({ scanned: 0, updated: 0, failed: 0 });
    await expect(reconcileMemberships(
      env(db, { CHAIN_URL: undefined }),
      deps({ "acct-due": null }),
    )).resolves.toEqual({ scanned: 0, updated: 0, failed: 0 });
  });
});
