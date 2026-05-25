BEGIN;

ALTER TABLE archives
  ADD COLUMN IF NOT EXISTS valid_from TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS valid_until TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS citizen_status_updated_at BIGINT NOT NULL DEFAULT 0;

UPDATE archives
SET
  valid_from = CASE
    WHEN valid_from = '' THEN to_char((to_timestamp(created_at) AT TIME ZONE 'UTC')::date, 'YYYY-MM-DD')
    ELSE valid_from
  END,
  valid_until = CASE
    WHEN valid_until = '' THEN to_char(((to_timestamp(created_at) AT TIME ZONE 'UTC')::date + INTERVAL '10 years' - INTERVAL '1 day')::date, 'YYYY-MM-DD')
    ELSE valid_until
  END,
  citizen_status_updated_at = CASE
    WHEN citizen_status_updated_at = 0 THEN updated_at
    ELSE citizen_status_updated_at
  END;

COMMIT;
