export type PostCategory = 'normal' | 'campaign';

export type PostContentFormat = 'normal' | 'article';

export type MediaKind = 'image' | 'video' | 'cover';

export type UploadStatus = 'prepared' | 'completed';

export type FeedKind = 'recommended' | 'following' | 'campaign';

export type MediaProvider = 'cloudflare_images' | 'cloudflare_stream';

export type MediaUploadMethod = 'worker' | 'tus';

export type MediaAssetState = 'prepared' | 'uploaded' | 'processing' | 'ready' | 'error';

/// 广场发帖通知扇出队列消息：一条 = 一次发帖事件，或一页续跑（cursor 空=首页）。
/// author_name 入队时读一次作者展示名、续跑复用，避免每页重读；cursor 为 keyset 续跑游标。
export interface SquareNotifyJob {
  author_cid_number: string;
  author_name: string;
  content_format: 'normal' | 'article';
  post_id: string;
  cursor?: { created_at: number; cid_number: string };
}

/// Wrangler 根据 wrangler.toml 生成普通变量与资源绑定；Secret 仅保留名称契约，不写入配置或生成物。
interface WorkerSecretsAndOptionalVars {
  // 平台推送只发送无内容 Chat 唤醒；私钥只允许使用 Worker Secret 配置。
  APNS_KEY?: string;
  APNS_KID?: string;
  APNS_TEAM?: string;
  APNS_TOPIC?: string;
  FCM_PROJECT?: string;
  FCM_EMAIL?: string;
  FCM_KEY?: string;
  // Cloudflare 账户只用于 Images / Stream 服务端 API。
  CF_ACCOUNT_ID?: string;
  // Worker 通过 Access + Tunnel 调用权威节点回环 RPC；URL 和服务令牌只放远端 Secret。
  CHAIN_URL?: string;
  CHAIN_ID?: string;
  CHAIN_SECRET?: string;
  // 官网「公民宪法」tab 读链文档的 KV 短缓存 TTL（秒，缺省 300）。修宪后一个 TTL 内自动刷新。
  CONSTITUTION_TTL_SECONDS?: string;
  // Cloudflare Images / Stream API token 只放 Worker Secret；App 只拿一次性上传 URL。
  CF_API_TOKEN?: string;
  STREAM_HOOK_SECRET?: string;
  IMAGES_SIGNING_KEY?: string;
  TURNSTILE_SECRET?: string;
  HASH_KEY?: string;
  // 本地部署控制台↔Worker 结算接口鉴权令牌，只放 Worker Secret。
  TOPUP_SETTLE_TOKEN?: string;
  // 付款意图 HMAC 密钥，只放 Worker Secret；用于把登录账户、付款钱包和报价绑定为短期令牌。
  TOPUP_INTENT_SECRET?: string;
}

/// Wrangler 会把配置值推导为字面量；Worker 运行期仍需接受测试覆盖值和控制台注入的字符串。
type WidenWorkerVar<T> = T extends string ? string : T;
type GeneratedBindings = {
  [K in keyof CloudflareBindings]: WidenWorkerVar<CloudflareBindings[K]>;
};

/// 数据库、广场媒体桶和缓存是基础能力；其余变量与资源延续原契约，缺失时由业务入口 fail-closed。
type RequiredRuntimeBinding = 'DB' | 'SQUARE_MEDIA' | 'SQUARE_CACHE';
type SpecializedRuntimeBinding = 'CHAT_REALTIME' | 'SQUARE_NOTIFY_QUEUE';
type RuntimeBindings =
  Pick<GeneratedBindings, RequiredRuntimeBinding>
  & Partial<Omit<GeneratedBindings, RequiredRuntimeBinding | SpecializedRuntimeBinding>>
  & {
    CHAT_REALTIME?: DurableObjectNamespace;
    SQUARE_NOTIFY_QUEUE?: Queue<SquareNotifyJob>;
  };

/// Worker 唯一环境类型 = Wrangler 生成的真实绑定 + 不可写入 wrangler.toml 的 Secret 名称。
export type Env = RuntimeBindings & WorkerSecretsAndOptionalVars;

export interface SessionState {
  /// 用户唯一身份主键。会话即以 cid_number 为身份。
  cid_number: string;
  /// 签发本会话时的 CID 单调绑定版本；每请求必须与 finalized 精确一致。
  binding_revision: number;
  /// 签发本会话时该 cid_number 链上绑定的钱包账户;用于定位设备子钥 + 每请求复查绑定。
  account_id: string;
  device_key_hash: string;
  created_at: number;
  expires_at: number;
}

export interface LoginChallengeRow {
  challenge_id: string;
  /// 挑战归属的唯一身份主键；account_id 只是该挑战要求的签名账户。
  cid_number: string;
  binding_revision: number;
  account_id: string;
  signing_payload: string;
  expires_at: number;
  used_at: number | null;
}

export interface DeviceSubkeyRow {
  /// 身份主键:设备子钥挂在其当前绑定钱包账户对应的 cid_number 下。
  cid_number: string;
  /// 设备标识 = P-256 公钥的 sha256(同一身份多设备各一行)。
  device_id: string;
  /// 该设备证明被当前账户授权时的 CID 绑定版本。
  binding_revision: number;
  /// 生成该子钥的钱包账户(换绑后由链上绑定校验判活/失效)。
  account_id: string;
  p256_public_key: string;
  issued_at: number;
  created_at: number;
  updated_at: number;
}

/// 端到端加密通讯录行。Worker 只保存不透明密文，绝不接收联系人账户或名称明文。
/// 属主键 = 身份主键 cid_number(换绑后随身份保留)。
export interface ContactCiphertextRow {
  cid_number: string;
  /// 密文派生上下文；只作钱包换绑版本隔离，不改变 CID 属主。
  binding_revision: number;
  account_id: string;
  contact_id: string;
  ciphertext: string;
  nonce: string;
  mac: string;
  updated_at: number;
}

export interface MembershipRow {
  /// 身份主键:会员镜像归属的 cid_number。
  cid_number: string;
  /// 当前绑定的付款/签名钱包账户(链上事实保留);不作身份归属键。
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
  /// 身份主键:发起上传的 cid_number。
  cid_number: string;
  /// 发起上传的钱包账户(当前绑定=发布签名者);作事实保留,不作身份归属键。
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
  /// 身份主键:媒体所属 cid_number(随其 upload 归属)。
  cid_number: string;
  /// 上传该媒体的钱包账户(当前绑定);作事实保留,不作身份归属键。
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
}

export interface SquarePostRow {
  post_id: string;
  /// 身份主键:发布者 cid_number(链上 SquarePostPublished 事件镜像)。
  cid_number: string;
  /// 发布该帖的钱包账户(链上签名者=当前绑定);作事实保留,不作身份归属键。
  account_id: string;
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

/// R2 公开资料包（citizenapp.square.profile）。
/// 头像/背景/签名/展示名等公开链下资料的唯一真源。
export interface CitizenProfileDoc {
  schema: 'citizenapp.square.profile';
  /// 身份主键:资料所属 cid_number(随身份走,换绑不丢)。
  cid_number: string;
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

/// GET /square/users/:account 响应载荷。
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
