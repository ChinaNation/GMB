-- CPMS 公民资料库：只保存资料元数据，文件正文保存在本机 data/archive-materials。

BEGIN;

CREATE TABLE IF NOT EXISTS archive_materials (
  material_id TEXT PRIMARY KEY,
  archive_id TEXT NOT NULL REFERENCES archives(archive_id) ON DELETE CASCADE,
  material_type TEXT NOT NULL CHECK (material_type IN (
    'PHOTO',
    'BIRTH_CERTIFICATE',
    'COPY',
    'VIDEO',
    'OTHER'
  )),
  original_file_name TEXT NOT NULL,
  stored_file_name TEXT NOT NULL UNIQUE,
  mime_type TEXT NOT NULL,
  file_size BIGINT NOT NULL CHECK (file_size > 0),
  sha256 TEXT NOT NULL,
  note TEXT NOT NULL DEFAULT '',
  uploaded_by TEXT NOT NULL,
  uploaded_at BIGINT NOT NULL,
  deleted_at BIGINT,
  deleted_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_archive_materials_archive_id
  ON archive_materials (archive_id, uploaded_at DESC)
  WHERE deleted_at IS NULL;

COMMIT;
