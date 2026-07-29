import type { Env } from '../types';
import { deletePostCloudflareDataByCid } from '../posts/confirm';

// 权益到期清理只接受同一轮订阅对账读取的 finalized 区块时间戳。
// 禁止使用 Worker/设备墙钟，也不保留“退订满 N 天”之类的第二触发时钟。
const MAX_IDENTITIES_PER_SWEEP = 20;
const MAX_CONTENT_ITEMS_PER_SWEEP = 100;

interface ExpiredIdentityRow {
  cid_number: string;
}

interface ContentItemRow {
  post_id: string;
}

export interface ExpiredMembershipCleanupResult {
  identity_count: number;
  deleted_content_items: number;
  failed_content_items: number;
}

/**
 * 删除权益已经在 finalized 链时间到期的身份所拥有的全部广场云端内容。
 *
 * 每个内容项先删除 Images/Stream/R2，全部成功后才事务删除 D1 索引；失败项保留
 * D1 行供下一轮继续定位。Cloudflare provider 的 404 和 R2 重复删除均为幂等成功。
 */
export async function runExpiredMembershipContentCleanup(
  env: Env,
  finalizedChainTimestamp: number,
): Promise<ExpiredMembershipCleanupResult> {
  if (!Number.isSafeInteger(finalizedChainTimestamp) || finalizedChainTimestamp < 0) {
    throw new Error('finalized chain timestamp is invalid');
  }

  const identities = await selectExpiredIdentities(
    env,
    finalizedChainTimestamp,
    MAX_IDENTITIES_PER_SWEEP,
  );
  let remaining = MAX_CONTENT_ITEMS_PER_SWEEP;
  let deleted = 0;
  let failed = 0;

  for (const identity of identities) {
    if (remaining <= 0) break;
    const items = await selectContentItems(env, identity.cid_number, remaining);
    remaining -= items.length;
    for (const item of items) {
      try {
        await deletePostCloudflareDataByCid(
          env,
          identity.cid_number,
          item.post_id,
          finalizedChainTimestamp,
        );
        deleted += 1;
      } catch (error) {
        failed += 1;
        console.error(JSON.stringify({
          event: 'expired_membership_content_cleanup_failed',
          cid_number: identity.cid_number,
          post_id: item.post_id,
          error: error instanceof Error ? error.message : String(error),
        }));
      }
    }
  }

  return {
    identity_count: identities.length,
    deleted_content_items: deleted,
    failed_content_items: failed,
  };
}

async function selectExpiredIdentities(
  env: Env,
  finalizedChainTimestamp: number,
  limit: number,
): Promise<ExpiredIdentityRow[]> {
  const rows = await env.DB.prepare(
    `SELECT m.cid_number
       FROM square_memberships m
      WHERE m.subscription_status IN ('cancelled', 'terminated')
        AND m.paid_until <= ?
        AND (
          EXISTS (SELECT 1 FROM square_posts p WHERE p.cid_number = m.cid_number)
          OR EXISTS (SELECT 1 FROM square_uploads u WHERE u.cid_number = m.cid_number)
        )
      ORDER BY m.paid_until ASC, m.cid_number ASC
      LIMIT ?`,
  )
    .bind(finalizedChainTimestamp, limit)
    .all<ExpiredIdentityRow>();
  return rows.results ?? [];
}

async function selectContentItems(
  env: Env,
  cidNumber: string,
  limit: number,
): Promise<ContentItemRow[]> {
  const rows = await env.DB.prepare(
    `SELECT post_id
       FROM (
         SELECT post_id FROM square_posts WHERE cid_number = ?
         UNION
         SELECT post_id FROM square_uploads WHERE cid_number = ?
       )
      ORDER BY post_id ASC
      LIMIT ?`,
  )
    .bind(cidNumber, cidNumber, limit)
    .all<ContentItemRow>();
  return rows.results ?? [];
}
