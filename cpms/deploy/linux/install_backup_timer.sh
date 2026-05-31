#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAYLOAD_DIR="${SCRIPT_DIR}/payload"
BACKUP_SCRIPT_SRC="${PAYLOAD_DIR}/bin/backup_to_storage.sh"
SERVICE_SRC="${PAYLOAD_DIR}/systemd/cpms-backup.service"
TIMER_SRC="${PAYLOAD_DIR}/systemd/cpms-backup.timer"

if [[ ! -f "${BACKUP_SCRIPT_SRC}" || ! -f "${SERVICE_SRC}" || ! -f "${TIMER_SRC}" ]]; then
  echo "ERROR: backup payload is incomplete"
  exit 1
fi

install -d -m 0755 /opt/cpms/bin
install -m 0755 "${BACKUP_SCRIPT_SRC}" /opt/cpms/bin/backup_to_storage.sh
install -m 0644 "${SERVICE_SRC}" /etc/systemd/system/cpms-backup.service
install -m 0644 "${TIMER_SRC}" /etc/systemd/system/cpms-backup.timer

install -d -m 0750 /etc/cpms
if [[ ! -f /etc/cpms/backup.env ]]; then
  cat >/etc/cpms/backup.env <<'EOF'
# 备份存储机 SSH 地址
STORAGE_HOST=CHANGE_ME
STORAGE_PORT=22
STORAGE_USER=CHANGE_ME

# 备份存储机上的绝对目录，例如 /data/cpms-backups
STORAGE_PATH=/data/cpms-backups

# 远端保留天数；0 表示永久保留
RETENTION_DAYS=0

# 本机保留天数；0 表示永久保留
LOCAL_RETENTION_DAYS=0
EOF
  chmod 0600 /etc/cpms/backup.env
fi

systemctl daemon-reload
systemctl enable --now cpms-backup.timer

echo "Backup timer installed and enabled."
echo "Edit /etc/cpms/backup.env and set STORAGE_HOST/STORAGE_USER/STORAGE_PATH."
echo "Test once now: sudo /opt/cpms/bin/backup_to_storage.sh"
echo "Check schedule: systemctl list-timers cpms-backup.timer"
