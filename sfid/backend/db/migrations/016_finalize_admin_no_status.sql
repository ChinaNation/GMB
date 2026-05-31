-- 016_finalize_admin_no_status.sql
-- 中文注释:管理员模型最终收口为无状态字段、无云端管理员签名私钥字段。
-- 本迁移用于清理旧本地库残留,并把管理员查询视图改为 sheng/shi 命名。

BEGIN;

DROP VIEW IF EXISTS v_sheng_admins;
DROP VIEW IF EXISTS v_shi_admins;

DROP INDEX IF EXISTS idx_admins_role_status;
DROP INDEX IF EXISTS idx_sfid_admins_role_status;

ALTER TABLE IF EXISTS admins DROP CONSTRAINT IF EXISTS admins_status_check;
ALTER TABLE IF EXISTS admins DROP COLUMN IF EXISTS status;
ALTER TABLE IF EXISTS admins DROP COLUMN IF EXISTS encrypted_signing_privkey;
ALTER TABLE IF EXISTS admins DROP COLUMN IF EXISTS signing_pubkey;
ALTER TABLE IF EXISTS admins DROP COLUMN IF EXISTS signing_created_at;
ALTER TABLE IF EXISTS admins ADD COLUMN IF NOT EXISTS admin_name TEXT NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS admins ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
ALTER TABLE IF EXISTS admins ADD COLUMN IF NOT EXISTS city TEXT NOT NULL DEFAULT '';

DO $$
BEGIN
  IF to_regclass('public.admins') IS NOT NULL THEN
    EXECUTE 'CREATE INDEX IF NOT EXISTS idx_admins_role ON admins(role)';
  END IF;

  IF to_regclass('public.admins') IS NOT NULL
     AND to_regclass('public.sheng_admin_scope') IS NOT NULL THEN
    EXECUTE $view$
      CREATE OR REPLACE VIEW v_sheng_admins AS
      SELECT
        a.admin_id,
        a.admin_pubkey,
        a.admin_name,
        a.role,
        a.built_in,
        a.created_by,
        a.created_at,
        a.updated_at,
        a.city,
        s.province_name,
        s.scope_no
      FROM admins a
      JOIN sheng_admin_scope s ON s.admin_id = a.admin_id
      WHERE a.role = 'SHENG_ADMIN'
    $view$;
  END IF;

  IF to_regclass('public.admins') IS NOT NULL
     AND to_regclass('public.shi_admin_scope') IS NOT NULL THEN
    EXECUTE $view$
      CREATE OR REPLACE VIEW v_shi_admins AS
      SELECT
        a.admin_id,
        a.admin_pubkey,
        a.admin_name,
        a.role,
        a.built_in,
        a.created_by,
        a.created_at,
        a.updated_at,
        a.city,
        s.province_name,
        s.sheng_admin_id,
        sa.admin_pubkey AS sheng_admin_pubkey
      FROM admins a
      JOIN shi_admin_scope s ON s.admin_id = a.admin_id
      JOIN admins sa ON sa.admin_id = s.sheng_admin_id
      WHERE a.role = 'SHI_ADMIN'
    $view$;
  END IF;
END $$;

COMMIT;
