-- SFID initial schema (PostgreSQL)
-- Milestone 1: data model + constraints + indexes

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'admin_role') THEN
    CREATE TYPE admin_role AS ENUM ('SUPER_ADMIN', 'OPERATOR_ADMIN');
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'admin_status') THEN
    CREATE TYPE admin_status AS ENUM ('ACTIVE', 'DISABLED');
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'bind_request_status') THEN
    CREATE TYPE bind_request_status AS ENUM ('PENDING', 'APPROVED', 'REJECTED', 'EXPIRED');
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'binding_status') THEN
    CREATE TYPE binding_status AS ENUM ('ACTIVE', 'UNBOUND', 'SUSPENDED');
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'audit_result') THEN
    CREATE TYPE audit_result AS ENUM ('SUCCESS', 'FAILED');
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'challenge_status') THEN
    CREATE TYPE challenge_status AS ENUM ('ISSUED', 'CONSUMED', 'EXPIRED', 'FAILED');
  END IF;
END $$;

CREATE TABLE IF NOT EXISTS admin_users (
  id BIGSERIAL PRIMARY KEY,
  admin_pubkey TEXT NOT NULL UNIQUE,
  role admin_role NOT NULL,
  status admin_status NOT NULL DEFAULT 'ACTIVE',
  source TEXT NOT NULL DEFAULT 'MANUAL',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  disabled_at TIMESTAMPTZ NULL
);

CREATE TABLE IF NOT EXISTS admin_login_challenges (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  admin_pubkey TEXT NOT NULL REFERENCES admin_users(admin_pubkey),
  challenge_text TEXT NOT NULL,
  origin TEXT NOT NULL,
  domain TEXT NOT NULL,
  session_id TEXT NOT NULL,
  nonce TEXT NOT NULL UNIQUE,
  issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expire_at TIMESTAMPTZ NOT NULL,
  consumed_at TIMESTAMPTZ NULL,
  status challenge_status NOT NULL DEFAULT 'ISSUED',
  verify_fail_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_admin_login_challenges_admin_pubkey
  ON admin_login_challenges(admin_pubkey);
CREATE INDEX IF NOT EXISTS idx_admin_login_challenges_expire_at
  ON admin_login_challenges(expire_at);
CREATE INDEX IF NOT EXISTS idx_admin_login_challenges_status
  ON admin_login_challenges(status);

CREATE TABLE IF NOT EXISTS bind_requests (
  id BIGSERIAL PRIMARY KEY,
  account_pubkey TEXT NOT NULL UNIQUE,
  status bind_request_status NOT NULL DEFAULT 'PENDING',
  requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  source TEXT NOT NULL DEFAULT 'BLOCKCHAIN'
);

CREATE INDEX IF NOT EXISTS idx_bind_requests_status ON bind_requests(status);

CREATE TABLE IF NOT EXISTS cpms_site_keys (
  id BIGSERIAL PRIMARY KEY,
  site_sfid TEXT NOT NULL UNIQUE,
  pubkey_1 TEXT NOT NULL,
  pubkey_2 TEXT NOT NULL,
  pubkey_3 TEXT NOT NULL,
  status admin_status NOT NULL DEFAULT 'ACTIVE',
  created_by TEXT NOT NULL REFERENCES admin_users(admin_pubkey),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS qr_consumptions (
  id BIGSERIAL PRIMARY KEY,
  qr_id TEXT NOT NULL UNIQUE,
  site_sfid TEXT NOT NULL,
  archive_no TEXT NOT NULL,
  consumed_by TEXT NOT NULL REFERENCES admin_users(admin_pubkey),
  consumed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_qr_consumptions_site_sfid ON qr_consumptions(site_sfid);

CREATE TABLE IF NOT EXISTS archive_bindings (
  id BIGSERIAL PRIMARY KEY,
  archive_no TEXT NOT NULL UNIQUE,
  account_pubkey TEXT NOT NULL UNIQUE,
  identity_code TEXT NOT NULL UNIQUE,
  birth_date DATE NOT NULL,
  status binding_status NOT NULL DEFAULT 'ACTIVE',
  site_sfid TEXT NULL REFERENCES cpms_site_keys(site_sfid),
  bound_by TEXT NOT NULL REFERENCES admin_users(admin_pubkey),
  bound_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  unbound_by TEXT NULL REFERENCES admin_users(admin_pubkey),
  unbound_at TIMESTAMPTZ NULL,
  unbind_reason TEXT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_archive_bindings_status ON archive_bindings(status);
CREATE INDEX IF NOT EXISTS idx_archive_bindings_birth_date ON archive_bindings(birth_date);

CREATE TABLE IF NOT EXISTS audit_logs (
  id BIGSERIAL PRIMARY KEY,
  action TEXT NOT NULL,
  actor_pubkey TEXT NULL,
  target_pubkey TEXT NULL,
  target_archive_no TEXT NULL,
  request_digest TEXT NULL,
  ip_addr INET NULL,
  user_agent TEXT NULL,
  result audit_result NOT NULL,
  result_code INTEGER NULL,
  detail JSONB NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_action_created_at ON audit_logs(action, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_created_at ON audit_logs(actor_pubkey, created_at DESC);

-- Keep updated_at fresh
CREATE OR REPLACE FUNCTION sfid_touch_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_admin_users_touch_updated_at ON admin_users;
CREATE TRIGGER trg_admin_users_touch_updated_at
BEFORE UPDATE ON admin_users
FOR EACH ROW EXECUTE FUNCTION sfid_touch_updated_at();

DROP TRIGGER IF EXISTS trg_bind_requests_touch_updated_at ON bind_requests;
CREATE TRIGGER trg_bind_requests_touch_updated_at
BEFORE UPDATE ON bind_requests
FOR EACH ROW EXECUTE FUNCTION sfid_touch_updated_at();

DROP TRIGGER IF EXISTS trg_archive_bindings_touch_updated_at ON archive_bindings;
CREATE TRIGGER trg_archive_bindings_touch_updated_at
BEFORE UPDATE ON archive_bindings
FOR EACH ROW EXECUTE FUNCTION sfid_touch_updated_at();

DROP TRIGGER IF EXISTS trg_cpms_site_keys_touch_updated_at ON cpms_site_keys;
CREATE TRIGGER trg_cpms_site_keys_touch_updated_at
BEFORE UPDATE ON cpms_site_keys
FOR EACH ROW EXECUTE FUNCTION sfid_touch_updated_at();

-- Protect SUPER_ADMIN records from delete/role-downgrade.
CREATE OR REPLACE FUNCTION sfid_protect_super_admin()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  IF TG_OP = 'DELETE' AND OLD.role = 'SUPER_ADMIN' THEN
    RAISE EXCEPTION 'SUPER_ADMIN record cannot be deleted';
  END IF;

  IF TG_OP = 'UPDATE' AND OLD.role = 'SUPER_ADMIN' AND NEW.role <> 'SUPER_ADMIN' THEN
    RAISE EXCEPTION 'SUPER_ADMIN role cannot be downgraded';
  END IF;

  RETURN COALESCE(NEW, OLD);
END;
$$;

DROP TRIGGER IF EXISTS trg_admin_users_protect_super_admin_delete ON admin_users;
CREATE TRIGGER trg_admin_users_protect_super_admin_delete
BEFORE DELETE ON admin_users
FOR EACH ROW EXECUTE FUNCTION sfid_protect_super_admin();

DROP TRIGGER IF EXISTS trg_admin_users_protect_super_admin_update ON admin_users;
CREATE TRIGGER trg_admin_users_protect_super_admin_update
BEFORE UPDATE ON admin_users
FOR EACH ROW EXECUTE FUNCTION sfid_protect_super_admin();

COMMIT;
