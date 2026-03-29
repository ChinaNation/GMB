-- 角色重命名：SUPER_ADMIN → INSTITUTION_ADMIN，OPERATOR_ADMIN → SYSTEM_ADMIN，删除 QUERY_ONLY
-- 对应代码中 AdminRole 枚举变更

-- 先删除旧约束（否则 UPDATE 会被旧约束拒绝）
ALTER TABLE admins DROP CONSTRAINT IF EXISTS admins_role_check;

-- 更新已有数据
UPDATE admins SET role = 'INSTITUTION_ADMIN' WHERE role = 'SUPER_ADMIN';
UPDATE admins SET role = 'SYSTEM_ADMIN' WHERE role = 'OPERATOR_ADMIN';
DELETE FROM admins WHERE role = 'QUERY_ONLY';

-- 添加新约束
ALTER TABLE admins ADD CONSTRAINT admins_role_check
    CHECK (role IN ('KEY_ADMIN', 'INSTITUTION_ADMIN', 'SYSTEM_ADMIN'));
