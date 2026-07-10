import type { Env, FeedKind, SquarePostFeedItem, SquarePostRow } from '../types';
import { buildFeedPostItem } from './confirm';
import { resolveAuthorSignals } from '../social/author_signals';

export async function listFeedPosts(
  env: Env,
  feedKind: FeedKind,
  ownerAccount: string | null,
  limit: number
): Promise<SquarePostFeedItem[]> {
  const boundedLimit = Math.min(Math.max(limit, 1), 50);

  if (feedKind === 'campaign') {
    const result = await env.DB.prepare(
      `SELECT post_id, owner_account, cid_number, post_category, content_format, title,
          text, content_hash, storage_receipt_id, chain_block, created_at, post_state
        FROM square_posts
        WHERE post_state = 'published' AND post_category = 'campaign'
        ORDER BY created_at DESC
        LIMIT ?`
    )
      .bind(boundedLimit)
      .all<SquarePostRow>();
    return hydrateFeedItems(env, result.results ?? []);
  }

  if (feedKind === 'following' && !ownerAccount) {
    return [];
  }
  if (feedKind === 'following' && ownerAccount) {
    const result = await env.DB.prepare(
      `SELECT p.post_id, p.owner_account, p.cid_number, p.post_category, p.content_format,
          p.title, p.text, p.content_hash, p.storage_receipt_id, p.chain_block,
          p.created_at, p.post_state
        FROM square_posts p
        INNER JOIN square_follows f
          ON f.followed_account = p.owner_account
        WHERE f.owner_account = ? AND p.post_state = 'published'
        ORDER BY p.created_at DESC
        LIMIT ?`
    )
      .bind(ownerAccount, boundedLimit)
      .all<SquarePostRow>();
    return hydrateFeedItems(env, result.results ?? []);
  }

  const result = await env.DB.prepare(
    `SELECT post_id, owner_account, cid_number, post_category, content_format, title,
        text, content_hash, storage_receipt_id, chain_block, created_at, post_state
      FROM square_posts
      WHERE post_state = 'published'
      ORDER BY created_at DESC
      LIMIT ?`
  )
    .bind(boundedLimit)
    .all<SquarePostRow>();

  return hydrateFeedItems(env, result.results ?? []);
}

async function hydrateFeedItems(
  env: Env,
  rows: SquarePostRow[]
): Promise<SquarePostFeedItem[]> {
  // 本页去重作者统一读链上身份+批量读会员，回填每条帖子作者的徽章信号。
  const [signals, items] = await Promise.all([
    resolveAuthorSignals(env, rows.map((row) => row.owner_account)),
    Promise.all(rows.map((row) => buildFeedPostItem(env, row)))
  ]);
  return items.map((item) => {
    const signal = signals.get(item.owner_account);
    return {
      ...item,
      identity_level: signal?.identity_level ?? 'visitor',
      membership_level: signal?.membership_level ?? null,
      membership_active: signal?.membership_active ?? false
    };
  });
}
