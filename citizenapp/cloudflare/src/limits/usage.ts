import type { Env, MediaAssetRow, MembershipRow } from '../types';
import type { MembershipLevel } from '../membership/plans';
import { HttpError } from '../shared/http';
import { nowMs } from '../shared/time';
import { usageLimits } from './catalog';

export async function reserveUploadUsage(input: {
  env: Env;
  upload_id: string;
  cid_number: string;
  membership_level: MembershipLevel;
  membership: MembershipRow;
  byte_size: number;
  image_count: number;
  video_seconds: number;
  expires_at: number;
}): Promise<void> {
  await assertProfitBudget(input.env, input.video_seconds > 0);
  const limit = usageLimits[input.membership_level];
  const { periodStart, periodEnd } = membershipUsagePeriod(input.membership);
  const createdAt = nowMs();
  // 用量预留归属键 = 身份主键 cid_number(活动预留数、周期用量额度均按 cid 累计)。
  const result = await input.env.DB.prepare(
    `INSERT INTO resource_reservations
      (reservation_id, cid_number, resource_key, period_start, period_end, byte_size,
       image_count, video_seconds, expires_at, reservation_state, created_at, used_at)
      SELECT ?, ?, 'square_upload', ?, ?, ?, ?, ?, ?, 'reserved', ?, NULL
      WHERE
        (SELECT COUNT(*) FROM resource_reservations
          WHERE cid_number = ? AND resource_key = 'square_upload'
            AND reservation_state = 'reserved' AND expires_at > ?) < ?
        AND COALESCE((SELECT image_count FROM resource_usage
          WHERE cid_number = ? AND resource_key = 'square_upload' AND period_start = ?), 0)
          + COALESCE((SELECT SUM(image_count) FROM resource_reservations
            WHERE cid_number = ? AND resource_key = 'square_upload'
              AND reservation_state = 'reserved' AND expires_at > ?), 0) + ? <= ?
        AND COALESCE((SELECT video_seconds FROM resource_usage
          WHERE cid_number = ? AND resource_key = 'square_upload' AND period_start = ?), 0)
          + COALESCE((SELECT SUM(video_seconds) FROM resource_reservations
            WHERE cid_number = ? AND resource_key = 'square_upload'
              AND reservation_state = 'reserved' AND expires_at > ?), 0) + ? <= ?`
  ).bind(
    input.upload_id, input.cid_number, periodStart, periodEnd, input.byte_size,
    input.image_count, input.video_seconds, input.expires_at, createdAt,
    input.cid_number, createdAt, limit.active_uploads,
    input.cid_number, periodStart, input.cid_number, createdAt,
    input.image_count, limit.monthly_images,
    input.cid_number, periodStart, input.cid_number, createdAt,
    input.video_seconds, limit.monthly_video_seconds,
  ).run();
  if ((result.meta?.changes ?? 0) !== 1) {
    throw new HttpError(429, 'upload_usage_exceeded', '活动上传数或订阅周期媒体额度已达到上限');
  }
}

/**
 * 计费周期镜像给出周期起点；缺失或越界时按固定周期终点反推，保证同一周期
 * 的每次请求都命中同一个 D1 主键，不能用请求时间制造新周期绕过累计额度。
 */
export function membershipUsagePeriod(
  membership: Pick<MembershipRow, 'last_charged_at' | 'paid_until'>,
): { periodStart: number; periodEnd: number } {
  if (membership.last_charged_at < 0 || membership.last_charged_at >= membership.paid_until) {
    throw new HttpError(503, 'subscription_period_invalid', 'finalized 订阅用量周期不合法');
  }
  return { periodStart: membership.last_charged_at, periodEnd: membership.paid_until };
}

/** 完成上传时一次性把预留转为周期用量，重复 complete 不会重复计数。 */
export async function consumeUploadUsage(
  env: Env,
  uploadId: string,
  assets: MediaAssetRow[],
  contentHash: string,
  completedAt: number,
): Promise<void> {
  const usedAt = nowMs();
  const reservation = await env.DB.prepare(
    `UPDATE resource_reservations SET reservation_state = 'used', used_at = ?
      WHERE reservation_id = ? AND reservation_state = 'reserved'
      RETURNING cid_number, period_start, period_end, byte_size, image_count, video_seconds`
  ).bind(usedAt, uploadId).first<{
    cid_number: string;
    period_start: number;
    period_end: number;
    byte_size: number;
    image_count: number;
    video_seconds: number;
  }>();
  if (!reservation) throw new HttpError(409, 'upload_reservation_missing', '上传额度预留不存在或已核销');

  try {
    await env.DB.batch([
      env.DB.prepare(
      `INSERT INTO resource_usage
        (cid_number, resource_key, period_start, period_end, byte_size, image_count, video_seconds, updated_at)
        VALUES (?, 'square_upload', ?, ?, ?, ?, ?, ?)
        ON CONFLICT(cid_number, resource_key, period_start) DO UPDATE SET
          byte_size = resource_usage.byte_size + excluded.byte_size,
          image_count = resource_usage.image_count + excluded.image_count,
          video_seconds = resource_usage.video_seconds + excluded.video_seconds,
          updated_at = excluded.updated_at`
      ).bind(
      reservation.cid_number, reservation.period_start, reservation.period_end,
      reservation.byte_size, reservation.image_count, reservation.video_seconds, usedAt,
      ),
      totalStatement(env, 'square_image', assets.filter((asset) => asset.media_kind === 'image')),
      totalStatement(env, 'square_video', assets.filter((asset) => asset.media_kind === 'video')),
      env.DB.prepare(
        `UPDATE square_uploads SET content_hash = ?, status = 'completed', completed_at = ?
          WHERE upload_id = ? AND status = 'prepared'`
      ).bind(contentHash, completedAt, uploadId),
    ]);
  } catch (error) {
    await env.DB.prepare(
      `UPDATE resource_reservations SET reservation_state = 'reserved', used_at = NULL
        WHERE reservation_id = ? AND reservation_state = 'used' AND used_at = ?`
    ).bind(uploadId, usedAt).run();
    throw error;
  }
}

export async function releaseUploadReservation(env: Env, uploadId: string): Promise<void> {
  await env.DB.prepare(
    `DELETE FROM resource_reservations WHERE reservation_id = ? AND reservation_state = 'reserved'`
  ).bind(uploadId).run();
}

/** 删除帖子只回收当前存储总量，不返还已经消耗的订阅周期上传额度。 */
export async function releaseStoredMedia(env: Env, assets: MediaAssetRow[]): Promise<void> {
  await env.DB.batch(storedMediaReleaseStatements(env, assets));
}

/**
 * 构造存储总量回收语句，供内容删除与其 D1 行删除放进同一个原子 batch。
 *
 * 调用方不得先单独执行这些语句再删除内容行，否则后续失败重试会重复扣减总量。
 */
export function storedMediaReleaseStatements(
  env: Env,
  assets: MediaAssetRow[],
  updatedAt: number = nowMs(),
): D1PreparedStatement[] {
  const imageAssets = assets.filter((asset) => asset.media_kind === 'image');
  const videoAssets = assets.filter((asset) => asset.media_kind === 'video');
  return [
    releaseTotalStatement(env, 'square_image', imageAssets, updatedAt),
    releaseTotalStatement(env, 'square_video', videoAssets, updatedAt),
  ];
}

export async function cleanupExpiredReservations(env: Env): Promise<void> {
  await env.DB.prepare(
    `DELETE FROM resource_reservations WHERE reservation_state = 'reserved' AND expires_at <= ?`
  ).bind(nowMs()).run();
}

function totalStatement(env: Env, key: string, assets: MediaAssetRow[]): D1PreparedStatement {
  const byteSize = assets.reduce((sum, asset) => sum + asset.byte_size, 0);
  const imageCount = assets.filter((asset) => asset.media_kind === 'image').length;
  const videoSeconds = assets
    .filter((asset) => asset.media_kind === 'video')
    .reduce((sum, asset) => sum + Math.ceil(asset.duration_seconds ?? asset.declared_duration_seconds ?? 0), 0);
  return env.DB.prepare(
    `INSERT INTO resource_totals (resource_key, byte_size, object_count, video_seconds, updated_at)
      VALUES (?, ?, ?, ?, ?)
      ON CONFLICT(resource_key) DO UPDATE SET
        byte_size = resource_totals.byte_size + excluded.byte_size,
        object_count = resource_totals.object_count + excluded.object_count,
        video_seconds = resource_totals.video_seconds + excluded.video_seconds,
        updated_at = excluded.updated_at`
  ).bind(key, byteSize, assets.length, videoSeconds, nowMs());
}

function releaseTotalStatement(
  env: Env,
  key: string,
  assets: MediaAssetRow[],
  updatedAt: number,
): D1PreparedStatement {
  const byteSize = assets.reduce((sum, asset) => sum + asset.byte_size, 0);
  const videoSeconds = assets
    .filter((asset) => asset.media_kind === 'video')
    .reduce((sum, asset) => sum + Math.ceil(asset.duration_seconds ?? asset.declared_duration_seconds ?? 0), 0);
  return env.DB.prepare(
    `UPDATE resource_totals SET
      byte_size = MAX(0, byte_size - ?), object_count = MAX(0, object_count - ?),
      video_seconds = MAX(0, video_seconds - ?), updated_at = ? WHERE resource_key = ?`
  ).bind(byteSize, assets.length, videoSeconds, updatedAt, key);
}

/** 收入预算是成本熔断，不替代单账户额度；达到阈值时先停止新视频，再停止新媒体。 */
async function assertProfitBudget(env: Env, hasVideo: boolean): Promise<void> {
  const revenue = await env.DB.prepare(
    `SELECT COALESCE(SUM(m.last_charged_price_fen), 0) AS cents
      FROM square_memberships m
      JOIN chain_clock c ON c.clock_id = 1
      WHERE m.subscription_status IN ('active', 'cancelled')
        AND c.chain_timestamp < m.paid_until`
  ).first<{ cents: number }>();
  const totals = await env.DB.prepare(
    `SELECT
      COALESCE(SUM(CASE WHEN resource_key = 'square_video' THEN video_seconds ELSE 0 END), 0) AS video_seconds,
      COALESCE(SUM(CASE WHEN resource_key = 'square_image' THEN object_count ELSE 0 END), 0) AS image_count
      FROM resource_totals`
  ).first<{ video_seconds: number; image_count: number }>();
  const revenueCents = Math.max(revenue?.cents ?? 0, 299);
  const budgetCents = Math.max(100, revenueCents * 0.35);
  const costCents = ((totals?.video_seconds ?? 0) / 60) * 0.5 + (totals?.image_count ?? 0) * 0.005;
  const ratio = costCents / budgetCents;
  if (ratio >= 1) throw new HttpError(503, 'media_budget_reached', '媒体容量保护已启动，请稍后再试');
  if (hasVideo && ratio >= 0.85) {
    throw new HttpError(503, 'video_budget_reached', '视频容量保护已启动，请稍后再试');
  }
}
