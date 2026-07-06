export type PostCategory = 'normal' | 'campaign';

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
  text: string;
  content_hash: string;
  storage_receipt_id: string;
  chain_block: number | null;
  created_at: number;
  post_state: string;
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
