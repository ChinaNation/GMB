import type { Env, MembershipRow, SessionState } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';
import { fetchChainIdentityState, type ChainIdentityState } from '../chain/identity';
import {
  identitySatisfies,
  membershipPlan,
  membershipPlanList,
  type MembershipLevel
} from './plans';

export async function getMembership(env: Env, ownerAccount: string): Promise<MembershipRow | null> {
  return env.DB.prepare(
    `SELECT owner_account, membership_level, storage_quota_bytes, storage_used_bytes, expires_at,
        updated_at, subscription_source, stripe_customer_id, stripe_subscription_id, stripe_price_id,
        subscription_status, current_period_start, current_period_end, cancel_at_period_end,
        identity_level, identity_checked_at
      FROM square_memberships
      WHERE owner_account = ?`
  )
    .bind(ownerAccount)
    .first<MembershipRow>();
}

export async function requireActiveMembership(
  env: Env,
  ownerAccount: string,
  requiredBytes: number
): Promise<MembershipRow> {
  const membership = await getMembership(env, ownerAccount);
  if (!membership) {
    throw new HttpError(402, 'membership_required', '需要有效会员才能使用广场内容存储');
  }
  const effective = await resolveMembershipEntitlement(env, membership);
  if (!effective.active) {
    throw new HttpError(402, effective.inactive_code, effective.inactive_message);
  }

  const remainingBytes = membership.storage_quota_bytes - membership.storage_used_bytes;
  if (requiredBytes > remainingBytes) {
    throw new HttpError(402, 'storage_quota_exceeded', '会员存储容量不足');
  }

  return membership;
}

export async function addStorageUsage(
  env: Env,
  ownerAccount: string,
  usedBytes: number
): Promise<void> {
  await env.DB.prepare(
    `UPDATE square_memberships
      SET storage_used_bytes = storage_used_bytes + ?, updated_at = ?
      WHERE owner_account = ?`
  )
    .bind(usedBytes, nowMs(), ownerAccount)
    .run();
}

export async function membershipRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const membership = await getMembership(env, session.owner_account);
  const identity = await fetchIdentityForDisplay(env, session.owner_account);
  const entitlement = membership
    ? await resolveMembershipEntitlement(env, membership, identity.state)
    : null;

  return jsonResponse({
    ok: true,
    plans: membershipPlanList(),
    identity: identity.state,
    identity_error: identity.error,
    eligible_levels: eligibleMembershipLevels(identity.state),
    membership,
    subscription_active: membership ? subscriptionIsActive(membership) : false,
    active: entitlement?.active ?? false,
    inactive_code: entitlement?.active === false ? entitlement.inactive_code : null,
    inactive_message: entitlement?.active === false ? entitlement.inactive_message : null
  });
}

export function assertSessionOwner(session: SessionState, ownerAccount: string): void {
  if (session.owner_account !== ownerAccount) {
    throw new HttpError(403, 'owner_account_mismatch', '登录钱包与请求钱包不一致');
  }
}

export async function resolveMembershipEntitlement(
  env: Env,
  membership: MembershipRow,
  identity?: ChainIdentityState
): Promise<{
  active: boolean;
  inactive_code: string;
  inactive_message: string;
}> {
  if (!subscriptionIsActive(membership)) {
    return {
      active: false,
      inactive_code: 'membership_inactive',
      inactive_message: '会员订阅未生效或已过期'
    };
  }

  const plan = membershipPlan(membership.membership_level);
  if (plan.required_identity_level === 'visitor') {
    return {
      active: true,
      inactive_code: '',
      inactive_message: ''
    };
  }
  const chainIdentity =
    identity ?? (await fetchChainIdentityState(env, membership.owner_account));
  if (!identitySatisfies(chainIdentity.identity_level, plan.required_identity_level)) {
    return {
      active: false,
      inactive_code: 'membership_identity_required',
      inactive_message: '当前链上身份不满足该会员等级'
    };
  }

  return {
    active: true,
    inactive_code: '',
    inactive_message: ''
  };
}

export function subscriptionIsActive(membership: MembershipRow): boolean {
  const status = membership.subscription_status || 'active';
  return (
    (status === 'active' || status === 'trialing') &&
    membership.expires_at > nowMs()
  );
}

export function eligibleMembershipLevels(identity: ChainIdentityState): MembershipLevel[] {
  return membershipPlanList()
    .filter((plan) => identitySatisfies(identity.identity_level, plan.required_identity_level))
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
  const plan = membershipPlan(input.membershipLevel);
  const now = nowMs();
  await env.DB.prepare(
    `INSERT INTO square_memberships
      (owner_account, membership_level, storage_quota_bytes, storage_used_bytes, expires_at,
        updated_at, subscription_source, stripe_customer_id, stripe_subscription_id,
        stripe_price_id, subscription_status, current_period_start, current_period_end,
        cancel_at_period_end, identity_level, identity_checked_at)
      VALUES (?, ?, ?, 0, ?, ?, 'stripe', ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(owner_account) DO UPDATE SET
        membership_level = excluded.membership_level,
        storage_quota_bytes = excluded.storage_quota_bytes,
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
        identity_checked_at = excluded.identity_checked_at`
  )
    .bind(
      input.ownerAccount,
      input.membershipLevel,
      plan.legacy_storage_quota_bytes,
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
  await env.DB.prepare(
    `UPDATE square_memberships
      SET subscription_status = ?, expires_at = ?, updated_at = ?
      WHERE stripe_subscription_id = ?`
  )
    .bind(status, nowMs(), nowMs(), stripeSubscriptionId)
    .run();
}
