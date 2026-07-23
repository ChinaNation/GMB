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

/// 广场发帖通知扇出队列消息：一条 = 一次发帖事件，或一页续跑（cursor 空=首页）。
/// author_name 入队时读一次作者展示名、续跑复用，避免每页重读；cursor 为 keyset 续跑游标。
export interface SquareNotifyJob {
  author_account_id: string;
  author_name: string;
  content_format: 'normal' | 'article';
  post_id: string;
  cursor?: { created_at: number; account_id: string };
}

export interface Env {
  DB: D1Database;
  SQUARE_MEDIA: R2Bucket;
  // 大媒体(>100MB)瞬时中转专用桶(桶级 24h 生命周期 TTL);只存薪火 + >100MB 的 E2E 密文。
  CHAT_RELAY?: R2Bucket;
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
  // 广场发帖通知扇出队列（producer 入队、consumer 分页跨调用推完全部未静音粉丝）。
  SQUARE_NOTIFY_QUEUE?: Queue<SquareNotifyJob>;
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
  // 官网「公民宪法」tab 读链文档的 KV 短缓存 TTL（秒，缺省 300）。修宪后一个 TTL 内自动刷新。
  CONSTITUTION_TTL_SECONDS?: string;
  CHAIN_GENESIS_HASH?: string;
  CHAIN_STATE_ROOT?: string;
  // 已签名交易兜底广播：只转发完整 signed extrinsic，不提供通用 JSON-RPC proxy。
  RELAY_ENABLED?: string;
  RELAY_MAX_BYTES?: string;
  RELAY_PER_MINUTE?: string;
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
  // 会员镜像对账：平台/创作者各自开关（'1' 开）+ 共用每轮批量（缺省 50，上限 500）。
  // 均为 wrangler 默认值；运行期以 KV 开关（flag:membership_reconcile / flag:creator_reconcile）
  // 优先，供控制台即时开关。关闭时 Cron 对账内部直接返回。
  MEMBERSHIP_RECONCILE_ENABLED?: string;
  CREATOR_RECONCILE_ENABLED?: string;
  MEMBERSHIP_RECONCILE_BATCH?: string;
  // 稳定币充值购买公民币（topup）：网络 / 收款地址 / 各链 EVM RPC / 合约覆盖 / 确认数 / 结算令牌。
  // 'mainnet' | 'testnet'（缺省 testnet，沙箱期）。
  TOPUP_NETWORK?: string;
  // 平台/国储会 EVM 收款地址（同一 EOA 跨链复用）。
  TOPUP_RECV_ADDRESS?: string;
  // 各链 EVM JSON-RPC（必须 https）；若 URL 内嵌 API key 则改用 wrangler secret。
  TOPUP_BASE_RPC_URL?: string;
  TOPUP_ARBITRUM_RPC_URL?: string;
  // 覆盖代币合约地址（testnet mock USDT 必填；mainnet 用代码内置默认）。
  TOPUP_USDC_CONTRACT?: string;
  TOPUP_USDT_CONTRACT?: string;
  // 最小确认数；>0 按 latest 计算，=0（缺省）按 finalized 区块判定。
  TOPUP_MIN_CONFIRMATIONS?: string;
  // 本地部署控制台↔Worker 结算接口鉴权令牌，只放 Worker Secret。
  TOPUP_SETTLE_TOKEN?: string;
}

export interface SessionState {
  account_id: string;
  device_key_hash: string;
  created_at: number;
  expires_at: number;
}

export interface LoginChallengeRow {
  challenge_id: string;
  account_id: string;
  signing_payload: string;
  expires_at: number;
  used_at: number | null;
}

export interface DeviceSubkeyRow {
  account_id: string;
  p256_public_key: string;
  issued_at: number;
  created_at: number;
  updated_at: number;
}

/// 端到端加密通讯录行。Worker 只保存不透明密文，绝不接收联系人账户或名称明文。
export interface ContactCiphertextRow {
  account_id: string;
  contact_id: string;
  ciphertext: string;
  nonce: string;
  mac: string;
  updated_at: number;
}

export interface MembershipRow {
  account_id: string;
  membership_level: string;
  started_at: number;
  last_charged_at: number;
  last_charged_price_fen: number;
  paid_until: number;
  subscription_status: string;
  finalized_block_number: number;
  finalized_block_hash: string;
  verified_at: number;
  entitlement_lapsed_at: number | null;
  last_tx_hash: string | null;
  // 由查询与 chain_clock 单例联结；缺失或过期时所有边缘权益 fail-closed。
  chain_timestamp: number | null;
  chain_observed_at: number | null;
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
  account_id: string;
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
  account_id: string;
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
  account_id: string;
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
  // 作者徽章信号（公开）：身份档=颜色、会员有效=勾。由本页去重作者统一读链上身份+批量读会员填充。
  // identity_level 是链上身份档；membership_level 是已购买会员档；二者已解耦（ADR-036）。
  identity_level?: 'visitor' | 'voting' | 'candidate';
  membership_level?: 'freedom' | 'democracy' | 'spark' | null;
  membership_active?: boolean;
  // 作者展示名与头像对象键（取自作者 profile.json），供 feed 直出真名和真头像。
  display_name?: string;
  avatar_object_key?: string | null;
  // 文章正文图文块（内联图 media_index 引用 media_items）；动态/旧文章为 null。
  content_blocks?: { t: 'text' | 'image'; text?: string; media_index?: number }[] | null;
}

/// 按作者拉帖的分类过滤维度。'all' 表示不过滤。
export type AuthorPostCategory = 'all' | PostCategory;

/// 按作者拉帖的内容形态过滤。'all' 不过滤；'normal' 排除文章；'article' 只看文章。
export type AuthorContentFormat = 'all' | PostContentFormat;

/// R2 公开资料包（citizenapp.square.profile.v1）。
/// 头像/背景/签名/展示名等公开链下资料的唯一真源。
export interface CitizenProfileDoc {
  schema: 'citizenapp.square.profile.v1';
  account_id: string;
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
  account_id: string;
  display_name: string;
  bio: string;
  avatar_object_key: string | null;
  banner_object_key: string | null;
  cid_number: string | null;
  is_certified: boolean;
  /// 链上身份档位：visitor 未认证 / voting 认证投票公民 / candidate 认证竞选公民。
  identity_level: 'visitor' | 'voting' | 'candidate';
  /// 已购买的会员档位（公开，与身份解耦）；未购买为 null。徽章「勾」= 会员有效。
  membership_level: 'freedom' | 'democracy' | 'spark' | null;
  /// 会员是否当前有效（订阅生效且未过期）。
  membership_active: boolean;
  counts: UserProfileCounts;
  is_following: boolean;
  /// 当前登录者是否对该账户开启发帖通知（= 已关注且未静音）；本人视角恒为 false。
  is_notifying: boolean;
  updated_at: number;
}
