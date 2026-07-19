import type { Env } from "../types";
import { blake2AsU8a } from "@polkadot/util-crypto";
import { HttpError } from "../shared/http";
import { resourceLimit } from "../limits/catalog";
import { bytesToHex, hexToBytes } from "../shared/signing_message";
import {
  concat,
  decodeOwnerAccount,
  storageMapKey,
  storageValueKey,
} from "./storage_key";
import {
  fetchBlockHeader,
  fetchCanonicalBlockHash,
  fetchChainStorage,
  fetchFinalizedChainStorage,
  fetchFinalizedHead,
  fetchSignedBlock,
} from "./rpc";

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

export type SubscriptionBusinessAction =
  | { kind: "platform_subscribe"; membershipLevel: PlatformLevel }
  | { kind: "platform_cancel" }
  | { kind: "platform_change"; membershipLevel: PlatformLevel }
  | {
      kind: "creator_subscribe";
      creatorAccount: string;
      tierId: string;
      billingPeriod: BillingPeriod;
    }
  | { kind: "creator_cancel"; creatorAccount: string }
  | {
      kind: "creator_change";
      creatorAccount: string;
      tierId: string;
      billingPeriod: BillingPeriod;
    }
  | { kind: "creator_plans_set"; tiers: ChainCreatorTier[] };

export interface FinalizedTransactionProofInput {
  txHash: string;
  blockHash: string;
  signedExtrinsicHex: string;
}

export interface VerifiedFinalizedTransaction {
  txHash: string;
  blockHash: string;
  blockNumber: number;
  extrinsicIndex: number;
  chainTimestamp: number;
  action: SubscriptionBusinessAction;
}

interface TransactionConfirmationRow {
  owner_account: string;
  block_hash: string;
  block_number: number;
  extrinsic_index: number;
  action_kind: string;
  request_hash: string;
  chain_timestamp: number;
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

/** 在指定 finalized 区块读取订阅，确保交易证明、状态和链时间属于同一状态快照。 */
export async function readSubscriptionAtBlock(
  env: Env,
  subscriberAccount: string,
  issuer: SubscriptionIssuer,
  blockHash: string,
): Promise<ChainSubscriptionState | null> {
  const key = buildSubscriptionKey(subscriberAccount, issuer);
  const hex = await fetchChainStorage(env, `0x${bytesToHex(key)}`, blockHash);
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


/** 在指定 finalized 区块读取创作者付款档位，展示字段不得进入该链上结构。 */
export async function readCreatorPlansAtBlock(
  env: Env,
  creatorAccount: string,
  blockHash: string,
): Promise<ChainCreatorTier[]> {
  const key = buildCreatorPlansKey(creatorAccount);
  const hex = await fetchChainStorage(env, `0x${bytesToHex(key)}`, blockHash);
  return hex ? decodeCreatorPlans(hexToBytes(hex)) : [];
}

/** 读取指定区块的共识时间戳；Worker 只比较链上毫秒值，不复制任何公历算法。 */
export async function readChainTimestampAtBlock(
  env: Env,
  blockHash: string,
): Promise<number> {
  const key = storageValueKey("Timestamp", "Now");
  const hex = await fetchChainStorage(env, `0x${bytesToHex(key)}`, blockHash);
  if (!hex) {
    throw new HttpError(502, "chain_timestamp_missing", "最终区块缺少链上时间戳");
  }
  const data = hexToBytes(hex);
  if (data.length !== 8) {
    throw new HttpError(502, "chain_timestamp_invalid", "链上时间戳编码不合法");
  }
  return readU64Le(data, 0);
}

/**
 * 证明 App 提交的完整 signed extrinsic 确实位于指定 finalized 主链区块，并且签名账户与
 * Bearer 会话账户一致、SquarePost 调用及参数与本次镜像动作一致。链已验证交易签名；Worker
 * 不重复验签，也不保存完整交易字节，只保存不可变哈希与 finalized 定位，降低 D1 占用。
 */
export async function verifyFinalizedSubscriptionTransaction(
  env: Env,
  ownerAccount: string,
  expectedAction: SubscriptionBusinessAction,
  proof: FinalizedTransactionProofInput,
): Promise<VerifiedFinalizedTransaction> {
  const txHash = normalizeHash(proof.txHash, "交易哈希");
  const blockHash = normalizeHash(proof.blockHash, "区块哈希");
  const signedExtrinsicHex = normalizeExtrinsicHex(proof.signedExtrinsicHex);
  const encoded = hexToBytes(signedExtrinsicHex);
  if (encoded.length > resourceLimit("chain_extrinsic").max_bytes) {
    throw new HttpError(413, "signed_extrinsic_too_large", "订阅交易超过大小限制");
  }
  const calculatedTxHash = `0x${bytesToHex(blake2AsU8a(encoded, 256))}`;
  if (calculatedTxHash !== txHash) {
    throw new HttpError(409, "subscription_tx_hash_mismatch", "交易哈希与签名交易不一致");
  }

  const decoded = decodeSignedSubscriptionExtrinsic(encoded);
  if (!equalBytes(decoded.signerAccount, decodeOwnerAccount(ownerAccount))) {
    throw new HttpError(403, "subscription_tx_owner_mismatch", "交易签名账户与登录钱包不一致");
  }
  if (!businessActionsEqual(decoded.action, expectedAction)) {
    throw new HttpError(409, "subscription_tx_action_mismatch", "链上交易与镜像业务操作不一致");
  }

  const [finalizedHead, signedBlock] = await Promise.all([
    fetchFinalizedHead(env),
    fetchSignedBlock(env, blockHash),
  ]);
  const [finalizedHeader, canonicalBlockHash] = await Promise.all([
    fetchBlockHeader(env, finalizedHead),
    fetchCanonicalBlockHash(env, parseBlockNumber(signedBlock.block.header.number)),
  ]);
  const blockNumber = parseBlockNumber(signedBlock.block.header.number);
  const finalizedNumber = parseBlockNumber(finalizedHeader.number);
  if (blockNumber > finalizedNumber || canonicalBlockHash !== blockHash) {
    throw new HttpError(409, "subscription_block_not_finalized", "交易区块尚未成为 finalized 主链区块");
  }
  const normalizedExtrinsics = signedBlock.block.extrinsics.map((value) =>
    normalizeExtrinsicHex(value),
  );
  const extrinsicIndex = normalizedExtrinsics.indexOf(signedExtrinsicHex);
  if (extrinsicIndex < 0) {
    throw new HttpError(409, "subscription_tx_not_in_block", "指定区块不包含该订阅交易");
  }
  const chainTimestamp = await readChainTimestampAtBlock(env, blockHash);
  return {
    txHash,
    blockHash,
    blockNumber,
    extrinsicIndex,
    chainTimestamp,
    action: decoded.action,
  };
}

/**
 * 把 tx_hash 首次绑定到唯一账户、业务动作和规范化请求。相同请求可无限 HTTP 重试，
 * 同一链上交易不得改绑另一组展示名或另一项订阅关系。
 */
export async function bindFinalizedTransactionConfirmation(
  env: Env,
  ownerAccount: string,
  transaction: VerifiedFinalizedTransaction,
  requestHash: string,
  confirmedAt: number,
): Promise<void> {
  if (!/^[0-9a-f]{64}$/.test(requestHash)) {
    throw new HttpError(500, "invalid_request_hash", "镜像请求摘要不合法");
  }
  await env.DB.prepare(
    `INSERT OR IGNORE INTO chain_transaction_confirmations
      (tx_hash, owner_account, block_hash, block_number, extrinsic_index, action_kind,
       request_hash, chain_timestamp, confirmed_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
  )
    .bind(
      transaction.txHash,
      ownerAccount,
      transaction.blockHash,
      transaction.blockNumber,
      transaction.extrinsicIndex,
      transaction.action.kind,
      requestHash,
      transaction.chainTimestamp,
      confirmedAt,
    )
    .run();
  const saved = await env.DB.prepare(
    `SELECT owner_account, block_hash, block_number, extrinsic_index, action_kind,
        request_hash, chain_timestamp
      FROM chain_transaction_confirmations WHERE tx_hash = ?`,
  )
    .bind(transaction.txHash)
    .first<TransactionConfirmationRow>();
  if (
    !saved ||
    saved.owner_account !== ownerAccount ||
    saved.block_hash !== transaction.blockHash ||
    saved.block_number !== transaction.blockNumber ||
    saved.extrinsic_index !== transaction.extrinsicIndex ||
    saved.action_kind !== transaction.action.kind ||
    saved.request_hash !== requestHash ||
    saved.chain_timestamp !== transaction.chainTimestamp
  ) {
    throw new HttpError(409, "subscription_tx_already_bound", "该链上交易已绑定另一业务请求");
  }
}

/** 更新全局 finalized 链时间；旧交易的延迟镜像不得把时钟回退。 */
export async function updateChainClock(
  env: Env,
  input: {
    chainTimestamp: number;
    blockNumber: number;
    blockHash: string;
    observedAt: number;
  },
): Promise<void> {
  await env.DB.prepare(
    `INSERT INTO chain_clock
      (clock_id, chain_timestamp, finalized_block_number, finalized_block_hash, observed_at)
      VALUES (1, ?, ?, ?, ?)
      ON CONFLICT(clock_id) DO UPDATE SET
        chain_timestamp = excluded.chain_timestamp,
        finalized_block_number = excluded.finalized_block_number,
        finalized_block_hash = excluded.finalized_block_hash,
        observed_at = excluded.observed_at
      WHERE excluded.finalized_block_number > chain_clock.finalized_block_number`,
  )
    .bind(input.chainTimestamp, input.blockNumber, input.blockHash, input.observedAt)
    .run();
}

function decodeSignedSubscriptionExtrinsic(encoded: Uint8Array): {
  signerAccount: Uint8Array;
  action: SubscriptionBusinessAction;
} {
  const outerLength = readCompactUnsigned(encoded, 0);
  if (outerLength.value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new HttpError(400, "invalid_signed_extrinsic", "签名交易长度不合法");
  }
  if (outerLength.offset + Number(outerLength.value) !== encoded.length) {
    throw new HttpError(400, "invalid_signed_extrinsic", "签名交易长度与内容不一致");
  }
  let offset = outerLength.offset;
  if (encoded[offset++] !== 0x84 || encoded[offset++] !== 0x00) {
    throw new HttpError(400, "invalid_signed_extrinsic", "只接受账户签名的目标版本交易");
  }
  const signerAccount = sliceExact(encoded, offset, 32);
  offset += 32;
  if (encoded[offset++] !== 0x01) {
    throw new HttpError(400, "invalid_signed_extrinsic", "订阅交易签名类型不合法");
  }
  sliceExact(encoded, offset, 64);
  offset += 64;
  if (encoded[offset++] !== 0x00) {
    throw new HttpError(400, "invalid_signed_extrinsic", "订阅交易必须使用统一 immortal era");
  }
  offset = readCompactUnsigned(encoded, offset).offset;
  offset = readCompactUnsigned(encoded, offset).offset;
  const decodedCall = decodeSubscriptionCall(encoded, offset);
  if (decodedCall.offset !== encoded.length) {
    throw new HttpError(400, "invalid_signed_extrinsic", "订阅交易含有尾随字节");
  }
  return { signerAccount, action: decodedCall.action };
}

function decodeSubscriptionCall(
  data: Uint8Array,
  initialOffset: number,
): { action: SubscriptionBusinessAction; offset: number } {
  let offset = initialOffset;
  if (data[offset++] !== 34) {
    throw new HttpError(400, "invalid_subscription_call", "交易不是 SquarePost 订阅调用");
  }
  const callIndex = data[offset++];
  if (callIndex === 3) {
    const decoded = decodeCreatorTierVector(data, offset);
    return { action: { kind: "creator_plans_set", tiers: decoded.tiers }, offset: decoded.offset };
  }
  if (callIndex !== 1 && callIndex !== 2 && callIndex !== 4) {
    throw new HttpError(400, "invalid_subscription_call", "交易不是允许的订阅业务操作");
  }
  const issuer = decodeCallIssuer(data, offset);
  offset = issuer.offset;
  if (callIndex === 2) {
    return issuer.kind === "platform"
      ? { action: { kind: "platform_cancel" }, offset }
      : {
          action: { kind: "creator_cancel", creatorAccount: issuer.creatorAccount },
          offset,
        };
  }
  const decodedPlan = decodeCallPlan(data, offset);
  offset = decodedPlan.offset;
  sliceExact(data, offset, 16);
  offset += 16;
  if (issuer.kind === "platform" && decodedPlan.kind === "platform") {
    return {
      action: {
        kind: callIndex === 1 ? "platform_subscribe" : "platform_change",
        membershipLevel: decodedPlan.membershipLevel,
      },
      offset,
    };
  }
  if (issuer.kind === "creator" && decodedPlan.kind === "creator") {
    return {
      action: {
        kind: callIndex === 1 ? "creator_subscribe" : "creator_change",
        creatorAccount: issuer.creatorAccount,
        tierId: decodedPlan.tierId,
        billingPeriod: decodedPlan.billingPeriod,
      },
      offset,
    };
  }
  throw new HttpError(400, "invalid_subscription_call", "收款主体与订阅计划类型不一致");
}

function decodeCallIssuer(
  data: Uint8Array,
  offset: number,
):
  | { kind: "platform"; offset: number }
  | { kind: "creator"; creatorAccount: string; offset: number } {
  const tag = data[offset++];
  if (tag === PLATFORM_ISSUER_TAG) return { kind: "platform", offset };
  if (tag !== CREATOR_ISSUER_TAG) {
    throw new HttpError(400, "invalid_subscription_call", "订阅收款主体不合法");
  }
  const account = sliceExact(data, offset, 32);
  return { kind: "creator", creatorAccount: `0x${bytesToHex(account)}`, offset: offset + 32 };
}

function decodeCallPlan(
  data: Uint8Array,
  offset: number,
):
  | { kind: "platform"; membershipLevel: PlatformLevel; offset: number }
  | { kind: "creator"; tierId: string; billingPeriod: BillingPeriod; offset: number } {
  const tag = data[offset++];
  if (tag === PLAN_PLATFORM_TAG) {
    const membershipLevel = LEVEL_BY_BYTE[data[offset++]];
    if (!membershipLevel) throw new HttpError(400, "invalid_subscription_call", "平台档位不合法");
    return { kind: "platform", membershipLevel, offset };
  }
  if (tag !== PLAN_CREATOR_TAG) {
    throw new HttpError(400, "invalid_subscription_call", "订阅计划不合法");
  }
  const tier = readScaleBytes(data, offset);
  offset = tier.offset;
  const billingPeriod = PERIOD_BY_BYTE[data[offset++]];
  if (!billingPeriod) throw new HttpError(400, "invalid_subscription_call", "订阅周期不合法");
  return { kind: "creator", tierId: strictUtf8(tier.value), billingPeriod, offset };
}

function decodeCreatorTierVector(
  data: Uint8Array,
  initialOffset: number,
): { tiers: ChainCreatorTier[]; offset: number } {
  let offset = initialOffset;
  const count = readCompactUnsigned(data, offset);
  offset = count.offset;
  const tierCount = Number(count.value);
  if (!Number.isSafeInteger(tierCount) || tierCount > 10) {
    throw new HttpError(400, "invalid_subscription_call", "创作者档位数量不合法");
  }
  const tiers: ChainCreatorTier[] = [];
  const tierIds = new Set<string>();
  for (let index = 0; index < tierCount; index += 1) {
    const tier = readScaleBytes(data, offset);
    offset = tier.offset;
    const tierId = strictUtf8(tier.value);
    if (!tierId || tierIds.has(tierId)) {
      throw new HttpError(400, "invalid_subscription_call", "创作者档位标识不合法");
    }
    tierIds.add(tierId);
    const priceCountValue = readCompactUnsigned(data, offset);
    offset = priceCountValue.offset;
    const priceCount = Number(priceCountValue.value);
    if (!Number.isSafeInteger(priceCount) || priceCount < 1 || priceCount > 3) {
      throw new HttpError(400, "invalid_subscription_call", "创作者周期价格数量不合法");
    }
    const pricesFen: Partial<Record<BillingPeriod, bigint>> = {};
    for (let priceIndex = 0; priceIndex < priceCount; priceIndex += 1) {
      const period = PERIOD_BY_BYTE[data[offset++]];
      if (!period || pricesFen[period] !== undefined) {
        throw new HttpError(400, "invalid_subscription_call", "创作者周期价格不合法");
      }
      sliceExact(data, offset, 16);
      pricesFen[period] = readU128Le(data, offset);
      offset += 16;
    }
    tiers.push({ tierId, pricesFen });
  }
  return { tiers, offset };
}

function readScaleBytes(
  data: Uint8Array,
  offset: number,
): { value: Uint8Array; offset: number } {
  const length = readCompactUnsigned(data, offset);
  if (length.value > 32n) {
    throw new HttpError(400, "invalid_subscription_call", "订阅字段长度不合法");
  }
  const size = Number(length.value);
  return { value: sliceExact(data, length.offset, size), offset: length.offset + size };
}

function readCompactUnsigned(
  data: Uint8Array,
  offset: number,
): { value: bigint; offset: number } {
  if (offset >= data.length) throw new HttpError(400, "invalid_scale_compact", "SCALE compact 缺失");
  const first = data[offset];
  const mode = first & 0x03;
  if (mode === 0) return { value: BigInt(first >> 2), offset: offset + 1 };
  if (mode === 1) {
    sliceExact(data, offset, 2);
    return {
      value: BigInt(((data[offset + 1] << 8) | first) >> 2),
      offset: offset + 2,
    };
  }
  if (mode === 2) {
    sliceExact(data, offset, 4);
    const encoded = new DataView(data.buffer, data.byteOffset + offset, 4).getUint32(0, true);
    return { value: BigInt(encoded >>> 2), offset: offset + 4 };
  }
  const byteLength = (first >> 2) + 4;
  if (byteLength > 16) throw new HttpError(400, "invalid_scale_compact", "SCALE compact 过大");
  const bytes = sliceExact(data, offset + 1, byteLength);
  let value = 0n;
  for (let index = byteLength - 1; index >= 0; index -= 1) {
    value = (value << 8n) | BigInt(bytes[index]);
  }
  return { value, offset: offset + 1 + byteLength };
}

function normalizeHash(value: string, field: string): string {
  const normalized = value.trim().toLowerCase();
  if (!/^0x[0-9a-f]{64}$/.test(normalized)) {
    throw new HttpError(400, "invalid_transaction_proof", `${field}不合法`);
  }
  return normalized;
}

function normalizeExtrinsicHex(value: string): string {
  const normalized = value.trim().toLowerCase();
  if (!/^0x(?:[0-9a-f]{2})+$/.test(normalized)) {
    throw new HttpError(400, "invalid_signed_extrinsic", "完整签名交易编码不合法");
  }
  return normalized;
}

function parseBlockNumber(value: string): number {
  if (!/^0x[0-9a-fA-F]+$/.test(value)) {
    throw new HttpError(502, "chain_rpc_invalid_response", "链服务节点返回了无效区块高度");
  }
  const parsed = Number(BigInt(value));
  if (!Number.isSafeInteger(parsed) || parsed < 0) {
    throw new HttpError(502, "chain_rpc_invalid_response", "链服务节点区块高度超出范围");
  }
  return parsed;
}

function businessActionsEqual(
  actual: SubscriptionBusinessAction,
  expected: SubscriptionBusinessAction,
): boolean {
  if (actual.kind !== expected.kind) return false;
  if (actual.kind === "platform_cancel" && expected.kind === "platform_cancel") return true;
  if (
    (actual.kind === "platform_subscribe" || actual.kind === "platform_change") &&
    (expected.kind === "platform_subscribe" || expected.kind === "platform_change")
  ) {
    return actual.membershipLevel === expected.membershipLevel;
  }
  if (actual.kind === "creator_cancel" && expected.kind === "creator_cancel") {
    return accountHex(expected.creatorAccount) === actual.creatorAccount;
  }
  if (
    (actual.kind === "creator_subscribe" || actual.kind === "creator_change") &&
    (expected.kind === "creator_subscribe" || expected.kind === "creator_change")
  ) {
    return (
      accountHex(expected.creatorAccount) === actual.creatorAccount &&
      actual.tierId === expected.tierId &&
      actual.billingPeriod === expected.billingPeriod
    );
  }
  if (actual.kind === "creator_plans_set" && expected.kind === "creator_plans_set") {
    return creatorTiersEqual(actual.tiers, expected.tiers);
  }
  return false;
}

function creatorTiersEqual(actual: ChainCreatorTier[], expected: ChainCreatorTier[]): boolean {
  if (actual.length !== expected.length) return false;
  return actual.every((tier, index) => {
    const other = expected[index];
    if (!other || tier.tierId !== other.tierId) return false;
    return (["monthly", "quarterly", "yearly"] as BillingPeriod[]).every(
      (period) => tier.pricesFen[period] === other.pricesFen[period],
    );
  });
}

function accountHex(account: string): string {
  return `0x${bytesToHex(decodeOwnerAccount(account))}`;
}

function strictUtf8(value: Uint8Array): string {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(value);
  } catch {
    throw new HttpError(400, "invalid_subscription_call", "订阅文本不是合法 UTF-8");
  }
}

function sliceExact(data: Uint8Array, offset: number, length: number): Uint8Array {
  if (offset < 0 || length < 0 || offset + length > data.length) {
    throw new HttpError(400, "invalid_signed_extrinsic", "签名交易被截断");
  }
  return data.slice(offset, offset + length);
}

function equalBytes(left: Uint8Array, right: Uint8Array): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}
