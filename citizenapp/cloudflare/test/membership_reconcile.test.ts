import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { Miniflare } from "miniflare";
import {
  reconcileMembershipForCid,
  reconcileMemberships,
  type ReconcileDeps,
} from "../src/membership/reconcile";
import type { ChainSubscriptionState } from "../src/chain/subscription";
import type { Env } from "../src/types";

const POINT = {
  blockHash: `0x${"a".repeat(64)}`,
  blockNumber: 90,
  chainTimestamp: 9_000,
  observedAt: 10_000,
};
const SCHEMA_SQL = readFileSync(
  resolve(process.cwd(), "schema/citizenapp.sql"),
  "utf8",
);

// 链上订阅直接按身份主键 CID 读取；账户列只保留为历史审计字段。
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
  finalized_block_number: number;
  verified_at: number;
  entitlement_lapsed_at: number | null;
}

class FakeDb {
  // PK = cid_number；account_id 仅为当前绑定账户列。
  rows = new Map<string, Row>();
  chainClockBlock = 0;
  batchCalls = 0;

  seed(cidNumber: string, accountId: string, paidUntil: number, status = "active"): void {
    this.rows.set(cidNumber, {
      cid_number: cidNumber,
      account_id: accountId,
      membership_level: "freedom",
      paid_until: paidUntil,
      subscription_status: status,
      finalized_block_number: 1,
      verified_at: 1,
      entitlement_lapsed_at: null,
    });
  }

  prepare(sql: string): FakeStmt {
    return new FakeStmt(this, sql);
  }

  async batch(statements: FakeStmt[]): Promise<unknown[]> {
    this.batchCalls += 1;
    const results: unknown[] = [];
    for (const statement of statements) {
      results.push(await statement.run());
    }
    return results;
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
        .map((row) => ({
          cid_number: row.cid_number,
          account_id: row.account_id,
        }));
      return { results: results as T[] };
    }
    return { results: [] };
  }

  async run(): Promise<{ meta: { changes: number } }> {
    if (this.sql.includes("INSERT INTO chain_clock")) {
      const blockNumber = this.args[1] as number;
      if (blockNumber > this.db.chainClockBlock) {
        this.db.chainClockBlock = blockNumber;
      }
      return { meta: { changes: 1 } };
    }
    if (this.sql.includes("subscription_status = 'terminated'")) {
      const cidNumber = this.args[3] as string;
      const row = this.db.rows.get(cidNumber);
      const blockNumber = this.args[0] as number;
      if (
        row &&
        row.finalized_block_number <= blockNumber &&
        blockNumber >= this.db.chainClockBlock
      ) {
        row.subscription_status = "terminated";
        row.entitlement_lapsed_at = row.paid_until;
        row.finalized_block_number = blockNumber;
        row.verified_at = this.args[2] as number;
      }
      return { meta: { changes: row ? 1 : 0 } };
    }
    if (this.sql.includes("INSERT INTO square_memberships")) {
      const cidNumber = this.args[0] as string;
      const row = this.db.rows.get(cidNumber);
      const blockNumber = this.args[8] as number;
      if (
        blockNumber >= this.db.chainClockBlock &&
        (!row || row.finalized_block_number <= blockNumber)
      ) {
        this.db.rows.set(cidNumber, {
          cid_number: cidNumber,
          account_id: this.args[1] as string,
          membership_level: this.args[2] as string,
          paid_until: this.args[6] as number,
          subscription_status: this.args[7] as string,
          finalized_block_number: blockNumber,
          verified_at: this.args[10] as number,
          entitlement_lapsed_at: this.args[11] as number | null,
        });
      }
      return { meta: { changes: 1 } };
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
    readSubscriptionAtBlock: async (_env, cidNumber) => {
      if (fail.has(cidNumber)) throw new Error("chain failed");
      return states[cidNumber] ?? null;
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
    const result = await reconcileMemberships(
      env(db),
      deps({ [CID_DUE]: active("democracy") }),
    );
    expect(result).toEqual({ scanned: 1, updated: 1, failed: 0 });
    expect(db.rows.get(CID_DUE)?.membership_level).toBe("democracy");
    expect(db.rows.get(CID_DUE)?.paid_until).toBe(20_000);
    expect(db.rows.get(CID_FUTURE)?.paid_until).toBe(12_000);
  });

  it("链上查无时 fail-closed 为 terminated", async () => {
    const db = new FakeDb();
    db.seed(CID_DUE, "acct-due", 8_000);
    await reconcileMemberships(env(db), deps({ [CID_DUE]: null }));
    expect(db.rows.get(CID_DUE)?.subscription_status).toBe("terminated");
  });

  it("单条链读失败不阻断同批其它记录", async () => {
    const db = new FakeDb();
    db.seed(CID_BAD, "acct-bad", 7_000);
    db.seed(CID_GOOD, "acct-good", 8_000);
    const result = await reconcileMemberships(
      env(db),
      deps({ [CID_GOOD]: active("spark") }, new Set([CID_BAD])),
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
      deps({ [CID_DUE]: null }),
    )).resolves.toEqual({ scanned: 0, updated: 0, failed: 0 });
    await expect(reconcileMemberships(
      env(db, { CHAIN_URL: undefined }),
      deps({ [CID_DUE]: null }),
    )).resolves.toEqual({ scanned: 0, updated: 0, failed: 0 });
  });
});

describe("发布拒绝前单 CID 对账", () => {
  it("真实 D1 事务可原子补建会员行并遵守全局 finalized 单调保护", async () => {
    const miniflare = new Miniflare({
      modules: true,
      script: 'export default { fetch() { return new Response("test"); } }',
      compatibilityDate: "2026-07-29",
      d1Databases: ["DB"],
    });
    try {
      const bindings = await miniflare.getBindings<Env>();
      await applySchema(bindings.DB);
      const testEnv = {
        ...bindings,
        CHAIN_URL: "https://node.internal/rpc",
        CHAIN_ID: "id",
        CHAIN_SECRET: "secret",
      } as Env;
      const accountId = `0x${"7".repeat(64)}`;

      await reconcileMembershipForCid(
        testEnv,
        { cidNumber: CID_GOOD, accountId },
        deps({ [CID_GOOD]: active("spark") }),
      );
      const inserted = await bindings.DB.prepare(
        "SELECT * FROM square_memberships WHERE cid_number = ?",
      ).bind(CID_GOOD).first<Row>();
      expect(inserted).toMatchObject({
        cid_number: CID_GOOD,
        account_id: accountId,
        membership_level: "spark",
        finalized_block_number: POINT.blockNumber,
      });

      await bindings.DB.prepare(
        "UPDATE chain_clock SET finalized_block_number = ? WHERE clock_id = 1",
      ).bind(POINT.blockNumber + 1).run();
      await bindings.DB.prepare(
        "DELETE FROM square_memberships WHERE cid_number = ?",
      ).bind(CID_GOOD).run();
      await reconcileMembershipForCid(
        testEnv,
        { cidNumber: CID_GOOD, accountId },
        deps({ [CID_GOOD]: active("freedom") }),
      );
      const staleInsert = await bindings.DB.prepare(
        "SELECT cid_number FROM square_memberships WHERE cid_number = ?",
      ).bind(CID_GOOD).first<{ cid_number: string }>();
      expect(staleInsert).toBeNull();
    } finally {
      await miniflare.dispose();
    }
  });

  it("镜像缺失但 finalized 链订阅有效时原子补建并刷新链时钟", async () => {
    const db = new FakeDb();

    await reconcileMembershipForCid(
      env(db),
      { cidNumber: CID_GOOD, accountId: "acct-good" },
      deps({ [CID_GOOD]: active("spark") }),
    );

    expect(db.batchCalls).toBe(1);
    expect(db.chainClockBlock).toBe(POINT.blockNumber);
    expect(db.rows.get(CID_GOOD)).toMatchObject({
      account_id: "acct-good",
      membership_level: "spark",
      subscription_status: "active",
      paid_until: 20_000,
      finalized_block_number: POINT.blockNumber,
    });
  });

  it("较旧 finalized 结果不得覆盖较新的会员镜像", async () => {
    const db = new FakeDb();
    db.seed(CID_GOOD, "acct-new", 30_000);
    const row = db.rows.get(CID_GOOD)!;
    row.membership_level = "spark";
    row.finalized_block_number = POINT.blockNumber + 1;

    await reconcileMembershipForCid(
      env(db),
      { cidNumber: CID_GOOD, accountId: "acct-old" },
      deps({ [CID_GOOD]: active("freedom") }),
    );

    expect(db.rows.get(CID_GOOD)).toMatchObject({
      account_id: "acct-new",
      membership_level: "spark",
      paid_until: 30_000,
      finalized_block_number: POINT.blockNumber + 1,
    });
  });

  it("较新的全局链时钟存在时不补写更旧 finalized 的缺失会员行", async () => {
    const db = new FakeDb();
    db.chainClockBlock = POINT.blockNumber + 1;

    await reconcileMembershipForCid(
      env(db),
      { cidNumber: CID_GOOD, accountId: "acct-old" },
      deps({ [CID_GOOD]: active("freedom") }),
    );

    expect(db.chainClockBlock).toBe(POINT.blockNumber + 1);
    expect(db.rows.has(CID_GOOD)).toBe(false);
  });

  it("链上查无时只终止既有镜像，不伪造空会员行", async () => {
    const db = new FakeDb();

    await reconcileMembershipForCid(
      env(db),
      { cidNumber: CID_BAD, accountId: "acct-bad" },
      deps({ [CID_BAD]: null }),
    );

    expect(db.rows.has(CID_BAD)).toBe(false);
    expect(db.chainClockBlock).toBe(POINT.blockNumber);
  });
});

async function applySchema(db: D1Database): Promise<void> {
  const statements = SCHEMA_SQL
    .split("\n")
    .filter((line) => !line.trimStart().startsWith("--"))
    .join("\n")
    .split(";")
    .map((statement) => statement.trim())
    .filter((statement) => statement.length > 0);
  for (const statement of statements) {
    await db.prepare(statement).run();
  }
}
