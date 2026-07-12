import type { Env, MediaAssetRow } from '../types';
import { HttpError } from '../shared/http';
import type { MembershipLevel } from '../membership/plans';
import { nowMs } from '../shared/time';

interface UsageLimit {
  monthly_images: number;
  monthly_video_seconds: number;
  active_uploads: number;
}

const usageLimits: Record<MembershipLevel, UsageLimit> = {
  freedom: { monthly_images: 300, monthly_video_seconds: 30 * 60, active_uploads: 1 },
  democracy: { monthly_images: 1500, monthly_video_seconds: 180 * 60, active_uploads: 2 },
  voting: { monthly_images: 1500, monthly_video_seconds: 180 * 60, active_uploads: 2 },
  candidate: { monthly_images: 5000, monthly_video_seconds: 1800 * 60, active_uploads: 3 }
};

interface UsageTotal {
  image_count: number;
  video_seconds: number;
}

export function usageLimit(level: MembershipLevel): UsageLimit {
  return usageLimits[level];
}

export async function assertUploadUsage(
  env: Env,
  ownerAccount: string,
  level: MembershipLevel,
  imageCount: number,
  videoSeconds: number
): Promise<void> {
  const limit = usageLimits[level];
  const active = await env.DB.prepare(
    `SELECT COUNT(*) AS n FROM square_uploads
      WHERE owner_account = ? AND status = 'prepared' AND expires_at > ?`
  ).bind(ownerAccount, nowMs()).first<{ n: number }>();
  if ((active?.n ?? 0) >= limit.active_uploads) {
    throw new HttpError(429, 'active_upload_exceeded', '请先完成或等待当前上传任务过期');
  }

  const month = new Date().toISOString().slice(0, 7);
  const total = await env.DB.prepare(
    `SELECT COALESCE(SUM(image_count), 0) AS image_count,
        COALESCE(SUM(video_seconds), 0) AS video_seconds
      FROM square_usage_days WHERE owner_account = ? AND usage_day LIKE ?`
  ).bind(ownerAccount, `${month}-%`).first<UsageTotal>();
  if ((total?.image_count ?? 0) + imageCount > limit.monthly_images) {
    throw new HttpError(429, 'monthly_images_exceeded', '本月图片用量已达到会员保护阈值');
  }
  if ((total?.video_seconds ?? 0) + videoSeconds > limit.monthly_video_seconds) {
    throw new HttpError(429, 'monthly_video_exceeded', '本月视频用量已达到会员保护阈值');
  }
  await assertProfitBudget(env, videoSeconds > 0);
}

export async function recordCompletedMedia(
  env: Env,
  ownerAccount: string,
  assets: MediaAssetRow[]
): Promise<void> {
  const usageDay = new Date().toISOString().slice(0, 10);
  const imageCount = assets.filter((asset) => asset.media_kind === 'image').length;
  const videoSeconds = assets
    .filter((asset) => asset.media_kind === 'video')
    .reduce((sum, asset) => sum + Math.ceil(asset.duration_seconds ?? asset.declared_duration_seconds ?? 0), 0);
  await env.DB.prepare(
    `INSERT INTO square_usage_days
      (owner_account, usage_day, image_count, video_seconds, blocked_count, updated_at)
      VALUES (?, ?, ?, ?, 0, ?)
      ON CONFLICT(owner_account, usage_day) DO UPDATE SET
        image_count = square_usage_days.image_count + excluded.image_count,
        video_seconds = square_usage_days.video_seconds + excluded.video_seconds,
        updated_at = excluded.updated_at`
  ).bind(ownerAccount, usageDay, imageCount, videoSeconds, nowMs()).run();
}

/**
 * 以当前有效订阅毛收入的 35% 作为 Cloudflare 媒体成本预算。达到 85% 先停新视频，
 * 达到 100% 停全部新媒体；已有内容、账户、文字浏览和 Chat 不受影响。
 */
async function assertProfitBudget(env: Env, hasVideo: boolean): Promise<void> {
  const revenue = await env.DB.prepare(
    `SELECT COALESCE(SUM(CASE membership_level
        WHEN 'freedom' THEN 299 WHEN 'candidate' THEN 9999 ELSE 999 END), 0) AS cents
      FROM square_memberships
      WHERE subscription_status IN ('active', 'trialing') AND expires_at > ?`
  ).bind(nowMs()).first<{ cents: number }>();
  const assets = await env.DB.prepare(
    `SELECT
        COALESCE(SUM(CASE WHEN media_kind = 'video' AND archive_state = 'live'
          THEN COALESCE(duration_seconds, declared_duration_seconds, 0) ELSE 0 END), 0) AS video_seconds,
        COALESCE(SUM(CASE WHEN media_kind = 'image' THEN 1 ELSE 0 END), 0) AS image_count
      FROM square_media_assets`
  ).first<{ video_seconds: number; image_count: number }>();
  const revenueCents = Math.max(revenue?.cents ?? 0, 299);
  const budgetCents = Math.max(100, revenueCents * 0.35);
  const videoStorageCents = ((assets?.video_seconds ?? 0) / 60) * 0.5;
  const imageStorageCents = (assets?.image_count ?? 0) * 0.005;
  const ratio = (videoStorageCents + imageStorageCents) / budgetCents;
  if (ratio >= 1) {
    throw new HttpError(503, 'media_budget_reached', '媒体容量保护已启动，请稍后再试');
  }
  if (hasVideo && ratio >= 0.85) {
    throw new HttpError(503, 'video_budget_reached', '视频容量保护已启动，请稍后再试');
  }
}
