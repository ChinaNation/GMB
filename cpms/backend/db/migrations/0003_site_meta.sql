-- QR1 协议扩展：携带省/市/机构名称，初始化时存入 DB
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS province_name TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS city_name TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS institution_name TEXT;
-- QR2 持久化：generate_qr2 生成后存入，重新生成覆盖旧值
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS qr2_payload TEXT;
-- 管理员姓名
ALTER TABLE admin_users ADD COLUMN IF NOT EXISTS admin_name TEXT NOT NULL DEFAULT '';

-- 地址管理：镇/村路（超管可维护）
CREATE TABLE IF NOT EXISTS address_towns (
  town_code TEXT PRIMARY KEY,
  town_name TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS address_villages (
  village_id TEXT PRIMARY KEY,
  town_code TEXT NOT NULL REFERENCES address_towns(town_code) ON DELETE CASCADE,
  village_name TEXT NOT NULL
);
-- 公民档案增加地址字段
ALTER TABLE archives ADD COLUMN IF NOT EXISTS town_code TEXT NOT NULL DEFAULT '';
ALTER TABLE archives ADD COLUMN IF NOT EXISTS village_id TEXT NOT NULL DEFAULT '';
ALTER TABLE archives ADD COLUMN IF NOT EXISTS address TEXT NOT NULL DEFAULT '';
-- 选举资格独立存储 + QR4 二维码自动生成持久化
ALTER TABLE archives ADD COLUMN IF NOT EXISTS voting_eligible BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE archives ADD COLUMN IF NOT EXISTS qr4_payload TEXT NOT NULL DEFAULT '';
