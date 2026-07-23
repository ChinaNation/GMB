import type {
  AuthorContentFormat,
  AuthorPostCategory,
  CitizenProfileDoc,
  Env,
  SquarePostFeedItem,
  SquarePostRow,
  UserProfileCounts
} from '../types';
import { buildFeedPostItem } from '../posts/confirm';
import { resolveAuthorSignals } from '../social/author_signals';
import { profileObjectKey } from '../storage/r2_keys';
import { validateUploadBytes } from '../limits/upload';
import { putR2Object } from '../limits/storage';
import { resourceLimit } from '../limits/catalog';

const PROFILE_SCHEMA = 'citizenapp.square.profile.v1' as const;

/// 空资料包默认值。首次访问、从未编辑的账户返回此结构，展示名/签名交由客户端兜底。
export function defaultProfileDoc(accountId: string): CitizenProfileDoc {
  return {
    schema: PROFILE_SCHEMA,
    account_id: accountId,
    display_name: '',
    bio: '',
    avatar_object_key: null,
    avatar_content_hash: null,
    banner_object_key: null,
    banner_content_hash: null,
    updated_at: 0
  };
}

/// 读取 R2 公开资料包；对象不存在或 schema 不合法时返回 null（由调用方决定是否回落默认值）。
export async function readProfileDoc(
  env: Env,
  accountId: string
): Promise<CitizenProfileDoc | null> {
  const object = await env.SQUARE_MEDIA.get(profileObjectKey(accountId));
  if (!object) {
    return null;
  }
  if (object.size > resourceLimit('profile_doc').max_bytes) {
    await object.body.cancel();
    return null;
  }
  try {
    const parsed = JSON.parse(await object.text()) as Partial<CitizenProfileDoc>;
    if (parsed.schema !== PROFILE_SCHEMA) {
      return null;
    }
    return {
      schema: PROFILE_SCHEMA,
      account_id: accountId,
      display_name: typeof parsed.display_name === 'string' ? parsed.display_name : '',
      bio: typeof parsed.bio === 'string' ? parsed.bio : '',
      avatar_object_key: nullableString(parsed.avatar_object_key),
      avatar_content_hash: nullableString(parsed.avatar_content_hash),
      banner_object_key: nullableString(parsed.banner_object_key),
      banner_content_hash: nullableString(parsed.banner_content_hash),
      updated_at: typeof parsed.updated_at === 'number' ? parsed.updated_at : 0
    };
  } catch {
    return null;
  }
}

/// 写入 R2 公开资料包。account_id 由调用方从 session 派生，不接受客户端伪造。
export async function writeProfileDoc(env: Env, doc: CitizenProfileDoc): Promise<void> {
  const bytes = new TextEncoder().encode(JSON.stringify(doc));
  const ticket = await validateUploadBytes({
    resource_key: 'profile_doc',
    bytes,
    content_type: 'application/json',
  });
  await putR2Object(env, profileObjectKey(doc.account_id), bytes, ticket);
}

/// 主页三项计数，全部走 D1 实时聚合。
export async function countUserStats(
  env: Env,
  accountId: string
): Promise<UserProfileCounts> {
  const [following, followers, posts] = await Promise.all([
    countScalar(
      env,
      'SELECT COUNT(*) AS n FROM square_follows WHERE account_id = ?',
      accountId
    ),
    countScalar(
      env,
      'SELECT COUNT(*) AS n FROM square_follows WHERE followed_account_id = ?',
      accountId
    ),
    countScalar(
      env,
      "SELECT COUNT(*) AS n FROM square_posts WHERE account_id = ? AND post_state = 'published'",
      accountId
    )
  ]);
  return { following, followers, posts };
}


/// 当前登录者是否已关注目标账户。未登录 viewer 传 null，直接返回 false。
export async function isFollowing(
  env: Env,
  viewerAccountId: string | null,
  targetAccountId: string
): Promise<boolean> {
  if (!viewerAccountId || viewerAccountId === targetAccountId) {
    return false;
  }
  const row = await env.DB.prepare(
    'SELECT 1 AS n FROM square_follows WHERE account_id = ? AND followed_account_id = ? LIMIT 1'
  )
    .bind(viewerAccountId, targetAccountId)
    .first<{ n: number }>();
  return row !== null;
}

/// 当前登录者是否对目标账户开启发帖通知（= 已关注且未静音）。未登录/自看返回 false。
export async function isNotifying(
  env: Env,
  viewerAccountId: string | null,
  targetAccountId: string
): Promise<boolean> {
  if (!viewerAccountId || viewerAccountId === targetAccountId) {
    return false;
  }
  const row = await env.DB.prepare(
    'SELECT notify_enabled FROM square_follows WHERE account_id = ? AND followed_account_id = ? LIMIT 1'
  )
    .bind(viewerAccountId, targetAccountId)
    .first<{ notify_enabled: number }>();
  return row?.notify_enabled === 1;
}

/// 设置对某关注的发帖通知开关；仅对已存在的关注生效，返回是否命中一条关注记录。
/// 未关注（0 命中）时上层据此提示「先关注」，通知归属永远挂在关注关系上。
export async function setFollowNotify(
  env: Env,
  accountId: string,
  followedAccountId: string,
  enabled: boolean
): Promise<boolean> {
  const result = await env.DB.prepare(
    'UPDATE square_follows SET notify_enabled = ? WHERE account_id = ? AND followed_account_id = ?'
  )
    .bind(enabled ? 1 : 0, accountId, followedAccountId)
    .run();
  return (result.meta.changes ?? 0) > 0;
}

/// 按作者分页拉取已发布帖子；category 过滤 all/normal/campaign，contentFormat 过滤
/// all/normal/article（帖子 Tab 传 normal 排除文章，文章 Tab 传 article），
/// cursor 为上一页最后一条 created_at（keyset 游标）。
export async function listAuthorPosts(
  env: Env,
  accountId: string,
  category: AuthorPostCategory,
  contentFormat: AuthorContentFormat,
  limit: number,
  cursor: number | null
): Promise<SquarePostFeedItem[]> {
  const boundedLimit = Math.min(Math.max(limit, 1), 50);
  const conditions = ["account_id = ?", "post_state = 'published'"];
  const binds: Array<string | number> = [accountId];
  if (category !== 'all') {
    conditions.push('post_category = ?');
    binds.push(category);
  }
  if (contentFormat !== 'all') {
    conditions.push('content_format = ?');
    binds.push(contentFormat);
  }
  if (cursor !== null) {
    conditions.push('created_at < ?');
    binds.push(cursor);
  }
  binds.push(boundedLimit);

  const result = await env.DB.prepare(
    `SELECT post_id, account_id, cid_number, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state
      FROM square_posts
      WHERE ${conditions.join(' AND ')}
      ORDER BY created_at DESC
      LIMIT ?`
  )
    .bind(...binds)
    .all<SquarePostRow>();

  const rows = result.results ?? [];
  // 作者主页所有帖子同一作者：去重后仅读一次链上身份+会员，回填作者徽章信号。
  const [signals, items] = await Promise.all([
    resolveAuthorSignals(env, [accountId]),
    Promise.all(rows.map((row) => buildFeedPostItem(env, row)))
  ]);
  return items.map((item) => {
    const signal = signals.get(item.account_id);
    return {
      ...item,
      identity_level: signal?.identity_level ?? 'visitor',
      membership_level: signal?.membership_level ?? null,
      membership_active: signal?.membership_active ?? false,
      display_name: signal?.display_name ?? '',
      avatar_object_key: signal?.avatar_object_key ?? null
    };
  });
}

export interface FollowEntry {
  account_id: string;
  created_at: number;
}

/// 关注/粉丝列表分页。type='following' 返回该账户关注的人，'followers' 返回关注该账户的人。
/// 均按 created_at 倒序 + keyset 游标。
export async function listFollows(
  env: Env,
  account: string,
  type: 'following' | 'followers',
  limit: number,
  cursor: number | null
): Promise<FollowEntry[]> {
  const boundedLimit = Math.min(Math.max(limit, 1), 50);
  const selectCol = type === 'following' ? 'followed_account_id' : 'account_id';
  const whereCol = type === 'following' ? 'account_id' : 'followed_account_id';
  const binds: Array<string | number> = [account];
  let cursorClause = '';
  if (cursor !== null) {
    cursorClause = ' AND created_at < ?';
    binds.push(cursor);
  }
  binds.push(boundedLimit);

  const result = await env.DB.prepare(
    `SELECT ${selectCol} AS account_id, created_at
      FROM square_follows
      WHERE ${whereCol} = ?${cursorClause}
      ORDER BY created_at DESC
      LIMIT ?`
  )
    .bind(...binds)
    .all<FollowEntry>();
  return result.results ?? [];
}

async function countScalar(env: Env, sql: string, bind: string): Promise<number> {
  const row = await env.DB.prepare(sql).bind(bind).first<{ n: number }>();
  return typeof row?.n === 'number' ? row.n : 0;
}

function nullableString(value: unknown): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null;
}
