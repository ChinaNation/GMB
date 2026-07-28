import type { Env } from '../types';
import { jsonResponse, requireSession } from '../shared/http';
import { revokeRebindOldAccount } from '../account/purge';

/// POST /v1/square/rebind/revoke —— 换绑后吊销**旧身份账户**的隐私/鉴权云端数据。
///
/// 鉴权 = 旧账户自己的广场会话(P-256 设备子钥静默登录;request_guard 默认拒已确保有效
/// 会话,且 session.account_id 即被吊销账户,故只能吊销「自己已登录的账户」,无法吊销他人、
/// 也无匿名枚举)。CID 及其通讯录/动态/文章/粉丝随换绑迁到新账户,旧账户的云端敏感副本
/// (通讯录密文 / Chat 材料 / 设备子钥 / 会话)删除后,即便旧私钥泄漏也无法重建旧会话解密
/// 旧通讯录——补上换绑「止损」缺口。幂等。
export async function rebindRevokeRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const deleted = await revokeRebindOldAccount(env, session.account_id);
  return jsonResponse({ ok: true, account_id: session.account_id, deleted });
}
