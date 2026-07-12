CREATE TABLE IF NOT EXISTS square_login_challenges (
  challenge_id TEXT PRIMARY KEY,
  owner_account TEXT NOT NULL,
  signing_payload TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  used_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_square_login_challenges_owner
  ON square_login_challenges(owner_account, expires_at);

CREATE TABLE IF NOT EXISTS square_memberships (
  owner_account TEXT PRIMARY KEY,
  membership_level TEXT NOT NULL,
  storage_quota_bytes INTEGER NOT NULL,
  storage_used_bytes INTEGER NOT NULL DEFAULT 0,
  expires_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS square_uploads (
  upload_id TEXT PRIMARY KEY,
  post_id TEXT NOT NULL,
  owner_account TEXT NOT NULL,
  post_category TEXT NOT NULL,
  manifest_hash TEXT NOT NULL,
  content_hash TEXT,
  storage_receipt_id TEXT,
  estimated_bytes INTEGER NOT NULL,
  object_keys_json TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  completed_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_square_uploads_owner
  ON square_uploads(owner_account, created_at);

CREATE TABLE IF NOT EXISTS square_posts (
  post_id TEXT PRIMARY KEY,
  owner_account TEXT NOT NULL,
  cid_number TEXT,
  post_category TEXT NOT NULL,
  text TEXT NOT NULL DEFAULT '',
  content_hash TEXT NOT NULL,
  storage_receipt_id TEXT NOT NULL,
  chain_block INTEGER,
  created_at INTEGER NOT NULL,
  post_state TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_square_posts_feed
  ON square_posts(post_category, created_at);

CREATE TABLE IF NOT EXISTS square_follows (
  owner_account TEXT NOT NULL,
  followed_account TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, followed_account)
);

CREATE TABLE IF NOT EXISTS square_user_signals (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  owner_account TEXT NOT NULL,
  post_id TEXT NOT NULL,
  signal_type TEXT NOT NULL,
  weight REAL NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_square_user_signals_owner
  ON square_user_signals(owner_account, created_at);

-- Chat 只保存建立设备间端到端通道所需的最小公开材料。
-- 消息、会话、联系人和附件禁止进入 D1、KV、R2 或 Durable Object Storage。
CREATE TABLE IF NOT EXISTS chat_devices (
  owner_account TEXT NOT NULL,
  device_id TEXT NOT NULL,
  device_public_key_hex TEXT NOT NULL,
  push_provider TEXT NOT NULL,
  push_token TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, device_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_devices_owner
  ON chat_devices(owner_account, expires_at);

CREATE TABLE IF NOT EXISTS chat_keypackages (
  owner_account TEXT NOT NULL,
  device_id TEXT NOT NULL,
  key_package_id TEXT PRIMARY KEY,
  key_package TEXT NOT NULL,
  cipher_suite TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chat_keypackages_available
  ON chat_keypackages(owner_account, expires_at, created_at);

CREATE TABLE IF NOT EXISTS chat_device_binding_nonces (
  owner_account TEXT NOT NULL,
  nonce_hash TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, nonce_hash)
);

CREATE INDEX IF NOT EXISTS idx_chat_device_binding_nonces_expires
  ON chat_device_binding_nonces(expires_at);

CREATE TABLE IF NOT EXISTS chat_turn_credentials (
  owner_account TEXT NOT NULL,
  username TEXT PRIMARY KEY,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chat_turn_credentials_owner
  ON chat_turn_credentials(owner_account, expires_at);

CREATE TABLE IF NOT EXISTS square_browse_days (
  owner_account TEXT NOT NULL,
  browse_day TEXT NOT NULL,
  browse_count INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, browse_day)
);
