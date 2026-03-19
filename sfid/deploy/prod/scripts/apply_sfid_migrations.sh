#!/usr/bin/env bash
set -euo pipefail

DATABASE_URL_INPUT="${1:-${DATABASE_URL:-}}"
MIGRATIONS_DIR="${2:-}"

if [[ -z "${DATABASE_URL_INPUT}" || -z "${MIGRATIONS_DIR}" ]]; then
  echo "用法: apply_sfid_migrations.sh <DATABASE_URL> <migrations_dir>"
  exit 1
fi

if ! command -v psql >/dev/null 2>&1; then
  echo "未检测到 psql，请先安装 PostgreSQL 客户端。"
  exit 1
fi

if ! command -v sha256sum >/dev/null 2>&1; then
  echo "未检测到 sha256sum，请先安装 coreutils。"
  exit 1
fi

if [[ ! -d "${MIGRATIONS_DIR}" ]]; then
  echo "迁移目录不存在: ${MIGRATIONS_DIR}"
  exit 1
fi

psql "${DATABASE_URL_INPUT}" -v ON_ERROR_STOP=1 <<'SQL'
CREATE TABLE IF NOT EXISTS schema_migrations (
  name TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL,
  applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
SQL

mapfile -t migration_files < <(find "${MIGRATIONS_DIR}" -maxdepth 1 -type f -name '*.sql' | sort)

if [[ ${#migration_files[@]} -eq 0 ]]; then
  echo "未找到 SQL 迁移文件，跳过。"
  exit 0
fi

for file in "${migration_files[@]}"; do
  name="$(basename "${file}")"
  sha256="$(sha256sum "${file}" | awk '{print $1}')"
  applied_sha="$(
    psql "${DATABASE_URL_INPUT}" -At -v ON_ERROR_STOP=1 \
      -c "SELECT sha256 FROM schema_migrations WHERE name='${name}' LIMIT 1"
  )"

  if [[ -n "${applied_sha}" ]]; then
    if [[ "${applied_sha}" != "${sha256}" ]]; then
      echo "迁移文件校验和不一致: ${name}"
      echo "数据库中记录: ${applied_sha}"
      echo "当前文件校验和: ${sha256}"
      exit 1
    fi
    echo "跳过已执行迁移: ${name}"
    continue
  fi

  echo "执行迁移: ${name}"
  psql "${DATABASE_URL_INPUT}" -v ON_ERROR_STOP=1 -f "${file}"
  psql "${DATABASE_URL_INPUT}" -v ON_ERROR_STOP=1 \
    -c "INSERT INTO schema_migrations(name, sha256) VALUES ('${name}', '${sha256}')"
done
