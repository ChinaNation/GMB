import type { Env } from '../types';
import { fetchChainIdentityStateCached } from '../chain/identity';
import { batchMemberships, subscriptionIsActive } from '../membership/service';
import type { IdentityLevel, MembershipLevel } from '../membership/plans';

/// 帖子作者徽章信号（公开）：身份档=颜色、会员匹配身份档且有效=勾。
/// identity_level 是链上身份档（visitor/voting/candidate）；membership_level 是
/// 已购买会员档（可含 visitor_pro 民主），二者已解耦。
export interface AuthorSignals {
  identity_level: IdentityLevel;
  membership_level: MembershipLevel | null;
  membership_active: boolean;
}

/// 为一页帖子的去重作者集统一解析徽章信号。
///
/// 身份走 [fetchChainIdentityStateCached]（KV 45s 缓存 + 读链失败软降级为访客）对去重作者
/// 并发读；会员用一条 IN() 批量读。与主页 buildProfileResponse 的单作者路径同源，口径一致。
export async function resolveAuthorSignals(
  env: Env,
  ownerAccounts: string[]
): Promise<Map<string, AuthorSignals>> {
  const distinct = [...new Set(ownerAccounts)];
  const map = new Map<string, AuthorSignals>();
  if (distinct.length === 0) {
    return map;
  }
  const [identities, membershipMap] = await Promise.all([
    Promise.all(distinct.map((owner) => fetchChainIdentityStateCached(env, owner))),
    batchMemberships(env, distinct)
  ]);
  distinct.forEach((owner, index) => {
    const membership = membershipMap.get(owner);
    map.set(owner, {
      identity_level: identities[index].identity_level,
      membership_level: (membership?.membership_level ?? null) as MembershipLevel | null,
      membership_active: membership ? subscriptionIsActive(membership) : false
    });
  });
  return map;
}
