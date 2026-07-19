import type { Env } from "../types";
import { HttpError, jsonResponse, readJson, requireSession } from "../shared/http";
import { sha256Hex } from "../shared/hash";
import { nowMs } from "../shared/time";
import {
  bindFinalizedTransactionConfirmation,
  readCreatorPlansAtBlock,
  readSubscriptionAtBlock,
  updateChainClock,
  verifyFinalizedSubscriptionTransaction,
  type BillingPeriod,
  type ChainCreatorTier,
  type ChainSubscriptionState,
  type FinalizedTransactionProofInput,
  type SubscriptionBusinessAction,
  type VerifiedFinalizedTransaction,
} from "../chain/subscription";
import {
  CHAIN_CLOCK_MAX_STALENESS_MS,
  getMembership,
  isSubscriptionMirrorEffective,
  requireActiveMembership,
  subscriptionIsActive,
} from "./service";

/**
 * 创作者会员 BFF：钱包账户是唯一身份；档位展示、订阅关系和统计只保存 finalized 镜像。
 * 付款字段与订阅有效性来自链上，Cloudflare 不扣款、不续费、不计算订阅公历。
 */

const PERIODS = ["monthly", "quarterly", "yearly"] as const;
type Period = (typeof PERIODS)[number];
type CreatorAction = "subscribe" | "cancel" | "change";
const MAX_TIERS = 10;

export interface CreatorTierInput {
  tier_id: string;
  name: string;
  prices_fen: Partial<Record<Period, number>>;
}

interface CreatorPlanRow {
  creator_account: string;
  tier_id: string;
  name: string;
  tier_order: number;
  monthly_price_fen: number | null;
  quarterly_price_fen: number | null;
  yearly_price_fen: number | null;
  verified_at: number;
}

interface CreatorConfirmBody {
  tx_hash?: unknown;
  block_hash?: unknown;
  signed_extrinsic_hex?: unknown;
  action?: unknown;
  creator_account?: unknown;
  tier_id?: unknown;
  billing_period?: unknown;
}

interface CreatorPlanBody {
  tx_hash?: unknown;
  block_hash?: unknown;
  signed_extrinsic_hex?: unknown;
  tiers?: unknown;
}

export interface CreatorSubscriptionConfirmDeps {
  verifyTransaction: typeof verifyFinalizedSubscriptionTransaction;
  readSubscriptionAtBlock: (
    env: Env,
    subscriberAccount: string,
    creatorAccount: string,
    blockHash: string,
  ) => Promise<ChainSubscriptionState | null>;
}

export interface CreatorPlanSaveDeps {
  verifyTransaction: typeof verifyFinalizedSubscriptionTransaction;
  readCreatorPlansAtBlock: typeof readCreatorPlansAtBlock;
  readPlatformSubscriptionAtBlock: (
    env: Env,
    creatorAccount: string,
    blockHash: string,
  ) => Promise<ChainSubscriptionState | null>;
}

const defaultCreatorPlanSaveDeps: CreatorPlanSaveDeps = {
  verifyTransaction: verifyFinalizedSubscriptionTransaction,
  readCreatorPlansAtBlock,
  readPlatformSubscriptionAtBlock: (env, creatorAccount, blockHash) =>
    readSubscriptionAtBlock(env, creatorAccount, { kind: "platform" }, blockHash),
};

const defaultSubscriptionConfirmDeps: CreatorSubscriptionConfirmDeps = {
  verifyTransaction: verifyFinalizedSubscriptionTransaction,
  readSubscriptionAtBlock: (env, subscriberAccount, creatorAccount, blockHash) =>
    readSubscriptionAtBlock(
      env,
      subscriberAccount,
      { kind: "creator", creatorAccount },
      blockHash,
    ),
};

/** 严格校验并归一化档位；价格只用于与链上 signed call 和 finalized storage 对照。 */
function validateTiers(raw: unknown): CreatorTierInput[] {
  if (!Array.isArray(raw)) throw new HttpError(400, "invalid_tiers", "档位必须是数组");
  if (raw.length > MAX_TIERS) {
    throw new HttpError(400, "too_many_tiers", `最多 ${MAX_TIERS} 个会员档`);
  }
  const tiers: CreatorTierInput[] = [];
  const seen = new Set<string>();
  for (const item of raw as Array<Record<string, unknown>>) {
    const tierId = typeof item.tier_id === "string" ? item.tier_id : "";
    const name = typeof item.name === "string" ? item.name.trim() : "";
    if (!tierId || !name) throw new HttpError(400, "invalid_tier", "档位需含标识与名称");
    if (seen.has(tierId)) throw new HttpError(400, "duplicate_tier", "档位标识重复");
    seen.add(tierId);
    const rawPrices =
      typeof item.prices_fen === "object" && item.prices_fen !== null
        ? (item.prices_fen as Record<string, unknown>)
        : {};
    const pricesFen: Partial<Record<Period, number>> = {};
    for (const period of PERIODS) {
      const value = rawPrices[period];
      if (value === undefined || value === null) continue;
      if (typeof value !== "number" || !Number.isSafeInteger(value) || value <= 0) {
        throw new HttpError(400, "invalid_price", "价格必须为正整数分");
      }
      pricesFen[period] = value;
    }
    if (Object.keys(pricesFen).length === 0) {
      throw new HttpError(400, "no_period", "每档至少开一个周期并填价");
    }
    tiers.push({ tier_id: tierId, name, prices_fen: pricesFen });
  }
  return tiers;
}

function chainTiersFromInput(tiers: CreatorTierInput[]): ChainCreatorTier[] {
  return tiers.map((tier) => ({
    tierId: tier.tier_id,
    pricesFen: Object.fromEntries(
      PERIODS.flatMap((period) => {
        const value = tier.prices_fen[period];
        return value === undefined ? [] : [[period, BigInt(value)]];
      }),
    ) as Partial<Record<BillingPeriod, bigint>>,
  }));
}

function verifiedDisplayTiers(
  requested: CreatorTierInput[],
  chainTiers: ChainCreatorTier[],
): CreatorTierInput[] {
  const expected = chainTiersFromInput(requested);
  if (!creatorTiersEqual(expected, chainTiers)) {
    throw new HttpError(409, "creator_plans_not_finalized", "链上创作者档位尚未最终确认");
  }
  return requested;
}

async function readPlan(env: Env, creatorAccount: string): Promise<unknown> {
  const rows = await env.DB.prepare(
    `SELECT creator_account, tier_id, name, tier_order, monthly_price_fen,
        quarterly_price_fen, yearly_price_fen, verified_at
      FROM square_creator_tiers
      WHERE creator_account = ? ORDER BY tier_order ASC`,
  )
    .bind(creatorAccount)
    .all<CreatorPlanRow>();
  const items = rows.results ?? [];
  if (items.length === 0) return null;
  return {
    creator_account: creatorAccount,
    tiers: items.map(rowToTier),
    updated_at: Math.max(...items.map((row) => row.verified_at)),
  };
}

function rowToTier(row: CreatorPlanRow): CreatorTierInput {
  const pricesFen: Partial<Record<Period, number>> = {};
  if (row.monthly_price_fen !== null) pricesFen.monthly = row.monthly_price_fen;
  if (row.quarterly_price_fen !== null) pricesFen.quarterly = row.quarterly_price_fen;
  if (row.yearly_price_fen !== null) pricesFen.yearly = row.yearly_price_fen;
  return {
    tier_id: row.tier_id,
    name: row.name,
    prices_fen: pricesFen,
  };
}

function monthStartMs(): number {
  const now = new Date(nowMs());
  return Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1);
}

/** GET /v1/square/creator/plan —— 当前钱包的档位；平台订阅门禁在服务端复核。 */
export async function creatorPlanRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  await requireActiveMembership(env, session.owner_account);
  return jsonResponse({ plan: await readPlan(env, session.owner_account) });
}

/** GET /v1/square/creator/plan/:account —— 仅返回当前仍具平台订阅资格的创作者档位。 */
export async function creatorPlanOfRoute(
  request: Request,
  env: Env,
  account: string,
): Promise<Response> {
  await requireSession(request, env);
  const creatorAccount = decodeURIComponent(account);
  const membership = await getMembership(env, creatorAccount);
  if (!membership || !subscriptionIsActive(membership)) return jsonResponse({ plan: null });
  return jsonResponse({ plan: await readPlan(env, creatorAccount) });
}

/** GET /v1/square/creator/overview —— 仅统计链时钟下仍有效的订阅关系。 */
export async function creatorOverviewRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  await requireActiveMembership(env, session.owner_account);
  const owner = session.owner_account;
  const observedAt = nowMs();
  const countRow = await env.DB.prepare(
    `SELECT COUNT(*) AS cnt
      FROM square_creator_subscriptions s
      JOIN chain_clock c ON c.clock_id = 1
      WHERE s.creator_account = ?
        AND s.subscription_status IN ('active', 'cancelled')
        AND c.chain_timestamp < s.paid_until
        AND c.observed_at <= ? AND c.observed_at >= ?`,
  )
    .bind(owner, observedAt, observedAt - CHAIN_CLOCK_MAX_STALENESS_MS)
    .first<{ cnt: number }>();
  const incomeRow = await env.DB.prepare(
    `SELECT COALESCE(SUM(last_charged_price_fen), 0) AS total
      FROM square_creator_subscriptions
      WHERE creator_account = ? AND last_charged_at >= ?`,
  )
    .bind(owner, monthStartMs())
    .first<{ total: number }>();
  const plan = await readPlan(env, owner) as { tiers?: unknown[] } | null;
  return jsonResponse({
    overview: {
      subscriber_count: Number(countRow?.cnt ?? 0),
      month_income_fen: Number(incomeRow?.total ?? 0),
      tier_count: plan?.tiers?.length ?? 0,
    },
  });
}

/** POST /v1/square/creator/plan —— 一次链签名后的 finalized 展示镜像。 */
export async function creatorPlanSaveRoute(
  request: Request,
  env: Env,
  deps: CreatorPlanSaveDeps = defaultCreatorPlanSaveDeps,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<CreatorPlanBody>(request);
  const requested = validateTiers(body.tiers);
  const proof = transactionProof(body);
  const transaction = await deps.verifyTransaction(
    env,
    session.owner_account,
    { kind: "creator_plans_set", tiers: chainTiersFromInput(requested) },
    proof,
  );
  const [chainTiers, platformState] = await Promise.all([
    deps.readCreatorPlansAtBlock(env, session.owner_account, transaction.blockHash),
    deps.readPlatformSubscriptionAtBlock(env, session.owner_account, transaction.blockHash),
  ]);
  const tiers = verifiedDisplayTiers(requested, chainTiers);
  if (!subscriptionStateEffective(platformState, transaction.chainTimestamp)) {
    throw new HttpError(402, "membership_required", "需要有效平台订阅才能开通创作者会员");
  }
  const verifiedAt = nowMs();
  const requestHash = await sha256Hex(JSON.stringify({ action: "set_creator_plans", tiers }));
  await bindFinalizedTransactionConfirmation(
    env,
    session.owner_account,
    transaction,
    requestHash,
    verifiedAt,
  );
  await updateChainClock(env, {
    chainTimestamp: transaction.chainTimestamp,
    blockNumber: transaction.blockNumber,
    blockHash: transaction.blockHash,
    observedAt: verifiedAt,
  });
  await replaceCreatorTiers(env, session.owner_account, tiers, transaction, verifiedAt);
  return jsonResponse({
    plan: {
      creator_account: session.owner_account,
      tiers,
      updated_at: verifiedAt,
    },
  });
}

/** POST /v1/square/creator/subscription/confirm —— finalized 创作者订阅镜像。 */
export async function creatorSubscriptionConfirmRoute(
  request: Request,
  env: Env,
  deps: CreatorSubscriptionConfirmDeps = defaultSubscriptionConfirmDeps,
): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<CreatorConfirmBody>(request);
  const action = creatorAction(body.action);
  const creatorAccount = requireString(body.creator_account, "创作者钱包账户缺失");
  const tierId = action === "cancel" ? null : requireString(body.tier_id, "创作者档位缺失");
  const billingPeriod = action === "cancel" ? null : billingPeriodValue(body.billing_period);
  const expectedAction = expectedCreatorAction(action, creatorAccount, tierId, billingPeriod);
  const proof = transactionProof(body);
  const transaction = await deps.verifyTransaction(
    env,
    session.owner_account,
    expectedAction,
    proof,
  );
  const state = await deps.readSubscriptionAtBlock(
    env,
    session.owner_account,
    creatorAccount,
    transaction.blockHash,
  );
  assertCreatorStateMatches(state, action, tierId, billingPeriod);
  const verifiedAt = nowMs();
  const requestHash = await sha256Hex(
    JSON.stringify({ action, creator_account: creatorAccount, tier_id: tierId, billing_period: billingPeriod }),
  );
  await bindFinalizedTransactionConfirmation(
    env,
    session.owner_account,
    transaction,
    requestHash,
    verifiedAt,
  );
  await updateChainClock(env, {
    chainTimestamp: transaction.chainTimestamp,
    blockNumber: transaction.blockNumber,
    blockHash: transaction.blockHash,
    observedAt: verifiedAt,
  });
  await mirrorCreatorSubscription(
    env,
    session.owner_account,
    creatorAccount,
    state!,
    transaction,
    verifiedAt,
  );
  return jsonResponse({
    ok: true,
    subscription_status: state!.status,
    paid_until: state!.paidUntil,
  });
}

/** 创作者付费内容的统一服务端门禁；未知、陈旧、终止或过期全部拒绝。 */
export async function requireCreatorSubscription(
  env: Env,
  subscriberAccount: string,
  creatorAccount: string,
): Promise<void> {
  const row = await env.DB.prepare(
    `SELECT s.subscription_status, s.paid_until, c.chain_timestamp,
        c.observed_at AS chain_observed_at
      FROM square_creator_subscriptions s
      LEFT JOIN chain_clock c ON c.clock_id = 1
      WHERE s.subscriber_account = ? AND s.creator_account = ?`,
  )
    .bind(subscriberAccount, creatorAccount)
    .first<{
      subscription_status: string;
      paid_until: number;
      chain_timestamp: number | null;
      chain_observed_at: number | null;
    }>();
  if (!row || !isSubscriptionMirrorEffective(row)) {
    throw new HttpError(402, "creator_subscription_required", "需订阅该创作者会员");
  }
}

async function replaceCreatorTiers(
  env: Env,
  creatorAccount: string,
  tiers: CreatorTierInput[],
  transaction: VerifiedFinalizedTransaction,
  verifiedAt: number,
): Promise<void> {
  const statements: D1PreparedStatement[] = [
    env.DB.prepare("DELETE FROM square_creator_tiers WHERE creator_account = ?").bind(creatorAccount),
  ];
  tiers.forEach((tier, index) => {
    statements.push(
      env.DB.prepare(
        `INSERT INTO square_creator_tiers
          (creator_account, tier_id, name, tier_order, monthly_price_fen,
           quarterly_price_fen, yearly_price_fen, finalized_block_number,
           finalized_block_hash, verified_at, last_tx_hash)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      ).bind(
        creatorAccount,
        tier.tier_id,
        tier.name,
        index,
        tier.prices_fen.monthly ?? null,
        tier.prices_fen.quarterly ?? null,
        tier.prices_fen.yearly ?? null,
        transaction.blockNumber,
        transaction.blockHash,
        verifiedAt,
        transaction.txHash,
      ),
    );
  });
  await env.DB.batch(statements);
}

async function mirrorCreatorSubscription(
  env: Env,
  subscriberAccount: string,
  creatorAccount: string,
  state: ChainSubscriptionState,
  transaction: VerifiedFinalizedTransaction,
  verifiedAt: number,
): Promise<void> {
  if (state.plan.kind !== "creator") {
    throw new HttpError(409, "subscription_state_not_finalized", "链上创作者订阅计划不合法");
  }
  const pending = state.pendingPlan?.kind === "creator" ? state.pendingPlan : null;
  await env.DB.prepare(
    `INSERT INTO square_creator_subscriptions
      (subscriber_account, creator_account, tier_id, billing_period,
       pending_tier_id, pending_billing_period, started_at, last_charged_at,
       last_charged_price_fen, paid_until, subscription_status,
       finalized_block_number, finalized_block_hash, verified_at, last_tx_hash)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(subscriber_account, creator_account) DO UPDATE SET
        tier_id = excluded.tier_id,
        billing_period = excluded.billing_period,
        pending_tier_id = excluded.pending_tier_id,
        pending_billing_period = excluded.pending_billing_period,
        started_at = excluded.started_at,
        last_charged_at = excluded.last_charged_at,
        last_charged_price_fen = excluded.last_charged_price_fen,
        paid_until = excluded.paid_until,
        subscription_status = excluded.subscription_status,
        finalized_block_number = excluded.finalized_block_number,
        finalized_block_hash = excluded.finalized_block_hash,
        verified_at = excluded.verified_at,
        last_tx_hash = excluded.last_tx_hash
      WHERE excluded.finalized_block_number >= square_creator_subscriptions.finalized_block_number`,
  )
    .bind(
      subscriberAccount,
      creatorAccount,
      state.plan.tierId,
      state.plan.billingPeriod,
      pending?.tierId ?? null,
      pending?.billingPeriod ?? null,
      state.startedAt,
      state.lastChargedAt,
      safePrice(state.lastChargedPriceFen),
      state.paidUntil,
      state.status,
      transaction.blockNumber,
      transaction.blockHash,
      verifiedAt,
      transaction.txHash,
    )
    .run();
}

function assertCreatorStateMatches(
  state: ChainSubscriptionState | null,
  action: CreatorAction,
  tierId: string | null,
  billingPeriod: BillingPeriod | null,
): void {
  if (state === null || state.plan.kind !== "creator") {
    throw new HttpError(409, "subscription_state_not_finalized", "链上创作者订阅状态尚未最终确认");
  }
  if (action === "cancel") {
    if (state.status !== "cancelled") {
      throw new HttpError(409, "subscription_state_not_finalized", "链上取消状态尚未最终确认");
    }
    return;
  }
  const currentMatches = state.plan.tierId === tierId && state.plan.billingPeriod === billingPeriod;
  const pendingMatches =
    state.pendingPlan?.kind === "creator" &&
    state.pendingPlan.tierId === tierId &&
    state.pendingPlan.billingPeriod === billingPeriod;
  if (state.status !== "active" || (!currentMatches && !pendingMatches)) {
    throw new HttpError(409, "subscription_state_not_finalized", "链上创作者订阅或换档状态尚未最终确认");
  }
}

function expectedCreatorAction(
  action: CreatorAction,
  creatorAccount: string,
  tierId: string | null,
  billingPeriod: BillingPeriod | null,
): SubscriptionBusinessAction {
  if (action === "cancel") return { kind: "creator_cancel", creatorAccount };
  if (!tierId || !billingPeriod) throw new HttpError(400, "invalid_request", "创作者订阅计划缺失");
  return action === "subscribe"
    ? { kind: "creator_subscribe", creatorAccount, tierId, billingPeriod }
    : { kind: "creator_change", creatorAccount, tierId, billingPeriod };
}

function transactionProof(body: CreatorConfirmBody | CreatorPlanBody): FinalizedTransactionProofInput {
  if (
    typeof body.tx_hash !== "string" ||
    typeof body.block_hash !== "string" ||
    typeof body.signed_extrinsic_hex !== "string"
  ) {
    throw new HttpError(400, "invalid_transaction_proof", "finalized 交易证明不完整");
  }
  return {
    txHash: body.tx_hash,
    blockHash: body.block_hash,
    signedExtrinsicHex: body.signed_extrinsic_hex,
  };
}

function creatorAction(value: unknown): CreatorAction {
  if (value === "subscribe" || value === "cancel" || value === "change") return value;
  throw new HttpError(400, "invalid_subscription_action", "创作者订阅操作不合法");
}

function billingPeriodValue(value: unknown): BillingPeriod {
  if (value === "monthly" || value === "quarterly" || value === "yearly") return value;
  throw new HttpError(400, "invalid_billing_period", "创作者订阅周期不合法");
}

function requireString(value: unknown, message: string): string {
  if (typeof value === "string" && value.length > 0) return value;
  throw new HttpError(400, "invalid_request", message);
}

function creatorTiersEqual(left: ChainCreatorTier[], right: ChainCreatorTier[]): boolean {
  return left.length === right.length && left.every((tier, index) => {
    const other = right[index];
    return !!other && tier.tierId === other.tierId && PERIODS.every(
      (period) => tier.pricesFen[period] === other.pricesFen[period],
    );
  });
}

function subscriptionStateEffective(
  state: ChainSubscriptionState | null,
  chainTimestamp: number,
): boolean {
  return !!state &&
    (state.status === "active" || state.status === "cancelled") &&
    chainTimestamp < state.paidUntil;
}

function safePrice(value: bigint): number {
  if (value <= 0n || value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new HttpError(502, "creator_price_out_of_range", "链上创作者价格超出边缘服务范围");
  }
  return Number(value);
}
