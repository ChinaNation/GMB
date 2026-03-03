BEGIN;

CREATE TABLE IF NOT EXISTS runtime_cache_entries (
  entry_key TEXT PRIMARY KEY,
  payload JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Backfill from legacy single-blob runtime_misc if present.
INSERT INTO runtime_cache_entries(entry_key, payload, updated_at)
SELECT k.key, k.value, rm.updated_at
FROM runtime_misc rm,
LATERAL jsonb_each(rm.payload) AS k(key, value)
ON CONFLICT (entry_key) DO UPDATE
SET payload = EXCLUDED.payload,
    updated_at = now();

-- PII column-level encryption materialization.
ALTER TABLE archive_bindings
  ADD COLUMN IF NOT EXISTS identity_code_enc BYTEA,
  ADD COLUMN IF NOT EXISTS birth_date_enc BYTEA,
  ADD COLUMN IF NOT EXISTS pii_key_version SMALLINT NOT NULL DEFAULT 1;

DO $$
DECLARE
  key_text TEXT;
BEGIN
  key_text := current_setting('app.pii_key', true);
  IF key_text IS NOT NULL AND length(trim(key_text)) > 0 THEN
    UPDATE archive_bindings
    SET
      identity_code_enc = pgp_sym_encrypt(identity_code, key_text, 'cipher-algo=aes256,compress-algo=1'),
      birth_date_enc = pgp_sym_encrypt(birth_date::text, key_text, 'cipher-algo=aes256,compress-algo=1')
    WHERE identity_code_enc IS NULL OR birth_date_enc IS NULL;
  END IF;
END$$;

CREATE INDEX IF NOT EXISTS idx_archive_bindings_pii_key_version
  ON archive_bindings(pii_key_version);

COMMIT;
