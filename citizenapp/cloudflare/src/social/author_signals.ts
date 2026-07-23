import type { Env } from '../types';
import { fetchChainIdentityStateCached, type IdentityLevel } from '../chain/identity';
import { batchMemberships, subscriptionIsActive } from '../membership/service';
import type { MembershipLevel } from '../membership/plans';
import { readProfileDoc } from '../profiles/repository';

/// 帖子作者展示信号（公开）：徽章身份/会员 + 展示名 + 头像对象键。
/// identity_level 是链上身份档（visitor/voting/candidate）；membership_level 是
/// 已购买会员档（freedom/democracy/spark），二者已彻底解耦（ADR-036）。
/// display_name / avatar_object_key 取自作者 profile.json（链下公开资料），供 feed 直出真名和真头像。
export interface AuthorSignals {
  identity_level: IdentityLevel;
  membership_level: MembershipLevel | null;
  membership_active: boolean;
  display_name: string;
  avatar_object_key: string | null;
}

/// 为一页帖子的去重作者集统一解析徽章信号。
///
/// 身份走 [fetchChainIdentityStateCached]（KV 45s 缓存 + 读链失败软降级为访客）对去重作者
/// 并发读；会员用一条 IN() 批量读。与主页 buildProfileResponse 的单作者路径同源，口径一致。
export async function resolveAuthorSignals(
  env: Env,
  accountIds: string[]
): Promise<Map<string, AuthorSignals>> {
  const distinct = [...new Set(accountIds)];
  const map = new Map<string, AuthorSignals>();
  if (distinct.length === 0) {
    return map;
  }
  const [identities, membershipMap, profiles] = await Promise.all([
    Promise.all(distinct.map((accountId) => fetchChainIdentityStateCached(env, accountId))),
    batchMemberships(env, distinct),
    // 去重作者的 profile.json 并行读；缺失（未建资料）软降级为空名 + 无头像。
    Promise.all(distinct.map((accountId) => readProfileDoc(env, accountId).catch(() => null)))
  ]);
  distinct.forEach((accountId, index) => {
    const membership = membershipMap.get(accountId);
    const profile = profiles[index];
    map.set(accountId, {
      identity_level: identities[index].identity_level,
      membership_level: (membership?.membership_level ?? null) as MembershipLevel | null,
      membership_active: membership ? subscriptionIsActive(membership) : false,
      display_name: profile?.display_name ?? '',
      avatar_object_key: profile?.avatar_object_key ?? null
    });
  });
  return map;
}
