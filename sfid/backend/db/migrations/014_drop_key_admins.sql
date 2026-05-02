-- 014_drop_key_admins.sql
-- 中文注释:Step 1 / Phase 2 — KEY_ADMIN 角色彻底废止(ADR-008,2026-05-01)
--
-- 决议:省管理员 main / backup_1 / backup_2 三槽自治,删除全国"超管"。
-- 本迁移把 admins 表内残留的 KEY_ADMIN 行清掉、放宽 role 检查约束、
-- 并 DROP 与 KEY_ADMIN 配套的 key_admin_keyring 表。
--
-- 注意:开发期一次性彻底切换(见 feedback_no_compatibility.md),
-- 不留迁移回退路径;down 仅作为占位说明。

BEGIN;

-- 1. 删除残留的 KEY_ADMIN 行(创世期 SFID 启动会再 sync,这里只清掉历史)
DELETE FROM admins WHERE role = 'KEY_ADMIN';

-- 2. 收紧 role 检查约束:KEY_ADMIN 不再合法
ALTER TABLE admins DROP CONSTRAINT IF EXISTS admins_role_check;
ALTER TABLE admins ADD CONSTRAINT admins_role_check
    CHECK (role IN ('SHENG_ADMIN', 'SHI_ADMIN'));

-- 3. DROP 与 KEY_ADMIN keyring 配套的表
DROP TABLE IF EXISTS key_admin_keyring;

-- down: 不提供回滚。开发期 chain 重启重发即可恢复。
COMMIT;
