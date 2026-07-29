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

/// 为一页帖子的去重作者集统一解析徽章信号,返回 Map(键 = 身份主键 cid_number)。
///
/// 身份走 [fetchChainIdentityStateCached]（KV 45s 缓存 + 读链失败软降级为访客）按作者
/// **当前绑定钱包账户** account_id 并发读；会员镜像同按 account_id 一条 IN() 批量读;
/// 公开资料 profile.json 按**身份主键 cid_number** 读(换绑不丢)。与主页 buildProfileResponse
/// 的单作者路径同源，口径一致。入参每项含 (cid_number, 当前 account_id),二者来自 post 行两列。
export async function resolveAuthorSignals(
  env: Env,
  authors: { cid_number: string; account_id: string }[]
): Promise<Map<string, AuthorSignals>> {
  const map = new Map<string, AuthorSignals>();
  // 按身份主键 cid_number 去重(同一身份多帖只解析一次);值取该身份当前绑定账户。
  const distinct = new Map<string, string>();
  for (const author of authors) {
    if (!distinct.has(author.cid_number)) {
      distinct.set(author.cid_number, author.account_id);
    }
  }
  if (distinct.size === 0) {
    return map;
  }
  const cidList = [...distinct.keys()];
  const accountList = [...distinct.values()];
  const [identities, membershipMap, profiles] = await Promise.all([
    // 链身份按当前绑定账户读(链查入口 = account_id);会员镜像与资料均按身份主键 cid_number 读。
    Promise.all(accountList.map((accountId) => fetchChainIdentityStateCached(env, accountId))),
    batchMemberships(env, cidList),
    // 去重作者的 profile.json 并行读；缺失（未建资料）软降级为空名 + 无头像。
    Promise.all(cidList.map((cidNumber) => readProfileDoc(env, cidNumber).catch(() => null)))
  ]);
  cidList.forEach((cidNumber, index) => {
    const membership = membershipMap.get(cidNumber);
    const profile = profiles[index];
    map.set(cidNumber, {
      identity_level: identities[index].identity_level,
      membership_level: (membership?.membership_level ?? null) as MembershipLevel | null,
      membership_active: membership ? subscriptionIsActive(membership) : false,
      display_name: profile?.display_name ?? '',
      avatar_object_key: profile?.avatar_object_key ?? null
    });
  });
  return map;
}
