import { describe, expect, it } from "vitest";
import {
  CHAIN_CLOCK_MAX_STALENESS_MS,
  isSubscriptionMirrorEffective,
  subscriptionIsActive,
} from "../src/membership/service";
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

function membershipRow(overrides: Partial<MembershipRow> = {}): MembershipRow {
  return {
    owner_account: "owner",
    membership_level: "freedom",
    pending_membership_level: null,
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
