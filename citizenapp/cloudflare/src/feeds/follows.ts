import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { setFollowNotify } from '../profiles/repository';
import { assertAccountId } from '../shared/ids';
import { nowMs } from '../shared/time';

interface FollowRequest {
  followed_account_id?: unknown;
}

interface NotifyRequest {
  enabled?: unknown;
}

export async function followRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<FollowRequest>(request);
  let followedAccountId: string;
  try {
    followedAccountId = assertAccountId(body.followed_account_id);
  } catch {
    throw new HttpError(400, 'invalid_followed_account_id', '关注账户格式不合法');
  }
  if (followedAccountId === session.account_id) {
    throw new HttpError(400, 'self_follow_forbidden', '不能关注自己');
  }

  await env.DB.prepare(
    `INSERT OR REPLACE INTO square_follows
      (account_id, followed_account_id, created_at)
      VALUES (?, ?, ?)`
  )
    .bind(session.account_id, followedAccountId, nowMs())
    .run();

  return jsonResponse({
    ok: true,
    followed_account_id: followedAccountId
  });
}

export async function unfollowRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const url = new URL(request.url);
  let followedAccountId: string;
  try {
    followedAccountId = assertAccountId(decodeURIComponent(url.pathname.split('/').pop() ?? ''));
  } catch {
    throw new HttpError(400, 'invalid_followed_account_id', '关注账户格式不合法');
  }

  await env.DB.prepare(
    `DELETE FROM square_follows
      WHERE account_id = ? AND followed_account_id = ?`
  )
    .bind(session.account_id, followedAccountId)
    .run();

  return jsonResponse({
    ok: true,
    followed_account_id: followedAccountId
  });
}

/// PUT /v1/square/follows/:account/notify —— 开/关对某关注的发帖通知。
/// 通知归属挂在关注关系上：只有已关注才能设置，未关注返回 409 让客户端提示先关注。
export async function setFollowNotifyRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const url = new URL(request.url);
  const segments = url.pathname.split('/').filter((segment) => segment.length > 0);
  // 路径 .../follows/:account/notify → 账户是 notify 的前一段。
  let followedAccountId: string;
  try {
    followedAccountId = assertAccountId(decodeURIComponent(segments[segments.length - 2] ?? ''));
  } catch {
    throw new HttpError(400, 'invalid_followed_account_id', '关注账户格式不合法');
  }

  const body = await readJson<NotifyRequest>(request);
  if (typeof body.enabled !== 'boolean') {
    throw new HttpError(400, 'invalid_enabled', 'enabled 必须是布尔值');
  }

  const hit = await setFollowNotify(
    env,
    session.account_id,
    followedAccountId,
    body.enabled
  );
  if (!hit) {
    throw new HttpError(409, 'not_following', '请先关注 TA 再设置通知');
  }

  return jsonResponse({
    ok: true,
    followed_account_id: followedAccountId,
    notify_enabled: body.enabled
  });
}
