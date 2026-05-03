BEGIN;

-- 统一管理员主表（按角色区分）
CREATE TABLE IF NOT EXISTS admins (
  admin_id BIGSERIAL PRIMARY KEY,
  admin_pubkey TEXT NOT NULL UNIQUE,
  role TEXT NOT NULL CHECK (role IN ('SUPER_ADMIN', 'OPERATOR_ADMIN')),
  status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'DISABLED')),
  built_in BOOLEAN NOT NULL DEFAULT FALSE,
  created_by TEXT NOT NULL DEFAULT 'SYSTEM',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_admins_role_status
  ON admins(role, status);

-- 省份维度（用于超级管理员分省）
CREATE TABLE IF NOT EXISTS provinces (
  province_name TEXT PRIMARY KEY
);

-- 超级管理员省域归属：一个省份只能有一个超级管理员
CREATE TABLE IF NOT EXISTS super_admin_scope (
  admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
  province_name TEXT NOT NULL UNIQUE REFERENCES provinces(province_name) ON DELETE RESTRICT,
  scope_no INTEGER UNIQUE
);

-- 操作管理员归属：归属于某个超级管理员
CREATE TABLE IF NOT EXISTS operator_admin_scope (
  admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
  super_admin_id BIGINT NOT NULL REFERENCES admins(admin_id) ON DELETE RESTRICT,
  province_name TEXT NULL REFERENCES provinces(province_name) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_operator_admin_scope_super
  ON operator_admin_scope(super_admin_id);

-- 从 runtime_store(id=1) 回填管理员
WITH raw AS (
  SELECT payload
  FROM runtime_store
  WHERE id = 1
),
admins AS (
  SELECT
    e.key AS admin_pubkey,
    UPPER(COALESCE(e.value->>'role', 'OPERATOR_ADMIN')) AS role,
    UPPER(COALESCE(e.value->>'status', 'ACTIVE')) AS status,
    COALESCE((e.value->>'built_in')::boolean, FALSE) AS built_in,
    COALESCE(NULLIF(e.value->>'created_by', ''), 'SYSTEM') AS created_by,
    COALESCE((e.value->>'created_at')::timestamptz, now()) AS created_at
  FROM raw, LATERAL jsonb_each(raw.payload->'admin_users_by_pubkey') AS e(key, value)
)
INSERT INTO admins(admin_pubkey, role, status, built_in, created_by, created_at)
SELECT
  admin_pubkey,
  CASE
    WHEN role IN ('SUPER_ADMIN', 'OPERATOR_ADMIN') THEN role
    ELSE 'OPERATOR_ADMIN'
  END,
  CASE
    WHEN status IN ('ACTIVE', 'DISABLED') THEN status
    ELSE 'ACTIVE'
  END,
  built_in,
  created_by,
  created_at
FROM admins
ON CONFLICT (admin_pubkey) DO UPDATE SET
  role = EXCLUDED.role,
  status = EXCLUDED.status,
  built_in = EXCLUDED.built_in,
  created_by = EXCLUDED.created_by,
  created_at = EXCLUDED.created_at;

-- 回填省份维度（按 runtime_store 的超级管理员映射）
WITH raw AS (
  SELECT payload
  FROM runtime_store
  WHERE id = 1
),
provinces AS (
  SELECT DISTINCT value AS province_name
  FROM raw, LATERAL jsonb_each_text(raw.payload->'super_admin_province_by_pubkey')
)
INSERT INTO provinces(province_name)
SELECT province_name
FROM provinces
WHERE province_name IS NOT NULL AND province_name <> ''
ON CONFLICT (province_name) DO NOTHING;

-- 回填超级管理员省域映射
WITH raw AS (
  SELECT payload
  FROM runtime_store
  WHERE id = 1
),
mapping AS (
  SELECT key AS admin_pubkey, value AS province_name
  FROM raw, LATERAL jsonb_each_text(raw.payload->'super_admin_province_by_pubkey')
),
rows_to_upsert AS (
  SELECT a.admin_id, m.province_name
  FROM mapping m
  JOIN admins a ON a.admin_pubkey = m.admin_pubkey
  WHERE a.role = 'SUPER_ADMIN'
)
INSERT INTO super_admin_scope(admin_id, province_name)
SELECT admin_id, province_name
FROM rows_to_upsert
ON CONFLICT (admin_id) DO UPDATE SET
  province_name = EXCLUDED.province_name;

-- 回填操作管理员归属（按 created_by 指向的超级管理员）
WITH op AS (
  SELECT admin_id, created_by
  FROM admins
  WHERE role = 'OPERATOR_ADMIN'
),
sa AS (
  SELECT a.admin_id, a.admin_pubkey, s.province_name
  FROM admins a
  JOIN super_admin_scope s ON s.admin_id = a.admin_id
  WHERE a.role = 'SUPER_ADMIN'
),
rows_to_upsert AS (
  SELECT
    op.admin_id,
    sa.admin_id AS super_admin_id,
    sa.province_name
  FROM op
  JOIN sa ON sa.admin_pubkey = op.created_by
)
INSERT INTO operator_admin_scope(admin_id, super_admin_id, province_name)
SELECT admin_id, super_admin_id, province_name
FROM rows_to_upsert
ON CONFLICT (admin_id) DO UPDATE SET
  super_admin_id = EXCLUDED.super_admin_id,
  province_name = EXCLUDED.province_name;

-- 便于按“类”直接查看
CREATE OR REPLACE VIEW v_super_admins AS
SELECT a.*, s.province_name, s.scope_no
FROM admins a
JOIN super_admin_scope s ON s.admin_id = a.admin_id
WHERE a.role = 'SUPER_ADMIN';

CREATE OR REPLACE VIEW v_operator_admins AS
SELECT
  a.*,
  o.province_name,
  o.super_admin_id,
  sa.admin_pubkey AS super_admin_pubkey
FROM admins a
JOIN operator_admin_scope o ON o.admin_id = a.admin_id
JOIN admins sa ON sa.admin_id = o.super_admin_id
WHERE a.role = 'OPERATOR_ADMIN';

COMMIT;
