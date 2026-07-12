export type PostCategory = 'normal' | 'campaign';

export type PostContentFormat = 'normal' | 'article';

export type MediaKind = 'image' | 'video' | 'cover';

export type UploadStatus = 'prepared' | 'completed';

export type FeedKind = 'recommended' | 'following' | 'campaign';

export type MediaProvider = 'cloudflare_images' | 'cloudflare_stream';

export type MediaUploadMethod = 'worker' | 'tus';

export type MediaAssetState = 'prepared' | 'uploaded' | 'processing' | 'ready' | 'error';

// 视频冷归档态：live=Stream 可播 / archived=已移入 R2 冷存不可播 / restoring=重订后回灌中。
export type MediaArchiveState = 'live' | 'archived' | 'restoring';

export interface Env {
  DB: D1Database;
  SQUARE_MEDIA: R2Bucket;
  SQUARE_CACHE: KVNamespace;
  CHAT_REALTIME?: DurableObjectNamespace;
  STREAM?: StreamBinding;
  // 平台推送只发送无内容 Chat 唤醒；私钥只允许使用 Worker Secret 配置。
  APNS_KEY?: string;
  APNS_KID?: string;
  APNS_TEAM?: string;
  APNS_TOPIC?: string;
  APNS_ENV?: string;
  FCM_PROJECT?: string;
  FCM_EMAIL?: string;
  FCM_KEY?: string;
  // Cloudflare 账户由 R2 冷归档、Images、Stream 共用；S3 密钥只用于内部归档读取。
  CF_ACCOUNT_ID?: string;
  R2_ACCESS_ID?: string;
  R2_SECRET_KEY?: string;
  R2_BUCKET?: string;
  SESSION_TTL_SECONDS?: string;
  UPLOAD_TTL_SECONDS?: string;
  // Worker 通过 Access + Tunnel 调用权威节点回环 RPC；URL 和服务令牌只放远端 Secret。
  CHAIN_URL?: string;
  CHAIN_ID?: string;
  CHAIN_SECRET?: string;
  // 轻节点启动清单只下发公开 bootnodes 和冻结链身份，不下发 checkpoint 或 RPC 地址。
  CHAIN_BOOTNODES?: string;
  BOOT_TTL_SECONDS?: string;
  CHAIN_GENESIS_HASH?: string;
  CHAIN_STATE_ROOT?: string;
  // 已签名交易兜底广播：只转发完整 signed extrinsic，不提供通用 JSON-RPC proxy。
  RELAY_ENABLED?: string;
  RELAY_MAX_BYTES?: string;
  RELAY_PER_MINUTE?: string;
  // Stripe webhook secret 必须使用 Cloudflare secret/变量配置，不能写入仓库或下发 App。
  STRIPE_HOOK_SECRET?: string;
  STRIPE_HOOK_WINDOW?: string;
  // 只允许本地 Miniflare 验证使用；生产环境必须保持关闭。
  STRIPE_DEV_PROXY?: string;
  // Stripe secret key 只允许放 Worker Secret，用于官网创建 Checkout Session。
  STRIPE_API_KEY?: string;
  FREEDOM_PRICE_ID?: string;
  // 民主会员价格 ID：访客身份的 $9.99 高权益档。
  DEMOCRACY_PRICE_ID?: string;
  VOTING_PRICE_ID?: string;
  CANDIDATE_PRICE_ID?: string;
  CHECKOUT_SUCCESS_URL?: string;
  CHECKOUT_CANCEL_URL?: string;
  // Cloudflare Images / Stream API token 只放 Worker Secret；App 只拿一次性上传 URL。
  CF_API_TOKEN?: string;
  IMAGES_URL?: string;
  STREAM_URL?: string;
  STREAM_HOOK_SECRET?: string;
  MEDIA_TTL_SECONDS?: string;
  IMAGES_SIGNING_KEY?: string;
  TURNSTILE_SITEKEY?: string;
  TURNSTILE_SECRET?: string;
  WEB_ORIGIN?: string;
  HASH_KEY?: string;
  // 退订视频冷归档：开关（'1' 开）与阈值（天，缺省 90）。关闭时 Cron 不做任何归档。
  ARCHIVE_ENABLED?: string;
  ARCHIVE_LAPSE_DAYS?: string;
}

export interface SessionState {
  owner_account: string;
  device_key_hash: string;
  created_at: number;
  expires_at: number;
}

export interface LoginChallengeRow {
  challenge_id: string;
  owner_account: string;
  signing_payload: string;
  expires_at: number;
  used_at: number | null;
}

export interface DeviceSubkeyRow {
  owner_account: string;
  p256_pubkey: string;
  issued_at: number;
  created_at: number;
  updated_at: number;
}

export interface MembershipRow {
  owner_account: string;
  membership_level: string;
  expires_at: number;
  updated_at: number;
  subscription_source: string;
  stripe_customer_id: string | null;
  stripe_subscription_id: string | null;
  stripe_price_id: string | null;
  subscription_status: string;
  current_period_start: number | null;
  current_period_end: number | null;
  cancel_at_period_end: number;
  identity_level: string;
  identity_checked_at: number | null;
  // 会员权益失效时刻（退订满 N 月冷归档的时钟起点；重订置 NULL）。
  entitlement_lapsed_at: number | null;
}

export interface UploadItemInput {
  media_kind: MediaKind;
  content_type: string;
  byte_size: number;
  duration_seconds?: number;
  file_ext?: string;
}

export interface PreparedUploadRow {
  upload_id: string;
  post_id: string;
  owner_account: string;
  post_category: PostCategory;
  manifest_hash: string;
  content_hash: string | null;
  storage_receipt_id: string | null;
  estimated_bytes: number;
  object_keys_json: string;
  status: UploadStatus;
  expires_at: number;
  created_at: number;
  completed_at: number | null;
}

export interface MediaAssetRow {
  upload_id: string;
  post_id: string;
  owner_account: string;
  media_index: number;
  media_kind: 'image' | 'video';
  provider: MediaProvider;
  provider_asset_id: string;
  upload_method: MediaUploadMethod;
  resource_key: string;
  content_type: string;
  byte_size: number;
  asset_state: MediaAssetState;
  declared_duration_seconds: number | null;
  duration_seconds: number | null;
  width: number | null;
  height: number | null;
  error_code: string | null;
  created_at: number;
  updated_at: number;
  ready_at: number | null;
  // 视频冷归档：仅视频行使用，图片恒 'live'。
  archive_state: MediaArchiveState;
  archived_at: number | null;
  r2_archive_key: string | null;
}

export interface SquarePostRow {
  post_id: string;
  owner_account: string;
  cid_number: string | null;
  post_category: PostCategory;
  content_format: PostContentFormat;
  title: string | null;
  text: string;
  content_hash: string;
  storage_receipt_id: string;
  chain_block: number | null;
  created_at: number;
  post_state: string;
  // 竞选目标（预留，待公民身份上链完成后落地）：竞选哪个机构的哪个岗位。
  // 公民 CID 复用 cid_number；下面两项待落地时新增 D1 列
  // campaign_institution_cid / campaign_position 并在此补类型与查询。
}

export interface SquareFeedMediaItem {
  media_kind: 'image' | 'video';
  object_key: string;
  url: string;
  provider: MediaProvider;
  provider_asset_id: string;
  asset_state: MediaAssetState;
  thumbnail_url?: string | null;
  content_type: string;
  byte_size: number;
  sha256: string;
  duration_seconds?: number | null;
  width?: number | null;
  height?: number | null;
  // 视频冷归档态：archived=已归档不可播（作者未续订），restoring=恢复中；缺省视为 live。
  archive_state?: MediaArchiveState;
}

export interface SquarePostFeedItem extends SquarePostRow {
  media_items?: SquareFeedMediaItem[];
  // 作者徽章信号（公开）：身份档=颜色、会员匹配身份档=勾。由本页去重作者统一读链上身份+批量读会员填充。
  // identity_level 是链上身份档；membership_level 是已购买会员档（含 democracy 民主）。
  identity_level?: 'visitor' | 'voting' | 'candidate';
  membership_level?: 'freedom' | 'democracy' | 'voting' | 'candidate' | null;
  membership_active?: boolean;
}

/// 按作者拉帖的分类过滤维度。'all' 表示不过滤。
export type AuthorPostCategory = 'all' | PostCategory;

/// 按作者拉帖的内容形态过滤。'all' 不过滤；'normal' 排除文章；'article' 只看文章。
export type AuthorContentFormat = 'all' | PostContentFormat;

/// R2 公开资料包（citizenapp.square.profile.v1）。
/// 头像/背景/签名/展示名等公开链下资料的唯一真源。
export interface CitizenProfileDoc {
  schema: 'citizenapp.square.profile.v1';
  owner_account: string;
  display_name: string;
  bio: string;
  avatar_object_key: string | null;
  avatar_content_hash: string | null;
  banner_object_key: string | null;
  banner_content_hash: string | null;
  updated_at: number;
}

/// 主页计数：均为 D1 实时聚合，不写入 profile.json。
export interface UserProfileCounts {
  following: number;
  followers: number;
  posts: number;
}

/// GET /v1/square/users/:account 响应载荷。
export interface UserProfileResponse {
  owner_account: string;
  display_name: string;
  bio: string;
  avatar_object_key: string | null;
  banner_object_key: string | null;
  cid_number: string | null;
  is_certified: boolean;
  /// 链上身份档位：visitor 未认证 / voting 认证投票公民 / candidate 认证竞选公民。
  identity_level: 'visitor' | 'voting' | 'candidate';
  /// 已购买的会员档位（公开）；未购买为 null。含 democracy（民主）。徽章「勾」= 会员有效。
  membership_level: 'freedom' | 'democracy' | 'voting' | 'candidate' | null;
  /// 会员是否当前有效（订阅生效且未过期）。
  membership_active: boolean;
  counts: UserProfileCounts;
  is_following: boolean;
  updated_at: number;
}
