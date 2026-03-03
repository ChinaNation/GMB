BEGIN;

-- 兼容已执行旧迁移的环境：将 sfid_ 前缀表重命名为无前缀
ALTER TABLE IF EXISTS sfid_admins RENAME TO admins;
ALTER TABLE IF EXISTS sfid_provinces RENAME TO provinces;
ALTER TABLE IF EXISTS sfid_super_admin_scope RENAME TO super_admin_scope;
ALTER TABLE IF EXISTS sfid_operator_admin_scope RENAME TO operator_admin_scope;
ALTER TABLE IF EXISTS sfid_key_admin_keyring RENAME TO key_admin_keyring;

-- 约束/索引命名同步（存在才改）
ALTER INDEX IF EXISTS idx_sfid_admins_role_status RENAME TO idx_admins_role_status;
ALTER INDEX IF EXISTS idx_sfid_operator_admin_scope_super RENAME TO idx_operator_admin_scope_super;
ALTER TABLE IF EXISTS admins DROP CONSTRAINT IF EXISTS sfid_admins_role_check;
ALTER TABLE IF EXISTS admins ADD CONSTRAINT admins_role_check
  CHECK (role IN ('KEY_ADMIN', 'SUPER_ADMIN', 'OPERATOR_ADMIN', 'QUERY_ONLY'));

DROP VIEW IF EXISTS v_key_admins;
DROP VIEW IF EXISTS v_super_admins;
DROP VIEW IF EXISTS v_operator_admins;

CREATE OR REPLACE VIEW v_key_admins AS
SELECT a.*, k.slot, k.keyring_version, k.updated_at AS slot_updated_at
FROM admins a
JOIN key_admin_keyring k ON k.admin_id = a.admin_id
WHERE a.role = 'KEY_ADMIN';

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
