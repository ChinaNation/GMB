#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

BACKEND_ENV="/etc/cpms/cpms-backend.env"
BACKUP_ENV="/etc/cpms/backup.env"

if [[ ! -f "${BACKEND_ENV}" ]]; then
  echo "ERROR: missing ${BACKEND_ENV}"
  exit 1
fi
if [[ ! -f "${BACKUP_ENV}" ]]; then
  echo "ERROR: missing ${BACKUP_ENV}"
  exit 1
fi

set -a
source "${BACKEND_ENV}"
source "${BACKUP_ENV}"
set +a

required_vars=(
  CPMS_DATABASE_URL
  STORAGE_HOST
  STORAGE_USER
  STORAGE_PATH
)
for v in "${required_vars[@]}"; do
  if [[ -z "${!v:-}" ]]; then
    echo "ERROR: ${v} is required in ${BACKUP_ENV}"
    exit 1
  fi
done

if [[ "${STORAGE_HOST}" == "CHANGE_ME" || "${STORAGE_USER}" == "CHANGE_ME" ]]; then
  echo "ERROR: please edit ${BACKUP_ENV} first."
  exit 1
fi

STORAGE_PORT="${STORAGE_PORT:-22}"
RETENTION_DAYS="${RETENTION_DAYS:-0}"
LOCAL_RETENTION_DAYS="${LOCAL_RETENTION_DAYS:-0}"

HOST_TAG="$(hostname -s)"
DATE_TAG="$(date +%F)"
TIME_TAG="$(date +%Y%m%d_%H%M%S)"
LOCAL_BACKUP_DIR="/var/backups/cpms/${DATE_TAG}"

mkdir -p "${LOCAL_BACKUP_DIR}"

DB_DUMP_FILE="${LOCAL_BACKUP_DIR}/${HOST_TAG}_cpms_${TIME_TAG}.dump"
RUNTIME_TAR_FILE="${LOCAL_BACKUP_DIR}/${HOST_TAG}_runtime_${TIME_TAG}.tar.gz"
CHECKSUM_FILE="${LOCAL_BACKUP_DIR}/${HOST_TAG}_backup_${TIME_TAG}.sha256"

pg_dump --format=custom --file="${DB_DUMP_FILE}" "${CPMS_DATABASE_URL}"
tar -C /var/lib/cpms -czf "${RUNTIME_TAR_FILE}" runtime
sha256sum "${DB_DUMP_FILE}" "${RUNTIME_TAR_FILE}" >"${CHECKSUM_FILE}"

REMOTE_DIR="${STORAGE_PATH%/}/${HOST_TAG}/${DATE_TAG}"
SSH_TARGET="${STORAGE_USER}@${STORAGE_HOST}"

ssh -p "${STORAGE_PORT}" "${SSH_TARGET}" "mkdir -p '${REMOTE_DIR}'"
rsync -az --partial --progress -e "ssh -p ${STORAGE_PORT}" \
  "${DB_DUMP_FILE}" "${RUNTIME_TAR_FILE}" "${CHECKSUM_FILE}" \
  "${SSH_TARGET}:${REMOTE_DIR}/"

# Remote retention: 0 means keep forever.
if [[ "${RETENTION_DAYS}" =~ ^[0-9]+$ ]] && [[ "${RETENTION_DAYS}" -gt 0 ]]; then
  ssh -p "${STORAGE_PORT}" "${SSH_TARGET}" \
    "find '${STORAGE_PATH%/}/${HOST_TAG}' -mindepth 1 -maxdepth 1 -type d -mtime +${RETENTION_DAYS} -exec rm -rf {} +"
fi

# Local retention: 0 means keep forever.
if [[ "${LOCAL_RETENTION_DAYS}" =~ ^[0-9]+$ ]] && [[ "${LOCAL_RETENTION_DAYS}" -gt 0 ]]; then
  find /var/backups/cpms -mindepth 1 -maxdepth 1 -type d -mtime +"${LOCAL_RETENTION_DAYS}" -exec rm -rf {} +
fi

echo "Backup completed: ${DB_DUMP_FILE} -> ${SSH_TARGET}:${REMOTE_DIR}"
