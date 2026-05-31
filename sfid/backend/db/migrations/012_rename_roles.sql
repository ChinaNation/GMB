-- 012_rename_roles.sql
-- 中文注释:SFID 管理员角色基准已直接使用 SHENG_ADMIN / SHI_ADMIN。

BEGIN;

ALTER TABLE admins DROP CONSTRAINT IF EXISTS admins_role_check;
ALTER TABLE admins ADD CONSTRAINT admins_role_check
    CHECK (role IN ('SHENG_ADMIN', 'SHI_ADMIN'));

COMMIT;
