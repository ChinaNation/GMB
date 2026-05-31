BEGIN;

-- 兼容已执行旧迁移的环境：将 sfid_ 前缀表重命名为无前缀
ALTER TABLE IF EXISTS sfid_admins RENAME TO admins;
ALTER TABLE IF EXISTS sfid_provinces RENAME TO provinces;
ALTER TABLE IF EXISTS sfid_sheng_admin_scope RENAME TO sheng_admin_scope;
ALTER TABLE IF EXISTS sfid_shi_admin_scope RENAME TO shi_admin_scope;

-- 约束/索引命名同步（存在才改）
ALTER INDEX IF EXISTS idx_sfid_admins_role_status RENAME TO idx_admins_role_status;
ALTER INDEX IF EXISTS idx_sfid_shi_admin_scope_sheng RENAME TO idx_shi_admin_scope_sheng;
ALTER TABLE IF EXISTS admins DROP CONSTRAINT IF EXISTS sfid_admins_role_check;
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'admins_role_check'
      AND conrelid = 'admins'::regclass
  ) THEN
    ALTER TABLE admins ADD CONSTRAINT admins_role_check
      CHECK (role IN ('SHENG_ADMIN', 'SHI_ADMIN'));
  END IF;
END
$$;

DROP VIEW IF EXISTS v_sheng_admins;
DROP VIEW IF EXISTS v_shi_admins;

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
