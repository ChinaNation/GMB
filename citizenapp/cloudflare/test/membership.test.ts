import { describe, expect, it, vi } from "vitest";
import {
  CHAIN_CLOCK_MAX_STALENESS_MS,
  getMembershipForAuthorization,
  isSubscriptionMirrorEffective,
  requireActiveMembership,
  subscriptionIsActive,
} from "../src/membership/service";
import type { MembershipAuthorizationDeps } from "../src/membership/service";
import type { ChainSubscriptionState } from "../src/chain/subscription";
import type { Env } from "../src/types";
import type { MembershipRow } from "../src/types";

const NOW = 2_000_000;

describe("平台与创作者统一订阅门禁", () => {
  it("Active 且链时间早于 paid_until 时放行", () => {
    expect(subscriptionIsActive(membershipRow(), NOW)).toBe(true);
  });

  it("Cancelled 在已付周期内继续放行，到期后立即拒绝", () => {
    expect(subscriptionIsActive(membershipRow({ subscription_status: "cancelled" }), NOW)).toBe(true);
    expect(subscriptionIsActive(membershipRow({
      subscription_status: "cancelled",
      paid_until: 1_999_999,
      chain_timestamp: 2_000_000,
    }), NOW)).toBe(false);
  });

  it("Terminated 无论 paid_until 是否在未来都拒绝", () => {
    expect(subscriptionIsActive(membershipRow({ subscription_status: "terminated" }), NOW)).toBe(false);
  });

  it("无链时钟、未来观测值或时钟陈旧都 fail-closed", () => {
    expect(subscriptionIsActive(membershipRow({ chain_timestamp: null }), NOW)).toBe(false);
    expect(subscriptionIsActive(membershipRow({ chain_observed_at: NOW + 1 }), NOW)).toBe(false);
    expect(subscriptionIsActive(membershipRow({
      chain_observed_at: NOW - CHAIN_CLOCK_MAX_STALENESS_MS - 1,
    }), NOW)).toBe(false);
  });

  it("创作者关系复用同一有效口径", () => {
    expect(isSubscriptionMirrorEffective({
      subscription_status: "cancelled",
      paid_until: 2_100_000,
      chain_timestamp: 2_000_000,
      chain_observed_at: NOW,
    }, NOW)).toBe(true);
  });
});

describe("发布授权拒绝前 finalized 复核", () => {
  it("D1 快路径有效时不读取链", async () => {
    const current = currentMembershipRow();
    const reconcile = vi.fn(async () => null);

    const result = await getMembershipForAuthorization(
      membershipEnv(() => current),
      current.cid_number,
      current.account_id,
      authorizationDeps(reconcile),
    );

    expect(result).toBe(current);
    expect(reconcile).not.toHaveBeenCalled();
  });

  it("镜像缺失时按当前 CID 复核并返回补建后的有效会员", async () => {
    const refreshed = currentMembershipRow({ membership_level: "spark" });
    let current: MembershipRow | null = null;
    const reconcile = vi.fn(async () => {
      current = refreshed;
      return activeChainMembership("spark");
    });

    const result = await getMembershipForAuthorization(
      membershipEnv(() => current),
      refreshed.cid_number,
      refreshed.account_id,
      authorizationDeps(reconcile),
    );

    expect(reconcile).toHaveBeenCalledTimes(1);
    expect(result).toBe(refreshed);
  });

  it("链服务异常返回可重试 503，不伪装成没有会员", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const reconcile = vi.fn(async () => {
      throw new Error("chain unavailable");
    });

    await expect(getMembershipForAuthorization(
      membershipEnv(() => null),
      "CN220-CTZN2-198805200-2026",
      `0x${"9".repeat(64)}`,
      authorizationDeps(reconcile),
    )).rejects.toMatchObject({
      status: 503,
      code: "membership_verification_unavailable",
    });
    consoleError.mockRestore();
  });

  it("finalized 链确认无会员后才返回 402 membership_required", async () => {
    const reconcile = vi.fn(async () => null);

    await expect(requireActiveMembership(
      membershipEnv(() => null),
      "CN220-CTZN2-198805200-2026",
      `0x${"9".repeat(64)}`,
      authorizationDeps(reconcile),
    )).rejects.toMatchObject({
      status: 402,
      code: "membership_required",
    });
    expect(reconcile).toHaveBeenCalledTimes(1);
  });

  it("链确认可能有效但镜像仍无法放行时返回 503", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const reconcile = vi.fn(async () => activeChainMembership("freedom"));

    await expect(getMembershipForAuthorization(
      membershipEnv(() => null),
      "CN220-CTZN2-198805200-2026",
      `0x${"9".repeat(64)}`,
      authorizationDeps(reconcile),
    )).rejects.toMatchObject({
      status: 503,
      code: "membership_verification_unavailable",
    });
    consoleError.mockRestore();
  });
});

function authorizationDeps(
  reconcileMembershipForCid: MembershipAuthorizationDeps["reconcileMembershipForCid"],
): MembershipAuthorizationDeps {
  return { reconcileMembershipForCid };
}

function membershipEnv(read: () => MembershipRow | null): Env {
  const statement = {
    bind() {
      return statement;
    },
    async first<T>() {
      return read() as T | null;
    },
  };
  return {
    DB: {
      prepare: () => statement,
    },
  } as unknown as Env;
}

function currentMembershipRow(
  overrides: Partial<MembershipRow> = {},
): MembershipRow {
  const now = Date.now();
  return membershipRow({
    chain_timestamp: now,
    chain_observed_at: now,
    paid_until: now + 60_000,
    verified_at: now,
    ...overrides,
  });
}

function activeChainMembership(
  membershipLevel: "freedom" | "democracy" | "spark",
): ChainSubscriptionState {
  return {
    plan: { kind: "platform", membershipLevel },
    startedAt: 1,
    lastChargedAt: 2,
    lastChargedPriceFen: 100n,
    paidUntil: Date.now() + 60_000,
    status: "active",
    authorizedPriceFen: 100n,
    suspendReason: null,
  };
}

function membershipRow(overrides: Partial<MembershipRow> = {}): MembershipRow {
  return {
    cid_number: "CN220-CTZN2-198805200-2026",
    account_id: "0x9999999999999999999999999999999999999999999999999999999999999999",
    membership_level: "freedom",
    started_at: 1_000_000,
    last_charged_at: 1_000_000,
    last_charged_price_fen: 100,
    paid_until: 2_100_000,
    subscription_status: "active",
    finalized_block_number: 10,
    finalized_block_hash: `0x${"1".repeat(64)}`,
    verified_at: NOW,
    entitlement_lapsed_at: null,
    last_tx_hash: `0x${"2".repeat(64)}`,
    chain_timestamp: 2_000_000,
    chain_observed_at: NOW,
    ...overrides,
  };
}
