#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

systemctl disable --now cpms-backend >/dev/null 2>&1 || true
systemctl disable --now cpms-backup.timer >/dev/null 2>&1 || true
rm -f /etc/systemd/system/cpms-backend.service
rm -f /etc/systemd/system/cpms-backup.service /etc/systemd/system/cpms-backup.timer
systemctl daemon-reload

rm -f /etc/nginx/sites-enabled/cpms.conf /etc/nginx/sites-available/cpms.conf
if command -v nginx >/dev/null 2>&1; then
  nginx -t >/dev/null 2>&1 && systemctl reload nginx >/dev/null 2>&1 || true
fi

rm -f /opt/cpms/bin/cpms-backend /opt/cpms/bin/backup_to_storage.sh /opt/cpms/bin/generate_cpms_certs.sh
rmdir /opt/cpms/bin >/dev/null 2>&1 || true
rmdir /opt/cpms >/dev/null 2>&1 || true

echo "CPMS backend service removed."
echo "PostgreSQL, CPMS database, /etc/cpms, /var/lib/cpms and /var/backups/cpms were kept intentionally."
