// 会员套餐真源（ADR-036：会员与身份彻底解耦）。会员档 `membership_level` 是纯付费订阅轴，
// **不再绑定任何身份档**——任意身份（访客/投票/竞选）可订阅任意会员档，全组合放行。
// 三档：freedom 自由 / democracy 民主 / spark 薪火。发帖额度、媒体质量、聊天文件上限均按
// 所购套餐（membershipPlan(level)）。**价格真源与实际扣款属于链上 `square-post`，真实公历
// 到期时间由 runtime 根据共识时间戳确定**；本表只定档位与配额，不涉计价。一改此表须同步 App 卡片。
import { resourceLimit } from '../limits/catalog';

export type MembershipLevel = 'freedom' | 'democracy' | 'spark';

export type MediaQuality = 'sd' | 'hd';

const mib = 1024 * 1024;

export interface DynamicQuota {
  text_max_chars: number;
  image_quality: MediaQuality;
  max_images: number;
  video_quality: MediaQuality;
  max_videos: number;
  max_video_seconds: number;
  /// 单个视频体积上限来自 limits 唯一资源表，会员接口只负责展示同一值。
  max_video_bytes: number;
}

export interface ArticleQuota {
  title_min_chars: number;
  title_max_chars: number;
  body_max_chars: number;
  cover_quality: MediaQuality;
  cover_required: true;
  image_quality: MediaQuality;
  max_images: number;
}

export interface MembershipPlan {
  membership_level: MembershipLevel;
  display_name: string;
  /// 聊天文件大小上限（字节，会员权益之一，ADR-036）。媒体走 WebRTC P2P，客户端按此档强制；
  /// >100MB（仅 spark）的 Cloudflare 瞬时中转 transport 归卡2 阶段3，本表只定档位上限值。
  chat_file_max_bytes: number;
  dynamic: DynamicQuota;
  article: ArticleQuota;
}

export const membershipPlans: Record<MembershipLevel, MembershipPlan> = {
  freedom: {
    membership_level: 'freedom',
    display_name: '自由会员',
    chat_file_max_bytes: 10 * mib,
    dynamic: {
      text_max_chars: 300,
      image_quality: 'sd',
      max_images: 9,
      video_quality: 'sd',
      max_videos: 1,
      max_video_seconds: 60,
      max_video_bytes: resourceLimit('square_video_sd').max_bytes
    },
    article: {
      title_min_chars: 10,
      title_max_chars: 50,
      body_max_chars: 20_000,
      cover_quality: 'hd',
      cover_required: true,
      image_quality: 'sd',
      max_images: 50
    }
  },
  democracy: {
    membership_level: 'democracy',
    display_name: '民主会员',
    chat_file_max_bytes: 100 * mib,
    dynamic: {
      text_max_chars: 300,
      image_quality: 'hd',
      max_images: 9,
      video_quality: 'hd',
      max_videos: 1,
      max_video_seconds: 30 * 60,
      max_video_bytes: resourceLimit('square_video_hd').max_bytes
    },
    article: {
      title_min_chars: 10,
      title_max_chars: 50,
      body_max_chars: 30_000,
      cover_quality: 'hd',
      cover_required: true,
      image_quality: 'hd',
      max_images: 100
    }
  },
  spark: {
    membership_level: 'spark',
    display_name: '薪火会员',
    chat_file_max_bytes: 5120 * mib,
    dynamic: {
      text_max_chars: 300,
      image_quality: 'hd',
      max_images: 9,
      video_quality: 'hd',
      max_videos: 1,
      max_video_seconds: 3 * 60 * 60,
      max_video_bytes: resourceLimit('square_video_spark').max_bytes
    },
    article: {
      title_min_chars: 10,
      title_max_chars: 50,
      body_max_chars: 30_000,
      cover_quality: 'hd',
      cover_required: true,
      image_quality: 'hd',
      max_images: 100
    }
  }
};

export function assertMembershipLevel(value: unknown): MembershipLevel {
  if (value === 'freedom' || value === 'democracy' || value === 'spark') {
    return value;
  }
  throw new Error('invalid membership level');
}

export function membershipPlan(level: string): MembershipPlan {
  if (level === 'democracy' || level === 'spark') {
    return membershipPlans[level];
  }
  return membershipPlans.freedom;
}

export function membershipPlanList(): MembershipPlan[] {
  return [membershipPlans.freedom, membershipPlans.democracy, membershipPlans.spark];
}
