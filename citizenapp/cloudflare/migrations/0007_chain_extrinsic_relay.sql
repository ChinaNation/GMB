-- 已签名交易受控广播兜底。
-- 本表只记录完整 signed extrinsic 的哈希、广播状态和限流去重信息；
-- 不保存私钥、助记词、签名 payload、RPC URL 或原始 extrinsic body。

CREATE TABLE IF NOT EXISTS chain_extrinsic_relays (
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

CREATE INDEX IF NOT EXISTS idx_chain_extrinsic_relays_extrinsic
  ON chain_extrinsic_relays(extrinsic_sha256, relay_status, created_at);

CREATE INDEX IF NOT EXISTS idx_chain_extrinsic_relays_request_ip
  ON chain_extrinsic_relays(request_ip_hash, created_at);

CREATE INDEX IF NOT EXISTS idx_chain_extrinsic_relays_tx_hash
  ON chain_extrinsic_relays(tx_hash)
  WHERE tx_hash IS NOT NULL;
