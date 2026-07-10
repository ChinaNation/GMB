-- P-256 设备子钥绑定：后台握手（广场 session / IM 设备绑定）改用硬件 P-256 子钥
-- 静默签名，不再读 sr25519 seed。一账户一活跃子钥（换机/轮换时重注册覆盖）。
CREATE TABLE IF NOT EXISTS square_device_subkeys (
  owner_account TEXT PRIMARY KEY,
  p256_pubkey TEXT NOT NULL,
  issued_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
