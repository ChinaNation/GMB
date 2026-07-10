import type { Env, MediaAssetRow } from '../types';
import { sanitizeOwnerAccount } from '../storage/r2_keys';
import { deleteProviderAsset } from '../media/cloudflare_assets';
import { getMembership } from '../membership/service';
import { cancelStripeSubscriptionNow } from '../membership/stripe_api';
import { clearOwnerSessions } from '../auth/session_index';

export interface PurgeAccountResult {
  stripe_canceled: boolean;
  deleted_media_assets: number;
  deleted_r2_objects: number;
  deleted_rows: number;
}

/// 硬删除某账户在 Cloudflare 的**全部**数据。原则：
/// - 只删 A 自己的：B 的数据（B 收到 A 发的密文、别人关注 A 的关注行）绝不碰。
/// - 顺序：先退 Stripe（失败即中止，绝不「删了库还在扣费」）→ 先删 Images/Stream
///   provider 再删 D1（否则丢 provider_asset_id 成永久孤儿）→ R2 前缀 → D1 → KV。
export async function purgeAccount(
  env: Env,
  ownerAccount: string
): Promise<PurgeAccountResult> {
  // 1. 先取会员，拿 stripe_subscription_id（删了就取不到了）。
  const membership = await getMembership(env, ownerAccount);

  // 2. Stripe 立即退订；失败则抛出、整个 purge 中止，用户可重试，绝不留订阅继续扣费。
  let stripeCanceled = false;
  if (membership?.stripe_subscription_id) {
    await cancelStripeSubscriptionNow(env, membership.stripe_subscription_id);
    stripeCanceled = true;
  }

  // 3. Images/Stream：先按 owner 取 provider_asset_id 删 provider 本体，再删 D1 行。
  const mediaRows =
    (
      await env.DB.prepare(
        `SELECT provider, provider_asset_id FROM square_media_assets WHERE owner_account = ?`
      )
        .bind(ownerAccount)
        .all<Pick<MediaAssetRow, 'provider' | 'provider_asset_id'>>()
    ).results ?? [];
  for (const row of mediaRows) {
    await deleteProviderAsset(env, row);
  }

  // 4. R2 前缀清扫。chat/{A}/ 下「A 发给 B 但 B 未 ack」的附件属于 B，保留。
  const safeOwner = sanitizeOwnerAccount(ownerAccount);
  const survivingRefs =
    (
      await env.DB.prepare(
        `SELECT attachment_manifest_key FROM chat_envelopes
          WHERE sender_account = ? AND recipient_account != ? AND attachment_manifest_key IS NOT NULL`
      )
        .bind(ownerAccount, ownerAccount)
        .all<{ attachment_manifest_key: string }>()
    ).results ?? [];
  const keepPrefixes = new Set(
    survivingRefs.map((ref) => attachmentDirPrefix(ref.attachment_manifest_key))
  );

  let deletedR2 = 0;
  deletedR2 += await deleteR2Prefix(env, `profile/${safeOwner}/`);
  deletedR2 += await deleteR2Prefix(env, `square/${safeOwner}/posts/`);
  deletedR2 += await deleteR2Prefix(env, `chat/${safeOwner}/`, keepPrefixes);

  // 5. D1 批删（只删 A 的）。保留：chat_envelopes(recipient!=A)、square_follows(followed_account=A)。
  const bind = (sql: string) => env.DB.prepare(sql).bind(ownerAccount);
  const results = await env.DB.batch([
    bind(`DELETE FROM square_memberships WHERE owner_account = ?`),
    bind(`DELETE FROM square_uploads WHERE owner_account = ?`),
    bind(`DELETE FROM square_posts WHERE owner_account = ?`),
    bind(`DELETE FROM square_user_signals WHERE owner_account = ?`),
    bind(`DELETE FROM square_media_assets WHERE owner_account = ?`),
    // A 关注别人（owner=A）删；别人关注 A（followed_account=A）是别人的，保留。
    bind(`DELETE FROM square_follows WHERE owner_account = ?`),
    bind(`DELETE FROM chat_devices WHERE owner_account = ?`),
    bind(`DELETE FROM chat_keypackages WHERE owner_account = ?`),
    // 只删 A 的收件箱（recipient=A）；A 发给 B（recipient=B）是 B 的，保留。
    bind(`DELETE FROM chat_envelopes WHERE recipient_account = ?`),
    bind(`DELETE FROM square_device_subkeys WHERE owner_account = ?`),
    bind(`DELETE FROM square_login_challenges WHERE owner_account = ?`)
  ]);
  const deletedRows = results.reduce((sum, result) => sum + (result.meta?.changes ?? 0), 0);

  // 6. KV：身份缓存 + 该账户全部会话。
  await env.FEED_CACHE.delete(`square_identity:${ownerAccount}`);
  await clearOwnerSessions(env, ownerAccount);

  return {
    stripe_canceled: stripeCanceled,
    deleted_media_assets: mediaRows.length,
    deleted_r2_objects: deletedR2,
    deleted_rows: deletedRows
  };
}

/// 附件 manifest key → 其所在附件目录前缀（保留整目录，含分片）。
function attachmentDirPrefix(manifestKey: string): string {
  const lastSlash = manifestKey.lastIndexOf('/');
  return lastSlash >= 0 ? manifestKey.slice(0, lastSlash + 1) : manifestKey;
}

/// 翻页删除某 R2 前缀下全部对象；keepPrefixes 命中的对象跳过（保留 B 的数据）。
async function deleteR2Prefix(
  env: Env,
  prefix: string,
  keepPrefixes?: Set<string>
): Promise<number> {
  const keep = keepPrefixes ? [...keepPrefixes] : [];
  let deleted = 0;
  let cursor: string | undefined;
  do {
    const listed = await env.SQUARE_MEDIA.list({ prefix, cursor, limit: 1000 });
    const keys = listed.objects
      .map((object) => object.key)
      .filter((key) => !keep.some((keepPrefix) => key.startsWith(keepPrefix)));
    if (keys.length > 0) {
      await env.SQUARE_MEDIA.delete(keys);
      deleted += keys.length;
    }
    cursor = listed.truncated ? listed.cursor : undefined;
  } while (cursor);
  return deleted;
}
