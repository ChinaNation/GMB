import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { setFollowNotify } from '../profiles/repository';
import { assertOwnerAccount } from '../shared/ids';
import { nowMs } from '../shared/time';

interface FollowRequest {
  followed_account?: unknown;
}

interface NotifyRequest {
  enabled?: unknown;
}

export async function followRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<FollowRequest>(request);
  let followedAccount: string;
  try {
    followedAccount = assertOwnerAccount(body.followed_account);
  } catch {
    throw new HttpError(400, 'invalid_followed_account', '关注账户格式不合法');
  }
  if (followedAccount === session.owner_account) {
    throw new HttpError(400, 'self_follow_forbidden', '不能关注自己');
  }

  await env.DB.prepare(
    `INSERT OR REPLACE INTO square_follows
      (owner_account, followed_account, created_at)
      VALUES (?, ?, ?)`
  )
    .bind(session.owner_account, followedAccount, nowMs())
    .run();

  return jsonResponse({
    ok: true,
    followed_account: followedAccount
  });
}

export async function unfollowRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const url = new URL(request.url);
  const followedAccount = url.pathname.split('/').pop();
  if (!followedAccount) {
    throw new HttpError(400, 'invalid_followed_account', '关注账户格式不合法');
  }

  await env.DB.prepare(
    `DELETE FROM square_follows
      WHERE owner_account = ? AND followed_account = ?`
  )
    .bind(session.owner_account, decodeURIComponent(followedAccount))
    .run();

  return jsonResponse({
    ok: true,
    followed_account: decodeURIComponent(followedAccount)
  });
}

/// PUT /v1/square/follows/:account/notify —— 开/关对某关注的发帖通知。
/// 通知归属挂在关注关系上：只有已关注才能设置，未关注返回 409 让客户端提示先关注。
export async function setFollowNotifyRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const url = new URL(request.url);
  const segments = url.pathname.split('/').filter((segment) => segment.length > 0);
  // 路径 .../follows/:account/notify → 账户是 notify 的前一段。
  const followedAccount = decodeURIComponent(segments[segments.length - 2] ?? '');
  if (!followedAccount) {
    throw new HttpError(400, 'invalid_followed_account', '关注账户格式不合法');
  }

  const body = await readJson<NotifyRequest>(request);
  if (typeof body.enabled !== 'boolean') {
    throw new HttpError(400, 'invalid_enabled', 'enabled 必须是布尔值');
  }

  const hit = await setFollowNotify(
    env,
    session.owner_account,
    followedAccount,
    body.enabled
  );
  if (!hit) {
    throw new HttpError(409, 'not_following', '请先关注 TA 再设置通知');
  }

  return jsonResponse({
    ok: true,
    followed_account: followedAccount,
    notify_enabled: body.enabled
  });
}
