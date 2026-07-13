// 会员套餐真源。会员档 `membership_level` 与身份档 `required_identity_level` 解耦：
// 访客身份含自由(freedom $2.99)/民主(democracy $9.99，权益=投票、身份匿名)两档，
// 投票/竞选各一档。订阅资格精确匹配身份档（identityEligibleForPlan，禁止降档/越级）；
// 发帖额度按所购套餐（membershipPlan(level).quota）。四档一改，须同步 App 卡片、官网
// Membership.tsx、Stripe price 映射与 webhook 反查。
import { resourceLimit } from '../limits/catalog';

export type MembershipLevel = 'freedom' | 'democracy' | 'voting' | 'candidate';

/// 链上身份档位（与会员档位解耦）：访客 / 投票公民 / 竞选公民。
/// 会员档 `democracy`（民主）不是身份档，其 required_identity_level 仍为 'visitor'。
export type IdentityLevel = 'visitor' | 'voting' | 'candidate';

export type RequiredIdentityLevel = IdentityLevel;

export type MediaQuality = 'sd' | 'hd';

export type MembershipPriceCurrency = 'usd';

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
  price_currency: MembershipPriceCurrency;
  price_usd_cents: number;
  price_usd_monthly: string;
  required_identity_level: RequiredIdentityLevel;
  dynamic: DynamicQuota;
  article: ArticleQuota;
}

export const membershipPlans: Record<MembershipLevel, MembershipPlan> = {
  freedom: {
    membership_level: 'freedom',
    display_name: '自由会员',
    price_currency: 'usd',
    price_usd_cents: 299,
    price_usd_monthly: '2.99',
    required_identity_level: 'visitor',
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
  // 民主会员：媒体权益与投票公民会员完全一致（仅身份不同——民主匿名、投票为
  // 公民认证）。required_identity_level 仍为 'visitor'，访客身份即可订阅。
  democracy: {
    membership_level: 'democracy',
    display_name: '民主会员',
    price_currency: 'usd',
    price_usd_cents: 999,
    price_usd_monthly: '9.99',
    required_identity_level: 'visitor',
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
  voting: {
    membership_level: 'voting',
    display_name: '投票公民会员',
    price_currency: 'usd',
    price_usd_cents: 999,
    price_usd_monthly: '9.99',
    required_identity_level: 'voting',
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
  candidate: {
    membership_level: 'candidate',
    display_name: '竞选公民会员',
    price_currency: 'usd',
    price_usd_cents: 9999,
    price_usd_monthly: '99.99',
    required_identity_level: 'candidate',
    dynamic: {
      text_max_chars: 300,
      image_quality: 'hd',
      max_images: 9,
      video_quality: 'hd',
      max_videos: 1,
      max_video_seconds: 3 * 60 * 60,
      max_video_bytes: resourceLimit('square_video_candidate').max_bytes
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
  if (
    value === 'freedom' ||
    value === 'democracy' ||
    value === 'voting' ||
    value === 'candidate'
  ) {
    return value;
  }
  throw new Error('invalid membership level');
}

export function membershipPlan(level: string): MembershipPlan {
  if (level === 'voting' || level === 'candidate' || level === 'democracy') {
    return membershipPlans[level];
  }
  return membershipPlans.freedom;
}

/// 订阅资格：精确匹配——只能订阅"本身份档对应"的会员，禁止降档/越级。
/// 例：voting 身份只能订 voting；candidate 只能订 candidate；visitor 身份可订
/// freedom 与 democracy（二者 required_identity_level 均为 'visitor'）。
export function identityEligibleForPlan(
  identity: RequiredIdentityLevel,
  plan: MembershipPlan
): boolean {
  return plan.required_identity_level === identity;
}

export function membershipPlanList(): MembershipPlan[] {
  return [
    membershipPlans.freedom,
    membershipPlans.democracy,
    membershipPlans.voting,
    membershipPlans.candidate
  ];
}
