#!/usr/bin/env bash
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "请使用 root 或 sudo 运行 update_sfid_app.sh"
  exit 1
fi

RELEASE_ID="${1:-}"
BUNDLE_DIR="${2:-}"
APP_HOME="${APP_HOME:-/opt/sfid}"
FRONTEND_ROOT="${FRONTEND_ROOT:-/var/www/sfid}"
SERVICE_NAME="${SERVICE_NAME:-sfid-backend}"
ENV_FILE="${ENV_FILE:-/etc/sfid/sfid.env}"
WEB_SERVICE_NAME="${WEB_SERVICE_NAME:-nginx}"
HEALTHCHECK_URL="${HEALTHCHECK_URL:-http://127.0.0.1:8899/api/v1/health}"
KEEP_RELEASES="${KEEP_RELEASES:-5}"

if [[ -z "${RELEASE_ID}" || -z "${BUNDLE_DIR}" ]]; then
  echo "用法: update_sfid_app.sh <release_id> <bundle_dir>"
  exit 1
fi

if [[ ! -x "${BUNDLE_DIR}/backend/sfid-backend" ]]; then
  echo "缺少后端二进制: ${BUNDLE_DIR}/backend/sfid-backend"
  exit 1
fi

if [[ ! -d "${BUNDLE_DIR}/backend/db/migrations" ]]; then
  echo "缺少迁移目录: ${BUNDLE_DIR}/backend/db/migrations"
  exit 1
fi

if [[ ! -x "${BUNDLE_DIR}/deploy/apply_sfid_migrations.sh" ]]; then
  echo "缺少迁移脚本: ${BUNDLE_DIR}/deploy/apply_sfid_migrations.sh"
  exit 1
fi

mkdir -p \
  "${APP_HOME}/bin" \
  "${APP_HOME}/backend/db/migrations" \
  "${APP_HOME}/scripts" \
  "${APP_HOME}/releases/${RELEASE_ID}/backend"

install -m 755 "${BUNDLE_DIR}/backend/sfid-backend" \
  "${APP_HOME}/releases/${RELEASE_ID}/backend/sfid-backend"
install -m 755 "${BUNDLE_DIR}/backend/sfid-backend" \
  "${APP_HOME}/bin/sfid-backend"
rsync -a --delete "${BUNDLE_DIR}/backend/db/migrations/" \
  "${APP_HOME}/backend/db/migrations/"
install -m 755 "${BUNDLE_DIR}/deploy/apply_sfid_migrations.sh" \
  "${APP_HOME}/scripts/apply_sfid_migrations.sh"
install -m 755 "${BUNDLE_DIR}/deploy/update_sfid_app.sh" \
  "${APP_HOME}/scripts/update_sfid_app.sh"
printf '%s\n' "${RELEASE_ID}" > "${APP_HOME}/REVISION"

if [[ -d "${BUNDLE_DIR}/frontend/dist" ]]; then
  mkdir -p "${FRONTEND_ROOT}/releases/${RELEASE_ID}"
  rsync -a --delete "${BUNDLE_DIR}/frontend/dist/" \
    "${FRONTEND_ROOT}/releases/${RELEASE_ID}/"
  chmod -R a+rX "${FRONTEND_ROOT}/releases/${RELEASE_ID}"
  ln -sfn "${FRONTEND_ROOT}/releases/${RELEASE_ID}" "${FRONTEND_ROOT}/current"
  printf '%s\n' "${RELEASE_ID}" > "${FRONTEND_ROOT}/REVISION"
fi

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "环境文件不存在: ${ENV_FILE}"
  exit 1
fi

set -a
source "${ENV_FILE}"
set +a

if [[ -z "${DATABASE_URL:-}" ]]; then
  echo "环境文件中缺少 DATABASE_URL: ${ENV_FILE}"
  exit 1
fi

"${APP_HOME}/scripts/apply_sfid_migrations.sh" \
  "${DATABASE_URL}" \
  "${APP_HOME}/backend/db/migrations"

systemctl daemon-reload
systemctl restart "${SERVICE_NAME}"

backend_ready=0
for _ in $(seq 1 30); do
  if curl -fsS "${HEALTHCHECK_URL}" >/dev/null 2>&1; then
    backend_ready=1
    break
  fi
  sleep 1
done

if [[ "${backend_ready}" -ne 1 ]]; then
  echo "后端健康检查失败: ${HEALTHCHECK_URL}"
  systemctl status "${SERVICE_NAME}" --no-pager || true
  exit 1
fi

if systemctl list-unit-files "${WEB_SERVICE_NAME}.service" >/dev/null 2>&1; then
  systemctl reload "${WEB_SERVICE_NAME}" || true
fi

prune_releases() {
  local releases_root="$1"
  local keep_count="$2"
  local paths=()

  if [[ ! -d "${releases_root}" ]]; then
    return 0
  fi

  mapfile -t paths < <(ls -1dt "${releases_root}"/* 2>/dev/null || true)
  if [[ ${#paths[@]} -le "${keep_count}" ]]; then
    return 0
  fi

  for ((i=keep_count; i<${#paths[@]}; i++)); do
    rm -rf "${paths[$i]}"
  done
}

prune_releases "${APP_HOME}/releases" "${KEEP_RELEASES}"
if [[ -d "${FRONTEND_ROOT}/releases" ]]; then
  prune_releases "${FRONTEND_ROOT}/releases" "${KEEP_RELEASES}"
fi

echo "SFID 已更新到版本: ${RELEASE_ID}"
