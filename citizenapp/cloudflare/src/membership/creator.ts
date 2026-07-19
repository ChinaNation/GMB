import type { Env } from "../types";
import { HttpError, jsonResponse, requireSession } from "../shared/http";
import { nowMs } from "../shared/time";
import {
  readCreatorPlans,
  readSubscription,
  type ChainCreatorTier,
  type ChainSubscriptionState,
} from "../chain/subscription";

/// 创作者会员 BFF：finalized 链上档位的展示镜像 + 概览 + 订阅镜像 + 门禁。
/// CreatorPlans 的 tier_id/周期/价格是唯一真源；Worker 只在链上最终确认后保存名称等展示数据，
/// 不再发起第二次账户业务签名。金额一律「分」。

const PERIODS = ["monthly", "quarterly", "yearly"] as const;
type Period = (typeof PERIODS)[number];
const MAX_TIERS = 10;

export interface CreatorSubscriptionConfirmDeps {
  readSubscription: (
    env: Env,
    subscriberAccount: string,
    creatorAccount: string,
  ) => Promise<ChainSubscriptionState | null>;
}

export interface CreatorPlanSaveDeps {
  readCreatorPlans: (
    env: Env,
    creatorAccount: string,
  ) => Promise<ChainCreatorTier[]>;
}

const defaultCreatorPlanSaveDeps: CreatorPlanSaveDeps = { readCreatorPlans };

const defaultSubscriptionConfirmDeps: CreatorSubscriptionConfirmDeps = {
  readSubscription: (env, subscriberAccount, creatorAccount) =>
    readSubscription(env, subscriberAccount, {
      kind: "creator",
      creatorAccount,
    }),
};

export interface CreatorTierInput {
  tier_id: string;
  name: string;
  prices_fen: Partial<Record<Period, number>>;
}

/// 严格校验并归一化档位（≤10、档名非空、id 唯一、每档至少一个正整数分周期价）。
function validateTiers(raw: unknown): CreatorTierInput[] {
  if (!Array.isArray(raw)) {
    throw new HttpError(400, "invalid_tiers", "档位必须是数组");
  }
  if (raw.length > MAX_TIERS) {
    throw new HttpError(400, "too_many_tiers", `最多 ${MAX_TIERS} 个会员档`);
  }
  const tiers: CreatorTierInput[] = [];
  const seen = new Set<string>();
  for (const item of raw as Array<Record<string, unknown>>) {
    const tierId = typeof item.tier_id === "string" ? item.tier_id : "";
    const name = typeof item.name === "string" ? item.name.trim() : "";
    if (!tierId || !name) {
      throw new HttpError(400, "invalid_tier", "档位需含 id 与名称");
    }
    if (seen.has(tierId)) {
      throw new HttpError(400, "duplicate_tier", "档位 id 重复");
    }
    seen.add(tierId);
    const rawPrices =
      typeof item.prices_fen === "object" && item.prices_fen !== null
        ? (item.prices_fen as Record<string, unknown>)
        : {};
    const prices: Partial<Record<Period, number>> = {};
    for (const period of PERIODS) {
      const value = rawPrices[period];
      if (value === undefined || value === null) continue;
      if (typeof value !== "number" || !Number.isInteger(value) || value <= 0) {
        throw new HttpError(400, "invalid_price", "价格必须为正整数分");
      }
      prices[period] = value;
    }
    if (Object.keys(prices).length === 0) {
      throw new HttpError(400, "no_period", "每档至少开一个周期并填价");
    }
    tiers.push({ tier_id: tierId, name, prices_fen: prices });
  }
  return tiers;
}

/// 用 finalized CreatorPlans 校验请求并生成可写入 D1 的展示镜像；请求价不得覆盖链上价。
function verifiedDisplayTiers(
  requested: CreatorTierInput[],
  chainTiers: ChainCreatorTier[],
): CreatorTierInput[] {
  if (requested.length !== chainTiers.length) {
    throw new HttpError(409, "creator_plans_not_finalized", "链上创作者档位尚未最终确认");
  }
  const requestedById = new Map(requested.map((tier) => [tier.tier_id, tier]));
  return chainTiers.map((chainTier) => {
    const input = requestedById.get(chainTier.tierId);
    if (!input) {
      throw new HttpError(409, "creator_plans_not_finalized", "链上创作者档位尚未最终确认");
    }
    const prices: Partial<Record<Period, number>> = {};
    for (const period of PERIODS) {
      const chainPrice = chainTier.pricesFen[period];
      const requestedPrice = input.prices_fen[period];
      if (chainPrice === undefined) {
        if (requestedPrice !== undefined) {
          throw new HttpError(409, "creator_plans_not_finalized", "链上创作者档位价格尚未最终确认");
        }
        continue;
      }
      if (chainPrice > BigInt(Number.MAX_SAFE_INTEGER)) {
        throw new HttpError(502, "creator_price_out_of_range", "链上创作者档位价格超出服务范围");
      }
      const chainPriceNumber = Number(chainPrice);
      if (requestedPrice !== chainPriceNumber) {
        throw new HttpError(409, "creator_plans_not_finalized", "链上创作者档位价格尚未最终确认");
      }
      prices[period] = chainPriceNumber;
    }
    return { tier_id: chainTier.tierId, name: input.name, prices_fen: prices };
  });
}

async function readPlanTiers(
  env: Env,
  creatorAccount: string,
): Promise<CreatorTierInput[]> {
  const row = await env.DB.prepare(
    "SELECT tiers_json FROM square_creator_plans WHERE creator_account = ?",
  )
    .bind(creatorAccount)
    .first<{ tiers_json: string }>();
  if (!row) return [];
  try {
    return JSON.parse(row.tiers_json) as CreatorTierInput[];
  } catch {
    return [];
  }
}

async function readPlan(env: Env, creatorAccount: string): Promise<unknown> {
  const row = await env.DB.prepare(
    "SELECT creator_account, tiers_json, updated_at FROM square_creator_plans WHERE creator_account = ?",
  )
    .bind(creatorAccount)
    .first<{
      creator_account: string;
      tiers_json: string;
      updated_at: number;
    }>();
  if (!row) return null;
  return {
    creator_account: row.creator_account,
    tiers: JSON.parse(row.tiers_json),
    updated_at: row.updated_at,
  };
}

/// 当前日历月起点（UTC 毫秒），用于本月已收入统计。
function monthStartMs(): number {
  const now = new Date(nowMs());
  return Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1);
}

// ── 端点 ──

/// GET /v1/square/creator/plan —— 我的档位。
export async function creatorPlanRoute(
  request: Request,
  env: Env,
): Promise<Response> {
  const session = await requireSession(request, env);
  return jsonResponse({ plan: await readPlan(env, session.owner_account) });
}

/// GET /v1/square/creator/plan/:account —— 他人档位（订阅者选档用）。
export async function creatorPlanOfRoute(
  request: Request,
  env: Env,
  account: string,
): Promise<Response> {
  await requireSession(request, env);
  const creatorAccount = decodeURIComponent(account);
  return jsonResponse({
    plan: await readPlan(env, creatorAccount),
  });
}

/// GET /v1/square/creator/overview —— 订阅人数 + 本月已收入（真实，非摊算）+ 档位数。
export async function creatorOverviewRoute(
  request: Request,
  env: Env,
): Promise<Response> {
  const session = await requireSession(request, env);
  const owner = session.owner_account;
  const countRow = await env.DB.prepare(
    "SELECT COUNT(*) AS cnt FROM square_creator_subscriptions WHERE creator_account = ? AND status = 'active'",
  )
    .bind(owner)
    .first<{ cnt: number }>();
  const incomeRow = await env.DB.prepare(
    "SELECT COALESCE(SUM(price_fen), 0) AS total FROM square_creator_subscriptions WHERE creator_account = ? AND status = 'active' AND last_charged_at >= ?",
  )
    .bind(owner, monthStartMs())
    .first<{ total: number }>();
  const tiers = await readPlanTiers(env, owner);
  return jsonResponse({
    overview: {
      subscriber_count: Number(countRow?.cnt ?? 0),
      month_income_fen: Number(incomeRow?.total ?? 0),
      tier_count: tiers.length,
    },
  });
}

/// POST /v1/square/creator/plan —— 校验 finalized CreatorPlans 后保存展示镜像。
export async function creatorPlanSaveRoute(
  request: Request,
  env: Env,
  deps: CreatorPlanSaveDeps = defaultCreatorPlanSaveDeps,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = (await request.json()) as {
    tx_hash?: unknown;
    tiers?: unknown;
  };
  const txHash = typeof body.tx_hash === "string" ? body.tx_hash : "";
  if (!/^0x[0-9a-f]{64}$/.test(txHash)) {
    throw new HttpError(400, "invalid_request", "缺少有效的链上交易哈希");
  }
  const requested = validateTiers(body.tiers);
  const chainTiers = await deps.readCreatorPlans(env, session.owner_account);
  const tiers = verifiedDisplayTiers(requested, chainTiers);
  const updatedAt = nowMs();
  await env.DB.prepare(
    `INSERT INTO square_creator_plans (creator_account, tiers_json, updated_at)
      VALUES (?, ?, ?)
      ON CONFLICT(creator_account) DO UPDATE SET
        tiers_json = excluded.tiers_json, updated_at = excluded.updated_at`,
  )
    .bind(session.owner_account, JSON.stringify(tiers), updatedAt)
    .run();
  return jsonResponse({
    plan: {
      creator_account: session.owner_account,
      tiers,
      updated_at: updatedAt,
    },
  });
}

/// POST /v1/square/creator/subscription/confirm —— 订阅/取消上链后镜像（幂等）。
/// subscriber 由 session 派生（不采信 body）；带 tier_id+period=订阅→active，缺=取消→cancelled。
/// confirm 先核对 finalized 链上真态；定时对账只负责后续续费/欠费同步。
export async function creatorSubscriptionConfirmRoute(
  request: Request,
  env: Env,
  deps: CreatorSubscriptionConfirmDeps = defaultSubscriptionConfirmDeps,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = (await request.json()) as {
    tx_hash?: unknown;
    creator_account?: unknown;
    tier_id?: unknown;
    period?: unknown;
  };
  const txHash = typeof body.tx_hash === "string" ? body.tx_hash : "";
  const creator =
    typeof body.creator_account === "string" ? body.creator_account : "";
  const tierId = typeof body.tier_id === "string" ? body.tier_id : "";
  const period = typeof body.period === "string" ? body.period : "";
  if (!/^0x[0-9a-f]{64}$/.test(txHash) || !creator) {
    throw new HttpError(400, "invalid_request", "确认参数不完整");
  }
  const subscriber = session.owner_account;
  const now = nowMs();
  const isSubscribe = tierId !== "" && period !== "";
  const chainState = await deps.readSubscription(env, subscriber, creator);
  if (!isSubscribe) {
    if (chainState === null || chainState.status !== "cancelled") {
      throw new HttpError(
        409,
        "subscription_state_not_finalized",
        "链上取消状态尚未最终确认",
      );
    }
    // 取消：仅在已有订阅时翻 cancelled，保留档位/价用于展示。
    await env.DB.prepare(
      `UPDATE square_creator_subscriptions
        SET status = 'cancelled', last_tx_hash = ?, updated_at = ?
        WHERE subscriber_account = ? AND creator_account = ?`,
    )
      .bind(txHash, now, subscriber, creator)
      .run();
    return jsonResponse({ ok: true, status: "cancelled" });
  }
  const chainPriceFen = chainState?.lastChargedPriceFen;
  const currentPlan = chainState?.plan;
  const pendingPlan = chainState?.pendingPlan;
  const requestMatchesCurrent =
    currentPlan?.kind === "creator" &&
    currentPlan.tierId === tierId &&
    currentPlan.billingPeriod === period;
  const requestMatchesPending =
    pendingPlan?.kind === "creator" &&
    pendingPlan.tierId === tierId &&
    pendingPlan.billingPeriod === period;
  if (
    chainState === null ||
    chainState.status !== "active" ||
    currentPlan?.kind !== "creator" ||
    (!requestMatchesCurrent && !requestMatchesPending) ||
    chainPriceFen === undefined ||
    chainPriceFen <= 0n ||
    chainPriceFen > BigInt(Number.MAX_SAFE_INTEGER)
  ) {
    throw new HttpError(
      409,
      "subscription_state_not_finalized",
      "链上订阅状态或价格尚未最终确认",
    );
  }
  const priceFen = Number(chainPriceFen);
  await env.DB.prepare(
    `INSERT INTO square_creator_subscriptions
      (subscriber_account, creator_account, tier_id, period, price_fen, status, last_charged_at, last_tx_hash, updated_at)
      VALUES (?, ?, ?, ?, ?, 'active', ?, ?, ?)
      ON CONFLICT(subscriber_account, creator_account) DO UPDATE SET
        tier_id = excluded.tier_id, period = excluded.period, price_fen = excluded.price_fen,
        status = 'active', last_charged_at = excluded.last_charged_at,
        last_tx_hash = excluded.last_tx_hash, updated_at = excluded.updated_at`,
  )
    .bind(
      subscriber,
      creator,
      currentPlan.tierId,
      currentPlan.billingPeriod,
      priceFen,
      chainState.lastChargedAt,
      txHash,
      now,
    )
    .run();
  return jsonResponse({
    ok: true,
    status: "active",
    pending_plan: requestMatchesPending
      ? { tier_id: tierId, period }
      : null,
  });
}

/// 门禁 helper（下游内容门禁用）：镜像 active 才放行；镜像滞后可由调用方加链读兜底。
export async function requireCreatorSubscription(
  env: Env,
  subscriberAccount: string,
  creatorAccount: string,
): Promise<void> {
  const row = await env.DB.prepare(
    "SELECT status FROM square_creator_subscriptions WHERE subscriber_account = ? AND creator_account = ? AND status = 'active'",
  )
    .bind(subscriberAccount, creatorAccount)
    .first<{ status: string }>();
  if (!row) {
    throw new HttpError(
      402,
      "creator_subscription_required",
      "需订阅该创作者会员",
    );
  }
}
