-- 013_rename_roles_sheng_shi.sql
-- 中文注释:确认管理员表、scope 表和索引均已收口到 sheng/shi 命名。

BEGIN;

ALTER TABLE admins DROP CONSTRAINT IF EXISTS admins_role_check;
ALTER TABLE admins ADD CONSTRAINT admins_role_check
    CHECK (role IN ('SHENG_ADMIN', 'SHI_ADMIN'));

ALTER TABLE IF EXISTS sheng_admin_scope
  DROP CONSTRAINT IF EXISTS sheng_admin_scope_province_name_key;
DROP INDEX IF EXISTS sheng_admin_scope_province_name_key;
CREATE INDEX IF NOT EXISTS idx_sheng_admin_scope_province_name
  ON sheng_admin_scope(province_name);

CREATE INDEX IF NOT EXISTS idx_shi_admin_scope_sheng
  ON shi_admin_scope(sheng_admin_id);

COMMIT;
