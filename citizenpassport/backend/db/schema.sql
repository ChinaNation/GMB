-- CPMS PostgreSQL schema (CID_CPMS_V1 two-code baseline)

BEGIN;

CREATE TABLE IF NOT EXISTS system_install (
  id SMALLINT PRIMARY KEY CHECK (id = 1),
  cid_number TEXT,
  install_secret TEXT,
  install_secret_hash TEXT,
  install_sig TEXT,
  province_code TEXT,
  city_code TEXT,
  province_name TEXT,
  city_name TEXT,
  cpms_pubkey TEXT,
  initialized_at BIGINT
);

INSERT INTO system_install (id)
VALUES (1)
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
  admin_account TEXT NOT NULL UNIQUE,
  admin_display_name TEXT NOT NULL DEFAULT '',
  user_group TEXT NOT NULL CHECK (user_group IN ('admins', 'operators')),
  immutable BOOLEAN NOT NULL DEFAULT FALSE,
  managed_key_id TEXT UNIQUE,
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_admin_users_user_group ON admin_users (user_group);

CREATE TABLE IF NOT EXISTS sessions (
  access_token TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  user_group TEXT NOT NULL,
  expires_at BIGINT NOT NULL,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);

CREATE TABLE IF NOT EXISTS login_challenges (
  challenge_id TEXT PRIMARY KEY,
  admin_account TEXT NOT NULL,
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
  user_group TEXT NOT NULL,
  created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_qr_login_results_created_at ON qr_login_results (created_at);

CREATE TABLE IF NOT EXISTS archives (
  archive_id TEXT PRIMARY KEY,
  archive_no TEXT NOT NULL UNIQUE,
  province_code TEXT NOT NULL,
  city_code TEXT NOT NULL,
  last_name TEXT NOT NULL,
  first_name TEXT NOT NULL,
  birth_date DATE NOT NULL,
  gender_code TEXT NOT NULL CHECK (gender_code IN ('M', 'W')),
  height_cm REAL NOT NULL CHECK (height_cm BETWEEN 30 AND 260),
  passport_no TEXT NOT NULL UNIQUE,
  town_code TEXT NOT NULL DEFAULT '',
  address_unit_id TEXT NOT NULL DEFAULT '',
  address_unit_name_snapshot TEXT NOT NULL DEFAULT '',
  address_detail TEXT NOT NULL DEFAULT '',
  address_full_snapshot TEXT NOT NULL DEFAULT '',
  birth_province_code TEXT NOT NULL,
  birth_city_code TEXT NOT NULL,
  birth_town_code TEXT NOT NULL,
  election_scope_level TEXT NOT NULL DEFAULT 'PROVINCE' CHECK (election_scope_level IN ('PROVINCE', 'CITY', 'TOWN')),
  status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'DELETED')),
  citizen_status TEXT NOT NULL CHECK (citizen_status IN ('NORMAL', 'REVOKED')),
  voting_eligible BOOLEAN NOT NULL DEFAULT TRUE,
  valid_from DATE NOT NULL,
  valid_until DATE NOT NULL CHECK (valid_until >= valid_from),
  citizen_status_updated_at BIGINT NOT NULL DEFAULT 0,
  wallet_address TEXT,
  wallet_pubkey TEXT,
  wallet_sig_alg TEXT NOT NULL DEFAULT 'sr25519',
  wallet_bound_at BIGINT,
  wallet_bound_by TEXT,
  archive_qr_payload TEXT NOT NULL DEFAULT '',
  deleted_at BIGINT,
  deleted_by TEXT,
  delete_reason TEXT,
  created_at BIGINT NOT NULL,
  updated_at BIGINT NOT NULL,
  CHECK (citizen_status <> 'REVOKED' OR voting_eligible = FALSE),
  CHECK (
    (status = 'ACTIVE' AND deleted_at IS NULL)
    OR
    (status = 'DELETED' AND deleted_at IS NOT NULL AND citizen_status = 'REVOKED' AND voting_eligible = FALSE)
  )
);

CREATE INDEX IF NOT EXISTS idx_archives_status ON archives (status);
CREATE INDEX IF NOT EXISTS idx_archives_active_created_cursor
  ON archives (created_at DESC, archive_id DESC)
  WHERE status = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_archives_active_archive_no
  ON archives (archive_no)
  WHERE status = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_archives_active_passport_no
  ON archives (passport_no)
  WHERE status = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_archives_active_name_birth_cursor
  ON archives ((last_name || first_name), birth_date, created_at DESC, archive_id DESC)
  WHERE status = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_archives_active_area_cursor
  ON archives (town_code, address_unit_id, created_at DESC, archive_id DESC)
  WHERE status = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_archives_active_citizen_status_cursor
  ON archives (citizen_status, created_at DESC, archive_id DESC)
  WHERE status = 'ACTIVE';
CREATE UNIQUE INDEX IF NOT EXISTS uq_archives_wallet_pubkey_lifetime
  ON archives (wallet_pubkey)
  WHERE wallet_pubkey IS NOT NULL;

CREATE TABLE IF NOT EXISTS archive_stats (
  id SMALLINT PRIMARY KEY CHECK (id = 1),
  active_count BIGINT NOT NULL DEFAULT 0 CHECK (active_count >= 0),
  deleted_count BIGINT NOT NULL DEFAULT 0 CHECK (deleted_count >= 0),
  updated_at BIGINT NOT NULL
);

INSERT INTO archive_stats (id, active_count, deleted_count, updated_at)
VALUES (1, 0, 0, 0)
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS sequence_counters (
  seq_key TEXT PRIMARY KEY,
  next_seq BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS archive_number_recycle_pool (
  pool_id TEXT PRIMARY KEY,
  archive_no TEXT NOT NULL,
  passport_no TEXT NOT NULL,
  source_archive_id TEXT NOT NULL UNIQUE,
  deleted_at BIGINT NOT NULL,
  released_at BIGINT NOT NULL,
  used_at BIGINT,
  used_by_archive_id TEXT,
  CHECK (released_at >= deleted_at),
  CHECK (
    (used_at IS NULL AND used_by_archive_id IS NULL)
    OR
    (used_at IS NOT NULL AND used_by_archive_id IS NOT NULL)
  )
);

CREATE INDEX IF NOT EXISTS idx_archive_number_recycle_pool_available
  ON archive_number_recycle_pool (released_at, pool_id)
  WHERE used_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_archive_number_recycle_pool_available_archive_no
  ON archive_number_recycle_pool (archive_no)
  WHERE used_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_archive_number_recycle_pool_available_passport_no
  ON archive_number_recycle_pool (passport_no)
  WHERE used_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_archive_number_recycle_pool_used_by_archive_id
  ON archive_number_recycle_pool (used_by_archive_id)
  WHERE used_by_archive_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS archive_hard_delete_logs (
  hard_delete_id TEXT PRIMARY KEY,
  source_archive_id TEXT NOT NULL UNIQUE,
  archive_no TEXT NOT NULL,
  passport_no TEXT NOT NULL,
  deleted_at BIGINT NOT NULL,
  hard_deleted_at BIGINT NOT NULL,
  reason TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_archive_hard_delete_logs_deleted_at
  ON archive_hard_delete_logs (hard_deleted_at);

CREATE TABLE IF NOT EXISTS archive_materials (
  material_id TEXT PRIMARY KEY,
  archive_id TEXT NOT NULL REFERENCES archives(archive_id) ON DELETE CASCADE,
  material_type TEXT NOT NULL CHECK (material_type IN (
    'PHOTO',
    'BIRTH_CERTIFICATE',
    'COPY',
    'VIDEO',
    'OTHER'
  )),
  original_file_name TEXT NOT NULL,
  stored_file_name TEXT NOT NULL UNIQUE,
  mime_type TEXT NOT NULL,
  file_size BIGINT NOT NULL CHECK (file_size > 0),
  sha256 TEXT NOT NULL,
  note TEXT NOT NULL DEFAULT '',
  uploaded_by TEXT NOT NULL,
  uploaded_at BIGINT NOT NULL,
  deleted_at BIGINT,
  deleted_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_archive_materials_archive_id
  ON archive_materials (archive_id, uploaded_at DESC)
  WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS cpms_status_exports (
  export_year INT PRIMARY KEY,
  export_batch_id TEXT NOT NULL UNIQUE,
  exported_at BIGINT NOT NULL,
  records_hash TEXT NOT NULL,
  citizen_binding_records_count BIGINT NOT NULL,
  binding_release_records_count BIGINT NOT NULL,
  export_file JSONB NOT NULL
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

CREATE TABLE IF NOT EXISTS archive_delete_challenges (
  challenge_id TEXT PRIMARY KEY,
  archive_id TEXT NOT NULL,
  archive_no TEXT NOT NULL,
  admin_id TEXT NOT NULL,
  admin_account TEXT NOT NULL,
  delete_payload TEXT NOT NULL,
  expire_at BIGINT NOT NULL,
  consumed BOOLEAN NOT NULL DEFAULT FALSE,
  created_at BIGINT NOT NULL,
  consumed_at BIGINT
);

CREATE INDEX IF NOT EXISTS idx_archive_delete_challenges_archive_id
  ON archive_delete_challenges (archive_id);

CREATE INDEX IF NOT EXISTS idx_archive_delete_challenges_expire_at
  ON archive_delete_challenges (expire_at);

CREATE TABLE IF NOT EXISTS address_towns (
  town_code TEXT PRIMARY KEY,
  town_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS address_units (
  address_unit_id TEXT PRIMARY KEY,
  town_code TEXT NOT NULL REFERENCES address_towns(town_code) ON DELETE CASCADE,
  address_unit_name TEXT NOT NULL
);

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
