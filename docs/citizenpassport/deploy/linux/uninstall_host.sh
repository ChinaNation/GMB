#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

systemctl disable --now citizenpassport-backend >/dev/null 2>&1 || true
systemctl disable --now citizenpassport-backup.timer >/dev/null 2>&1 || true
rm -f /etc/systemd/system/citizenpassport-backend.service
rm -f /etc/systemd/system/citizenpassport-backup.service /etc/systemd/system/citizenpassport-backup.timer
systemctl daemon-reload

rm -f /etc/nginx/sites-enabled/citizenpassport.conf /etc/nginx/sites-available/citizenpassport.conf
if command -v nginx >/dev/null 2>&1; then
  nginx -t >/dev/null 2>&1 && systemctl reload nginx >/dev/null 2>&1 || true
fi

rm -f /opt/citizenpassport/bin/citizenpassport-backend /opt/citizenpassport/bin/backup_to_storage.sh /opt/citizenpassport/bin/generate_citizenpassport_certs.sh
rm -f /opt/citizenpassport/data/china.sqlite
rmdir /opt/citizenpassport/bin >/dev/null 2>&1 || true
rmdir /opt/citizenpassport/data >/dev/null 2>&1 || true
rmdir /opt/citizenpassport >/dev/null 2>&1 || true

echo "CPMS backend service removed."
echo "PostgreSQL, CPMS database, /etc/citizenpassport, /var/lib/citizenpassport and /var/backups/citizenpassport were kept intentionally."
