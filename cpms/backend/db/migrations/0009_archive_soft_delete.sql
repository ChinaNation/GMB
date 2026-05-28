-- CPMS 公民档案软删除与删除签名 challenge。
-- 中文注释：删除档案必须由当前登录管理员用 wumin 签名确认，真实档案数据不做物理删除。

ALTER TABLE archives
  ADD COLUMN IF NOT EXISTS deleted_at BIGINT,
  ADD COLUMN IF NOT EXISTS deleted_by TEXT,
  ADD COLUMN IF NOT EXISTS delete_reason TEXT;

CREATE TABLE IF NOT EXISTS archive_delete_challenges (
  challenge_id TEXT PRIMARY KEY,
  archive_id TEXT NOT NULL,
  archive_no TEXT NOT NULL,
  admin_id TEXT NOT NULL,
  admin_pubkey TEXT NOT NULL,
  delete_payload TEXT NOT NULL,
  expire_at BIGINT NOT NULL,
  consumed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at BIGINT NOT NULL,
  consumed_at BIGINT
);

CREATE INDEX IF NOT EXISTS idx_archive_delete_challenges_archive_id
  ON archive_delete_challenges (archive_id);

CREATE INDEX IF NOT EXISTS idx_archive_delete_challenges_expire_at
  ON archive_delete_challenges (expire_at);
