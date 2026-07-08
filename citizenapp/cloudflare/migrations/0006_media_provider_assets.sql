-- 广场主媒体迁移到 Cloudflare Images / Stream。
-- R2 只保留 post manifest；图片/视频的外部资产、状态和播放地址由本表承载。

CREATE TABLE IF NOT EXISTS square_media_assets (
  upload_id TEXT NOT NULL,
  post_id TEXT NOT NULL,
  owner_account TEXT NOT NULL,
  media_index INTEGER NOT NULL,
  media_kind TEXT NOT NULL,
  provider TEXT NOT NULL,
  provider_asset_id TEXT NOT NULL,
  upload_method TEXT NOT NULL,
  content_type TEXT NOT NULL,
  byte_size INTEGER NOT NULL,
  asset_state TEXT NOT NULL,
  delivery_url TEXT,
  playback_hls_url TEXT,
  playback_dash_url TEXT,
  thumbnail_url TEXT,
  duration_seconds REAL,
  width INTEGER,
  height INTEGER,
  error_code TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  ready_at INTEGER,
  PRIMARY KEY(upload_id, media_index)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_square_media_assets_provider_asset
  ON square_media_assets(provider, provider_asset_id);

CREATE INDEX IF NOT EXISTS idx_square_media_assets_post
  ON square_media_assets(post_id, media_index);

CREATE INDEX IF NOT EXISTS idx_square_media_assets_state
  ON square_media_assets(asset_state, updated_at);
