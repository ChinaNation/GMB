-- CID 稳定数据根：数据根归 cid_number 永久所有，不归任何钱包账户。
-- 钱包换绑只推进 active_binding_revision / active_account_id；密封数据根本身不变。
CREATE TABLE IF NOT EXISTS cid_data_roots (
  cid_number TEXT PRIMARY KEY,
  sealed_data_root TEXT NOT NULL,
  seal_nonce TEXT NOT NULL,
  data_root_hash TEXT NOT NULL CHECK(
    length(data_root_hash) = 64
    AND data_root_hash NOT GLOB '*[^0-9a-f]*'
  ),
  active_binding_revision INTEGER NOT NULL CHECK(active_binding_revision > 0),
  active_account_id TEXT NOT NULL CHECK(
    length(active_account_id) = 66
    AND substr(active_account_id, 1, 2) = '0x'
    AND substr(active_account_id, 3) NOT GLOB '*[^0-9a-f]*'
  ),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cid_data_roots_active_binding
  ON cid_data_roots(active_binding_revision, active_account_id);
