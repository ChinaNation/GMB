-- CitizenApp 端到端加密通讯录增量迁移。
-- Cloudflare 只保存密文和不透明索引，不保存联系人账户或名称明文。

CREATE TABLE IF NOT EXISTS square_contacts (
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
CREATE INDEX IF NOT EXISTS idx_square_contacts_owner_updated
  ON square_contacts(owner_account, updated_at DESC, contact_id DESC);
