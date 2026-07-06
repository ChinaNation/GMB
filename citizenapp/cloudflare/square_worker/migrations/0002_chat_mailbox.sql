CREATE TABLE IF NOT EXISTS chat_devices (
  owner_account TEXT NOT NULL,
  device_id TEXT NOT NULL,
  device_public_key_hex TEXT NOT NULL,
  binding_signature TEXT NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  revoked_at INTEGER,
  PRIMARY KEY(owner_account, device_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_devices_owner
  ON chat_devices(owner_account, revoked_at, expires_at);

CREATE TABLE IF NOT EXISTS chat_keypackages (
  owner_account TEXT NOT NULL,
  device_id TEXT NOT NULL,
  key_package_id TEXT PRIMARY KEY,
  key_package TEXT NOT NULL,
  cipher_suite TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  consumed_at INTEGER,
  consumed_by_account TEXT
);

CREATE INDEX IF NOT EXISTS idx_chat_keypackages_available
  ON chat_keypackages(owner_account, consumed_at, expires_at, created_at);

CREATE TABLE IF NOT EXISTS chat_envelopes (
  envelope_id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  sender_account TEXT NOT NULL,
  sender_device_id TEXT NOT NULL,
  recipient_account TEXT NOT NULL,
  recipient_device_id TEXT,
  mls_message_kind TEXT NOT NULL,
  encrypted_payload TEXT NOT NULL,
  attachment_manifest_key TEXT,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chat_envelopes_pending
  ON chat_envelopes(recipient_account, recipient_device_id, expires_at, created_at);

CREATE INDEX IF NOT EXISTS idx_chat_envelopes_conversation
  ON chat_envelopes(conversation_id, created_at);
