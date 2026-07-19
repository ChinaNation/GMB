import { describe, expect, it } from "vitest";
import { encodeAddress } from "@polkadot/util-crypto";
import { hexToBytes } from "../src/shared/signing_message";
import { storageValueKey } from "../src/chain/storage_key";
import {
  buildCreatorPlansKey,
  buildSubscriptionKey,
  decodeCreatorPlans,
  decodeSubscriptionState,
} from "../src/chain/subscription";
import {
  creatorPlanSaveRoute,
  type CreatorPlanSaveDeps,
} from "../src/membership/creator";
import type { Env, SessionState } from "../src/types";

const STATE_PLATFORM =
  "0002000068e5cf8b0100000068e5cf8b0100001c8d5b0000000000000000000000000000fc1a478c01000000";

describe("decodeSubscriptionState", () => {
  it("严格解码平台 Active 状态和链上 paid_until", () => {
    const state = decodeSubscriptionState(hexToBytes(STATE_PLATFORM));
    expect(state).toEqual({
      plan: { kind: "platform", membershipLevel: "spark" },
      pendingPlan: null,
      startedAt: 1_700_000_000_000,
      lastChargedAt: 1_700_000_000_000,
      lastChargedPriceFen: 5_999_900n,
      paidUntil: 1_702_000_000_000,
      status: "active",
    });
  });

  it("解码扣款失败后的 Terminated 状态", () => {
    const terminated = STATE_PLATFORM.slice(0, -2) + "02";
    const state = decodeSubscriptionState(hexToBytes(terminated));
    expect(state?.status).toBe("terminated");
  });

  it("解码创作者 tier_id 和自然周期枚举", () => {
    const plan = "0124737570706f7274657201";
    const stateHex =
      plan +
      "00" +
      "0068e5cf8b010000" +
      "0068e5cf8b010000" +
      "32000000000000000000000000000000" +
      "00fc1a478c010000" +
      "00";
    const state = decodeSubscriptionState(hexToBytes(stateHex));
    expect(state?.plan).toEqual({
      kind: "creator",
      tierId: "supporter",
      billingPeriod: "quarterly",
    });
  });

  it("拒绝非法枚举、非法到期时间、截断和尾随字节", () => {
    expect(() => decodeSubscriptionState(hexToBytes("0003"))).toThrow();
    expect(() => decodeSubscriptionState(hexToBytes(STATE_PLATFORM.slice(0, -18) + "000000000000000000"))).toThrow();
    expect(() => decodeSubscriptionState(hexToBytes(STATE_PLATFORM + "00"))).toThrow();
    expect(() => decodeSubscriptionState(new Uint8Array())).toThrow();
  });
});

describe("decodeCreatorPlans", () => {
  it("严格解码 tier_id 与链上月/年价格", () => {
    const price50 = "32000000000000000000000000000000";
    const price500 = "f4010000000000000000000000000000";
    const tiers = decodeCreatorPlans(
      hexToBytes(`0424737570706f727465720800${price50}02${price500}`),
    );
    expect(tiers).toEqual([
      {
        tierId: "supporter",
        pricesFen: { monthly: 50n, yearly: 500n },
      },
    ]);
  });

  it("拒绝重复周期、截断和尾随字节", () => {
    const price = "32000000000000000000000000000000";
    expect(() => decodeCreatorPlans(hexToBytes(`0404740800${price}00${price}`))).toThrow();
    expect(() => decodeCreatorPlans(hexToBytes("04047404"))).toThrow();
    expect(() => decodeCreatorPlans(hexToBytes("0000"))).toThrow();
  });
});

describe("buildSubscriptionKey", () => {
  it("平台键保持 Blake2_128Concat 单键布局", () => {
    const account = encodeAddress(
      Uint8Array.from(Array.from({ length: 32 }, (_, index) => index + 1)),
      2027,
    );
    const key = buildSubscriptionKey(account, { kind: "platform" });
    const prefix = storageValueKey("SquarePost", "Subscriptions");
    expect(Array.from(key.slice(0, 32))).toEqual(Array.from(prefix));
    expect(key.length).toBe(81);
    expect(key[key.length - 1]).toBe(0x00);
  });

  it("创作者键包含收款账户", () => {
    const subscriber = encodeAddress(new Uint8Array(32).fill(2), 2027);
    const creator = encodeAddress(new Uint8Array(32).fill(9), 2027);
    const key = buildSubscriptionKey(subscriber, {
      kind: "creator",
      creatorAccount: creator,
    });
    expect(key.length).toBe(113);
    expect(key[key.length - 33]).toBe(0x01);
  });

  it("CreatorPlans 键使用创作者账户作为 Blake2_128Concat 单键", () => {
    const creator = encodeAddress(new Uint8Array(32).fill(7), 2027);
    const key = buildCreatorPlansKey(creator);
    const prefix = storageValueKey("SquarePost", "CreatorPlans");
    expect(Array.from(key.slice(0, 32))).toEqual(Array.from(prefix));
    expect(key.length).toBe(80);
  });
});

describe("creatorPlanSaveRoute", () => {
  it("finalized CreatorPlans 一致后直接保存展示名，不要求第二次业务签名", async () => {
    const stored: unknown[][] = [];
    const env = creatorPlanEnv(stored);
    const response = await creatorPlanSaveRoute(
      creatorPlanRequest({
        tx_hash: "0x" + "a".repeat(64),
        tiers: [
          {
            tier_id: "supporter",
            name: "支持者",
            prices_fen: { monthly: 50 },
          },
        ],
      }),
      env,
      creatorPlanDeps(50n),
    );
    const body = (await response.json()) as {
      plan: { tiers: Array<{ name: string; prices_fen: { monthly: number } }> };
    };

    expect(body.plan.tiers[0]).toEqual({
      tier_id: "supporter",
      name: "支持者",
      prices_fen: { monthly: 50 },
    });
    expect(stored).toHaveLength(1);
    expect(JSON.parse(stored[0][1] as string)).toEqual(body.plan.tiers);
  });

  it("请求价格与 finalized 链价不一致时拒绝写 Cloudflare", async () => {
    const stored: unknown[][] = [];
    await expect(
      creatorPlanSaveRoute(
        creatorPlanRequest({
          tx_hash: "0x" + "b".repeat(64),
          tiers: [
            {
              tier_id: "supporter",
              name: "支持者",
              prices_fen: { monthly: 51 },
            },
          ],
        }),
        creatorPlanEnv(stored),
        creatorPlanDeps(50n),
      ),
    ).rejects.toMatchObject({
      status: 409,
      code: "creator_plans_not_finalized",
    });
    expect(stored).toHaveLength(0);
  });
});

function creatorPlanRequest(body: Record<string, unknown>): Request {
  return new Request("https://w/v1/square/creator/plan", {
    method: "POST",
    headers: {
      authorization: "Bearer creator-session",
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
}

function creatorPlanDeps(price: bigint): CreatorPlanSaveDeps {
  return {
    readCreatorPlans: async () => [
      { tierId: "supporter", pricesFen: { monthly: price } },
    ],
  };
}

function creatorPlanEnv(stored: unknown[][]): Env {
  const session: SessionState = {
    owner_account: encodeAddress(new Uint8Array(32).fill(8), 2027),
    device_key_hash: "d".repeat(64),
    created_at: Date.now(),
    expires_at: Date.now() + 60_000,
  };
  const statement = {
    args: [] as unknown[],
    bind(...args: unknown[]) {
      this.args = args;
      return this;
    },
    async run() {
      stored.push(this.args);
      return { success: true, meta: { changes: 1 } };
    },
  };
  return {
    DB: {
      prepare: () => statement,
    } as unknown as D1Database,
    SQUARE_CACHE: {
      get: async () => session,
    } as unknown as KVNamespace,
  } as Env;
}
