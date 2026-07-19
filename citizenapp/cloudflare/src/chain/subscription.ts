import type { Env } from "../types";
import { bytesToHex, hexToBytes } from "../shared/signing_message";
import { concat, decodeOwnerAccount, storageMapKey } from "./storage_key";
import { fetchFinalizedChainStorage } from "./rpc";

/// Cloudflare 只严格解码 finalized `Subscriptions`。真实公历和自动扣款都由 runtime
/// 根据共识时间戳完成；Worker 不计算日期、不提交续费。

export type SubscriptionIssuer =
  | { kind: "platform" }
  | { kind: "creator"; creatorAccount: string };

export type SubscriptionStatus = "active" | "cancelled" | "terminated";

export type PlatformLevel = "freedom" | "democracy" | "spark";
export type BillingPeriod = "monthly" | "quarterly" | "yearly";

export type ChainSubscriptionPlan =
  | { kind: "platform"; membershipLevel: PlatformLevel }
  | { kind: "creator"; tierId: string; billingPeriod: BillingPeriod };

export interface ChainSubscriptionState {
  plan: ChainSubscriptionPlan;
  pendingPlan: ChainSubscriptionPlan | null;
  startedAt: number;
  lastChargedAt: number;
  lastChargedPriceFen: bigint;
  paidUntil: number;
  status: SubscriptionStatus;
}

export interface ChainCreatorTier {
  tierId: string;
  pricesFen: Partial<Record<BillingPeriod, bigint>>;
}

const PLATFORM_ISSUER_TAG = 0x00;
const CREATOR_ISSUER_TAG = 0x01;
const PLAN_PLATFORM_TAG = 0x00;
const PLAN_CREATOR_TAG = 0x01;

const LEVEL_BY_BYTE: Record<number, PlatformLevel> = {
  0: "freedom",
  1: "democracy",
  2: "spark",
};

const PERIOD_BY_BYTE: Record<number, BillingPeriod> = {
  0: "monthly",
  1: "quarterly",
  2: "yearly",
};

const STATUS_BY_BYTE: Record<number, SubscriptionStatus> = {
  0: "active",
  1: "cancelled",
  2: "terminated",
};

function encodeIssuerKey(issuer: SubscriptionIssuer): Uint8Array {
  if (issuer.kind === "platform") {
    return new Uint8Array([PLATFORM_ISSUER_TAG]);
  }
  return concat([
    new Uint8Array([CREATOR_ISSUER_TAG]),
    decodeOwnerAccount(issuer.creatorAccount),
  ]);
}

export function buildSubscriptionKey(
  subscriberAccount: string,
  issuer: SubscriptionIssuer,
): Uint8Array {
  return storageMapKey(
    "SquarePost",
    "Subscriptions",
    concat([decodeOwnerAccount(subscriberAccount), encodeIssuerKey(issuer)]),
  );
}

export function buildCreatorPlansKey(creatorAccount: string): Uint8Array {
  return storageMapKey(
    "SquarePost",
    "CreatorPlans",
    decodeOwnerAccount(creatorAccount),
  );
}

function readU128Le(data: Uint8Array, offset: number): bigint {
  let value = 0n;
  for (let index = 15; index >= 0; index--) {
    value = (value << 8n) | BigInt(data[offset + index]);
  }
  return value;
}

function readU64Le(data: Uint8Array, offset: number): number {
  const value = new DataView(
    data.buffer,
    data.byteOffset + offset,
    8,
  ).getBigUint64(0, true);
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new RangeError("u64 timestamp exceeds JavaScript safe integer");
  }
  return Number(value);
}

function readCompactBytes(
  data: Uint8Array,
  offset: number,
): { value: Uint8Array; offset: number } {
  if (offset >= data.length) throw new RangeError("missing compact length");
  const first = data[offset];
  if ((first & 0x03) !== 0) {
    throw new RangeError("tier_id only accepts one-byte SCALE compact length");
  }
  const length = first >> 2;
  const start = offset + 1;
  const end = start + length;
  if (length === 0 || length > 32 || end > data.length) {
    throw new RangeError("invalid tier_id length");
  }
  return { value: data.slice(start, end), offset: end };
}

function readPlan(
  data: Uint8Array,
  initialOffset: number,
): { value: ChainSubscriptionPlan; offset: number } {
  let offset = initialOffset;
  if (offset >= data.length) throw new RangeError("missing plan tag");
  const tag = data[offset++];
  if (tag === PLAN_PLATFORM_TAG) {
    const membershipLevel = LEVEL_BY_BYTE[data[offset++]];
    if (!membershipLevel) throw new RangeError("invalid membership level");
    return { value: { kind: "platform", membershipLevel }, offset };
  }
  if (tag === PLAN_CREATOR_TAG) {
    const tier = readCompactBytes(data, offset);
    offset = tier.offset;
    const billingPeriod = PERIOD_BY_BYTE[data[offset++]];
    if (!billingPeriod) throw new RangeError("invalid billing period");
    const tierId = new TextDecoder("utf-8", { fatal: true }).decode(tier.value);
    return { value: { kind: "creator", tierId, billingPeriod }, offset };
  }
  throw new RangeError("invalid plan tag");
}

/// 严格解码目标 `SubscriptionState`；非法枚举、截断或尾随字节一律抛错，禁止伪装成无订阅。
export function decodeSubscriptionState(
  data: Uint8Array,
): ChainSubscriptionState {
  let decoded = readPlan(data, 0);
  const plan = decoded.value;
  let offset = decoded.offset;

  if (offset >= data.length) throw new RangeError("missing pending plan tag");
  const pendingTag = data[offset++];
  let pendingPlan: ChainSubscriptionPlan | null = null;
  if (pendingTag === 1) {
    decoded = readPlan(data, offset);
    pendingPlan = decoded.value;
    offset = decoded.offset;
  } else if (pendingTag !== 0) {
    throw new RangeError("invalid pending plan tag");
  }

  if (offset + 8 + 8 + 16 + 8 + 1 > data.length) {
    throw new RangeError("subscription state truncated");
  }
  const startedAt = readU64Le(data, offset);
  offset += 8;
  const lastChargedAt = readU64Le(data, offset);
  offset += 8;
  const lastChargedPriceFen = readU128Le(data, offset);
  offset += 16;

  const paidUntil = readU64Le(data, offset);
  offset += 8;

  const status = STATUS_BY_BYTE[data[offset++]];
  if (!status) throw new RangeError("invalid subscription status");
  if (offset !== data.length) throw new RangeError("subscription state has trailing bytes");
  if (paidUntil <= lastChargedAt) {
    throw new RangeError("paid_until must be after last_charged_at");
  }
  return {
    plan,
    pendingPlan,
    startedAt,
    lastChargedAt,
    lastChargedPriceFen,
    paidUntil,
    status,
  };
}

/// 严格解码 finalized `CreatorPlans`；名称等展示字段不属于该结构。
export function decodeCreatorPlans(data: Uint8Array): ChainCreatorTier[] {
  let offset = 0;
  const tierCount = readCompactCount(data, offset, 10, true);
  offset = tierCount.offset;
  const result: ChainCreatorTier[] = [];
  const tierIds = new Set<string>();
  for (let tierIndex = 0; tierIndex < tierCount.value; tierIndex++) {
    const tier = readCompactBytes(data, offset);
    offset = tier.offset;
    const tierId = new TextDecoder("utf-8", { fatal: true }).decode(tier.value);
    if (tierIds.has(tierId)) throw new RangeError("duplicate creator tier_id");
    tierIds.add(tierId);

    const priceCount = readCompactCount(data, offset, 3, false);
    offset = priceCount.offset;
    const pricesFen: Partial<Record<BillingPeriod, bigint>> = {};
    for (let priceIndex = 0; priceIndex < priceCount.value; priceIndex++) {
      if (offset >= data.length) throw new RangeError("missing billing period");
      const period = PERIOD_BY_BYTE[data[offset++]];
      if (!period || pricesFen[period] !== undefined) {
        throw new RangeError("invalid or duplicate billing period");
      }
      if (offset + 16 > data.length) throw new RangeError("creator price truncated");
      const price = readU128Le(data, offset);
      offset += 16;
      if (price <= 0n) throw new RangeError("creator price must be positive");
      pricesFen[period] = price;
    }
    result.push({ tierId, pricesFen });
  }
  if (offset !== data.length) throw new RangeError("creator plans have trailing bytes");
  return result;
}

function readCompactCount(
  data: Uint8Array,
  offset: number,
  max: number,
  allowZero: boolean,
): { value: number; offset: number } {
  if (offset >= data.length) throw new RangeError("missing compact count");
  const first = data[offset];
  if ((first & 0x03) !== 0) throw new RangeError("unsupported compact count");
  const value = first >> 2;
  if ((!allowZero && value === 0) || value > max) {
    throw new RangeError("invalid compact count");
  }
  return { value, offset: offset + 1 };
}

export async function readSubscription(
  env: Env,
  subscriberAccount: string,
  issuer: SubscriptionIssuer,
): Promise<ChainSubscriptionState | null> {
  const key = buildSubscriptionKey(subscriberAccount, issuer);
  const hex = await fetchFinalizedChainStorage(env, `0x${bytesToHex(key)}`);
  return hex ? decodeSubscriptionState(hexToBytes(hex)) : null;
}

export function readPlatformSubscription(
  env: Env,
  subscriberAccount: string,
): Promise<ChainSubscriptionState | null> {
  return readSubscription(env, subscriberAccount, { kind: "platform" });
}

export async function readCreatorPlans(
  env: Env,
  creatorAccount: string,
): Promise<ChainCreatorTier[]> {
  const key = buildCreatorPlansKey(creatorAccount);
  const hex = await fetchFinalizedChainStorage(env, `0x${bytesToHex(key)}`);
  return hex ? decodeCreatorPlans(hexToBytes(hex)) : [];
}
