import { describe, expect, it, vi } from "vitest";

// 链读桩：只为验证「缓存读到无 CID 时是否回源」这条时序，不打真链。
const chainCidHex = vi.fn<() => Promise<string | null>>();
vi.mock("../src/chain/rpc", () => ({
  fetchFinalizedHead: vi.fn(async () => "0x" + "11".repeat(32)),
  fetchChainStorage: vi.fn(async () => chainCidHex()),
}));

import {
  cidRecordIsActive,
  fetchChainIdentityStateFreshIfUnbound,
  decodeCandidateIdentity,
  decodeCidNumber,
  decodeVotingIdentity,
  encodeBoundedBytes,
  votingIdentityIsActive,
} from "../src/chain/identity";

const utf8 = new TextEncoder();

describe("citizen-identity 永久 CID 闭环解码", () => {
  it("CID 值只接受唯一完整的 BoundedVec 编码", () => {
    const cid = "GD-CTZN1-8F3A2B";
    const encoded = encodeBoundedBytes(utf8.encode(cid));
    expect(decodeCidNumber(encoded)).toBe(cid);
    expect(decodeCidNumber(Uint8Array.from([...encoded, 0]))).toBeNull();
  });

  it("CidRegistry 只接受 Active 状态", () => {
    expect(cidRecordIsActive(cidRecord(0))).toBe(true);
    expect(cidRecordIsActive(cidRecord(1))).toBe(false);
    expect(cidRecordIsActive(cidRecord(0, true))).toBe(false);
    expect(cidRecordIsActive(null)).toBe(false);
  });

  it("VotingIdentityByCid 不再重复保存 CID，并按 UTC+8 有效期判定", () => {
    const active = decodeVotingIdentity(votingIdentity(0));
    const revoked = decodeVotingIdentity(votingIdentity(1));
    expect(active).not.toBeNull();
    expect(
      votingIdentityIsActive(active!, new Date("2026-07-21T16:30:00Z")),
    ).toBe(true);
    expect(
      votingIdentityIsActive(active!, new Date("2040-01-01T00:00:00Z")),
    ).toBe(false);
    expect(
      votingIdentityIsActive(revoked!, new Date("2026-07-22T00:00:00Z")),
    ).toBe(false);
    expect(decodeVotingIdentity(votingIdentity(0).slice(0, 9))).toBeNull();
  });

  it("竞选身份必须是姓、名和出生日期完整的最终布局", () => {
    expect(decodeCandidateIdentity(candidateIdentity())).not.toBeNull();
    expect(
      decodeCandidateIdentity(candidateIdentity({ familyName: "" })),
    ).toBeNull();
    expect(
      decodeCandidateIdentity(candidateIdentity().slice(0, -1)),
    ).toBeNull();
  });
});

function bounded(value: string): number[] {
  const bytes = [...utf8.encode(value)];
  return [bytes.length << 2, ...bytes];
}

function u32(value: number): number[] {
  return [
    value & 0xff,
    (value >> 8) & 0xff,
    (value >> 16) & 0xff,
    (value >> 24) & 0xff,
  ];
}

function cidRecord(status: number, revokedAt = false): Uint8Array {
  return Uint8Array.from([
    ...bounded("FEDERAL_REGISTRY-CID"),
    ...new Array(32).fill(7),
    ...bounded("GD"),
    ...bounded("0755"),
    status,
    ...u32(1),
    revokedAt ? 1 : 0,
    ...(revokedAt ? u32(2) : []),
  ]);
}

function votingIdentity(status: number): Uint8Array {
  return Uint8Array.from([
    ...u32(20260101),
    ...u32(20310101),
    status,
    ...bounded("GD"),
    ...bounded("0755"),
    ...bounded("001"),
    ...u32(1),
  ]);
}

function candidateIdentity(options: { familyName?: string } = {}): Uint8Array {
  return Uint8Array.from([
    ...bounded("GD"),
    ...bounded("0755"),
    ...bounded("001"),
    ...bounded(options.familyName ?? "陈"),
    ...bounded("明"),
    0,
    ...u32(20000131),
    ...u32(1),
  ]);
}

describe("身份缓存旁路（子钥懒绑定时序）", () => {
  /// 缓存里没有 CID 就必须回源核实一次。
  ///
  /// 真实时序：用户占号 finalized 后几秒就进广场触发子钥绑定，而身份缓存 45 秒，
  /// 缓存里还留着占号前的空值。若直接采信，用户会被自己刚上链的身份挡在门外。
  it("缓存无 CID 时回源链读", async () => {
    chainCidHex.mockClear();
    chainCidHex.mockResolvedValue(null);
    const env = envWithCachedIdentity({ cid_number: null });
    await fetchChainIdentityStateFreshIfUnbound(env, ACCOUNT_ID);
    expect(chainCidHex).toHaveBeenCalled();
  });

  /// 正面结论不会凭空消失，继续走缓存，不为一次判定多打一次链。
  it("缓存已有 CID 时直接采信，不再回源", async () => {
    chainCidHex.mockClear();
    const env = envWithCachedIdentity({ cid_number: "GD-CTZN1-CACHED" });
    const state = await fetchChainIdentityStateFreshIfUnbound(env, ACCOUNT_ID);
    expect(state.cid_number).toBe("GD-CTZN1-CACHED");
    expect(chainCidHex).not.toHaveBeenCalled();
  });
});

const ACCOUNT_ID = `0x${"77".repeat(32)}`;

function envWithCachedIdentity(patch: { cid_number: string | null }): never {
  const cached = JSON.stringify({
    account_id: ACCOUNT_ID,
    identity_level: "visitor",
    has_voting_identity: false,
    has_candidate_identity: false,
    checked_at: 0,
    ...patch,
  });
  return {
    SQUARE_CACHE: {
      get: async () => cached,
      put: async () => {},
    },
  } as never;
}
