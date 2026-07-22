import type { Env } from '../types';
import { HttpError, jsonResponse, readJson, requireSession } from '../shared/http';
import { nowMs } from '../shared/time';

interface ReadRequest {
  scope?: unknown;
}

/// 数某游标之后、我未静音关注（notify_enabled=1）发布的新帖数。
/// 与关注流同源（square_posts JOIN square_follows），只多一层 created_at > 游标 与静音过滤。
async function countUnreadSince(
  env: Env,
  viewer: string,
  since: number
): Promise<number> {
  const row = await env.DB.prepare(
    `SELECT COUNT(*) AS n
       FROM square_posts p
       INNER JOIN square_follows f ON f.followed_account = p.owner_account
      WHERE f.owner_account = ?
        AND f.notify_enabled = 1
        AND p.post_state = 'published'
        AND p.created_at > ?`
  )
    .bind(viewer, since)
    .first<{ n: number }>();
  return row?.n ?? 0;
}

/// GET /v1/square/notify/unread —— 双游标红点计数。
/// square_unread 驱动广场底部 tab，following_unread 驱动关注子 tab；无已读行时游标视为 0。
export async function getNotifyUnreadRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const reads = await env.DB.prepare(
    'SELECT last_seen_square_at, last_seen_following_at FROM square_notify_reads WHERE owner_account = ?'
  )
    .bind(session.owner_account)
    .first<{ last_seen_square_at: number; last_seen_following_at: number }>();

  const squareSince = reads?.last_seen_square_at ?? 0;
  const followingSince = reads?.last_seen_following_at ?? 0;

  const [squareUnread, followingUnread] = await Promise.all([
    countUnreadSince(env, session.owner_account, squareSince),
    countUnreadSince(env, session.owner_account, followingSince)
  ]);

  return jsonResponse({
    ok: true,
    square_unread: squareUnread,
    following_unread: followingUnread
  });
}

/// POST /v1/square/notify/read {scope:'square'|'following'} —— 推进对应游标到当前时间，红点归零。
/// scope 只映射到固定列名，无注入面；另一游标保持不变（进广场只清广场，关注仍留）。
export async function markNotifyReadRoute(request: Request, env: Env): Promise<Response> {
  const session = await requireSession(request, env);
  const body = await readJson<ReadRequest>(request);
  if (body.scope !== 'square' && body.scope !== 'following') {
    throw new HttpError(400, 'invalid_scope', 'scope 必须是 square 或 following');
  }

  const now = nowMs();
  const column =
    body.scope === 'square' ? 'last_seen_square_at' : 'last_seen_following_at';
  await env.DB.prepare(
    `INSERT INTO square_notify_reads (owner_account, ${column})
       VALUES (?, ?)
     ON CONFLICT(owner_account) DO UPDATE SET ${column} = excluded.${column}`
  )
    .bind(session.owner_account, now)
    .run();

  return jsonResponse({ ok: true, scope: body.scope, last_seen_at: now });
}
