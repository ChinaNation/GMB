import type { Env, SquareNotifyJob } from '../types';
import { nowMs } from '../shared/time';
import { sendSquarePostAlert, type PushDevice } from '../chat/push';

/// 每页粉丝数：满页则 keyset 续跑下一页（不是丢弃上限，队列消费者跨调用推完全部）。
const FANOUT_PAGE = 100;

interface FollowerRow {
  account_id: string;
  created_at: number;
}

/// 扇出一页：拉一页「未静音粉丝」，取其未过期设备，逐台发可见推送；满页则续跑入队。
/// 分页按 (created_at, account_id) keyset，避免多设备粉丝跨页错位（先分页粉丝，再取设备）。
export async function fanOutPage(
  env: Env,
  job: SquareNotifyJob,
  pageSize: number = FANOUT_PAGE,
): Promise<void> {
  const cursorAt = job.cursor?.created_at ?? 0;
  const cursorAccountId = job.cursor?.account_id ?? '';

  const followers = await env.DB.prepare(
    `SELECT account_id, created_at
       FROM square_follows
      WHERE followed_account_id = ?
        AND notify_enabled = 1
        AND (created_at, account_id) > (?, ?)
      ORDER BY created_at ASC, account_id ASC
      LIMIT ?`,
  )
    .bind(job.author_account_id, cursorAt, cursorAccountId, pageSize)
    .all<FollowerRow>();
  const rows = followers.results ?? [];
  if (rows.length === 0) return;

  const accounts = rows.map((row) => row.account_id);
  const placeholders = accounts.map(() => '?').join(',');
  const devices = await env.DB.prepare(
    `SELECT push_provider, push_token
       FROM chat_devices
      WHERE account_id IN (${placeholders})
        AND expires_at > ?`,
  )
    .bind(...accounts, nowMs())
    .all<PushDevice>();

  const alert = buildAlert(job);
  await Promise.all(
    (devices.results ?? []).map((device) =>
      sendSquarePostAlert(env, device, alert).catch(() => false),
    ),
  );

  // 满页 → 续跑下一页（游标 = 本页末个粉丝）。不满页说明已到末尾，结束。
  if (rows.length >= pageSize) {
    const last = rows[rows.length - 1];
    await env.SQUARE_NOTIFY_QUEUE?.send({
      ...job,
      cursor: { created_at: last.created_at, account_id: last.account_id },
    });
  }
}

function buildAlert(job: SquareNotifyJob): {
  title: string;
  body: string;
  post_id: string;
} {
  const kind = job.content_format === 'article' ? '文章' : '动态';
  const name = job.author_name.trim().length > 0 ? job.author_name.trim() : '你关注的人';
  return { title: name, body: `发布了新${kind}`, post_id: job.post_id };
}
