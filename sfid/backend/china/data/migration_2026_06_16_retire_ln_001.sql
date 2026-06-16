-- ADR-021 / 任务卡 20260616-admin-district-single-source · B6
-- 收口本轮区划重构对镇 code 001 的复用:香港岛/新界/澳门半岛(LN 001/002/003 的 code=001)
-- 被删后,中西/葵青/大堂镇当时复用了 001。按「code 不可变不复用」铁律,给三镇换该市新 code,
-- 旧 001 永久退役进墓碑表,永不再分配。
-- 幂等:用 WHERE name= 定位,重复执行不会二次改(改完 code≠001 即不再命中)。

-- 1) 墓碑表:退役/撤并的镇级 code 永久占位,字典生成遇墓碑输出「已撤销」,区划脚本禁复用。
CREATE TABLE IF NOT EXISTS town_tombstones (
    province_code TEXT NOT NULL,
    city_code     TEXT NOT NULL,
    code          TEXT NOT NULL,
    retired_name  TEXT NOT NULL,   -- 退役时该 code 对应的镇名(历史留痕)
    retired_at    TEXT NOT NULL,   -- 退役日期 YYYY-MM-DD
    reason        TEXT NOT NULL,
    PRIMARY KEY (province_code, city_code, code)
);

-- 2) 三镇换新 code(取该市 max+1:LN001 005 / LN002 010 / LN003 006)
UPDATE towns SET code='005' WHERE province_code='LN' AND city_code='001' AND code='001' AND name='中西镇';
UPDATE towns SET code='010' WHERE province_code='LN' AND city_code='002' AND code='001' AND name='葵青镇';
UPDATE towns SET code='006' WHERE province_code='LN' AND city_code='003' AND code='001' AND name='大堂镇';

-- 3) 退役旧 001 进墓碑(原是香港岛/新界/澳门半岛,本轮重构删除,001 永不再分配)
INSERT OR IGNORE INTO town_tombstones(province_code, city_code, code, retired_name, retired_at, reason) VALUES
  ('LN','001','001','香港岛','2026-06-16','港澳新界中间层拆分为区/堂区直挂市,旧 code 退役不复用'),
  ('LN','002','001','新界','2026-06-16','同上'),
  ('LN','003','001','澳门半岛','2026-06-16','同上');

REINDEX towns;
