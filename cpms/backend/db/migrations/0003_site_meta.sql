-- QR1 协议扩展：携带省/市/机构名称，初始化时存入 DB
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS province_name TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS city_name TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS institution_name TEXT;
-- QR2 持久化：generate_qr2 生成后存入，重新生成覆盖旧值
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS qr2_payload TEXT;
-- 管理员姓名
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS admin_name TEXT NOT NULL DEFAULT '';
