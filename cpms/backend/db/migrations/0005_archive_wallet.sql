BEGIN;

ALTER TABLE archives
  ADD COLUMN IF NOT EXISTS wallet_address TEXT,
  ADD COLUMN IF NOT EXISTS wallet_pubkey TEXT,
  ADD COLUMN IF NOT EXISTS wallet_sig_alg TEXT NOT NULL DEFAULT 'sr25519',
  ADD COLUMN IF NOT EXISTS wallet_proof_payload TEXT,
  ADD COLUMN IF NOT EXISTS wallet_signature TEXT,
  ADD COLUMN IF NOT EXISTS wallet_bound_at BIGINT,
  ADD COLUMN IF NOT EXISTS wallet_bound_by TEXT;

CREATE TABLE IF NOT EXISTS archive_wallet_challenges (
  challenge_id TEXT PRIMARY KEY,
  archive_id TEXT NOT NULL,
  archive_no TEXT NOT NULL,
  wallet_address TEXT,
  wallet_pubkey TEXT,
  wallet_proof_payload TEXT NOT NULL,
  expire_at BIGINT NOT NULL,
  consumed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_archive_wallet_challenges_archive_id
  ON archive_wallet_challenges (archive_id);

CREATE INDEX IF NOT EXISTS idx_archive_wallet_challenges_expire_at
  ON archive_wallet_challenges (expire_at);

COMMIT;
