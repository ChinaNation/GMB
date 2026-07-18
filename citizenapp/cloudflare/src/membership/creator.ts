import type { Env } from '../types';
import { consumeActionSignature, issueActionChallenge } from '../account/action_challenge';
import { sha256Hex } from '../shared/hash';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';

/// 创作者会员 BFF：档位定义读写（全链下，Cloudflare 存）+ 概览（本月真实收入）+ 订阅镜像 + 门禁。
/// 档位写入复用现有广场账户动作统一签名（OP_SIGN_SQUARE_ACTION 0x1D，action='set_creator_plan'），
/// context 绑 tiers 规范化哈希防替换；不新增任何签名协议。金额一律「分」。

const PERIODS = ['monthly', 'quarterly', 'yearly'] as const;
type Period = (typeof PERIODS)[number];
const MAX_TIERS = 10;

interface CreatorTierInput {
  tier_id: string;
  name: string;
  prices_fen: Partial<Record<Period, number>>;
}

/// 规范化档位为确定性字符串（挑战与保存两次必须一致 → 绑定哈希才能对上）。
function canonicalTiers(tiers: CreatorTierInput[]): string {
  const normalized = tiers
    .map((tier) => {
      const prices: Partial<Record<Period, number>> = {};
      for (const period of PERIODS) {
        const value = tier.prices_fen[period];
        if (typeof value === 'number' && Number.isInteger(value) && value > 0) {
          prices[period] = value;
        }
      }
      return { tier_id: tier.tier_id, name: tier.name, prices_fen: prices };
    })
    .sort((a, b) => a.tier_id.localeCompare(b.tier_id));
  return JSON.stringify(normalized);
}

/// 严格校验并归一化档位（≤10、档名非空、id 唯一、每档至少一个正整数分周期价）。
function validateTiers(raw: unknown): CreatorTierInput[] {
  if (!Array.isArray(raw) || raw.length === 0) {
    throw new HttpError(400, 'invalid_tiers', '档位不能为空');
  }
  if (raw.length > MAX_TIERS) {
    throw new HttpError(400, 'too_many_tiers', `最多 ${MAX_TIERS} 个会员档`);
  }
  const tiers: CreatorTierInput[] = [];
  const seen = new Set<string>();
  for (const item of raw as Array<Record<string, unknown>>) {
    const tierId = typeof item.tier_id === 'string' ? item.tier_id : '';
    const name = typeof item.name === 'string' ? item.name.trim() : '';
    if (!tierId || !name) {
      throw new HttpError(400, 'invalid_tier', '档位需含 id 与名称');
    }
    if (seen.has(tierId)) {
      throw new HttpError(400, 'duplicate_tier', '档位 id 重复');
    }
    seen.add(tierId);
    const rawPrices =
      typeof item.prices_fen === 'object' && item.prices_fen !== null
        ? (item.prices_fen as Record<string, unknown>)
        : {};
    const prices: Partial<Record<Period, number>> = {};
    for (const period of PERIODS) {
      const value = rawPrices[period];
      if (value === undefined || value === null) continue;
      if (typeof value !== 'number' || !Number.isInteger(value) || value <= 0) {
        throw new HttpError(400, 'invalid_price', '价格必须为正整数分');
      }
      prices[period] = value;
    }
    if (Object.keys(prices).length === 0) {
      throw new HttpError(400, 'no_period', '每档至少开一个周期并填价');
    }
    tiers.push({ tier_id: tierId, name, prices_fen: prices });
  }
  return tiers;
}

async function readPlanTiers(env: Env, creatorAccount: string): Promise<CreatorTierInput[]> {
  const row = await env.DB.prepare(
    'SELECT tiers_json FROM square_creator_plans WHERE creator_account = ?'
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
    'SELECT creator_account, tiers_json, updated_at FROM square_creator_plans WHERE creator_account = ?'
  )
    .bind(creatorAccount)
    .first<{ creator_account: string; tiers_json: string; updated_at: number }>();
  if (!row) return null;
  return {
    creator_account: row.creator_account,
    tiers: JSON.parse(row.tiers_json),
    updated_at: row.updated_at,
  };
}

function priceForTierPeriod(
  tiers: CreatorTierInput[],
  tierId: string,
  period: string
): number {
  const tier = tiers.find((item) => item.tier_id === tierId);
  const value = tier?.prices_fen[period as Period];
  return typeof value === 'number' ? value : 0;
}

/// 当前日历月起点（UTC 毫秒），用于本月已收入统计。
function monthStartMs(): number {
  const now = new Date(nowMs());
  return Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1);
}

// ── 端点 ──

/// GET /v1/square/creator/plan —— 我的档位。
export async function creatorPlanRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  return jsonResponse({ plan: await readPlan(env, session.owner_account) });
}

/// GET /v1/square/creator/plan/:account —— 他人档位（订阅者选档用）。
export async function creatorPlanOfRoute(
  request: Request,
  env: Env,
  account: string
): Promise<Response> {
  await requireSession(request, env);
  return jsonResponse({ plan: await readPlan(env, decodeURIComponent(account)) });
}

/// GET /v1/square/creator/overview —— 订阅人数 + 本月已收入（真实，非摊算）+ 档位数。
export async function creatorOverviewRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const owner = session.owner_account;
  const countRow = await env.DB.prepare(
    "SELECT COUNT(*) AS cnt FROM square_creator_subscriptions WHERE creator_account = ? AND status = 'active'"
  )
    .bind(owner)
    .first<{ cnt: number }>();
  const incomeRow = await env.DB.prepare(
    "SELECT COALESCE(SUM(price_fen), 0) AS total FROM square_creator_subscriptions WHERE creator_account = ? AND status = 'active' AND last_charged_at >= ?"
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

/// GET /v1/square/creator/subscription/:account —— 我对该创作者的订阅态（按钮双态）。
export async function creatorSubscriptionStatusRoute(
  request: Request,
  env: Env,
  account: string
): Promise<Response> {
  const session = await requireSession(request, env);
  const row = await env.DB.prepare(
    'SELECT status FROM square_creator_subscriptions WHERE subscriber_account = ? AND creator_account = ?'
  )
    .bind(session.owner_account, decodeURIComponent(account))
    .first<{ status: string }>();
  return jsonResponse({ status: row ? row.status : null });
}

/// POST /v1/square/creator/plan/challenge —— 发起设档动作挑战（context 绑 tiers 哈希）。
export async function creatorPlanChallengeRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = (await request.json()) as { tiers?: unknown };
  const tiers = validateTiers(body.tiers);
  const context = await sha256Hex(canonicalTiers(tiers));
  const challenge = await issueActionChallenge(
    env,
    session.owner_account,
    'set_creator_plan',
    context
  );
  return jsonResponse({
    signing_payload_hex: challenge.signingPayloadHex,
    challenge_id: challenge.challengeId,
    expires_at: challenge.expiresAt,
  });
}

/// POST /v1/square/creator/plan —— 验 0x1D 签名 + tiers 哈希一致 → 覆盖写档位。
export async function creatorPlanSaveRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = (await request.json()) as {
    challenge_id?: unknown;
    signature?: unknown;
    tiers?: unknown;
  };
  const challengeId = typeof body.challenge_id === 'string' ? body.challenge_id : '';
  const signature = typeof body.signature === 'string' ? body.signature : '';
  if (!challengeId || !signature) {
    throw new HttpError(400, 'invalid_request', '缺少挑战或签名');
  }
  const tiers = validateTiers(body.tiers);
  const context = await sha256Hex(canonicalTiers(tiers));
  // 验签 + 一次性消费；context 必与挑战下发时一致（= tiers 未被替换）。
  await consumeActionSignature(env, {
    ownerAccount: session.owner_account,
    action: 'set_creator_plan',
    challengeId,
    signature,
    context,
  });
  const updatedAt = nowMs();
  await env.DB.prepare(
    `INSERT INTO square_creator_plans (creator_account, tiers_json, updated_at)
      VALUES (?, ?, ?)
      ON CONFLICT(creator_account) DO UPDATE SET
        tiers_json = excluded.tiers_json, updated_at = excluded.updated_at`
  )
    .bind(session.owner_account, JSON.stringify(tiers), updatedAt)
    .run();
  return jsonResponse({
    plan: { creator_account: session.owner_account, tiers, updated_at: updatedAt },
  });
}

/// POST /v1/square/creator/subscription/confirm —— 订阅/取消上链后镜像（幂等）。
/// subscriber 由 session 派生（不采信 body）；带 tier_id+period=订阅→active，缺=取消→cancelled。
/// TODO(硬化)：链读 Subscriptions[(subscriber, Creator(creator))] 核实后再镜像；当前信任 App 已上链 tx。
export async function creatorSubscriptionConfirmRoute(
  request: Request,
  env: Env
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = (await request.json()) as {
    tx_hash?: unknown;
    creator_account?: unknown;
    tier_id?: unknown;
    period?: unknown;
  };
  const txHash = typeof body.tx_hash === 'string' ? body.tx_hash : '';
  const creator = typeof body.creator_account === 'string' ? body.creator_account : '';
  const tierId = typeof body.tier_id === 'string' ? body.tier_id : '';
  const period = typeof body.period === 'string' ? body.period : '';
  if (!/^0x[0-9a-f]{64}$/.test(txHash) || !creator) {
    throw new HttpError(400, 'invalid_request', '确认参数不完整');
  }
  const subscriber = session.owner_account;
  const now = nowMs();
  const isSubscribe = tierId !== '' && period !== '';
  if (!isSubscribe) {
    // 取消：仅在已有订阅时翻 cancelled，保留档位/价用于展示。
    await env.DB.prepare(
      `UPDATE square_creator_subscriptions
        SET status = 'cancelled', last_tx_hash = ?, updated_at = ?
        WHERE subscriber_account = ? AND creator_account = ?`
    )
      .bind(txHash, now, subscriber, creator)
      .run();
    return jsonResponse({ ok: true, status: 'cancelled' });
  }
  const priceFen = priceForTierPeriod(await readPlanTiers(env, creator), tierId, period);
  await env.DB.prepare(
    `INSERT INTO square_creator_subscriptions
      (subscriber_account, creator_account, tier_id, period, price_fen, status, last_charged_at, last_tx_hash, updated_at)
      VALUES (?, ?, ?, ?, ?, 'active', ?, ?, ?)
      ON CONFLICT(subscriber_account, creator_account) DO UPDATE SET
        tier_id = excluded.tier_id, period = excluded.period, price_fen = excluded.price_fen,
        status = 'active', last_charged_at = excluded.last_charged_at,
        last_tx_hash = excluded.last_tx_hash, updated_at = excluded.updated_at`
  )
    .bind(subscriber, creator, tierId, period, priceFen, now, txHash, now)
    .run();
  return jsonResponse({ ok: true, status: 'active' });
}

/// 门禁 helper（下游内容门禁用）：镜像 active 才放行；镜像滞后可由调用方加链读兜底。
export async function requireCreatorSubscription(
  env: Env,
  subscriberAccount: string,
  creatorAccount: string
): Promise<void> {
  const row = await env.DB.prepare(
    "SELECT status FROM square_creator_subscriptions WHERE subscriber_account = ? AND creator_account = ? AND status = 'active'"
  )
    .bind(subscriberAccount, creatorAccount)
    .first<{ status: string }>();
  if (!row) {
    throw new HttpError(402, 'creator_subscription_required', '需订阅该创作者会员');
  }
}
