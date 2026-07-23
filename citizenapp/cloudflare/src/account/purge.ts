import type { Env, MediaAssetRow } from '../types';
import { accountIdPathSegment } from '../storage/r2_keys';
import { deleteProviderAsset } from '../media/cloudflare_assets';
import { clearAccountSessions } from '../auth/session_index';
import { closeChatRealtime } from '../chat/realtime';
import { releaseStoredMedia } from '../limits/usage';

export interface PurgeAccountResult {
  deleted_media_assets: number;
  deleted_r2_objects: number;
  deleted_rows: number;
}

/// 硬删除某账户在 Cloudflare 的**全部**数据。原则：
/// - Chat 不保存消息或附件；注销先断开连接并删除全部设备路由材料。
/// - 所有包含 A 的账户引用都删除，不保留粉丝、消费记录或影子关联。
/// - 会员订阅与注销解耦：注销只删本地数据，不代签链上退订；链上订阅由用户自行取消或欠费即停。
/// - 媒体提供商失败不得阻塞 Chat 隐私数据硬删除。
export async function purgeAccount(
  env: Env,
  accountId: string
): Promise<PurgeAccountResult> {
  // 1. Chat 活动连接先关闭；通信元数据和通讯录密文立即硬删除，故障不能阻塞隐私删除。
  await closeChatRealtime(env, accountId);
  await env.DB.batch([
    env.DB.prepare(`DELETE FROM chat_keypackages WHERE account_id = ?`).bind(accountId),
    env.DB.prepare(`DELETE FROM chat_devices WHERE account_id = ?`).bind(accountId),
    env.DB.prepare(`DELETE FROM chat_device_binding_nonces WHERE account_id = ?`).bind(accountId),
    env.DB.prepare(`DELETE FROM square_contacts WHERE account_id = ?`).bind(accountId),
  ]);

  // 2. Images/Stream：先按 account_id 取 provider_asset_id 删 provider 本体，再删 D1 行。
  const mediaRows =
    (
      await env.DB.prepare(
        `SELECT upload_id, post_id, account_id, media_index, media_kind, provider,
          provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
          declared_duration_seconds, duration_seconds, width, height, error_code,
          created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key
          FROM square_media_assets WHERE account_id = ?`
      )
        .bind(accountId)
        .all<MediaAssetRow>()
    ).results ?? [];
  for (const row of mediaRows) {
    await deleteProviderAsset(env, row);
  }
  await releaseStoredMedia(env, mediaRows);

  // 3. R2 只清理当前允许的资料、广场和归档对象；Chat 永远不创建 R2 对象。
  const accountSegment = accountIdPathSegment(accountId);
  let deletedR2 = 0;
  deletedR2 += await deleteR2Prefix(env, `profile/${accountSegment}/`);
  deletedR2 += await deleteR2Prefix(env, `square/${accountSegment}/posts/`);
  // 视频冷归档的 R2 冷存原片一并硬删（注销才删；退订只归档不删）。
  deletedR2 += await deleteR2Prefix(env, `archive/${accountSegment}/`);

  // 4. D1 批删账户全部引用。
  const bind = (sql: string) => env.DB.prepare(sql).bind(accountId);
  const results = await env.DB.batch([
    bind(`DELETE FROM square_memberships WHERE account_id = ?`),
    bind(`DELETE FROM square_uploads WHERE account_id = ?`),
    bind(`DELETE FROM square_posts WHERE account_id = ?`),
    bind(`DELETE FROM square_user_signals WHERE account_id = ?`),
    bind(`DELETE FROM square_media_assets WHERE account_id = ?`),
    env.DB.prepare(`DELETE FROM square_follows WHERE account_id = ? OR followed_account_id = ?`).bind(accountId, accountId),
    bind(`DELETE FROM square_browse_days WHERE account_id = ?`),
    bind(`DELETE FROM resource_reservations WHERE account_id = ?`),
    bind(`DELETE FROM resource_usage WHERE account_id = ?`),
    bind(`DELETE FROM square_request_nonces WHERE account_id = ?`),
    env.DB.prepare(`DELETE FROM square_rate_windows WHERE rate_key LIKE ?`).bind(`%:account_id:${accountId}`),
    bind(`DELETE FROM square_device_subkeys WHERE account_id = ?`),
    bind(`DELETE FROM square_login_challenges WHERE account_id = ?`)
  ]);
  const deletedRows = results.reduce((sum, result) => sum + (result.meta?.changes ?? 0), 0);

  // 5. KV：身份缓存 + 该账户全部会话。
  await env.SQUARE_CACHE.delete(`square_identity:${accountId}`);
  await clearAccountSessions(env, accountId);

  return {
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
