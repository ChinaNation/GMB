import type { Env } from "../types";
import { nowMs } from "../shared/time";
import { fetchBlockHeader, fetchFinalizedHead, isChainRpcConfigured } from "../chain/rpc";
import {
  readChainTimestampAtBlock,
  readSubscriptionAtBlock,
  updateChainClock,
  type ChainSubscriptionState,
} from "../chain/subscription";

/**
 * 订阅镜像对账只处理已经到期、可能发生自动续费或终止的候选项。每轮只读取一次 finalized
 * 头和 Timestamp.Now，不滚动扫描未到期全表，不承担扣款或续费职责。
 */

const DEFAULT_BATCH = 50;
const MAX_BATCH = 500;

export const MEMBERSHIP_RECONCILE_FLAG_KEY = "flag:membership_reconcile";
export const CREATOR_RECONCILE_FLAG_KEY = "flag:creator_reconcile";

interface FinalizedPoint {
  blockHash: string;
  blockNumber: number;
  chainTimestamp: number;
  observedAt: number;
}

export interface ReconcileDeps {
  finalizedPoint: (env: Env) => Promise<FinalizedPoint>;
  readSubscriptionAtBlock: typeof readSubscriptionAtBlock;
}

const defaultDeps: ReconcileDeps = {
  finalizedPoint: readFinalizedPoint,
  readSubscriptionAtBlock,
};

export interface ReconcileResult {
  scanned: number;
  updated: number;
  failed: number;
}

export interface SubscriptionReconcileResult {
  platform: ReconcileResult;
  creator: ReconcileResult;
}

const EMPTY_RESULT: ReconcileResult = { scanned: 0, updated: 0, failed: 0 };

/** Cron 唯一入口：共享一次 finalized 链锚点后分别处理平台与创作者到期候选。 */
export async function reconcileSubscriptions(
  env: Env,
  deps: ReconcileDeps = defaultDeps,
): Promise<SubscriptionReconcileResult> {
  const [platformEnabled, creatorEnabled] = await Promise.all([
    reconcileEnabled(env, MEMBERSHIP_RECONCILE_FLAG_KEY, env.MEMBERSHIP_RECONCILE_ENABLED),
    reconcileEnabled(env, CREATOR_RECONCILE_FLAG_KEY, env.CREATOR_RECONCILE_ENABLED),
  ]);
  if ((!platformEnabled && !creatorEnabled) || !isChainRpcConfigured(env)) {
    return { platform: { ...EMPTY_RESULT }, creator: { ...EMPTY_RESULT } };
  }

  const point = await deps.finalizedPoint(env);
  await updateChainClock(env, {
    chainTimestamp: point.chainTimestamp,
    blockNumber: point.blockNumber,
    blockHash: point.blockHash,
    observedAt: point.observedAt,
  });

  // 已取消订阅无需点读链：权益在链上 paid_until 到达时自然失效，归档起点固定为该时间。
  await env.DB.prepare(
    `UPDATE square_memberships
      SET entitlement_lapsed_at = paid_until
      WHERE subscription_status IN ('cancelled', 'terminated')
        AND entitlement_lapsed_at IS NULL AND paid_until <= ?`,
  ).bind(point.chainTimestamp).run();

  const [platform, creator] = await Promise.all([
    platformEnabled
      ? reconcilePlatformCandidates(env, point, deps, reconcileBatchSize(env))
      : Promise.resolve({ ...EMPTY_RESULT }),
    creatorEnabled
      ? reconcileCreatorCandidates(env, point, deps, reconcileBatchSize(env))
      : Promise.resolve({ ...EMPTY_RESULT }),
  ]);
  return { platform, creator };
}

/** 单模块测试与人工诊断入口；生产 Cron 使用 reconcileSubscriptions 避免重复读链。 */
export async function reconcileMemberships(
  env: Env,
  deps: ReconcileDeps = defaultDeps,
): Promise<ReconcileResult> {
  if (!(await reconcileEnabled(env, MEMBERSHIP_RECONCILE_FLAG_KEY, env.MEMBERSHIP_RECONCILE_ENABLED))) {
    return { ...EMPTY_RESULT };
  }
  if (!isChainRpcConfigured(env)) return { ...EMPTY_RESULT };
  const point = await deps.finalizedPoint(env);
  await updateChainClock(env, {
    chainTimestamp: point.chainTimestamp,
    blockNumber: point.blockNumber,
    blockHash: point.blockHash,
    observedAt: point.observedAt,
  });
  return reconcilePlatformCandidates(env, point, deps, reconcileBatchSize(env));
}

export async function reconcileCreatorSubscriptions(
  env: Env,
  deps: ReconcileDeps = defaultDeps,
): Promise<ReconcileResult> {
  if (!(await reconcileEnabled(env, CREATOR_RECONCILE_FLAG_KEY, env.CREATOR_RECONCILE_ENABLED))) {
    return { ...EMPTY_RESULT };
  }
  if (!isChainRpcConfigured(env)) return { ...EMPTY_RESULT };
  const point = await deps.finalizedPoint(env);
  await updateChainClock(env, {
    chainTimestamp: point.chainTimestamp,
    blockNumber: point.blockNumber,
    blockHash: point.blockHash,
    observedAt: point.observedAt,
  });
  return reconcileCreatorCandidates(env, point, deps, reconcileBatchSize(env));
}

async function readFinalizedPoint(env: Env): Promise<FinalizedPoint> {
  const blockHash = await fetchFinalizedHead(env);
  const [header, chainTimestamp] = await Promise.all([
    fetchBlockHeader(env, blockHash),
    readChainTimestampAtBlock(env, blockHash),
  ]);
  return {
    blockHash,
    blockNumber: parseBlockNumber(header.number),
    chainTimestamp,
    observedAt: nowMs(),
  };
}

async function reconcilePlatformCandidates(
  env: Env,
  point: FinalizedPoint,
  deps: ReconcileDeps,
  batch: number,
): Promise<ReconcileResult> {
  const rows = await env.DB.prepare(
    `SELECT owner_account FROM square_memberships
      WHERE subscription_status = 'active' AND paid_until <= ?
      ORDER BY paid_until ASC LIMIT ?`,
  ).bind(point.chainTimestamp, batch).all<{ owner_account: string }>();
  return runBatch(rows.results ?? [], async (row) => {
    const state = await deps.readSubscriptionAtBlock(
      env,
      row.owner_account,
      { kind: "platform" },
      point.blockHash,
    );
    await applyPlatformState(env, row.owner_account, state, point);
  });
}

async function reconcileCreatorCandidates(
  env: Env,
  point: FinalizedPoint,
  deps: ReconcileDeps,
  batch: number,
): Promise<ReconcileResult> {
  const rows = await env.DB.prepare(
    `SELECT subscriber_account, creator_account FROM square_creator_subscriptions
      WHERE subscription_status = 'active' AND paid_until <= ?
      ORDER BY paid_until ASC LIMIT ?`,
  ).bind(point.chainTimestamp, batch).all<{
    subscriber_account: string;
    creator_account: string;
  }>();
  return runBatch(rows.results ?? [], async (row) => {
    const state = await deps.readSubscriptionAtBlock(
      env,
      row.subscriber_account,
      { kind: "creator", creatorAccount: row.creator_account },
      point.blockHash,
    );
    await applyCreatorState(env, row.subscriber_account, row.creator_account, state, point);
  });
}

async function applyPlatformState(
  env: Env,
  ownerAccount: string,
  state: ChainSubscriptionState | null,
  point: FinalizedPoint,
): Promise<void> {
  if (!state || state.plan.kind !== "platform") {
    await env.DB.prepare(
      `UPDATE square_memberships SET subscription_status = 'terminated',
        pending_membership_level = NULL, entitlement_lapsed_at = paid_until,
        finalized_block_number = ?, finalized_block_hash = ?, verified_at = ?
        WHERE owner_account = ?`,
    ).bind(point.blockNumber, point.blockHash, point.observedAt, ownerAccount).run();
    return;
  }
  const pending = state.pendingPlan?.kind === "platform"
    ? state.pendingPlan.membershipLevel
    : null;
  await env.DB.prepare(
    `UPDATE square_memberships SET membership_level = ?, pending_membership_level = ?,
      started_at = ?, last_charged_at = ?, last_charged_price_fen = ?, paid_until = ?,
      subscription_status = ?, finalized_block_number = ?, finalized_block_hash = ?,
      verified_at = ?, entitlement_lapsed_at = ? WHERE owner_account = ?`,
  ).bind(
    state.plan.membershipLevel,
    pending,
    state.startedAt,
    state.lastChargedAt,
    safePrice(state.lastChargedPriceFen),
    state.paidUntil,
    state.status,
    point.blockNumber,
    point.blockHash,
    point.observedAt,
    state.status === "active" ? null : state.paidUntil,
    ownerAccount,
  ).run();
}

async function applyCreatorState(
  env: Env,
  subscriberAccount: string,
  creatorAccount: string,
  state: ChainSubscriptionState | null,
  point: FinalizedPoint,
): Promise<void> {
  if (!state || state.plan.kind !== "creator") {
    await env.DB.prepare(
      `UPDATE square_creator_subscriptions SET subscription_status = 'terminated',
        pending_tier_id = NULL, pending_billing_period = NULL,
        finalized_block_number = ?, finalized_block_hash = ?, verified_at = ?
        WHERE subscriber_account = ? AND creator_account = ?`,
    ).bind(
      point.blockNumber,
      point.blockHash,
      point.observedAt,
      subscriberAccount,
      creatorAccount,
    ).run();
    return;
  }
  const pending = state.pendingPlan?.kind === "creator" ? state.pendingPlan : null;
  await env.DB.prepare(
    `UPDATE square_creator_subscriptions SET tier_id = ?, billing_period = ?,
      pending_tier_id = ?, pending_billing_period = ?, started_at = ?,
      last_charged_at = ?, last_charged_price_fen = ?, paid_until = ?,
      subscription_status = ?, finalized_block_number = ?, finalized_block_hash = ?, verified_at = ?
      WHERE subscriber_account = ? AND creator_account = ?`,
  ).bind(
    state.plan.tierId,
    state.plan.billingPeriod,
    pending?.tierId ?? null,
    pending?.billingPeriod ?? null,
    state.startedAt,
    state.lastChargedAt,
    safePrice(state.lastChargedPriceFen),
    state.paidUntil,
    state.status,
    point.blockNumber,
    point.blockHash,
    point.observedAt,
    subscriberAccount,
    creatorAccount,
  ).run();
}

async function runBatch<T>(
  rows: T[],
  handle: (row: T) => Promise<void>,
): Promise<ReconcileResult> {
  const result: ReconcileResult = { scanned: 0, updated: 0, failed: 0 };
  for (const row of rows) {
    result.scanned += 1;
    try {
      await handle(row);
      result.updated += 1;
    } catch (error) {
      result.failed += 1;
      console.error(JSON.stringify({
        event: "subscription_reconcile_row_failed",
        error: error instanceof Error ? error.message : String(error),
      }));
    }
  }
  return result;
}

async function reconcileEnabled(
  env: Env,
  kvKey: string,
  fallbackVar: string | undefined,
): Promise<boolean> {
  if (env.SQUARE_CACHE) {
    try {
      const value = await env.SQUARE_CACHE.get(kvKey);
      if (value !== null) return value === "1";
    } catch {
      // KV 只承载动态开关；读取失败时回退部署变量，不改变订阅真态。
    }
  }
  return fallbackVar === "1";
}

function reconcileBatchSize(env: Env): number {
  const raw = Number.parseInt(env.MEMBERSHIP_RECONCILE_BATCH ?? "", 10);
  if (!Number.isFinite(raw) || raw <= 0) return DEFAULT_BATCH;
  return Math.min(raw, MAX_BATCH);
}

function parseBlockNumber(value: string): number {
  if (!/^0x[0-9a-fA-F]+$/.test(value)) throw new Error("finalized block number invalid");
  const number = Number(BigInt(value));
  if (!Number.isSafeInteger(number) || number < 0) throw new Error("finalized block number out of range");
  return number;
}

function safePrice(value: bigint): number {
  if (value <= 0n || value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new Error("subscription price out of D1 range");
  }
  return Number(value);
}
