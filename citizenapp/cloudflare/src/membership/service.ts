import type { Env, MembershipRow } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';
import { membershipPlan, membershipPlanList, type MembershipLevel } from './plans';

/// 会员与身份彻底解耦（ADR-036）：会员权益只看订阅是否有效（subscriptionIsActive），
/// 不再读链上身份、不再有「身份≠档位」冻结或暂停收款。身份展示由 chain/identity 与
/// profiles 各自负责，会员侧一概不涉身份。

export async function getMembership(env: Env, ownerAccount: string): Promise<MembershipRow | null> {
  return env.DB.prepare(
    `SELECT owner_account, membership_level, expires_at,
        updated_at, subscription_source, stripe_customer_id, stripe_subscription_id, stripe_price_id,
        subscription_status, current_period_start, current_period_end, cancel_at_period_end,
        entitlement_lapsed_at, prepaid_payment_ref
      FROM square_memberships
      WHERE owner_account = ?`
  )
    .bind(ownerAccount)
    .first<MembershipRow>();
}

/// 批量读会员：一页去重作者一条 IN() 查询（≤50 占位符），避免逐作者点查。
export async function batchMemberships(
  env: Env,
  ownerAccounts: string[]
): Promise<Map<string, MembershipRow>> {
  const distinct = [...new Set(ownerAccounts)];
  const map = new Map<string, MembershipRow>();
  if (distinct.length === 0) {
    return map;
  }
  const placeholders = distinct.map(() => '?').join(', ');
  const result = await env.DB.prepare(
    `SELECT owner_account, membership_level, expires_at,
        updated_at, subscription_source, stripe_customer_id, stripe_subscription_id, stripe_price_id,
        subscription_status, current_period_start, current_period_end, cancel_at_period_end,
        entitlement_lapsed_at, prepaid_payment_ref
      FROM square_memberships
      WHERE owner_account IN (${placeholders})`
  )
    .bind(...distinct)
    .all<MembershipRow>();
  for (const row of result.results ?? []) {
    map.set(row.owner_account, row);
  }
  return map;
}

/// 发布闸门（门禁2）：只要求订阅当前有效；解耦后不再校验身份、不再冻结。
export async function requireActiveMembership(
  env: Env,
  ownerAccount: string
): Promise<MembershipRow> {
  const membership = await getMembership(env, ownerAccount);
  if (!membership) {
    throw new HttpError(402, 'membership_required', '需要有效会员才能发布广场内容');
  }
  if (!subscriptionIsActive(membership)) {
    throw new HttpError(402, 'membership_inactive', '会员订阅未生效或已过期');
  }
  // 已移除账户总储存上限维度（对齐 YouTube/推特）：仅校验会员有效，不再核算容量。
  return membership;
}

export async function membershipRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const membership = await getMembership(env, session.owner_account);
  const active = membership ? subscriptionIsActive(membership) : false;
  return jsonResponse({
    ok: true,
    plans: membershipPlanList(),
    membership,
    // 解耦后权益态即订阅态（无身份冻结）；两字段等值，保留 subscription_active 供官网/App 判续订。
    subscription_active: active,
    active
  });
}

export function subscriptionIsActive(membership: MembershipRow): boolean {
  // USDC 预付无 Stripe 状态生命周期，只看到期（ADR-034）。
  if (membership.subscription_source === 'usdc_prepaid') {
    return membership.expires_at > nowMs();
  }
  const status = membership.subscription_status || 'active';
  return (
    (status === 'active' || status === 'trialing') &&
    membership.expires_at > nowMs()
  );
}

const DAY_MS = 86_400_000;

/// 给 [baseMs] 加 [months] 个日历月（按季/年授时长；跨月按自然日历，非固定 30 天）。
function addMonths(baseMs: number, months: number): number {
  const d = new Date(baseMs);
  d.setUTCMonth(d.getUTCMonth() + months);
  return d.getTime();
}

export interface PrepaidGrantInput {
  ownerAccount: string;
  membershipLevel: MembershipLevel;
  months: number;
  stripePayment: StripePaymentGrant;
}

export interface StripePaymentGrant {
  paymentIntentId: string;
  checkoutSessionId: string;
  paymentRoute: 'usdc_prepaid' | 'usdc_prepaid_upgrade';
}

async function stripePaymentAlreadyGranted(env: Env, paymentIntentId: string): Promise<boolean> {
  const row = await env.DB.prepare(
    `SELECT stripe_payment_intent_id
       FROM square_stripe_payments
      WHERE stripe_payment_intent_id = ?`
  )
    .bind(paymentIntentId)
    .first<{ stripe_payment_intent_id: string }>();
  return row !== null;
}

function stripePaymentInsert(
  env: Env,
  input: {
    ownerAccount: string;
    membershipLevel: MembershipLevel;
    stripePayment: StripePaymentGrant;
    grantedAt: number;
  }
): D1PreparedStatement {
  return env.DB.prepare(
    `INSERT INTO square_stripe_payments
      (stripe_payment_intent_id, checkout_session_id, owner_account,
       membership_level, payment_route, granted_at)
     VALUES (?, ?, ?, ?, ?, ?)`
  ).bind(
    input.stripePayment.paymentIntentId,
    input.stripePayment.checkoutSessionId,
    input.ownerAccount,
    input.membershipLevel,
    input.stripePayment.paymentRoute,
    input.grantedAt
  );
}

/// USDC 预付授时长（ADR-034）：expires_at 从 max(now, 现expires_at) 起叠 N 个日历月，
/// 无 stripe_subscription_id、source=usdc_prepaid、status=active；current_period_start
/// 保留首次激活（同路线叠加不覆盖）。权益真源仍是 expires_at（解耦后不再挂身份）。
export async function upsertPrepaidMembership(
  env: Env,
  input: PrepaidGrantInput
): Promise<boolean> {
  const now = nowMs();
  if (await stripePaymentAlreadyGranted(env, input.stripePayment.paymentIntentId)) {
    return false;
  }
  const existing = await getMembership(env, input.ownerAccount);

  // 结算侧兜底守卫（ADR-034 段4）：不同档 USDC 并存时——用户分别签两档挑战、在 confirm→
  // webhook 窗口内两笔购买都过了 confirm 前的档差守卫——不把便宜档的"剩余时长"直贴成贵档
  // （会少收档差、白得高档权益），而是把旧档剩余按**价值**折算成本次档的等值天数追加。
  // 同档续费仍按自然日历月叠加（口径不变）。
  const differentTierActive =
    existing != null &&
    existing.subscription_source === 'usdc_prepaid' &&
    existing.expires_at > now &&
    existing.membership_level !== input.membershipLevel;

  let expiresAt: number;
  if (differentTierActive) {
    const grantMonthly = membershipPlan(input.membershipLevel).price_usd_cents;
    const existingMonthly = membershipPlan(existing!.membership_level).price_usd_cents;
    const existingRemainingDays = Math.max(
      0,
      Math.floor((existing!.expires_at - now) / DAY_MS)
    );
    // 价值守恒：旧档剩余天数 × 旧月费 ÷ 新月费 = 本档等值天数（升档→更少天、降档→更多天）。
    const foldedDays =
      grantMonthly > 0
        ? Math.round((existingRemainingDays * existingMonthly) / grantMonthly)
        : existingRemainingDays;
    expiresAt = addMonths(now, input.months) + foldedDays * DAY_MS;
  } else {
    const base = existing && existing.expires_at > now ? existing.expires_at : now;
    expiresAt = addMonths(base, input.months);
  }

  // 起算点：同档 USDC 续费保留原激活时刻（叠加）；异档折算 = 换到本档，起点重置为现在。
  const periodStart =
    existing &&
    existing.subscription_source === 'usdc_prepaid' &&
    existing.current_period_start &&
    !differentTierActive
      ? existing.current_period_start
      : now;
  const membershipWrite = env.DB.prepare(
    `INSERT INTO square_memberships
      (owner_account, membership_level, expires_at, updated_at, subscription_source,
       stripe_customer_id, stripe_subscription_id, stripe_price_id, subscription_status,
       current_period_start, current_period_end, cancel_at_period_end,
       entitlement_lapsed_at, prepaid_payment_ref)
      VALUES (?, ?, ?, ?, 'usdc_prepaid', NULL, NULL, NULL, 'active', ?, ?, 0, NULL, ?)
      ON CONFLICT(owner_account) DO UPDATE SET
        membership_level = excluded.membership_level,
        expires_at = excluded.expires_at,
        updated_at = excluded.updated_at,
        subscription_source = excluded.subscription_source,
        stripe_subscription_id = NULL,
        stripe_price_id = NULL,
        subscription_status = 'active',
        current_period_start = excluded.current_period_start,
        current_period_end = excluded.current_period_end,
        cancel_at_period_end = 0,
        entitlement_lapsed_at = NULL,
        prepaid_payment_ref = excluded.prepaid_payment_ref`
  )
    .bind(
      input.ownerAccount,
      input.membershipLevel,
      expiresAt,
      now,
      periodStart,
      expiresAt,
      input.stripePayment.paymentIntentId
    );
  try {
    // D1 batch 具备事务语义：付款凭证占位与会员授时长同成同败，重放不能重复延长。
    await env.DB.batch([
      stripePaymentInsert(env, {
        ownerAccount: input.ownerAccount,
        membershipLevel: input.membershipLevel,
        stripePayment: input.stripePayment,
        grantedAt: now
      }),
      membershipWrite
    ]);
    return true;
  } catch (error) {
    // 并发重复事件可能同时通过前置查询；唯一 payment_intent 由 D1 决胜，后来者按幂等成功处理。
    if (await stripePaymentAlreadyGranted(env, input.stripePayment.paymentIntentId)) {
      return false;
    }
    throw error;
  }
}

/// USDC 预付换档落库（ADR-034 段2）：切 level + 设新到期（降档=折算后的新到期、
/// 升档=沿用原到期）；current_period_start 置为本次换档时刻（新档块从现在起算）。
/// 仅在已存在 usdc_prepaid 行上 UPDATE。
export async function applyPrepaidTierChange(
  env: Env,
  input: {
    ownerAccount: string;
    membershipLevel: MembershipLevel;
    expiresAt: number;
    stripePayment?: StripePaymentGrant;
  }
): Promise<boolean> {
  const now = nowMs();
  if (
    input.stripePayment &&
    (await stripePaymentAlreadyGranted(env, input.stripePayment.paymentIntentId))
  ) {
    return false;
  }
  const membershipWrite = env.DB.prepare(
    `UPDATE square_memberships
       SET membership_level = ?, expires_at = ?, current_period_start = ?, current_period_end = ?,
           subscription_source = 'usdc_prepaid', stripe_subscription_id = NULL,
           subscription_status = 'active', cancel_at_period_end = 0,
           updated_at = ?
       WHERE owner_account = ?`
  )
    .bind(
      input.membershipLevel,
      input.expiresAt,
      now,
      input.expiresAt,
      now,
      input.ownerAccount
    );
  if (!input.stripePayment) {
    await membershipWrite.run();
    return true;
  }
  try {
    // 升档付款与切档同一事务落库；重复 payment_intent 永远不会二次切档。
    await env.DB.batch([
      stripePaymentInsert(env, {
        ownerAccount: input.ownerAccount,
        membershipLevel: input.membershipLevel,
        stripePayment: input.stripePayment,
        grantedAt: now
      }),
      membershipWrite
    ]);
    return true;
  } catch (error) {
    if (await stripePaymentAlreadyGranted(env, input.stripePayment.paymentIntentId)) {
      return false;
    }
    throw error;
  }
}

export async function upsertStripeMembership(
  env: Env,
  input: {
    ownerAccount: string;
    membershipLevel: MembershipLevel;
    stripeCustomerId: string | null;
    stripeSubscriptionId: string;
    stripePriceId: string | null;
    subscriptionStatus: string;
    currentPeriodStart: number | null;
    currentPeriodEnd: number;
    cancelAtPeriodEnd: boolean;
    allowPrepaidSwitch: boolean;
  }
): Promise<boolean> {
  const now = nowMs();
  const result = await env.DB.prepare(
    `INSERT INTO square_memberships
      (owner_account, membership_level, expires_at,
        updated_at, subscription_source, stripe_customer_id, stripe_subscription_id,
        stripe_price_id, subscription_status, current_period_start, current_period_end,
        cancel_at_period_end, entitlement_lapsed_at)
      VALUES (?, ?, ?, ?, 'stripe', ?, ?, ?, ?, ?, ?, ?, NULL)
      ON CONFLICT(owner_account) DO UPDATE SET
        membership_level = excluded.membership_level,
        expires_at = excluded.expires_at,
        updated_at = excluded.updated_at,
        subscription_source = excluded.subscription_source,
        stripe_customer_id = excluded.stripe_customer_id,
        stripe_subscription_id = excluded.stripe_subscription_id,
        stripe_price_id = excluded.stripe_price_id,
        subscription_status = excluded.subscription_status,
        current_period_start = excluded.current_period_start,
        current_period_end = excluded.current_period_end,
        cancel_at_period_end = excluded.cancel_at_period_end,
        entitlement_lapsed_at = NULL
      WHERE square_memberships.subscription_source <> 'usdc_prepaid' OR ? = 1`
  )
    .bind(
      input.ownerAccount,
      input.membershipLevel,
      input.currentPeriodEnd,
      now,
      input.stripeCustomerId,
      input.stripeSubscriptionId,
      input.stripePriceId,
      input.subscriptionStatus,
      input.currentPeriodStart,
      input.currentPeriodEnd,
      input.cancelAtPeriodEnd ? 1 : 0,
      input.allowPrepaidSwitch ? 1 : 0
    )
    .run();
  return (result.meta?.changes ?? 0) === 1;
}

export async function markStripeMembershipInactive(
  env: Env,
  stripeSubscriptionId: string,
  status: string
): Promise<void> {
  const now = nowMs();
  await env.DB.prepare(
    `UPDATE square_memberships
      SET subscription_status = ?, expires_at = ?, updated_at = ?,
        entitlement_lapsed_at = COALESCE(entitlement_lapsed_at, ?)
      WHERE stripe_subscription_id = ?`
  )
    .bind(status, now, now, now, stripeSubscriptionId)
    .run();
}
