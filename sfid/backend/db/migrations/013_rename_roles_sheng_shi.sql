-- 013_rename_roles_sheng_shi.sql
-- 角色命名最终统一:INSTITUTION_ADMIN → SHENG_ADMIN, SYSTEM_ADMIN → SHI_ADMIN
-- 对应 Rust 枚举 AdminRole::{ShengAdmin, ShiAdmin}
-- 对应代码目录 super-admins/ → sheng-admins/, operator-admins/ → shi-admins/
-- 见任务卡:memory/08-tasks/open/20260408-sfid-三角色命名统一-任务卡0.5.md

BEGIN;

-- 中文注释:admins 表 role 字段值迁移
-- 先删约束再 UPDATE,再加新约束。与 012 迁移一致的模式。
ALTER TABLE admins DROP CONSTRAINT IF EXISTS admins_role_check;

UPDATE admins SET role = 'SHENG_ADMIN' WHERE role = 'INSTITUTION_ADMIN';
UPDATE admins SET role = 'SHI_ADMIN' WHERE role = 'SYSTEM_ADMIN';

ALTER TABLE admins ADD CONSTRAINT admins_role_check
    CHECK (role IN ('KEY_ADMIN', 'SHENG_ADMIN', 'SHI_ADMIN'));

-- 中文注释:super_admin_scope 表改名为 sheng_admin_scope
-- 索引/约束会自动跟着表名改(PostgreSQL RENAME TABLE 语义)
ALTER TABLE IF EXISTS super_admin_scope RENAME TO sheng_admin_scope;

-- 相关索引/约束显式重命名以保持一致性
ALTER INDEX IF EXISTS super_admin_scope_pkey RENAME TO sheng_admin_scope_pkey;
ALTER INDEX IF EXISTS super_admin_scope_province_name_key RENAME TO sheng_admin_scope_province_name_key;
ALTER INDEX IF EXISTS super_admin_scope_scope_no_key RENAME TO sheng_admin_scope_scope_no_key;
ALTER TABLE sheng_admin_scope RENAME CONSTRAINT super_admin_scope_admin_id_fkey TO sheng_admin_scope_admin_id_fkey;
ALTER TABLE sheng_admin_scope RENAME CONSTRAINT super_admin_scope_province_name_fkey TO sheng_admin_scope_province_name_fkey;

-- 中文注释:operator_admin_scope 表改名为 shi_admin_scope
-- 列 super_admin_id 同时改名为 sheng_admin_id(指向 sheng_admin_scope.admin_id 的外键)
ALTER TABLE IF EXISTS operator_admin_scope RENAME TO shi_admin_scope;
ALTER TABLE shi_admin_scope RENAME COLUMN super_admin_id TO sheng_admin_id;

ALTER INDEX IF EXISTS operator_admin_scope_pkey RENAME TO shi_admin_scope_pkey;
ALTER INDEX IF EXISTS idx_operator_admin_scope_super RENAME TO idx_shi_admin_scope_sheng;
ALTER TABLE shi_admin_scope RENAME CONSTRAINT operator_admin_scope_admin_id_fkey TO shi_admin_scope_admin_id_fkey;
ALTER TABLE shi_admin_scope RENAME CONSTRAINT operator_admin_scope_province_name_fkey TO shi_admin_scope_province_name_fkey;
ALTER TABLE shi_admin_scope RENAME CONSTRAINT operator_admin_scope_super_admin_id_fkey TO shi_admin_scope_sheng_admin_id_fkey;

COMMIT;
