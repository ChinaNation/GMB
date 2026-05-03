BEGIN;

-- 允许查询管理员角色（若已存在旧约束名则先删）
ALTER TABLE admins DROP CONSTRAINT IF EXISTS admins_role_check;
ALTER TABLE admins
  ADD CONSTRAINT admins_role_check
  CHECK (role IN ('SUPER_ADMIN', 'OPERATOR_ADMIN', 'QUERY_ONLY'));

-- 运行态杂项表（不再使用 runtime_store）
CREATE TABLE IF NOT EXISTS runtime_misc (
  id INTEGER PRIMARY KEY,
  payload JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 首次迁移：把 runtime_store 中与管理员无关的字段转到 runtime_misc
INSERT INTO runtime_misc(id, payload, updated_at)
SELECT
  id,
  payload
    - 'admin_users_by_pubkey'
    - 'super_admin_province_by_pubkey',
  now()
FROM runtime_store
WHERE id = 1
ON CONFLICT (id) DO UPDATE SET
  payload = EXCLUDED.payload,
  updated_at = EXCLUDED.updated_at;

DROP VIEW IF EXISTS v_super_admins;
DROP VIEW IF EXISTS v_operator_admins;

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

-- 旧整包状态表下线
DROP TABLE IF EXISTS runtime_store;

COMMIT;
