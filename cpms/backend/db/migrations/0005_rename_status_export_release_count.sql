-- CPMS 当前基线：年度报告只表达 SFID 需要的档案号释放记录，不再使用号码释放旧命名。

BEGIN;

DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_name = 'cpms_status_exports'
      AND column_name = 'number_release_records_count'
  ) AND NOT EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_name = 'cpms_status_exports'
      AND column_name = 'archive_release_records_count'
  ) THEN
    ALTER TABLE cpms_status_exports
      RENAME COLUMN number_release_records_count TO archive_release_records_count;
  END IF;
END $$;

COMMIT;
