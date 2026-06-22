#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DB_CONTAINER="${CPMS_DEV_DB_CONTAINER:-cpms-pg}"
DB_NAME="${CPMS_DEV_DB_NAME:-cpms_dev}"
DB_USER="${CPMS_DEV_DB_USER:-cpms}"
DB_PASSWORD="${CPMS_DEV_DB_PASSWORD:-cpms_dev_change_me}"
DB_PORT="${CPMS_DEV_DB_PORT:-5432}"
DB_IMAGE="${CPMS_DEV_DB_IMAGE:-postgres:16-alpine}"

echo "[CPMS] Ensuring dev PostgreSQL container exists ..."
if ! docker ps -a --format '{{.Names}}' | grep -qx "${DB_CONTAINER}"; then
  docker run -d \
    --name "${DB_CONTAINER}" \
    -e POSTGRES_DB="${DB_NAME}" \
    -e POSTGRES_USER="${DB_USER}" \
    -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
    -p "${DB_PORT}:5432" \
    "${DB_IMAGE}"
fi

docker start "${DB_CONTAINER}" >/dev/null

echo "[CPMS] Waiting for PostgreSQL ..."
for _ in {1..30}; do
  if docker exec "${DB_CONTAINER}" pg_isready -U "${DB_USER}" -d "${DB_NAME}" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

echo "[CPMS] Applying schema and seed ..."
docker exec -i "${DB_CONTAINER}" psql -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 \
  < "${ROOT_DIR}/backend/db/schema.sql"
docker exec -i "${DB_CONTAINER}" psql -U "${DB_USER}" -d "${DB_NAME}" -v ON_ERROR_STOP=1 \
  < "${ROOT_DIR}/backend/db/seed.sql"

cat <<EOF
[CPMS] Dev database is ready.
Host: 127.0.0.1
Port: ${DB_PORT}
DB:   ${DB_NAME}
User: ${DB_USER}
Pass: ${DB_PASSWORD}
URL:  postgresql://${DB_USER}:${DB_PASSWORD}@127.0.0.1:${DB_PORT}/${DB_NAME}
EOF
