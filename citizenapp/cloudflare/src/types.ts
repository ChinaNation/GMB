export type PostCategory = 'normal' | 'campaign';

export type PostContentFormat = 'normal' | 'article';

export type MediaKind = 'image' | 'video' | 'cover';

export type UploadStatus = 'prepared' | 'completed';

export type FeedKind = 'recommended' | 'following' | 'campaign';

export interface Env {
  DB: D1Database;
  SQUARE_MEDIA: R2Bucket;
  FEED_CACHE: KVNamespace;
  CHAT_REALTIME?: DurableObjectNamespace;
  // 生产环境只在 Worker Secret/变量中配置 R2 S3 凭证，绝不下发到 CitizenApp。
  R2_ACCOUNT_ID?: string;
  R2_ACCESS_KEY_ID?: string;
  R2_SECRET_ACCESS_KEY?: string;
  R2_BUCKET_NAME?: string;
  SQUARE_SESSION_TTL_SECONDS?: string;
  SQUARE_UPLOAD_URL_TTL_SECONDS?: string;
  // Worker 只读取链上事件用于确认发布，不托管钱包、不代签交易。
  SQUARE_CHAIN_RPC_URL?: string;
  // 只允许本地 Miniflare 验证使用；生产环境必须保持关闭。
  SQUARE_DEV_UPLOAD_PROXY?: string;
}

export interface SessionState {
  owner_account: string;
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

export interface MembershipRow {
  owner_account: string;
  membership_level: string;
  storage_quota_bytes: number;
  storage_used_bytes: number;
  expires_at: number;
  updated_at: number;
}

export interface UploadItemInput {
  media_kind: MediaKind;
  content_type: string;
  byte_size: number;
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
  created_at: number;
  completed_at: number | null;
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
  content_type: string;
  byte_size: number;
  sha256: string;
}

export interface SquarePostFeedItem extends SquarePostRow {
  media_items?: SquareFeedMediaItem[];
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
  counts: UserProfileCounts;
  is_following: boolean;
  updated_at: number;
}
