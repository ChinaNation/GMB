import type { Env } from '../types';
import { nowMs } from '../shared/time';
import { isChainRpcConfigured } from '../chain/rpc';
import {
  readSubscription,
  type ChainSubscriptionState,
  type SubscriptionIssuer
} from '../chain/subscription';

/// 会员镜像对账器：把 D1 镜像持续对齐链上 `Subscriptions` 真源。
///
/// 两个对账器同构、各自独立开关：
///  - `reconcileMemberships`：平台会员，镜像表 `square_memberships`（单键 owner）。
///  - `reconcileCreatorSubscriptions`：创作者会员，镜像表 `square_creator_subscriptions`（复合键 subscriber+creator）。
///
/// confirm（`citizen_coin.ts`/`creator.ts`）是乐观快路；本对账器是权威兜底——按 Cron 取一批镜像行，
/// 逐个链读订阅真态，**fail-closed** 回填：链上 Active→刷新有效；Terminated/Cancelled/查无→镜像收紧。
/// 由此解决自动续扣后镜像假过期、欠费即停跟进、以及 confirm 被伪造的纠正。
///
/// 状态对账（点读现态，非事件流）：无游标、天然幂等、限流分批（`updated_at` 最旧优先滚动覆盖）。
/// 开关从 KV 读（供控制台即时开关），未设则回退 wrangler var 默认。

const DEFAULT_BATCH = 50;
const MAX_BATCH = 500;

/// KV 开关键（控制台以同名键 + `--binding SQUARE_CACHE --env production` 写 '1'/'0'）。
export const MEMBERSHIP_RECONCILE_FLAG_KEY = 'flag:membership_reconcile';
export const CREATOR_RECONCILE_FLAG_KEY = 'flag:creator_reconcile';

/// 可注入依赖（测试用假实现替换链读与时钟）。
export interface ReconcileDeps {
  readSubscription: (
    env: Env,
    subscriberAccount: string,
    issuer: SubscriptionIssuer
  ) => Promise<ChainSubscriptionState | null>;
  now: () => number;
}

const defaultDeps: ReconcileDeps = {
  readSubscription,
  now: nowMs
};

export interface ReconcileResult {
  scanned: number;
  updated: number;
  failed: number;
}

const EMPTY_RESULT: ReconcileResult = { scanned: 0, updated: 0, failed: 0 };

/// 平台会员对账（issuer=Platform）。开关 KV `flag:membership_reconcile`（未设回退 var）。
export async function reconcileMemberships(
  env: Env,
  deps: ReconcileDeps = defaultDeps
): Promise<ReconcileResult> {
  if (!(await reconcileEnabled(env, MEMBERSHIP_RECONCILE_FLAG_KEY, env.MEMBERSHIP_RECONCILE_ENABLED))) {
    return { ...EMPTY_RESULT };
  }
  if (!isChainRpcConfigured(env)) return { ...EMPTY_RESULT };

  const owners = await selectOldest(env, 'square_memberships', 'owner_account', reconcileBatchSize(env));
  return runBatch(owners, async (row) => {
    const account = row.owner_account as string;
    const state = await deps.readSubscription(env, account, { kind: 'platform' });
    await applyPlatformState(env, account, state, deps.now());
  });
}

/// 创作者会员对账（issuer=Creator）。开关 KV `flag:creator_reconcile`（未设回退 var）。
export async function reconcileCreatorSubscriptions(
  env: Env,
  deps: ReconcileDeps = defaultDeps
): Promise<ReconcileResult> {
  if (!(await reconcileEnabled(env, CREATOR_RECONCILE_FLAG_KEY, env.CREATOR_RECONCILE_ENABLED))) {
    return { ...EMPTY_RESULT };
  }
  if (!isChainRpcConfigured(env)) return { ...EMPTY_RESULT };

  const rows = await selectOldest(
    env,
    'square_creator_subscriptions',
    'subscriber_account, creator_account',
    reconcileBatchSize(env)
  );
  return runBatch(rows, async (row) => {
    const subscriber = row.subscriber_account as string;
    const creator = row.creator_account as string;
    const state = await deps.readSubscription(env, subscriber, {
      kind: 'creator',
      creatorAccount: creator
    });
    await applyCreatorState(env, subscriber, creator, state, deps.now());
  });
}

/// 开关：优先读 KV（供控制台即时开关），未设或读失败回退 wrangler var 默认。
async function reconcileEnabled(
  env: Env,
  kvKey: string,
  fallbackVar: string | undefined
): Promise<boolean> {
  if (env.SQUARE_CACHE) {
    try {
      const value = await env.SQUARE_CACHE.get(kvKey);
      if (value !== null) return value === '1';
    } catch {
      // KV 读失败：回退 wrangler 默认，不因缓存抖动误停/误开。
    }
  }
  return fallbackVar === '1';
}

function reconcileBatchSize(env: Env): number {
  const raw = Number.parseInt(env.MEMBERSHIP_RECONCILE_BATCH ?? '', 10);
  if (!Number.isFinite(raw) || raw <= 0) return DEFAULT_BATCH;
  return Math.min(raw, MAX_BATCH);
}

/// 取一批「最旧对账」的镜像行（成功回写 updated_at=now → 滚到队尾，全表滚动覆盖）。
async function selectOldest(
  env: Env,
  table: string,
  columns: string,
  batch: number
): Promise<Record<string, unknown>[]> {
  const rows = await env.DB.prepare(
    `SELECT ${columns} FROM ${table} ORDER BY updated_at ASC LIMIT ?`
  )
    .bind(batch)
    .all<Record<string, unknown>>();
  return rows.results ?? [];
}

/// 逐条处理一批：单条失败不阻断整批（不动其 updated_at → 下轮自然重试）。
async function runBatch(
  rows: Record<string, unknown>[],
  handle: (row: Record<string, unknown>) => Promise<void>
): Promise<ReconcileResult> {
  const result: ReconcileResult = { scanned: 0, updated: 0, failed: 0 };
  for (const row of rows) {
    result.scanned += 1;
    try {
      await handle(row);
      result.updated += 1;
    } catch (error) {
      result.failed += 1;
      console.error(
        `[reconcile] row failed: ${error instanceof Error ? error.message : error}`
      );
    }
  }
  return result;
}

// ── 平台镜像回填 ──

/// 据链上真态回填平台镜像（fail-closed）。
async function applyPlatformState(
  env: Env,
  owner: string,
  state: ChainSubscriptionState | null,
  now: number
): Promise<void> {
  // 链上无订阅或已取消：镜像翻 cancelled，记权益失效时刻（视频冷归档时钟起点）。
  if (state === null || state.status === 'cancelled') {
    await lapsePlatform(env, owner, 'cancelled', now);
    return;
  }
  if (state.status === 'terminated') {
    await lapsePlatform(env, owner, 'terminated', now);
    return;
  }
  // 平台订阅理应携带档位；异常缺档保守收紧，不放行。
  if (state.plan.kind !== 'platform') {
    await lapsePlatform(env, owner, 'cancelled', now);
    return;
  }
  const level = state.plan.membershipLevel;
  // Active：直接镜像链上已确认时间戳，不在 Worker 复制自然日历算法。
  const periodStart = state.lastChargedAt;
  const periodEnd = state.paidUntil;
  await env.DB.prepare(
    `UPDATE square_memberships
       SET membership_level = ?, subscription_status = 'active',
           current_period_start = ?, current_period_end = ?, expires_at = ?,
           entitlement_lapsed_at = NULL, updated_at = ?
       WHERE owner_account = ?`
  )
    .bind(level, periodStart, periodEnd, periodEnd, now, owner)
    .run();
}

/// 收紧平台镜像为 terminated/cancelled，记权益失效时刻（已记则保留最早值）。
async function lapsePlatform(
  env: Env,
  owner: string,
  status: 'terminated' | 'cancelled',
  now: number
): Promise<void> {
  await env.DB.prepare(
    `UPDATE square_memberships
       SET subscription_status = ?,
           entitlement_lapsed_at = COALESCE(entitlement_lapsed_at, ?),
           updated_at = ?
       WHERE owner_account = ?`
  )
    .bind(status, now, now, owner)
    .run();
}

// ── 创作者镜像回填 ──

/// 据链上真态回填创作者订阅镜像（fail-closed）。
/// Active 时 tier_id/period/price_fen 也必须随当前链上计划更新，处理待换档到期生效。
async function applyCreatorState(
  env: Env,
  subscriber: string,
  creator: string,
  state: ChainSubscriptionState | null,
  now: number
): Promise<void> {
  if (state === null || state.status === 'cancelled') {
    await setCreatorStatus(env, subscriber, creator, 'cancelled', null, now);
    return;
  }
  if (state.status === 'terminated') {
    await setCreatorStatus(env, subscriber, creator, 'terminated', null, now);
    return;
  }
  if (
    state.plan.kind !== 'creator' ||
    state.lastChargedPriceFen <= 0n ||
    state.lastChargedPriceFen > BigInt(Number.MAX_SAFE_INTEGER)
  ) {
    await setCreatorStatus(env, subscriber, creator, 'cancelled', null, now);
    return;
  }
  // Active：状态、当前计划、最近实扣价与扣款时间全部回填 finalized 真态。
  await env.DB.prepare(
    `UPDATE square_creator_subscriptions
       SET tier_id = ?, period = ?, price_fen = ?, status = 'active',
           last_charged_at = ?, updated_at = ?
       WHERE subscriber_account = ? AND creator_account = ?`
  )
    .bind(
      state.plan.tierId,
      state.plan.billingPeriod,
      Number(state.lastChargedPriceFen),
      state.lastChargedAt,
      now,
      subscriber,
      creator
    )
    .run();
}

/// 写创作者订阅镜像 status（active 时一并刷新 last_charged_at）。
async function setCreatorStatus(
  env: Env,
  subscriber: string,
  creator: string,
  status: 'active' | 'terminated' | 'cancelled',
  lastChargedAt: number | null,
  now: number
): Promise<void> {
  if (lastChargedAt !== null) {
    await env.DB.prepare(
      `UPDATE square_creator_subscriptions
         SET status = ?, last_charged_at = ?, updated_at = ?
         WHERE subscriber_account = ? AND creator_account = ?`
    )
      .bind(status, lastChargedAt, now, subscriber, creator)
      .run();
    return;
  }
  await env.DB.prepare(
    `UPDATE square_creator_subscriptions
       SET status = ?, updated_at = ?
       WHERE subscriber_account = ? AND creator_account = ?`
  )
    .bind(status, now, subscriber, creator)
    .run();
}
