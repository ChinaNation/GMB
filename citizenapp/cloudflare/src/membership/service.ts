import type { Env, MembershipRow } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { membershipPlanList } from './plans';
import { reconcileMembershipForCid } from './reconcile';
import { nowMs } from '../shared/time';

/// 会员状态读取 + 门禁（写入镜像见 `citizen_coin.ts`）。
/// 会员与身份彻底解耦（ADR-036）：会员权益只看订阅是否有效（subscriptionIsActive），
/// 不再读链上身份、不再有「身份≠档位」冻结或暂停收款。身份展示由 chain/identity 与
/// profiles 各自负责，会员侧一概不涉身份。
/// 价格、状态和到期时间都来自 finalized `square-post`；BFF 不计算公历。

const MEMBERSHIP_COLUMNS =
  `m.cid_number, m.account_id, m.membership_level, m.started_at,
    m.last_charged_at, m.last_charged_price_fen, m.paid_until,
    m.subscription_status, m.finalized_block_number, m.finalized_block_hash,
    m.verified_at, m.entitlement_lapsed_at, m.last_tx_hash,
    c.chain_timestamp, c.observed_at AS chain_observed_at`;

/// 链时钟超过三个计划 Cron 周期仍未刷新即拒绝，防止停更镜像无限放行已过期权益。
export const CHAIN_CLOCK_MAX_STALENESS_MS = 15 * 60 * 1000;

/// 会员镜像按身份主键 cid_number 读取(account_id 仅为当前付款账户,不作归属键)。
export async function getMembership(env: Env, cidNumber: string): Promise<MembershipRow | null> {
  return env.DB.prepare(
    `SELECT ${MEMBERSHIP_COLUMNS}
      FROM square_memberships m
      LEFT JOIN chain_clock c ON c.clock_id = 1
      WHERE m.cid_number = ?`
  )
    .bind(cidNumber)
    .first<MembershipRow>();
}

/// 批量读会员：一页去重作者(身份主键 cid_number)一条 IN() 查询（≤50 占位符），避免逐作者点查。
export async function batchMemberships(
  env: Env,
  cidNumbers: string[]
): Promise<Map<string, MembershipRow>> {
  const distinct = [...new Set(cidNumbers)];
  const map = new Map<string, MembershipRow>();
  if (distinct.length === 0) {
    return map;
  }
  const placeholders = distinct.map(() => '?').join(', ');
  const result = await env.DB.prepare(
    `SELECT ${MEMBERSHIP_COLUMNS}
      FROM square_memberships m
      LEFT JOIN chain_clock c ON c.clock_id = 1
      WHERE m.cid_number IN (${placeholders})`
  )
    .bind(...distinct)
    .all<MembershipRow>();
  for (const row of result.results ?? []) {
    map.set(row.cid_number, row);
  }
  return map;
}

export interface MembershipAuthorizationDeps {
  reconcileMembershipForCid: typeof reconcileMembershipForCid;
}

const defaultAuthorizationDeps: MembershipAuthorizationDeps = {
  reconcileMembershipForCid,
};

/**
 * 授权读取先走 D1 快路径；只有当前结果即将拒绝时，才按会话绑定的 CID 在 finalized 链上
 * 点查并重建镜像。App 会员自报绝不进入本函数，链服务异常也不得伪装成“没有会员”。
 */
export async function getMembershipForAuthorization(
  env: Env,
  cidNumber: string,
  accountId: string,
  deps: MembershipAuthorizationDeps = defaultAuthorizationDeps,
): Promise<MembershipRow | null> {
  const current = await getMembership(env, cidNumber);
  if (current && subscriptionIsActive(current)) {
    return current;
  }
  let chainConfirmedPotentiallyActive = false;
  try {
    const state = await deps.reconcileMembershipForCid(env, {
      cidNumber,
      accountId,
    });
    chainConfirmedPotentiallyActive =
      state?.plan.kind === 'platform' &&
      (state.status === 'active' || state.status === 'cancelled');
  } catch (error) {
    throw membershipVerificationUnavailable(error);
  }
  const refreshed = await getMembership(env, cidNumber);
  // finalized 链确认仍可能有效，但较新并发写或未前进的链时钟使 D1 仍无法放行时，属于
  // “暂时无法验证”而不是“没有会员”，继续 fail-closed 并允许客户端重试。
  if (
    chainConfirmedPotentiallyActive &&
    (!refreshed || !subscriptionIsActive(refreshed))
  ) {
    throw membershipVerificationUnavailable(
      new Error('finalized membership mirror remains unavailable'),
    );
  }
  return refreshed;
}

function membershipVerificationUnavailable(error: unknown): HttpError {
  console.error(JSON.stringify({
    event: 'membership_authorization_reconcile_failed',
    error: error instanceof Error ? error.message : String(error),
  }));
  return new HttpError(
    503,
    'membership_verification_unavailable',
    '暂时无法验证会员状态，请稍后重试',
  );
}

/// 发布闸门（门禁2）：只要求订阅当前有效；解耦后不再校验身份、不再冻结。
export async function requireActiveMembership(
  env: Env,
  cidNumber: string,
  accountId: string,
  deps: MembershipAuthorizationDeps = defaultAuthorizationDeps,
): Promise<MembershipRow> {
  const membership = await getMembershipForAuthorization(
    env,
    cidNumber,
    accountId,
    deps,
  );
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
  // 普通头像/资料读取只查镜像；发布前显式要求 verify_on_deny 时才在拒绝路径点查链，
  // 避免无会员用户每次打开页面都产生链 RPC。
  const verifyOnDeny =
    new URL(request.url).searchParams.get('verify_on_deny') === '1';
  const membership = verifyOnDeny
    ? await getMembershipForAuthorization(
        env,
        session.cid_number,
        session.account_id,
      )
    : await getMembership(env, session.cid_number);
  const active = membership ? subscriptionIsActive(membership) : false;
  return jsonResponse({
    ok: true,
    plans: membershipPlanList(),
    membership,
    // 解耦后权益态即订阅态（无身份冻结）；两字段等值，保留 subscription_active 供 App 判续订。
    subscription_active: active,
    active
  });
}

/// Active 或已签名取消但尚在已付周期内的 Cancelled 都有效；终止、过期、无链时钟或时钟陈旧拒绝。
export function subscriptionIsActive(
  membership: MembershipRow,
  observedNow: number = nowMs(),
): boolean {
  return isSubscriptionMirrorEffective({
    subscription_status: membership.subscription_status,
    paid_until: membership.paid_until,
    chain_timestamp: membership.chain_timestamp,
    chain_observed_at: membership.chain_observed_at,
  }, observedNow);
}

export function isSubscriptionMirrorEffective(
  mirror: {
    subscription_status: string;
    paid_until: number;
    chain_timestamp: number | null;
    chain_observed_at: number | null;
  },
  observedNow: number = nowMs(),
): boolean {
  if (mirror.subscription_status !== 'active' && mirror.subscription_status !== 'cancelled') {
    return false;
  }
  if (
    mirror.chain_timestamp === null ||
    mirror.chain_observed_at === null ||
    observedNow < mirror.chain_observed_at ||
    observedNow - mirror.chain_observed_at > CHAIN_CLOCK_MAX_STALENESS_MS
  ) {
    return false;
  }
  return mirror.chain_timestamp < mirror.paid_until;
}
