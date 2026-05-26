-- 015_store_reset.sql
-- 中文注释:SFID Store 目标化重置。
-- 本迁移不迁旧数据:旧 runtime 整包 JSON 与旧持久分片允许直接删除。

BEGIN;

DROP TABLE IF EXISTS runtime_store;
DROP TABLE IF EXISTS runtime_misc;
DROP TABLE IF EXISTS runtime_cache_entries;
DROP TABLE IF EXISTS store_shards;

CREATE TABLE IF NOT EXISTS store_citizens (
  id SMALLINT PRIMARY KEY CHECK (id = 1),
  payload JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_cpms (
  id SMALLINT PRIMARY KEY CHECK (id = 1),
  payload JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_institutions (
  id SMALLINT PRIMARY KEY CHECK (id = 1),
  payload JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS store_ops (
  id SMALLINT PRIMARY KEY CHECK (id = 1),
  payload JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMIT;
