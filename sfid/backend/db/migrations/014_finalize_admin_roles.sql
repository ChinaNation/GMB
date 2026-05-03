-- 014_finalize_admin_roles.sql
-- 中文注释:管理员角色最终收口为 SHENG_ADMIN / SHI_ADMIN。

BEGIN;

-- 中文注释:删除不属于当前二角色模型的数据行,再收紧 role 检查约束。
DELETE FROM admins WHERE role NOT IN ('SHENG_ADMIN', 'SHI_ADMIN');

ALTER TABLE admins DROP CONSTRAINT IF EXISTS admins_role_check;
ALTER TABLE admins ADD CONSTRAINT admins_role_check
    CHECK (role IN ('SHENG_ADMIN', 'SHI_ADMIN'));

-- down: 不提供回滚。开发期链和本地库按当前二角色模型重建。
COMMIT;
