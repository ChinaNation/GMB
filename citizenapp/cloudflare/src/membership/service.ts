import type { Env, MembershipRow, SessionState } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';
import { fetchChainIdentityState, type ChainIdentityState } from '../chain/identity';
import { putKvJson } from '../limits/storage';
import { pauseStripeCollection, resumeStripeCollection } from './stripe_api';
import {
  identityEligibleForPlan,
  membershipPlan,
  membershipPlanList,
  type MembershipLevel,
  type RequiredIdentityLevel
} from './plans';

/// 冻结判定身份读取的 KV 短缓存 TTL（秒），与展示路径共用同一 square_identity 缓存。
const IDENTITY_FREEZE_CACHE_TTL_SECONDS = 45;
const IDENTITY_LEVELS = new Set(['visitor', 'voting', 'candidate']);

function normalizeIdentityLevel(level: string): RequiredIdentityLevel {
  return (IDENTITY_LEVELS.has(level) ? level : 'visitor') as RequiredIdentityLevel;
}

export async function getMembership(env: Env, ownerAccount: string): Promise<MembershipRow | null> {
  return env.DB.prepare(
    `SELECT owner_account, membership_level, expires_at,
        updated_at, subscription_source, stripe_customer_id, stripe_subscription_id, stripe_price_id,
        subscription_status, current_period_start, current_period_end, cancel_at_period_end,
        identity_level, identity_checked_at, entitlement_lapsed_at,
        frozen_at, collection_paused, prepaid_payment_ref
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
        identity_level, identity_checked_at, entitlement_lapsed_at,
        frozen_at, collection_paused, prepaid_payment_ref
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

export async function requireActiveMembership(
  env: Env,
  ownerAccount: string
): Promise<MembershipRow> {
  const membership = await getMembership(env, ownerAccount);
  if (!membership) {
    throw new HttpError(402, 'membership_required', '需要有效会员才能发布广场内容');
  }
  const effective = await resolveMembershipEntitlement(env, membership);
  // 懒判定同步收款态（冻结即暂停 / 身份重新匹配即恢复）；best-effort，失败不影响拦截。
  try {
    await syncCollectionState(env, membership, effective);
  } catch {
    // 忽略：拦截以 effective.active 为准，下次读重试。
  }
  if (!effective.active) {
    throw new HttpError(402, effective.inactive_code, effective.inactive_message);
  }
  // 已移除账户总储存上限维度（对齐 YouTube/推特）：仅校验会员有效，不再核算容量。
  return membership;
}

export async function membershipRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const membership = await getMembership(env, session.owner_account);
  const identity = await fetchIdentityForDisplay(env, session.owner_account);
  // 展示已读到身份时复用（读失败传 undefined，让 resolve 走安全回退，不误冻）。
  const entitlement = membership
    ? await resolveMembershipEntitlement(
        env,
        membership,
        identity.error ? undefined : identity.state
      )
    : null;
  if (membership && entitlement) {
    // 展示端也顺手同步收款态（幂等，best-effort）。
    try {
      await syncCollectionState(env, membership, entitlement);
    } catch {
      // 忽略：展示不因 Stripe 收款态同步失败而中断。
    }
  }

  return jsonResponse({
    ok: true,
    plans: membershipPlanList(),
    identity: identity.state,
    identity_error: identity.error,
    eligible_levels: eligibleMembershipLevels(identity.state),
    membership,
    subscription_active: membership ? subscriptionIsActive(membership) : false,
    active: entitlement?.active ?? false,
    frozen: entitlement?.frozen ?? false,
    required_identity_level: entitlement?.required_identity_level ?? null,
    actual_identity_level: entitlement?.actual_identity_level ?? null,
    inactive_code: entitlement?.active === false ? entitlement.inactive_code : null,
    inactive_message: entitlement?.active === false ? entitlement.inactive_message : null
  });
}

export interface MembershipEntitlement {
  active: boolean;
  /// true=因链上身份≠会员档位被冻结（区别于普通失效/过期）。
  frozen: boolean;
  inactive_code: string;
  inactive_message: string;
  required_identity_level: RequiredIdentityLevel | null;
  actual_identity_level: RequiredIdentityLevel | null;
}

/// 会员权益判定（ADR-033 规则5）：订阅有效 且 链上身份**精确匹配**会员档所属身份档。
/// 身份不匹配（升/降任一方向）→ 冻结（active=false, frozen=true），供上层拦使用 + 暂停收款。
///
/// [identity] 为「已成功读到」的链上身份（如展示路径已读）时直接复用，省一次读；
/// 未提供时走 identityLevelForFreeze（缓存 + 读失败回退上次已知身份，绝不误冻）。
/// 展示路径务必只在读成功时传入，读失败传 undefined 以走安全回退。
export async function resolveMembershipEntitlement(
  env: Env,
  membership: MembershipRow,
  identity?: ChainIdentityState
): Promise<MembershipEntitlement> {
  if (!subscriptionIsActive(membership)) {
    return {
      active: false,
      frozen: false,
      inactive_code: 'membership_inactive',
      inactive_message: '会员订阅未生效或已过期',
      required_identity_level: null,
      actual_identity_level: null
    };
  }

  const plan = membershipPlan(membership.membership_level);
  const actual = identity
    ? normalizeIdentityLevel(identity.identity_level)
    : await identityLevelForFreeze(env, membership);
  // 精确匹配双向：档位必须恰等身份档；否则冻结（含身份升级导致「会员<身份」）。
  if (actual !== plan.required_identity_level) {
    return {
      active: false,
      frozen: true,
      inactive_code: 'membership_frozen_identity_mismatch',
      inactive_message: '链上身份已变更，会员权益已冻结，请换档到与身份匹配的会员档',
      required_identity_level: plan.required_identity_level,
      actual_identity_level: actual
    };
  }

  return {
    active: true,
    frozen: false,
    inactive_code: '',
    inactive_message: '',
    required_identity_level: plan.required_identity_level,
    actual_identity_level: actual
  };
}

/// 冻结判定专用身份读取：新鲜 KV 缓存命中直接用；否则回链——成功回写缓存，失败回退
/// 「上次已知身份」membership.identity_level（**绝不软降级为访客**，避免瞬时 RPC 抖动误冻
/// 付费用户）。与展示路径共用 square_identity 缓存（缓存只存成功读结果，故读缓存安全）。
async function identityLevelForFreeze(
  env: Env,
  membership: MembershipRow
): Promise<RequiredIdentityLevel> {
  const cacheKey = `square_identity:${membership.owner_account}`;
  try {
    const cached = await env.SQUARE_CACHE.get(cacheKey);
    if (cached) {
      return normalizeIdentityLevel((JSON.parse(cached) as ChainIdentityState).identity_level);
    }
  } catch {
    // 缓存读失败忽略，继续回链。
  }
  try {
    const state = await fetchChainIdentityState(env, membership.owner_account);
    try {
      await putKvJson(env, cacheKey, state, 'identity_cache', {
        expirationTtl: IDENTITY_FREEZE_CACHE_TTL_SECONDS
      });
    } catch {
      // 缓存写失败忽略。
    }
    return state.identity_level;
  } catch {
    // 回链失败：回退上次已知身份，不误冻。
    return normalizeIdentityLevel(membership.identity_level);
  }
}

/// 懒判定双向同步 Stripe 收款态（冻结即暂停、身份重新匹配即恢复），均原子占位
/// （命中 1 行才调 Stripe，防并发/重复读重复调用）；Stripe 失败则回滚标记，下次读重试。
/// 换档到匹配档 / 身份自然恢复后的解冻都走此路径，无需 webhook 或换档链路另做。
async function syncCollectionState(
  env: Env,
  membership: MembershipRow,
  entitlement: MembershipEntitlement
): Promise<void> {
  const subscriptionId = membership.stripe_subscription_id;
  if (!subscriptionId) {
    return;
  }
  // 冻结 且 尚未暂停 → 暂停收款。
  if (entitlement.frozen && !membership.collection_paused) {
    const claimed = await env.DB.prepare(
      `UPDATE square_memberships SET collection_paused = 1, frozen_at = ?
         WHERE owner_account = ? AND collection_paused = 0`
    )
      .bind(nowMs(), membership.owner_account)
      .run();
    if ((claimed.meta?.changes ?? 0) !== 1) {
      return; // 别的请求已占位。
    }
    try {
      await pauseStripeCollection(env, subscriptionId);
    } catch (error) {
      await env.DB.prepare(
        `UPDATE square_memberships SET collection_paused = 0, frozen_at = NULL WHERE owner_account = ?`
      )
        .bind(membership.owner_account)
        .run();
      throw error;
    }
    return;
  }
  // 权益有效（身份已重新匹配）且仍暂停 → 恢复收款、清冻结标记。
  if (entitlement.active && membership.collection_paused) {
    const claimed = await env.DB.prepare(
      `UPDATE square_memberships SET collection_paused = 0, frozen_at = NULL
         WHERE owner_account = ? AND collection_paused = 1`
    )
      .bind(membership.owner_account)
      .run();
    if ((claimed.meta?.changes ?? 0) !== 1) {
      return; // 别的请求已恢复。
    }
    try {
      await resumeStripeCollection(env, subscriptionId);
    } catch (error) {
      await env.DB.prepare(
        `UPDATE square_memberships SET collection_paused = 1, frozen_at = ? WHERE owner_account = ?`
      )
        .bind(nowMs(), membership.owner_account)
        .run();
      throw error;
    }
  }
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
  paymentRef: string | null;
  identity: ChainIdentityState;
}

/// USDC 预付授时长（ADR-034）：expires_at 从 max(now, 现expires_at) 起叠 N 个日历月，
/// 无 stripe_subscription_id、source=usdc_prepaid、status=active；current_period_start
/// 保留首次激活（同路线叠加不覆盖）。权益真源仍是 expires_at + 身份匹配。
export async function upsertPrepaidMembership(
  env: Env,
  input: PrepaidGrantInput
): Promise<void> {
  const now = nowMs();
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
  await env.DB.prepare(
    `INSERT INTO square_memberships
      (owner_account, membership_level, expires_at, updated_at, subscription_source,
       stripe_customer_id, stripe_subscription_id, stripe_price_id, subscription_status,
       current_period_start, current_period_end, cancel_at_period_end,
       identity_level, identity_checked_at, entitlement_lapsed_at, frozen_at, collection_paused,
       prepaid_payment_ref)
      VALUES (?, ?, ?, ?, 'usdc_prepaid', NULL, NULL, NULL, 'active', ?, ?, 0, ?, ?, NULL, NULL, 0, ?)
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
        identity_level = excluded.identity_level,
        identity_checked_at = excluded.identity_checked_at,
        entitlement_lapsed_at = NULL,
        frozen_at = NULL,
        collection_paused = 0,
        prepaid_payment_ref = excluded.prepaid_payment_ref`
  )
    .bind(
      input.ownerAccount,
      input.membershipLevel,
      expiresAt,
      now,
      periodStart,
      expiresAt,
      input.identity.identity_level,
      input.identity.checked_at,
      input.paymentRef
    )
    .run();
}

/// USDC 预付换档落库（ADR-034 段2）：切 level + 设新到期（降档=折算后的新到期、
/// 升档=沿用原到期）；current_period_start 置为本次换档时刻（新档块从现在起算）。
/// 仅在已存在 usdc_prepaid 行上 UPDATE；解冻标记一并清（换到匹配档即解冻）。
export async function applyPrepaidTierChange(
  env: Env,
  input: {
    ownerAccount: string;
    membershipLevel: MembershipLevel;
    expiresAt: number;
    identity: ChainIdentityState;
  }
): Promise<void> {
  const now = nowMs();
  await env.DB.prepare(
    `UPDATE square_memberships
       SET membership_level = ?, expires_at = ?, current_period_start = ?, current_period_end = ?,
           subscription_source = 'usdc_prepaid', stripe_subscription_id = NULL,
           subscription_status = 'active', cancel_at_period_end = 0,
           identity_level = ?, identity_checked_at = ?, frozen_at = NULL, collection_paused = 0,
           updated_at = ?
       WHERE owner_account = ?`
  )
    .bind(
      input.membershipLevel,
      input.expiresAt,
      now,
      input.expiresAt,
      input.identity.identity_level,
      input.identity.checked_at,
      now,
      input.ownerAccount
    )
    .run();
}

/// 可订阅档位：精确匹配本身份档（禁止降档/越级）。visitor 身份 → [freedom,
/// democracy]；voting → [voting]；candidate → [candidate]。
export function eligibleMembershipLevels(identity: ChainIdentityState): MembershipLevel[] {
  return membershipPlanList()
    .filter((plan) => identityEligibleForPlan(identity.identity_level, plan))
    .map((plan) => plan.membership_level);
}

async function fetchIdentityForDisplay(
  env: Env,
  ownerAccount: string
): Promise<{ state: ChainIdentityState; error: string | null }> {
  try {
    return {
      state: await fetchChainIdentityState(env, ownerAccount),
      error: null
    };
  } catch (error) {
    return {
      state: {
        owner_account: ownerAccount,
        identity_level: 'visitor',
        has_voting_identity: false,
        has_candidate_identity: false,
        cid_number: null,
        checked_at: nowMs()
      },
      error: error instanceof Error ? error.message : '链上身份读取失败'
    };
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
    identity: ChainIdentityState;
  }
): Promise<void> {
  const now = nowMs();
  await env.DB.prepare(
    `INSERT INTO square_memberships
      (owner_account, membership_level, expires_at,
        updated_at, subscription_source, stripe_customer_id, stripe_subscription_id,
        stripe_price_id, subscription_status, current_period_start, current_period_end,
        cancel_at_period_end, identity_level, identity_checked_at, entitlement_lapsed_at)
      VALUES (?, ?, ?, ?, 'stripe', ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
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
        identity_level = excluded.identity_level,
        identity_checked_at = excluded.identity_checked_at,
        entitlement_lapsed_at = NULL`
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
      input.identity.identity_level,
      input.identity.checked_at
    )
    .run();
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
