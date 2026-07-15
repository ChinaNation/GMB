import type { Env, MediaAssetRow } from '../types';
import { sanitizeOwnerAccount } from '../storage/r2_keys';
import { deleteProviderAsset } from '../media/cloudflare_assets';
import { getMembership } from '../membership/service';
import { cancelStripeSubscriptionNow } from '../membership/stripe_api';
import { clearOwnerSessions } from '../auth/session_index';
import { closeChatRealtime } from '../chat/realtime';
import { releaseStoredMedia } from '../limits/usage';

export interface PurgeAccountResult {
  stripe_canceled: boolean;
  deleted_media_assets: number;
  deleted_r2_objects: number;
  deleted_rows: number;
}

/// 硬删除某账户在 Cloudflare 的**全部**数据。原则：
/// - Chat 不保存消息或附件；注销先断开连接并删除全部设备路由材料。
/// - 所有包含 A 的账户引用都删除，不保留粉丝、消费记录或影子关联。
/// - Stripe 或媒体提供商失败不得阻塞 Chat 隐私数据硬删除。
export async function purgeAccount(
  env: Env,
  ownerAccount: string
): Promise<PurgeAccountResult> {
  // 1. 先取会员，拿 stripe_subscription_id（删了就取不到了）。
  const membership = await getMembership(env, ownerAccount);

  // 2. Chat 活动连接先关闭；通信元数据和通讯录密文立即硬删除，支付故障不能阻塞隐私删除。
  await closeChatRealtime(env, ownerAccount);
  await env.DB.batch([
    env.DB.prepare(`DELETE FROM chat_keypackages WHERE owner_account = ?`).bind(ownerAccount),
    env.DB.prepare(`DELETE FROM chat_devices WHERE owner_account = ?`).bind(ownerAccount),
    env.DB.prepare(`DELETE FROM chat_device_binding_nonces WHERE owner_account = ?`).bind(ownerAccount),
    env.DB.prepare(`DELETE FROM square_contacts WHERE owner_account = ?`).bind(ownerAccount),
  ]);

  // 3. Stripe 立即退订；Chat 已先删除，失败时账户可用主钥签名再次执行剩余清理。
  let stripeCanceled = false;
  if (membership?.stripe_subscription_id) {
    await cancelStripeSubscriptionNow(env, membership.stripe_subscription_id);
    stripeCanceled = true;
  }

  // 4. Images/Stream：先按 owner 取 provider_asset_id 删 provider 本体，再删 D1 行。
  const mediaRows =
    (
      await env.DB.prepare(
        `SELECT upload_id, post_id, owner_account, media_index, media_kind, provider,
          provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
          declared_duration_seconds, duration_seconds, width, height, error_code,
          created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key
          FROM square_media_assets WHERE owner_account = ?`
      )
        .bind(ownerAccount)
        .all<MediaAssetRow>()
    ).results ?? [];
  for (const row of mediaRows) {
    await deleteProviderAsset(env, row);
  }
  await releaseStoredMedia(env, mediaRows);

  // 5. R2 只清理当前允许的资料、广场和归档对象；Chat 永远不创建 R2 对象。
  const safeOwner = sanitizeOwnerAccount(ownerAccount);
  let deletedR2 = 0;
  deletedR2 += await deleteR2Prefix(env, `profile/${safeOwner}/`);
  deletedR2 += await deleteR2Prefix(env, `square/${safeOwner}/posts/`);
  // 视频冷归档的 R2 冷存原片一并硬删（注销才删；退订只归档不删）。
  deletedR2 += await deleteR2Prefix(env, `archive/${safeOwner}/`);

  // 6. D1 批删账户全部引用。
  const bind = (sql: string) => env.DB.prepare(sql).bind(ownerAccount);
  const results = await env.DB.batch([
    bind(`DELETE FROM square_memberships WHERE owner_account = ?`),
    bind(`DELETE FROM square_uploads WHERE owner_account = ?`),
    bind(`DELETE FROM square_posts WHERE owner_account = ?`),
    bind(`DELETE FROM square_user_signals WHERE owner_account = ?`),
    bind(`DELETE FROM square_media_assets WHERE owner_account = ?`),
    env.DB.prepare(`DELETE FROM square_follows WHERE owner_account = ? OR followed_account = ?`).bind(ownerAccount, ownerAccount),
    bind(`DELETE FROM square_browse_days WHERE owner_account = ?`),
    bind(`DELETE FROM resource_reservations WHERE owner_account = ?`),
    bind(`DELETE FROM resource_usage WHERE owner_account = ?`),
    bind(`DELETE FROM square_request_nonces WHERE owner_account = ?`),
    env.DB.prepare(`DELETE FROM square_rate_windows WHERE rate_key LIKE ?`).bind(`%:owner:${ownerAccount}`),
    bind(`DELETE FROM square_device_subkeys WHERE owner_account = ?`),
    bind(`DELETE FROM square_login_challenges WHERE owner_account = ?`)
  ]);
  const deletedRows = results.reduce((sum, result) => sum + (result.meta?.changes ?? 0), 0);

  // 7. KV：身份缓存 + 该账户全部会话。
  await env.SQUARE_CACHE.delete(`square_identity:${ownerAccount}`);
  await clearOwnerSessions(env, ownerAccount);

  return {
    stripe_canceled: stripeCanceled,
    deleted_media_assets: mediaRows.length,
    deleted_r2_objects: deletedR2,
    deleted_rows: deletedRows
  };
}

/// 翻页硬删除某 R2 前缀下全部对象。
async function deleteR2Prefix(
  env: Env,
  prefix: string
): Promise<number> {
  let deleted = 0;
  let cursor: string | undefined;
  do {
    const listed = await env.SQUARE_MEDIA.list({ prefix, cursor, limit: 1000 });
    const keys = listed.objects.map((object) => object.key);
    if (keys.length > 0) {
      await env.SQUARE_MEDIA.delete(keys);
      deleted += keys.length;
    }
    cursor = listed.truncated ? listed.cursor : undefined;
  } while (cursor);
  return deleted;
}
