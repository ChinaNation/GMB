import type { Env, MembershipRow } from '../types';
import { HttpError, jsonResponse, requireSession } from '../shared/http';
import { membershipPlanList } from './plans';

/// 会员状态读取 + 门禁（写入镜像见 `citizen_coin.ts`）。
/// 会员与身份彻底解耦（ADR-036）：会员权益只看订阅是否有效（subscriptionIsActive），
/// 不再读链上身份、不再有「身份≠档位」冻结或暂停收款。身份展示由 chain/identity 与
/// profiles 各自负责，会员侧一概不涉身份。
/// 价格与扣款真源是链上 `square-post`；到期时间由 CitizenApp 计算并上链，BFF 只镜像 finalized 订阅态。

const MEMBERSHIP_COLUMNS =
  `owner_account, membership_level, expires_at, updated_at, subscription_status,
    current_period_start, current_period_end, entitlement_lapsed_at, last_tx_hash`;

export async function getMembership(env: Env, ownerAccount: string): Promise<MembershipRow | null> {
  return env.DB.prepare(
    `SELECT ${MEMBERSHIP_COLUMNS} FROM square_memberships WHERE owner_account = ?`
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
    `SELECT ${MEMBERSHIP_COLUMNS} FROM square_memberships WHERE owner_account IN (${placeholders})`
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
    // 解耦后权益态即订阅态（无身份冻结）；两字段等值，保留 subscription_active 供 App 判续订。
    subscription_active: active,
    active
  });
}

/// 会员是否有效：镜像订阅态为 active 即放行。按月自动续扣发生在链上，客户端确认之间
/// 镜像不应按 expires_at 假过期误拦（与创作者门禁 `requireCreatorSubscription` 同口径）。
export function subscriptionIsActive(membership: MembershipRow): boolean {
  return membership.subscription_status === 'active';
}
