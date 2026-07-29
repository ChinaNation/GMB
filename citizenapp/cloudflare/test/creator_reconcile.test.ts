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

// 复合主键落在身份 cid 上;回链读订阅仍按各自绑定账户 account_id。两套键分离。
interface Party {
  cid: string;
  account: string;
}

const CREATOR: Party = {
  cid: "CN220-CTZN2-900000001-2026",
  account: `0x${"c".repeat(64)}`,
};

function subscriberParty(serial: string, fill: string): Party {
  return {
    cid: `CN220-CTZN2-${serial}-2026`,
    account: `0x${fill.repeat(64)}`,
  };
}

const S1 = subscriberParty("100000001", "1");
const S2 = subscriberParty("100000002", "2");
const BAD = subscriberParty("100000003", "3");
const GOOD = subscriberParty("100000004", "4");
const FUTURE = subscriberParty("100000005", "5");

interface Row {
  subscriber_cid_number: string;
  creator_cid_number: string;
  subscriber_account_id: string;
  creator_account_id: string;
  tier_id: string;
  billing_period: string;
  paid_until: number;
  subscription_status: string;
  verified_at: number;
}

// 行主键 = 双 cid;链读键 = 双 account。
const rowKey = (subscriberCid: string, creatorCid: string) => `${subscriberCid}|${creatorCid}`;
const acctKey = (subscriberAccount: string, creatorAccount: string) => `${subscriberAccount}|${creatorAccount}`;

class FakeDb {
  rows = new Map<string, Row>();
  seed(subscriber: Party, creator: Party, paidUntil: number): void {
    this.rows.set(rowKey(subscriber.cid, creator.cid), {
      subscriber_cid_number: subscriber.cid,
      creator_cid_number: creator.cid,
      subscriber_account_id: subscriber.account,
      creator_account_id: creator.account,
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
    if (this.sql.includes("SELECT subscriber_cid_number, creator_cid_number, subscriber_account_id, creator_account_id")) {
      const [chainTimestamp, limit] = this.args as [number, number];
      const results = [...this.db.rows.values()]
        .filter((row) => row.subscription_status === "active" && row.paid_until <= chainTimestamp)
        .sort((a, b) => a.paid_until - b.paid_until)
        .slice(0, limit)
        .map((row) => ({
          subscriber_cid_number: row.subscriber_cid_number,
          creator_cid_number: row.creator_cid_number,
          subscriber_account_id: row.subscriber_account_id,
          creator_account_id: row.creator_account_id,
        }));
      return { results: results as unknown as T[] };
    }
    return { results: [] };
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes("INSERT INTO chain_clock")) return { meta: { changes: 1 } };
    if (this.sql.includes("subscription_status = 'terminated'")) {
      const subscriberCid = this.args[3] as string;
      const creatorCid = this.args[4] as string;
      const row = this.db.rows.get(rowKey(subscriberCid, creatorCid));
      if (row) {
        row.subscription_status = "terminated";
        row.verified_at = this.args[2] as number;
      }
      return { meta: { changes: row ? 1 : 0 } };
    }
    if (this.sql.includes("UPDATE square_creator_subscriptions SET tier_id")) {
      const subscriberCid = this.args[10] as string;
      const creatorCid = this.args[11] as string;
      const row = this.db.rows.get(rowKey(subscriberCid, creatorCid));
      if (row) {
        row.tier_id = this.args[0] as string;
        row.billing_period = this.args[1] as string;
        row.paid_until = this.args[5] as number;
        row.subscription_status = this.args[6] as string;
        row.verified_at = this.args[9] as number;
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
    readSubscriptionAtBlock: async (_env, subscriberAccount, issuer) => {
      const creatorAccount = issuer.kind === "creator" ? issuer.creatorAccountId : "";
      const key = acctKey(subscriberAccount, creatorAccount);
      if (fail.has(key)) throw new Error("chain failed");
      return states[key] ?? null;
    },
  };
}

function active(): ChainSubscriptionState {
  return {
    plan: { kind: "creator", tierId: "gold", billingPeriod: "yearly" },
    startedAt: 1_000,
    lastChargedAt: 9_000,
    lastChargedPriceFen: 500n,
    paidUntil: 20_000,
    status: "active",
    authorizedPriceFen: 500n,
    suspendReason: null,
  };
}

describe("创作者订阅复合主键到期对账", () => {
  it("同一创作者的多个订阅者按复合主键独立更新", async () => {
    const db = new FakeDb();
    db.seed(S1, CREATOR, 7_000);
    db.seed(S2, CREATOR, 8_000);
    await reconcileCreatorSubscriptions(env(db), deps({
      [acctKey(S1.account, CREATOR.account)]: active(),
      [acctKey(S2.account, CREATOR.account)]: null,
    }));
    expect(db.rows.get(rowKey(S1.cid, CREATOR.cid))?.tier_id).toBe("gold");
    expect(db.rows.get(rowKey(S1.cid, CREATOR.cid))?.billing_period).toBe("yearly");
    expect(db.rows.get(rowKey(S2.cid, CREATOR.cid))?.subscription_status).toBe("terminated");
  });

  it("未到期记录不扫描，失败行不阻断同批", async () => {
    const db = new FakeDb();
    db.seed(BAD, CREATOR, 7_000);
    db.seed(GOOD, CREATOR, 8_000);
    db.seed(FUTURE, CREATOR, 12_000);
    const result = await reconcileCreatorSubscriptions(
      env(db),
      deps(
        { [acctKey(GOOD.account, CREATOR.account)]: active() },
        new Set([acctKey(BAD.account, CREATOR.account)]),
      ),
    );
    expect(result).toEqual({ scanned: 2, updated: 1, failed: 1 });
    expect(db.rows.get(rowKey(BAD.cid, CREATOR.cid))?.verified_at).toBe(1);
    expect(db.rows.get(rowKey(FUTURE.cid, CREATOR.cid))?.verified_at).toBe(1);
  });
});
