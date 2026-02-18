-- CPMS PostgreSQL schema (offline LAN edition)
-- Source of truth: cpms/DB-SCHEMA.md

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION cpms_set_updated_at()
RETURNS trigger AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,
  user_id VARCHAR(64) UNIQUE NOT NULL,
  username VARCHAR(64) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  role VARCHAR(32) NOT NULL CHECK (role IN ('SUPER_ADMIN', 'ADMIN')),
  status VARCHAR(16) NOT NULL CHECK (status IN ('ACTIVE', 'DISABLED', 'LOCKED')),
  failed_login_count INT NOT NULL DEFAULT 0 CHECK (failed_login_count >= 0),
  locked_until TIMESTAMPTZ NULL,
  last_login_at TIMESTAMPTZ NULL,
  created_by VARCHAR(64) NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);

CREATE TABLE IF NOT EXISTS citizen_archives (
  id BIGSERIAL PRIMARY KEY,
  archive_id VARCHAR(64) UNIQUE NOT NULL,
  archive_index_no VARCHAR(32) UNIQUE NOT NULL,
  passport_no VARCHAR(32) UNIQUE NOT NULL,
  full_name VARCHAR(128) NOT NULL,
  birth_date DATE NOT NULL,
  gender_code VARCHAR(1) NOT NULL CHECK (gender_code IN ('M', 'W')),
  height_cm NUMERIC(5,2) NULL CHECK (height_cm > 0 AND height_cm < 300),
  province_code VARCHAR(2) NOT NULL CHECK (province_code ~ '^[A-Z]{2}$'),
  status VARCHAR(16) NOT NULL CHECK (status IN ('ACTIVE', 'SUSPENDED', 'ARCHIVED')),
  remark TEXT NULL,
  created_by VARCHAR(64) NOT NULL,
  updated_by VARCHAR(64) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  deleted_at TIMESTAMPTZ NULL,
  CONSTRAINT ck_archive_index_no_format
    CHECK (archive_index_no ~ '^[A-Z]{2}(M|W)[0-9]{8}[0-9]{6}$')
);

CREATE INDEX IF NOT EXISTS idx_archives_name ON citizen_archives(full_name);
CREATE INDEX IF NOT EXISTS idx_archives_birth_gender ON citizen_archives(birth_date, gender_code);
CREATE INDEX IF NOT EXISTS idx_archives_status ON citizen_archives(status);

CREATE TABLE IF NOT EXISTS archive_sequence_counters (
  id BIGSERIAL PRIMARY KEY,
  province_code VARCHAR(2) NOT NULL CHECK (province_code ~ '^[A-Z]{2}$'),
  gender_code VARCHAR(1) NOT NULL CHECK (gender_code IN ('M', 'W')),
  birth_yyyymmdd VARCHAR(8) NOT NULL CHECK (birth_yyyymmdd ~ '^[0-9]{8}$'),
  next_seq INT NOT NULL CHECK (next_seq >= 1),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT uq_archive_seq_counter UNIQUE (province_code, gender_code, birth_yyyymmdd)
);

CREATE TABLE IF NOT EXISTS biometric_assets (
  id BIGSERIAL PRIMARY KEY,
  asset_id VARCHAR(64) UNIQUE NOT NULL,
  archive_id VARCHAR(64) NOT NULL,
  asset_type VARCHAR(16) NOT NULL CHECK (asset_type IN ('PHOTO', 'FINGERPRINT')),
  file_path TEXT NOT NULL,
  file_sha256 VARCHAR(64) NOT NULL CHECK (file_sha256 ~ '^[a-fA-F0-9]{64}$'),
  file_size BIGINT NOT NULL CHECK (file_size > 0),
  mime_type VARCHAR(64) NOT NULL,
  version_no INT NOT NULL CHECK (version_no >= 1),
  status VARCHAR(16) NOT NULL CHECK (status IN ('ACTIVE', 'REPLACED', 'DELETED')),
  created_by VARCHAR(64) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  deleted_at TIMESTAMPTZ NULL,
  CONSTRAINT fk_biometric_archive
    FOREIGN KEY (archive_id) REFERENCES citizen_archives(archive_id)
);

CREATE INDEX IF NOT EXISTS idx_assets_archive_type ON biometric_assets(archive_id, asset_type);
CREATE INDEX IF NOT EXISTS idx_assets_status ON biometric_assets(status);

CREATE TABLE IF NOT EXISTS archive_materials (
  id BIGSERIAL PRIMARY KEY,
  material_id VARCHAR(64) UNIQUE NOT NULL,
  archive_id VARCHAR(64) NOT NULL,
  material_type VARCHAR(32) NOT NULL,
  title VARCHAR(255) NOT NULL,
  file_path TEXT NOT NULL,
  file_sha256 VARCHAR(64) NOT NULL CHECK (file_sha256 ~ '^[a-fA-F0-9]{64}$'),
  file_size BIGINT NOT NULL CHECK (file_size > 0),
  version_no INT NOT NULL CHECK (version_no >= 1),
  status VARCHAR(16) NOT NULL CHECK (status IN ('ACTIVE', 'REPLACED', 'DELETED')),
  created_by VARCHAR(64) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  deleted_at TIMESTAMPTZ NULL,
  CONSTRAINT fk_material_archive
    FOREIGN KEY (archive_id) REFERENCES citizen_archives(archive_id)
);

CREATE INDEX IF NOT EXISTS idx_materials_archive_type ON archive_materials(archive_id, material_type);
CREATE INDEX IF NOT EXISTS idx_materials_status ON archive_materials(status);

CREATE TABLE IF NOT EXISTS audit_logs (
  id BIGSERIAL PRIMARY KEY,
  log_id VARCHAR(64) UNIQUE NOT NULL,
  operator_user_id VARCHAR(64) NULL,
  action VARCHAR(64) NOT NULL,
  target_type VARCHAR(32) NOT NULL,
  target_id VARCHAR(64) NULL,
  result VARCHAR(16) NOT NULL CHECK (result IN ('SUCCESS', 'FAILED')),
  trace_id VARCHAR(64) NULL,
  client_host VARCHAR(128) NULL,
  detail JSONB NULL,
  prev_hash VARCHAR(64) NULL CHECK (prev_hash ~ '^[a-fA-F0-9]{64}$' OR prev_hash IS NULL),
  curr_hash VARCHAR(64) NOT NULL CHECK (curr_hash ~ '^[a-fA-F0-9]{64}$'),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_operator_time ON audit_logs(operator_user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_action_time ON audit_logs(action, created_at);

CREATE TABLE IF NOT EXISTS backup_records (
  id BIGSERIAL PRIMARY KEY,
  backup_id VARCHAR(64) UNIQUE NOT NULL,
  operation VARCHAR(16) NOT NULL CHECK (operation IN ('BACKUP', 'RESTORE')),
  package_path TEXT NOT NULL,
  package_sha256 VARCHAR(64) NOT NULL CHECK (package_sha256 ~ '^[a-fA-F0-9]{64}$'),
  result VARCHAR(16) NOT NULL CHECK (result IN ('SUCCESS', 'FAILED')),
  operator_user_id VARCHAR(64) NOT NULL,
  detail JSONB NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DROP TRIGGER IF EXISTS trg_users_updated_at ON users;
CREATE TRIGGER trg_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION cpms_set_updated_at();

DROP TRIGGER IF EXISTS trg_citizen_archives_updated_at ON citizen_archives;
CREATE TRIGGER trg_citizen_archives_updated_at
BEFORE UPDATE ON citizen_archives
FOR EACH ROW
EXECUTE FUNCTION cpms_set_updated_at();

DROP TRIGGER IF EXISTS trg_archive_sequence_counters_updated_at ON archive_sequence_counters;
CREATE TRIGGER trg_archive_sequence_counters_updated_at
BEFORE UPDATE ON archive_sequence_counters
FOR EACH ROW
EXECUTE FUNCTION cpms_set_updated_at();

COMMIT;
