-- CitizenApp Cloudflare 唯一目标基线。
-- 新环境和清空后的 staging/production 只执行本文件，不保留历史迁移链。

CREATE TABLE square_login_challenges (
  challenge_id TEXT PRIMARY KEY,
  owner_account TEXT NOT NULL,
  signing_payload TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  used_at INTEGER
);
CREATE INDEX idx_square_login_challenges_owner
  ON square_login_challenges(owner_account, expires_at);

CREATE TABLE square_device_subkeys (
  owner_account TEXT PRIMARY KEY,
  p256_pubkey TEXT NOT NULL,
  issued_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE square_request_nonces (
  nonce_hash TEXT PRIMARY KEY,
  owner_account TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE INDEX idx_square_request_nonces_expires
  ON square_request_nonces(expires_at);

-- 通讯录只保存端到端密文；联系人账户、名称和关系明文不得进入 Cloudflare。
CREATE TABLE square_contacts (
  owner_account TEXT NOT NULL,
  contact_id TEXT NOT NULL CHECK(
    length(contact_id) = 64 AND contact_id NOT GLOB '*[^0-9a-f]*'
  ),
  ciphertext TEXT NOT NULL,
  nonce TEXT NOT NULL,
  mac TEXT NOT NULL,
  updated_at INTEGER NOT NULL CHECK(updated_at > 0),
  PRIMARY KEY(owner_account, contact_id)
);
CREATE INDEX idx_square_contacts_owner_updated
  ON square_contacts(owner_account, updated_at DESC, contact_id DESC);

CREATE TABLE square_rate_windows (
  rate_key TEXT PRIMARY KEY,
  request_count INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE INDEX idx_square_rate_windows_expires
  ON square_rate_windows(expires_at);

-- finalized 链时间单例。Worker 只用它判断订阅权益和镜像新鲜度，不计算公历日期。
CREATE TABLE chain_clock (
  clock_id INTEGER PRIMARY KEY CHECK(clock_id = 1),
  chain_timestamp INTEGER NOT NULL,
  finalized_block_number INTEGER NOT NULL,
  finalized_block_hash TEXT NOT NULL,
  observed_at INTEGER NOT NULL
);

-- 平台订阅 finalized 镜像。钱包账户是唯一业务主键；价格、状态和时间只来自链上。
CREATE TABLE square_memberships (
  owner_account TEXT PRIMARY KEY,
  membership_level TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  last_charged_at INTEGER NOT NULL,
  last_charged_price_fen INTEGER NOT NULL,
  paid_until INTEGER NOT NULL,
  subscription_status TEXT NOT NULL CHECK(subscription_status IN ('active', 'cancelled', 'terminated', 'suspended', 'creatorPaused')),
  finalized_block_number INTEGER NOT NULL,
  finalized_block_hash TEXT NOT NULL,
  verified_at INTEGER NOT NULL,
  entitlement_lapsed_at INTEGER,
  last_tx_hash TEXT
);
CREATE INDEX idx_square_memberships_state
  ON square_memberships(subscription_status, paid_until);
CREATE INDEX idx_square_memberships_lapsed
  ON square_memberships(entitlement_lapsed_at)
  WHERE entitlement_lapsed_at IS NOT NULL;
CREATE INDEX idx_square_memberships_reconcile
  ON square_memberships(subscription_status, paid_until, verified_at);

-- 创作者档位展示镜像。每档以创作者钱包账户 + tier_id 为关系主键；价格仍以链上为真源。
CREATE TABLE square_creator_tiers (
  creator_account TEXT NOT NULL,
  tier_id TEXT NOT NULL,
  name TEXT NOT NULL,
  tier_order INTEGER NOT NULL,
  monthly_price_fen INTEGER,
  quarterly_price_fen INTEGER,
  yearly_price_fen INTEGER,
  finalized_block_number INTEGER NOT NULL,
  finalized_block_hash TEXT NOT NULL,
  verified_at INTEGER NOT NULL,
  last_tx_hash TEXT NOT NULL,
  PRIMARY KEY(creator_account, tier_id)
);

-- 创作者订阅关系必须使用订阅者钱包 + 创作者钱包复合主键，允许同一账户订阅多个创作者。
CREATE TABLE square_creator_subscriptions (
  subscriber_account TEXT NOT NULL,
  creator_account TEXT NOT NULL,
  tier_id TEXT NOT NULL,
  billing_period TEXT NOT NULL CHECK(billing_period IN ('monthly', 'quarterly', 'yearly')),
  started_at INTEGER NOT NULL,
  last_charged_at INTEGER NOT NULL,
  last_charged_price_fen INTEGER NOT NULL,
  paid_until INTEGER NOT NULL,
  subscription_status TEXT NOT NULL CHECK(subscription_status IN ('active', 'cancelled', 'terminated', 'suspended', 'creatorPaused')),
  finalized_block_number INTEGER NOT NULL,
  finalized_block_hash TEXT NOT NULL,
  verified_at INTEGER NOT NULL,
  last_tx_hash TEXT NOT NULL,
  PRIMARY KEY(subscriber_account, creator_account)
);
CREATE INDEX idx_square_creator_subscriptions_creator
  ON square_creator_subscriptions(creator_account, subscription_status, paid_until);
CREATE INDEX idx_square_creator_subscriptions_reconcile
  ON square_creator_subscriptions(subscription_status, paid_until, verified_at);

-- Cloudflare 只保留 finalized 交易的最小不可变证明；完整交易仍在链上，避免重复占用 D1。
CREATE TABLE chain_transaction_confirmations (
  tx_hash TEXT PRIMARY KEY,
  owner_account TEXT NOT NULL,
  block_hash TEXT NOT NULL,
  block_number INTEGER NOT NULL,
  extrinsic_index INTEGER NOT NULL,
  action_kind TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  chain_timestamp INTEGER NOT NULL,
  confirmed_at INTEGER NOT NULL
);
CREATE INDEX idx_chain_transaction_confirmations_owner
  ON chain_transaction_confirmations(owner_account, confirmed_at DESC);

CREATE TABLE square_uploads (
  upload_id TEXT PRIMARY KEY,
  post_id TEXT NOT NULL UNIQUE,
  owner_account TEXT NOT NULL,
  post_category TEXT NOT NULL,
  manifest_hash TEXT NOT NULL,
  content_hash TEXT,
  storage_receipt_id TEXT,
  estimated_bytes INTEGER NOT NULL,
  object_keys_json TEXT NOT NULL,
  status TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  completed_at INTEGER
);
CREATE INDEX idx_square_uploads_owner
  ON square_uploads(owner_account, status, created_at);
CREATE INDEX idx_square_uploads_expires
  ON square_uploads(status, expires_at);

CREATE TABLE square_media_assets (
  upload_id TEXT NOT NULL,
  post_id TEXT NOT NULL,
  owner_account TEXT NOT NULL,
  media_index INTEGER NOT NULL,
  media_kind TEXT NOT NULL,
  provider TEXT NOT NULL,
  provider_asset_id TEXT NOT NULL,
  upload_method TEXT NOT NULL,
  resource_key TEXT NOT NULL,
  content_type TEXT NOT NULL,
  byte_size INTEGER NOT NULL,
  asset_state TEXT NOT NULL,
  declared_duration_seconds REAL,
  duration_seconds REAL,
  width INTEGER,
  height INTEGER,
  error_code TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  ready_at INTEGER,
  archive_state TEXT NOT NULL DEFAULT 'live',
  archived_at INTEGER,
  r2_archive_key TEXT,
  PRIMARY KEY(upload_id, media_index)
);
CREATE UNIQUE INDEX idx_square_media_provider_asset
  ON square_media_assets(provider, provider_asset_id);
CREATE INDEX idx_square_media_post
  ON square_media_assets(post_id, media_index);
CREATE INDEX idx_square_media_state
  ON square_media_assets(asset_state, updated_at);
CREATE INDEX idx_square_media_archive
  ON square_media_assets(owner_account, archive_state);

CREATE TABLE square_posts (
  post_id TEXT PRIMARY KEY,
  owner_account TEXT NOT NULL,
  cid_number TEXT,
  post_category TEXT NOT NULL,
  content_format TEXT NOT NULL,
  title TEXT,
  text TEXT NOT NULL DEFAULT '',
  content_hash TEXT NOT NULL,
  storage_receipt_id TEXT NOT NULL,
  chain_block INTEGER,
  created_at INTEGER NOT NULL,
  post_state TEXT NOT NULL
);
CREATE INDEX idx_square_posts_feed
  ON square_posts(post_category, post_state, created_at);
CREATE INDEX idx_square_posts_owner
  ON square_posts(owner_account, post_state, created_at);
CREATE INDEX idx_square_posts_owner_format
  ON square_posts(owner_account, post_state, content_format, created_at);

CREATE TABLE square_follows (
  owner_account TEXT NOT NULL,
  followed_account TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  notify_enabled INTEGER NOT NULL DEFAULT 1,  -- 关注即默认开发帖通知；0=对该关注静音（仍在关注流，只是不进红点/推送）
  PRIMARY KEY(owner_account, followed_account)
);
CREATE INDEX idx_square_follows_followed
  ON square_follows(followed_account, created_at);

-- 发帖通知「已读游标」：双游标分别驱动广场底部 tab 与关注子 tab 两个红点。
-- 红点数 = 我 notify_enabled=1 的关注在对应游标之后发布的新帖数。
-- 进广场清 last_seen_square_at、进关注子 tab 清 last_seen_following_at；只进广场不进关注→广场清、关注留。
CREATE TABLE square_notify_reads (
  owner_account TEXT NOT NULL PRIMARY KEY,
  last_seen_square_at INTEGER NOT NULL DEFAULT 0,
  last_seen_following_at INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE square_user_signals (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  owner_account TEXT NOT NULL,
  post_id TEXT NOT NULL,
  signal_type TEXT NOT NULL,
  weight REAL NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE INDEX idx_square_user_signals_owner
  ON square_user_signals(owner_account, created_at);

CREATE TABLE square_browse_days (
  owner_account TEXT NOT NULL,
  browse_day TEXT NOT NULL,
  browse_count INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, browse_day)
);

CREATE TABLE resource_reservations (
  reservation_id TEXT PRIMARY KEY,
  owner_account TEXT NOT NULL,
  resource_key TEXT NOT NULL,
  period_start INTEGER NOT NULL,
  period_end INTEGER NOT NULL,
  byte_size INTEGER NOT NULL,
  image_count INTEGER NOT NULL,
  video_seconds INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  reservation_state TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  used_at INTEGER
);
CREATE INDEX idx_resource_reservations_owner
  ON resource_reservations(owner_account, resource_key, reservation_state, expires_at);

CREATE TABLE resource_usage (
  owner_account TEXT NOT NULL,
  resource_key TEXT NOT NULL,
  period_start INTEGER NOT NULL,
  period_end INTEGER NOT NULL,
  byte_size INTEGER NOT NULL,
  image_count INTEGER NOT NULL,
  video_seconds INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, resource_key, period_start)
);

CREATE TABLE resource_totals (
  resource_key TEXT PRIMARY KEY,
  byte_size INTEGER NOT NULL,
  object_count INTEGER NOT NULL,
  video_seconds INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE chain_extrinsic_relays (
  relay_id TEXT PRIMARY KEY,
  extrinsic_sha256 TEXT NOT NULL,
  tx_hash TEXT,
  request_ip_hash TEXT NOT NULL,
  byte_size INTEGER NOT NULL,
  relay_status TEXT NOT NULL,
  error_code TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX idx_chain_extrinsic_relays_extrinsic
  ON chain_extrinsic_relays(extrinsic_sha256, relay_status, created_at);
CREATE INDEX idx_chain_extrinsic_relays_request_ip
  ON chain_extrinsic_relays(request_ip_hash, created_at);
CREATE INDEX idx_chain_extrinsic_relays_tx_hash
  ON chain_extrinsic_relays(tx_hash)
  WHERE tx_hash IS NOT NULL;

-- Chat 云端只保存建立端到端通道所需的最小公开材料。
-- Chat 消息、会话、附件及联系人明文禁止进入 D1、KV、R2 或 Durable Object Storage。
CREATE TABLE chat_devices (
  owner_account TEXT NOT NULL,
  device_id TEXT NOT NULL,
  device_public_key_hex TEXT NOT NULL,
  push_provider TEXT NOT NULL,
  push_token TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, device_id)
);
CREATE INDEX idx_chat_devices_owner
  ON chat_devices(owner_account, expires_at);

CREATE TABLE chat_keypackages (
  owner_account TEXT NOT NULL,
  device_id TEXT NOT NULL,
  key_package_id TEXT PRIMARY KEY,
  key_package TEXT NOT NULL,
  cipher_suite TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE INDEX idx_chat_keypackages_available
  ON chat_keypackages(owner_account, expires_at, created_at);

CREATE TABLE chat_device_binding_nonces (
  owner_account TEXT NOT NULL,
  nonce_hash TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, nonce_hash)
);
CREATE INDEX idx_chat_device_binding_nonces_expires
  ON chat_device_binding_nonces(expires_at);
