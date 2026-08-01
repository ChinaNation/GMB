import type { Env, MediaAssetRow } from '../types';
import { uploadObjectKeys } from '../storage/r2_keys';
import { deleteProviderAsset } from '../media/cloudflare_assets';
import {
  clearIdentityAccountSessions,
  clearIdentitySessions
} from '../auth/session_index';
import { closeChatRealtime } from '../chat/realtime';
import { storedMediaReleaseStatements } from '../limits/usage';
import { HttpError } from '../shared/http';
import { nowMs } from '../shared/time';

export interface PurgeIdentityResult {
  deleted_media_assets: number;
  deleted_r2_objects: number;
  deleted_rows: number;
}

interface PurgeUploadRow {
  upload_id: string;
  post_id: string;
  cid_number: string;
  account_id: string;
  object_keys_json: string;
}

/// 按唯一身份主键 CID 硬删除其在 Cloudflare 中可清除的身份、社交、鉴权、会话和媒体数据。
/// [authorizationAccountId] 只表示完成本次注销签名的当前链上绑定账户，不参与业务数据归属。
/// 边界：
/// - Chat 不保存消息或附件；注销先断开连接并删除全部设备路由材料。
/// - 身份内容、设备、会话和 off-chain 关系全部按 cid_number 删除。
/// - finalized 交易最小证明与充值订单都带 CID 归属并随身份完整删除；链上原始交易事实
///   仍由公链保存，不以 D1 台账残留为审计前提。
/// - 会员订阅与注销解耦：注销只删本地数据，不代签链上退订；链上订阅由用户自行取消或欠费即停。
/// - 媒体提供商失败不得阻塞 Chat 隐私数据硬删除。
export async function purgeIdentity(
  env: Env,
  cidNumber: string,
  authorizationAccountId: string
): Promise<PurgeIdentityResult> {
  // 1. Chat 实时信箱、设备名册、KeyPackage 和防重放材料均属于 CID；换绑前后全部清除。
  await closeChatRealtime(env, cidNumber);
  await env.DB.batch([
    env.DB.prepare(`DELETE FROM chat_keypackages WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM chat_devices WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM chat_device_binding_nonces WHERE cid_number = ?`).bind(cidNumber)
  ]);

  // 2. 先完整读取并校验该 CID 的上传对象索引；清单损坏或已发布帖缺上传索引时 fail-closed，
  //    禁止先删 R2/D1 后丢失继续清理所需的唯一对象事实。
  const uploads = (
    await env.DB.prepare(
      `SELECT upload_id, post_id, cid_number, account_id, object_keys_json
        FROM square_uploads WHERE cid_number = ?`
    )
      .bind(cidNumber)
      .all<PurgeUploadRow>()
  ).results ?? [];
  const postWithoutUpload = await env.DB.prepare(
    `SELECT p.post_id
      FROM square_posts p
      LEFT JOIN square_uploads u ON u.post_id = p.post_id AND u.cid_number = p.cid_number
      WHERE p.cid_number = ? AND u.upload_id IS NULL
      LIMIT 1`
  )
    .bind(cidNumber)
    .first<{ post_id: string }>();
  if (postWithoutUpload) {
    throw new HttpError(409, 'identity_upload_index_missing', '身份内容缺少上传对象索引');
  }
  const postObjectKeys = [
    ...new Set(uploads.flatMap((upload) => uploadObjectKeys(upload)))
  ];

  // 3. Images/Stream：注销=删身份,按身份主键 cid_number 取该身份**全部**媒体(跨换绑账户),
  //    先删 provider 本体,再删 D1 行。未绑定 CID 的账户不产生任何媒体(上传/发布均需 cid)。
  const mediaRows = (
    await env.DB.prepare(
      `SELECT upload_id, post_id, cid_number, account_id, media_index, media_kind, provider,
        provider_asset_id, upload_method, resource_key, content_type, byte_size, asset_state,
        declared_duration_seconds, duration_seconds, width, height, error_code,
        created_at, updated_at, ready_at
        FROM square_media_assets WHERE cid_number = ?`
    )
      .bind(cidNumber)
      .all<MediaAssetRow>()
  ).results ?? [];
  for (const row of mediaRows) {
    await deleteProviderAsset(env, row);
  }

  // 4. R2：资料按 CID 前缀；帖子只按上方已严格验证的 D1 对象清单精确删除，覆盖历次换绑
  //    账户的发布路径。禁止按当前账户前缀猜测，也不保留生产期迁移工具兜底。
  for (const objectKey of postObjectKeys) {
    await env.SQUARE_MEDIA.delete(objectKey);
  }
  const deletedProfileObjects = await deleteR2Prefix(env, `profile/${cidNumber}/`);
  const deletedR2 = postObjectKeys.length + deletedProfileObjects;

  // 5. KV：注销按 CID 失效历次换绑账户签发的全部会话和 CID 资料缓存。当前账户短缓存
  //    只因它是本次授权账户而清理，不用于推导身份数据范围。
  await clearIdentitySessions(env, cidNumber);
  await env.SQUARE_CACHE.delete(`square_identity_cid:${cidNumber}`);
  await env.SQUARE_CACHE.delete(`square_identity:${authorizationAccountId}`);

  // 6. D1 原子批删。存储总量回收必须和媒体/内容行删除同批提交：任一语句失败时
  //    D1 整批回滚，重试仍能从媒体行重建同一释放量；成功后媒体行已删除，后续重试
  //    不会再次扣减全局 resource_totals。
  //    所有有 CID 归属的身份、内容、关系、设备与用量数据均按 CID 删除。登录挑战
  //    记录 CID 归属，必须覆盖历次换绑账户，不能只删当前授权账户。
  const deletedAt = nowMs();
  const statements = [
    ...storedMediaReleaseStatements(env, mediaRows, deletedAt),
    env.DB.prepare(`DELETE FROM square_device_subkeys WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM square_sessions WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM square_login_challenges WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM square_uploads WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM square_posts WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM square_media_assets WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM square_contacts WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM square_memberships WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM chain_transaction_confirmations WHERE cid_number = ?`).bind(cidNumber),
    env.DB.prepare(`DELETE FROM topup_orders WHERE cid_number = ?`).bind(cidNumber),
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
  ];
  const results = await env.DB.batch(statements);
  const deletedRows = results.reduce((sum, result) => sum + (result.meta?.changes ?? 0), 0);

  return {
    deleted_media_assets: mediaRows.length,
    deleted_r2_objects: deletedR2,
    deleted_rows: deletedRows
  };
}

/// finalized CID 绑定版本推进后，删除**旧身份账户**在 Cloudflare 的鉴权敏感数据
/// (Chat 端到端材料、设备子钥、登录挑战、会话)，使已解绑账户无法再建立旧鉴权上下文。
/// 本 helper 不接收 HTTP 请求或旧账户签名；后续只能由可信 finalized 链事件消费者调用。
///
/// **不删** posts / media / memberships / follows / 通讯录 —— 这些随 CID 迁到新账户(「永不丢失」),
/// 由身份迁移单独处理,不属吊销范围。也不关闭按 CID 命名的实时 DO、不删 CID 级
/// `chat_device_binding_nonces`,否则会误伤换绑后的新账户连接和设备状态。幂等:重复调用为
/// 安全空操作。
export async function purgeFinalizedOldAccountCredentials(
  env: Env,
  cidNumber: string,
  accountId: string
): Promise<{ deleted_rows: number }> {
  // 通讯录密文、实时 DO 与绑定 nonce 均按 cid_number 归属，换绑后继续由新账户使用；
  // 本函数只按“旧 CID + 旧 account_id”交集删除账户级材料。旧账户释放后可能已绑定
  // 另一 CID，禁止延迟到达的旧版本事件误删其新身份凭证。
  const statements = [
    env.DB.prepare(
      `DELETE FROM chat_keypackages WHERE cid_number = ? AND account_id = ?`
    ).bind(cidNumber, accountId),
    env.DB.prepare(
      `DELETE FROM chat_devices WHERE cid_number = ? AND account_id = ?`
    ).bind(cidNumber, accountId),
    env.DB.prepare(
      `DELETE FROM square_login_challenges WHERE cid_number = ? AND account_id = ?`
    ).bind(cidNumber, accountId),
    // 设备子钥放最后；finalized 事件消费者重放同一版本事件时仍保持幂等。
    env.DB.prepare(
      `DELETE FROM square_device_subkeys WHERE cid_number = ? AND account_id = ?`
    ).bind(cidNumber, accountId)
  ];
  const results = await env.DB.batch(statements);
  const deletedRows = results.reduce((sum, result) => sum + (result.meta?.changes ?? 0), 0);
  await env.SQUARE_CACHE.delete(`square_identity:${accountId}`);
  await clearIdentityAccountSessions(env, cidNumber, accountId);
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
