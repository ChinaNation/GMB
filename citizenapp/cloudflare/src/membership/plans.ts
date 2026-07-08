export type MembershipLevel = 'visitor' | 'voting' | 'candidate';

export type RequiredIdentityLevel = MembershipLevel;

export type MediaQuality = 'sd' | 'hd';

export type MembershipPriceCurrency = 'usd';

export interface DynamicQuota {
  text_max_chars: number;
  image_quality: MediaQuality;
  max_images: number;
  video_quality: MediaQuality;
  max_videos: number;
  max_video_seconds: number;
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
  legacy_storage_quota_bytes: number;
}

const gib = 1024 * 1024 * 1024;

export const membershipPlans: Record<MembershipLevel, MembershipPlan> = {
  visitor: {
    membership_level: 'visitor',
    display_name: '访客会员',
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
      max_video_seconds: 60
    },
    article: {
      title_min_chars: 10,
      title_max_chars: 50,
      body_max_chars: 20_000,
      cover_quality: 'hd',
      cover_required: true,
      image_quality: 'sd',
      max_images: 50
    },
    legacy_storage_quota_bytes: 20 * gib
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
      max_video_seconds: 30 * 60
    },
    article: {
      title_min_chars: 10,
      title_max_chars: 50,
      body_max_chars: 30_000,
      cover_quality: 'hd',
      cover_required: true,
      image_quality: 'hd',
      max_images: 100
    },
    legacy_storage_quota_bytes: 200 * gib
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
      max_video_seconds: 3 * 60 * 60
    },
    article: {
      title_min_chars: 10,
      title_max_chars: 50,
      body_max_chars: 30_000,
      cover_quality: 'hd',
      cover_required: true,
      image_quality: 'hd',
      max_images: 100
    },
    legacy_storage_quota_bytes: 1024 * gib
  }
};

export function assertMembershipLevel(value: unknown): MembershipLevel {
  if (value === 'visitor' || value === 'voting' || value === 'candidate') {
    return value;
  }
  throw new Error('invalid membership level');
}

export function membershipPlan(level: string): MembershipPlan {
  if (level === 'voting' || level === 'candidate') {
    return membershipPlans[level];
  }
  return membershipPlans.visitor;
}

export function identityLevelRank(level: RequiredIdentityLevel): number {
  if (level === 'candidate') return 2;
  if (level === 'voting') return 1;
  return 0;
}

export function identitySatisfies(
  actual: RequiredIdentityLevel,
  required: RequiredIdentityLevel
): boolean {
  return identityLevelRank(actual) >= identityLevelRank(required);
}

export function membershipPlanList(): MembershipPlan[] {
  return [membershipPlans.visitor, membershipPlans.voting, membershipPlans.candidate];
}
