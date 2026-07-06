import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { assertOwnerAccount } from '../shared/ids';
import { nowMs } from '../shared/time';

interface FollowRequest {
  followed_account?: unknown;
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
