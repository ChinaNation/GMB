import type { Env } from '../types';
import { fetchChainIdentityStateByCidCached, type IdentityLevel } from '../chain/identity';
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
/// 三项全部按**身份主键 cid_number** 读:链身份走 [fetchChainIdentityStateByCidCached]
/// (KV 45s 缓存 + 读链失败软降级为访客)、会员镜像一条 IN() 批量读、公开资料 profile.json。
/// **不得按 post 行的 account_id 读链身份** —— 那是发帖当时的签名账户,作者换绑后该账户
/// 已不再绑定任何 CID,会让其历史帖徽章整体降级 visitor,与主页 buildProfileResponse
/// (按 cid 读)自相矛盾。与主页单作者路径同源,口径一致。
export async function resolveAuthorSignals(
  env: Env,
  authors: { cid_number: string; account_id: string }[]
): Promise<Map<string, AuthorSignals>> {
  const map = new Map<string, AuthorSignals>();
  // 按身份主键 cid_number 去重(同一身份多帖只解析一次)。
  const cidList = [...new Set(authors.map((author) => author.cid_number))];
  if (cidList.length === 0) {
    return map;
  }
  const [identities, membershipMap, profiles] = await Promise.all([
    Promise.all(cidList.map((cidNumber) => fetchChainIdentityStateByCidCached(env, cidNumber))),
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
