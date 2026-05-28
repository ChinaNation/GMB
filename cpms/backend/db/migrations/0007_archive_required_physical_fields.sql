-- CPMS 公民档案基础身份字段约束。
-- 中文注释：NOT VALID 避免历史空身高阻断迁移；新写入/更新数据必须满足约束。
ALTER TABLE archives
  ADD CONSTRAINT archives_birth_date_yyyy_mm_dd
  CHECK (birth_date ~ '^[0-9]{4}-[0-9]{2}-[0-9]{2}$') NOT VALID;

ALTER TABLE archives
  ADD CONSTRAINT archives_height_cm_required_range
  CHECK (height_cm IS NOT NULL AND height_cm BETWEEN 30 AND 260) NOT VALID;
