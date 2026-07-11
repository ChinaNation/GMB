import type { Env, MediaAssetRow } from '../types';
import { nowMs } from '../shared/time';
import { sanitizeOwnerAccount } from '../storage/r2_keys';
import { signR2GetUrl } from '../storage/presigned';
import {
  copyStreamFromUrl,
  createStreamDownloadUrl,
  deleteProviderAsset,
  streamPlaybackUrls
} from '../media/cloudflare_assets';

// 退订视频冷归档（任务卡 20260710-membership-video-archive-revamp）：
// 会员失效满 N 月 → 视频从 Stream 导出到 R2 冷存(IA)、删 Stream、对所有人不可播；
// 作者重新订阅 → 从 R2 回灌 Stream 解冻。文本/图片不归档；数据只在注销硬删。

const DAY_MS = 24 * 60 * 60 * 1000;
const DEFAULT_LAPSE_DAYS = 90; // 退订满 3 个月
const MAX_OWNERS_PER_SWEEP = 20; // 单次 Cron 限流，防 Worker 超时
const MAX_VIDEOS_PER_SWEEP = 100;
const RESTORE_MAX_DURATION_SECONDS = 4 * 60 * 60; // 回灌时长上限（覆盖最高档 3h）
const ARCHIVE_READ_URL_TTL_SECONDS = 3600;

const MEDIA_COLUMNS = `upload_id, post_id, owner_account, media_index, media_kind, provider,
  provider_asset_id, upload_method, content_type, byte_size, asset_state, delivery_url,
  playback_hls_url, playback_dash_url, thumbnail_url, duration_seconds, width, height,
  error_code, created_at, updated_at, ready_at, archive_state, archived_at, r2_archive_key`;

export function videoArchiveEnabled(env: Env): boolean {
  return env.ARCHIVE_ENABLED === '1';
}

function lapseDays(env: Env): number {
  const parsed = Number(env.ARCHIVE_LAPSE_DAYS);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : DEFAULT_LAPSE_DAYS;
}

/// R2 冷存对象键：archive/{owner}/{stream_uid}.mp4。
export function archiveObjectKey(ownerAccount: string, uid: string): string {
  return `archive/${sanitizeOwnerAccount(ownerAccount)}/${uid}.mp4`;
}

/// Cron 入口：扫描退订满 N 月的账户，冷归档其仍在播的视频。返回处理统计。
export async function runVideoArchiveSweep(env: Env): Promise<{ owners: number; archived: number }> {
  if (!videoArchiveEnabled(env)) {
    return { owners: 0, archived: 0 };
  }
  const cutoff = nowMs() - lapseDays(env) * DAY_MS;
  const owners = await selectLapsedOwners(env, cutoff, MAX_OWNERS_PER_SWEEP);
  let archived = 0;
  for (const owner of owners) {
    if (archived >= MAX_VIDEOS_PER_SWEEP) break;
    const videos = await selectVideoAssets(env, owner, 'live');
    for (const video of videos) {
      if (archived >= MAX_VIDEOS_PER_SWEEP) break;
      if (await archiveVideoAsset(env, video)) archived += 1;
    }
  }
  return { owners: owners.length, archived };
}

/// 重订解冻：把该 owner 已归档的视频回灌 Stream。由 Stripe 订阅重新生效时触发。
export async function restoreOwnerVideos(env: Env, ownerAccount: string): Promise<{ restored: number }> {
  const videos = await selectVideoAssets(env, ownerAccount, 'archived');
  let restored = 0;
  for (const video of videos) {
    if (await restoreVideoAsset(env, video)) restored += 1;
  }
  return { restored };
}

async function selectLapsedOwners(env: Env, cutoff: number, limit: number): Promise<string[]> {
  const result = await env.DB.prepare(
    `SELECT DISTINCT m.owner_account
      FROM square_memberships m
      JOIN square_media_assets a ON a.owner_account = m.owner_account
      WHERE m.entitlement_lapsed_at IS NOT NULL
        AND m.entitlement_lapsed_at <= ?
        AND m.subscription_status NOT IN ('active', 'trialing')
        AND a.media_kind = 'video'
        AND a.archive_state = 'live'
      LIMIT ?`
  )
    .bind(cutoff, limit)
    .all<{ owner_account: string }>();
  return (result.results ?? []).map((row) => row.owner_account);
}

async function selectVideoAssets(
  env: Env,
  ownerAccount: string,
  archiveState: 'live' | 'archived'
): Promise<MediaAssetRow[]> {
  const result = await env.DB.prepare(
    `SELECT ${MEDIA_COLUMNS} FROM square_media_assets
      WHERE owner_account = ? AND media_kind = 'video' AND archive_state = ?`
  )
    .bind(ownerAccount, archiveState)
    .all<MediaAssetRow>();
  return result.results ?? [];
}

async function archiveVideoAsset(env: Env, video: MediaAssetRow): Promise<boolean> {
  if (video.media_kind !== 'video' || video.archive_state !== 'live') return false;
  if (video.provider !== 'cloudflare_stream') return false;
  const uid = video.provider_asset_id;
  const r2Key = archiveObjectKey(video.owner_account, uid);
  try {
    // 本地验收无真实 Stream/R2：直接落归档态，验证状态机与 DB 迁移。
    if (env.DEV_UPLOAD_PROXY === '1') {
      await markArchived(env, video, r2Key);
      return true;
    }
    // 1) Stream 导出编码版 MP4（无冷层）。仍在生成则本轮跳过，下次扫描再归档。
    const mp4Url = await createStreamDownloadUrl(env, uid);
    if (!mp4Url) return false;
    // 2) 拉流写入 R2 冷存（Infrequent Access）。
    const response = await fetch(mp4Url);
    if (!response.ok || !response.body) {
      throw new Error(`stream download failed: ${response.status}`);
    }
    await env.SQUARE_MEDIA.put(r2Key, response.body, {
      storageClass: 'InfrequentAccess',
      httpMetadata: { contentType: 'video/mp4' }
    });
    // 3) 无损铁律：确认 R2 落成才删 Stream。
    const head = await env.SQUARE_MEDIA.head(r2Key);
    if (!head || head.size <= 0) {
      throw new Error('r2 archive object not persisted');
    }
    // 4) 删 Stream，停止其存储计费（$5/1000min）。
    await deleteProviderAsset(env, { provider: 'cloudflare_stream', provider_asset_id: uid });
    // 5) 落归档态。
    await markArchived(env, video, r2Key);
    return true;
  } catch (error) {
    // 失败保持 live（下次重扫）；R2 未落成前绝不删 Stream，无数据丢失风险。
    console.error(`[video-archive] archive uid=${uid} failed: ${errorText(error)}`);
    return false;
  }
}

async function restoreVideoAsset(env: Env, video: MediaAssetRow): Promise<boolean> {
  if (video.archive_state !== 'archived' || !video.r2_archive_key) return false;
  // 先切「恢复中」，客户端显示占位。
  await setArchiveState(env, video, 'restoring');
  try {
    // 本地验收：直接落回 live（无真实回灌）。
    if (env.DEV_UPLOAD_PROXY === '1') {
      await markRestoredLive(env, video, video.provider_asset_id);
      return true;
    }
    // 从 R2 冷存签发短期只读 URL，供 Stream copy-from-URL 回灌。
    const sourceUrl = await signR2GetUrl(env, video.r2_archive_key, ARCHIVE_READ_URL_TTL_SECONDS);
    if (!sourceUrl) {
      throw new Error('r2 presign unavailable');
    }
    const newUid = await copyStreamFromUrl(env, sourceUrl, RESTORE_MAX_DURATION_SECONDS);
    // 切到新 uid + 新播放地址，保持 restoring；转码完成由 Stream webhook 落 live。
    await markRestoring(env, video, newUid);
    return true;
  } catch (error) {
    // 恢复失败：回退 archived，下次重试。
    console.error(`[video-archive] restore uid=${video.provider_asset_id} failed: ${errorText(error)}`);
    await setArchiveState(env, video, 'archived');
    return false;
  }
}

async function markArchived(env: Env, video: MediaAssetRow, r2Key: string): Promise<void> {
  const now = nowMs();
  await env.DB.prepare(
    `UPDATE square_media_assets
      SET archive_state = 'archived', archived_at = ?, r2_archive_key = ?,
        playback_hls_url = NULL, playback_dash_url = NULL, updated_at = ?
      WHERE upload_id = ? AND media_index = ?`
  )
    .bind(now, r2Key, now, video.upload_id, video.media_index)
    .run();
}

async function markRestoring(env: Env, video: MediaAssetRow, newUid: string): Promise<void> {
  const playback = streamPlaybackUrls(env, newUid);
  await env.DB.prepare(
    `UPDATE square_media_assets
      SET provider_asset_id = ?, archive_state = 'restoring', asset_state = 'processing',
        playback_hls_url = ?, playback_dash_url = ?, thumbnail_url = ?, updated_at = ?
      WHERE upload_id = ? AND media_index = ?`
  )
    .bind(
      newUid,
      playback.playback_hls_url,
      playback.playback_dash_url,
      playback.thumbnail_url,
      nowMs(),
      video.upload_id,
      video.media_index
    )
    .run();
}

async function markRestoredLive(env: Env, video: MediaAssetRow, uid: string): Promise<void> {
  const playback = streamPlaybackUrls(env, uid);
  const now = nowMs();
  await env.DB.prepare(
    `UPDATE square_media_assets
      SET provider_asset_id = ?, archive_state = 'live', asset_state = 'ready',
        playback_hls_url = ?, playback_dash_url = ?, thumbnail_url = ?, updated_at = ?, ready_at = ?
      WHERE upload_id = ? AND media_index = ?`
  )
    .bind(
      uid,
      playback.playback_hls_url,
      playback.playback_dash_url,
      playback.thumbnail_url,
      now,
      now,
      video.upload_id,
      video.media_index
    )
    .run();
}

async function setArchiveState(
  env: Env,
  video: MediaAssetRow,
  archiveState: MediaAssetRow['archive_state']
): Promise<void> {
  await env.DB.prepare(
    `UPDATE square_media_assets SET archive_state = ?, updated_at = ?
      WHERE upload_id = ? AND media_index = ?`
  )
    .bind(archiveState, nowMs(), video.upload_id, video.media_index)
    .run();
}

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
