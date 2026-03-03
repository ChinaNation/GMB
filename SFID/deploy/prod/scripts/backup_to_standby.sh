#!/usr/bin/env bash
set -euo pipefail

# 主库定时执行：导出并传到备库（额外灾备层）
# 说明：主备流复制仍是主同步链路，此脚本用于形成可恢复备份文件。

BACKUP_DIR="/var/backups/sfid"
DATE="$(date +%F_%H%M%S)"
DB_URL="${DATABASE_URL:-}"
STANDBY_USER="backup"
STANDBY_IP="10.0.0.22"
STANDBY_DIR="/data/sfid-backups"
BACKUP_PASSPHRASE="${SFID_BACKUP_PASSPHRASE:-}"

if [[ -z "${DB_URL}" ]]; then
  echo "请先导出 DATABASE_URL"
  exit 1
fi

if [[ -z "${BACKUP_PASSPHRASE}" ]]; then
  echo "请先配置 SFID_BACKUP_PASSPHRASE"
  exit 1
fi

mkdir -p "${BACKUP_DIR}"
RAW_FILE="${BACKUP_DIR}/sfid_${DATE}.dump"
ENC_FILE="${RAW_FILE}.gpg"

pg_dump "${DB_URL}" -Fc -f "${RAW_FILE}"
gpg --batch --yes --symmetric --cipher-algo AES256 --passphrase "${BACKUP_PASSPHRASE}" -o "${ENC_FILE}" "${RAW_FILE}"
shred -u "${RAW_FILE}"

rsync -az "${ENC_FILE}" "${STANDBY_USER}@${STANDBY_IP}:${STANDBY_DIR}/"

# 本地保留 14 天
find "${BACKUP_DIR}" -name 'sfid_*.dump.gpg' -mtime +14 -delete
