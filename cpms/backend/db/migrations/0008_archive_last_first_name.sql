-- CPMS 公民姓名字段统一为 last_name / first_name。
-- 中文注释：迁移旧 full_name 后删除该字段，避免前后端继续使用混合姓名模型。
ALTER TABLE archives ADD COLUMN IF NOT EXISTS last_name TEXT;
ALTER TABLE archives ADD COLUMN IF NOT EXISTS first_name TEXT;

UPDATE archives
SET
  last_name = COALESCE(NULLIF(last_name, ''), SUBSTRING(full_name FROM 1 FOR 1)),
  first_name = COALESCE(NULLIF(first_name, ''), COALESCE(NULLIF(SUBSTRING(full_name FROM 2), ''), ''))
WHERE full_name IS NOT NULL;

UPDATE archives SET last_name = COALESCE(last_name, '');
UPDATE archives SET first_name = COALESCE(first_name, '');

ALTER TABLE archives ALTER COLUMN last_name SET NOT NULL;
ALTER TABLE archives ALTER COLUMN first_name SET NOT NULL;

DROP INDEX IF EXISTS idx_archives_full_name;
CREATE INDEX IF NOT EXISTS idx_archives_last_first_name ON archives (last_name, first_name);

ALTER TABLE archives DROP COLUMN IF EXISTS full_name;
