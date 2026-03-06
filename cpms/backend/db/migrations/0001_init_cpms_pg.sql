BEGIN;

CREATE TABLE IF NOT EXISTS system_install (
  id SMALLINT PRIMARY KEY CHECK (id = 1),
  site_sfid TEXT,
  initialized_at BIGINT
);

INSERT INTO system_install (id, site_sfid, initialized_at)
VALUES (1, NULL, NULL)
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS qr_sign_keys (
  key_id TEXT PRIMARY KEY,
  purpose TEXT NOT NULL,
  status TEXT NOT NULL,
  pubkey TEXT NOT NULL,
  secret TEXT NOT NULL,
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_users (
  user_id TEXT PRIMARY KEY,
  admin_pubkey TEXT NOT NULL UNIQUE,
  role TEXT NOT NULL CHECK (role IN ('SUPER_ADMIN', 'OPERATOR_ADMIN')),
  status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'DISABLED')),
  immutable BOOLEAN NOT NULL DEFAULT FALSE,
  managed_key_id TEXT UNIQUE,
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_admin_users_role ON admin_users (role);
CREATE INDEX IF NOT EXISTS idx_admin_users_status ON admin_users (status);

CREATE TABLE IF NOT EXISTS sessions (
  access_token TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  role TEXT NOT NULL,
  expires_at BIGINT NOT NULL,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);

CREATE TABLE IF NOT EXISTS login_challenges (
  challenge_id TEXT PRIMARY KEY,
  admin_pubkey TEXT NOT NULL,
  challenge_payload TEXT NOT NULL,
  session_id TEXT NOT NULL,
  expire_at BIGINT NOT NULL,
  consumed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_login_challenges_expire_at ON login_challenges (expire_at);

CREATE TABLE IF NOT EXISTS qr_login_results (
  challenge_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  access_token TEXT NOT NULL,
  expires_in BIGINT NOT NULL,
  user_id TEXT NOT NULL,
  role TEXT NOT NULL,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_qr_login_results_created_at ON qr_login_results (created_at);

CREATE TABLE IF NOT EXISTS archives (
  archive_id TEXT PRIMARY KEY,
  archive_no TEXT NOT NULL UNIQUE,
  province_code TEXT NOT NULL,
  city_code TEXT NOT NULL,
  full_name TEXT NOT NULL,
  birth_date TEXT NOT NULL,
  gender_code TEXT NOT NULL CHECK (gender_code IN ('M', 'W')),
  height_cm REAL,
  passport_no TEXT NOT NULL,
  status TEXT NOT NULL,
  citizen_status TEXT NOT NULL CHECK (citizen_status IN ('NORMAL', 'ABNORMAL')),
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_archives_full_name ON archives (full_name);
CREATE INDEX IF NOT EXISTS idx_archives_status ON archives (status);

CREATE TABLE IF NOT EXISTS sequence_counters (
  seq_key TEXT PRIMARY KEY,
  next_seq BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS qr_print_records (
  print_id TEXT PRIMARY KEY,
  archive_id TEXT NOT NULL,
  archive_no TEXT NOT NULL,
  citizen_status TEXT NOT NULL,
  voting_eligible BOOLEAN NOT NULL,
  printed_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_qr_print_records_archive_id ON qr_print_records (archive_id);

CREATE TABLE IF NOT EXISTS audit_logs (
  log_id TEXT PRIMARY KEY,
  operator_user_id TEXT,
  action TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_id TEXT,
  result TEXT NOT NULL CHECK (result IN ('SUCCESS', 'FAILED')),
  detail JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs (created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_created_at ON audit_logs (action, created_at);

COMMIT;
