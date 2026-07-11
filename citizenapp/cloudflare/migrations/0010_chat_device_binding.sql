-- Chat 设备绑定改为硬件 P-256 设备子钥签名。
-- 严格切换：清理旧钱包签名设备绑定和旧 KeyPackage；用户密文投递 chat_envelopes 保留。
DELETE FROM chat_keypackages;
DELETE FROM chat_devices;

-- owner + nonce 是一次性重放闸门；过期记录由设备登记接口按 expires_at 清理。
CREATE TABLE IF NOT EXISTS chat_device_binding_nonces (
  owner_account TEXT NOT NULL,
  nonce TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  PRIMARY KEY(owner_account, nonce)
);

CREATE INDEX IF NOT EXISTS idx_chat_device_binding_nonces_expires
  ON chat_device_binding_nonces(expires_at);
