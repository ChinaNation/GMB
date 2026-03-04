BEGIN;

-- Keep encrypted columns only; remove plaintext PII columns.
ALTER TABLE archive_bindings
  DROP COLUMN IF EXISTS identity_code,
  DROP COLUMN IF EXISTS birth_date;

DROP INDEX IF EXISTS idx_archive_bindings_birth_date;

COMMIT;
