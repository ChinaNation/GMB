#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

systemctl disable --now cpms-backend >/dev/null 2>&1 || true
rm -f /etc/systemd/system/cpms-backend.service
systemctl daemon-reload

rm -f /opt/cpms/bin/cpms-backend
rmdir /opt/cpms/bin >/dev/null 2>&1 || true
rmdir /opt/cpms >/dev/null 2>&1 || true

echo "CPMS backend service removed."
echo "PostgreSQL and CPMS database were kept intentionally."
