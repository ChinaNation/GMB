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
import { profileObjectKey } from '../storage/r2_keys';

const PROFILE_SCHEMA = 'citizenapp.square.profile.v1' as const;

/// 空资料包默认值。首次访问、从未编辑的账户返回此结构，展示名/签名交由客户端兜底。
export function defaultProfileDoc(ownerAccount: string): CitizenProfileDoc {
  return {
    schema: PROFILE_SCHEMA,
    owner_account: ownerAccount,
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
  ownerAccount: string
): Promise<CitizenProfileDoc | null> {
  const object = await env.SQUARE_MEDIA.get(profileObjectKey(ownerAccount));
  if (!object) {
    return null;
  }
  try {
    const parsed = JSON.parse(await object.text()) as Partial<CitizenProfileDoc>;
    if (parsed.schema !== PROFILE_SCHEMA) {
      return null;
    }
    return {
      schema: PROFILE_SCHEMA,
      owner_account: ownerAccount,
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

/// 写入 R2 公开资料包。owner_account 由调用方从 session 派生，不接受客户端伪造。
export async function writeProfileDoc(env: Env, doc: CitizenProfileDoc): Promise<void> {
  await env.SQUARE_MEDIA.put(profileObjectKey(doc.owner_account), JSON.stringify(doc), {
    httpMetadata: { contentType: 'application/json; charset=utf-8' }
  });
}

/// 主页三项计数，全部走 D1 实时聚合。
export async function countUserStats(
  env: Env,
  ownerAccount: string
): Promise<UserProfileCounts> {
  const [following, followers, posts] = await Promise.all([
    countScalar(
      env,
      'SELECT COUNT(*) AS n FROM square_follows WHERE owner_account = ?',
      ownerAccount
    ),
    countScalar(
      env,
      'SELECT COUNT(*) AS n FROM square_follows WHERE followed_account = ?',
      ownerAccount
    ),
    countScalar(
      env,
      "SELECT COUNT(*) AS n FROM square_posts WHERE owner_account = ? AND post_state = 'published'",
      ownerAccount
    )
  ]);
  return { following, followers, posts };
}

/// 认证真源：取该账户最近一条已发布帖子携带的 cid_number（confirm 时由链上事件写入）。
/// 从未发帖的账户返回 null（展示为未认证），链上直连 CID 查询留待后续增强。
export async function readLatestCidNumber(
  env: Env,
  ownerAccount: string
): Promise<string | null> {
  const row = await env.DB.prepare(
    `SELECT cid_number FROM square_posts
      WHERE owner_account = ? AND post_state = 'published' AND cid_number IS NOT NULL
      ORDER BY created_at DESC
      LIMIT 1`
  )
    .bind(ownerAccount)
    .first<{ cid_number: string | null }>();
  const cid = row?.cid_number?.trim();
  return cid && cid.length > 0 ? cid : null;
}

/// 当前登录者是否已关注目标账户。未登录 viewer 传 null，直接返回 false。
export async function isFollowing(
  env: Env,
  viewerAccount: string | null,
  targetAccount: string
): Promise<boolean> {
  if (!viewerAccount || viewerAccount === targetAccount) {
    return false;
  }
  const row = await env.DB.prepare(
    'SELECT 1 AS n FROM square_follows WHERE owner_account = ? AND followed_account = ? LIMIT 1'
  )
    .bind(viewerAccount, targetAccount)
    .first<{ n: number }>();
  return row !== null;
}

/// 按作者分页拉取已发布帖子；category 过滤 all/normal/campaign，contentFormat 过滤
/// all/normal/article（帖子 Tab 传 normal 排除文章，文章 Tab 传 article），
/// cursor 为上一页最后一条 created_at（keyset 游标）。
export async function listAuthorPosts(
  env: Env,
  ownerAccount: string,
  category: AuthorPostCategory,
  contentFormat: AuthorContentFormat,
  limit: number,
  cursor: number | null
): Promise<SquarePostFeedItem[]> {
  const boundedLimit = Math.min(Math.max(limit, 1), 50);
  const conditions = ["owner_account = ?", "post_state = 'published'"];
  const binds: Array<string | number> = [ownerAccount];
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
    `SELECT post_id, owner_account, cid_number, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state
      FROM square_posts
      WHERE ${conditions.join(' AND ')}
      ORDER BY created_at DESC
      LIMIT ?`
  )
    .bind(...binds)
    .all<SquarePostRow>();

  const rows = result.results ?? [];
  return Promise.all(rows.map((row) => buildFeedPostItem(env, row)));
}

export interface FollowEntry {
  owner_account: string;
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
  const selectCol = type === 'following' ? 'followed_account' : 'owner_account';
  const whereCol = type === 'following' ? 'owner_account' : 'followed_account';
  const binds: Array<string | number> = [account];
  let cursorClause = '';
  if (cursor !== null) {
    cursorClause = ' AND created_at < ?';
    binds.push(cursor);
  }
  binds.push(boundedLimit);

  const result = await env.DB.prepare(
    `SELECT ${selectCol} AS owner_account, created_at
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
