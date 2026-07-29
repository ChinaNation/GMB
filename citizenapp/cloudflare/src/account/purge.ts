import type { Env, MediaAssetRow } from '../types';
import { accountIdPathSegment } from '../storage/r2_keys';
import { deleteProviderAsset } from '../media/cloudflare_assets';
import { clearAccountSessions } from '../auth/session_index';
import { closeChatRealtime } from '../chat/realtime';
import { releaseStoredMedia } from '../limits/usage';
import { fetchChainIdentityStateCached } from '../chain/identity';

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
  // 该账户当前绑定的身份主键 cid_number(R2/R3/R4/R5 已切 cid 的对象/表/DO 按它清理)。
  const identity = await fetchChainIdentityStateCached(env, accountId);
  const cidNumber = identity.cid_number;

  // 1. Chat 实时信箱(按 cid 命名的 DO)先关闭；Chat 端到端材料(设备/密钥属账户,按 account_id
  //    删;绑定 nonce 按 cid 归属)立即硬删除，故障不能阻塞隐私删除。通讯录密文按 cid 归属,随
  //    step 4 的 cid 分支删除。
  if (cidNumber) {
    await closeChatRealtime(env, cidNumber);
  }
  const chatDeletes = [
    env.DB.prepare(`DELETE FROM chat_keypackages WHERE account_id = ?`).bind(accountId),
    env.DB.prepare(`DELETE FROM chat_devices WHERE account_id = ?`).bind(accountId),
  ];
  if (cidNumber) {
    chatDeletes.push(
      env.DB.prepare(`DELETE FROM chat_device_binding_nonces WHERE cid_number = ?`).bind(cidNumber)
    );
  }
  await env.DB.batch(chatDeletes);

  // 2. Images/Stream：注销=删身份,按身份主键 cid_number 取该身份**全部**媒体(跨换绑账户),
  //    先删 provider 本体,再删 D1 行。未绑定 CID 的账户不产生任何媒体(上传/发布均需 cid)。
  const mediaRows = cidNumber
    ? (
        await env.DB.prepare(
          `SELECT upload_id, post_id, cid_number, account_id, media_index, media_kind, provider,
            provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
            declared_duration_seconds, duration_seconds, width, height, error_code,
            created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key
            FROM square_media_assets WHERE cid_number = ?`
        )
          .bind(cidNumber)
          .all<MediaAssetRow>()
      ).results ?? []
    : [];
  for (const row of mediaRows) {
    await deleteProviderAsset(env, row);
  }
  await releaseStoredMedia(env, mediaRows);

  // 3. R2 只清理当前允许的资料、广场和归档对象；Chat 永远不创建 R2 对象。
  //    资料包/头像按身份主键 cid 路径(R2 起 profile/{cid_number}/);帖子 manifest 与视频冷归档
  //    原片按发布/上传时的 account_id 路径(不可变内容)。开发期零用户/无换绑历史,当前 account
  //    即唯一发布账户,故按当前段清理完整;跨换绑账户的历史对象前缀清理待生产期身份迁移工具补齐。
  const accountSegment = accountIdPathSegment(accountId);
  let deletedR2 = 0;
  if (cidNumber) {
    deletedR2 += await deleteR2Prefix(env, `profile/${cidNumber}/`);
  }
  deletedR2 += await deleteR2Prefix(env, `square/${accountSegment}/posts/`);
  // 视频冷归档的 R2 冷存原片一并硬删（注销才删；退订只归档不删）。
  deletedR2 += await deleteR2Prefix(env, `archive/${accountSegment}/`);

  // 4. D1 批删。注销=删身份:身份内容与 off-chain/镜像表按身份主键 cid_number 删(删该身份跨
  //    换绑账户的**全部**内容);仅账户级鉴权凭证(登录挑战/设备子钥)按当前 account_id 删。
  const bind = (sql: string) => env.DB.prepare(sql).bind(accountId);
  const statements = [
    // 账户级鉴权凭证按当前 account_id 删(设备子钥/登录挑战是特定账户的登录凭证)。
    bind(`DELETE FROM square_device_subkeys WHERE account_id = ?`),
    bind(`DELETE FROM square_login_challenges WHERE account_id = ?`)
  ];
  if (cidNumber) {
    // 身份内容(发布签名者账户列保留,但注销按 cid 删该身份全部内容)。
    statements.push(
      env.DB.prepare(`DELETE FROM square_uploads WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM square_posts WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM square_media_assets WHERE cid_number = ?`).bind(cidNumber)
    );
    // 身份主键 cid_number 归属的 off-chain / 镜像表(R1/R2/R3/R4/R5 已切)。
    statements.push(
      env.DB.prepare(`DELETE FROM square_contacts WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM square_memberships WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM resource_reservations WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM resource_usage WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM square_creator_tiers WHERE creator_cid_number = ?`).bind(cidNumber),
      env.DB.prepare(
        `DELETE FROM square_creator_subscriptions WHERE subscriber_cid_number = ? OR creator_cid_number = ?`
      ).bind(cidNumber, cidNumber),
      env.DB.prepare(`DELETE FROM square_user_signals WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(
        `DELETE FROM square_follows WHERE follower_cid_number = ? OR followed_cid_number = ?`
      ).bind(cidNumber, cidNumber),
      env.DB.prepare(`DELETE FROM square_browse_days WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM square_notify_reads WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM square_request_nonces WHERE cid_number = ?`).bind(cidNumber),
      env.DB.prepare(`DELETE FROM square_rate_windows WHERE rate_key LIKE ?`).bind(`%:cid_number:${cidNumber}`)
    );
  }
  const results = await env.DB.batch(statements);
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

/// 换绑吊销:删除**旧身份账户**在 Cloudflare 的鉴权敏感数据(Chat 端到端材料、设备子钥、
/// 登录挑战、会话),使换绑(常因私钥泄漏触发)后旧账户无法再重建会话。通讯录密文按身份主键
/// cid 归属、AEAD 密文本身对旧账户已无解密价值,随 CID 迁到新账户,不在吊销删除范围。
///
/// **不删** posts / media / memberships / follows / 通讯录 —— 这些随 CID 迁到新账户(「永不丢失」),
/// 由身份迁移单独处理,不属吊销范围。幂等:重复调用为安全空操作。设备子钥/会话放最后删,
/// 中途失败仍可用旧会话重试;删后旧账户彻底无法再登录。
export async function revokeRebindOldAccount(
  env: Env,
  accountId: string
): Promise<{ deleted_rows: number }> {
  // Chat 实时信箱按身份主键 cid 命名(DO),吊销旧账户时断开其当前连接。
  const identity = await fetchChainIdentityStateCached(env, accountId);
  const cidNumber = identity.cid_number;
  if (cidNumber) {
    await closeChatRealtime(env, cidNumber);
  }
  // 通讯录密文按身份主键 cid 归属,换绑后随身份保留给新账户(数据随 CID 迁移不丢),
  // 故换绑吊销不删 square_contacts;只删旧账户的 chat 端到端材料(设备/密钥属账户按 account_id;
  // 绑定 nonce 按 cid)/登录挑战/设备子钥(账户级)。
  const statements = [
    env.DB.prepare(`DELETE FROM chat_keypackages WHERE account_id = ?`).bind(accountId),
    env.DB.prepare(`DELETE FROM chat_devices WHERE account_id = ?`).bind(accountId),
    env.DB.prepare(`DELETE FROM square_login_challenges WHERE account_id = ?`).bind(accountId),
    // 设备子钥放最后：删前若中途失败，旧会话仍可重试；删后旧账户彻底无法再登录。
    env.DB.prepare(`DELETE FROM square_device_subkeys WHERE account_id = ?`).bind(accountId)
  ];
  if (cidNumber) {
    statements.push(
      env.DB.prepare(`DELETE FROM chat_device_binding_nonces WHERE cid_number = ?`).bind(cidNumber)
    );
  }
  const results = await env.DB.batch(statements);
  const deletedRows = results.reduce((sum, result) => sum + (result.meta?.changes ?? 0), 0);
  await env.SQUARE_CACHE.delete(`square_identity:${accountId}`);
  await clearAccountSessions(env, accountId);
  return { deleted_rows: deletedRows };
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
