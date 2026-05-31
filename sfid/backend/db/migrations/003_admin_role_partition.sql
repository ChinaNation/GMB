BEGIN;

-- 统一管理员主表（按角色区分）
CREATE TABLE IF NOT EXISTS admins (
  admin_id BIGSERIAL PRIMARY KEY,
  admin_pubkey TEXT NOT NULL UNIQUE,
  role TEXT NOT NULL CHECK (role IN ('SHENG_ADMIN', 'SHI_ADMIN')),
  built_in BOOLEAN NOT NULL DEFAULT FALSE,
  created_by TEXT NOT NULL DEFAULT 'SYSTEM',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_admins_role
  ON admins(role);

-- 省份维度（用于省级管理员分省）
CREATE TABLE IF NOT EXISTS provinces (
  province_name TEXT PRIMARY KEY
);

-- 省级管理员省域归属：每省最多 5 个省级管理员，由后端业务规则强制校验。
CREATE TABLE IF NOT EXISTS sheng_admin_scope (
  admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
  province_name TEXT NOT NULL REFERENCES provinces(province_name) ON DELETE RESTRICT,
  scope_no INTEGER UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_sheng_admin_scope_province_name
  ON sheng_admin_scope(province_name);

-- 市级管理员归属：归属于某个省级管理员
CREATE TABLE IF NOT EXISTS shi_admin_scope (
  admin_id BIGINT PRIMARY KEY REFERENCES admins(admin_id) ON DELETE CASCADE,
  sheng_admin_id BIGINT NOT NULL REFERENCES admins(admin_id) ON DELETE RESTRICT,
  province_name TEXT NULL REFERENCES provinces(province_name) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_shi_admin_scope_sheng
  ON shi_admin_scope(sheng_admin_id);

-- 从 runtime_store(id=1) 回填管理员
WITH raw AS (
  SELECT payload
  FROM runtime_store
  WHERE id = 1
),
admins AS (
  SELECT
    e.key AS admin_pubkey,
    UPPER(COALESCE(e.value->>'role', 'SHI_ADMIN')) AS role,
    COALESCE((e.value->>'built_in')::boolean, FALSE) AS built_in,
    COALESCE(NULLIF(e.value->>'created_by', ''), 'SYSTEM') AS created_by,
    COALESCE((e.value->>'created_at')::timestamptz, now()) AS created_at
  FROM raw, LATERAL jsonb_each(raw.payload->'admin_users_by_pubkey') AS e(key, value)
)
INSERT INTO admins(admin_pubkey, role, built_in, created_by, created_at)
SELECT
  admin_pubkey,
  CASE
    WHEN role IN ('SHENG_ADMIN', 'SHI_ADMIN') THEN role
    ELSE 'SHI_ADMIN'
  END,
  built_in,
  created_by,
  created_at
FROM admins
ON CONFLICT (admin_pubkey) DO UPDATE SET
  role = EXCLUDED.role,
  built_in = EXCLUDED.built_in,
  created_by = EXCLUDED.created_by,
  created_at = EXCLUDED.created_at;

-- 回填省份维度（按 runtime_store 的省级管理员映射）
WITH raw AS (
  SELECT payload
  FROM runtime_store
  WHERE id = 1
),
provinces AS (
  SELECT DISTINCT value AS province_name
  FROM raw, LATERAL jsonb_each_text(raw.payload->'sheng_admin_province_by_pubkey')
)
INSERT INTO provinces(province_name)
SELECT province_name
FROM provinces
WHERE province_name IS NOT NULL AND province_name <> ''
ON CONFLICT (province_name) DO NOTHING;

-- 回填省级管理员省域映射
WITH raw AS (
  SELECT payload
  FROM runtime_store
  WHERE id = 1
),
mapping AS (
  SELECT key AS admin_pubkey, value AS province_name
  FROM raw, LATERAL jsonb_each_text(raw.payload->'sheng_admin_province_by_pubkey')
),
rows_to_upsert AS (
  SELECT a.admin_id, m.province_name
  FROM mapping m
  JOIN admins a ON a.admin_pubkey = m.admin_pubkey
  WHERE a.role = 'SHENG_ADMIN'
)
INSERT INTO sheng_admin_scope(admin_id, province_name)
SELECT admin_id, province_name
FROM rows_to_upsert
ON CONFLICT (admin_id) DO UPDATE SET
  province_name = EXCLUDED.province_name;

-- 回填市级管理员归属（按 created_by 指向的省级管理员）
WITH op AS (
  SELECT admin_id, created_by
  FROM admins
  WHERE role = 'SHI_ADMIN'
),
sa AS (
  SELECT a.admin_id, a.admin_pubkey, s.province_name
  FROM admins a
  JOIN sheng_admin_scope s ON s.admin_id = a.admin_id
  WHERE a.role = 'SHENG_ADMIN'
),
rows_to_upsert AS (
  SELECT
    op.admin_id,
    sa.admin_id AS sheng_admin_id,
    sa.province_name
  FROM op
  JOIN sa ON sa.admin_pubkey = op.created_by
)
INSERT INTO shi_admin_scope(admin_id, sheng_admin_id, province_name)
SELECT admin_id, sheng_admin_id, province_name
FROM rows_to_upsert
ON CONFLICT (admin_id) DO UPDATE SET
  sheng_admin_id = EXCLUDED.sheng_admin_id,
  province_name = EXCLUDED.province_name;

-- 便于按“类”直接查看
CREATE OR REPLACE VIEW v_sheng_admins AS
SELECT a.*, s.province_name, s.scope_no
FROM admins a
JOIN sheng_admin_scope s ON s.admin_id = a.admin_id
WHERE a.role = 'SHENG_ADMIN';

CREATE OR REPLACE VIEW v_shi_admins AS
SELECT
  a.*,
  o.province_name,
  o.sheng_admin_id,
  sa.admin_pubkey AS sheng_admin_pubkey
FROM admins a
JOIN shi_admin_scope o ON o.admin_id = a.admin_id
JOIN admins sa ON sa.admin_id = o.sheng_admin_id
WHERE a.role = 'SHI_ADMIN';

COMMIT;
