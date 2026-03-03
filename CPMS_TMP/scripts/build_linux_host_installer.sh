#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT_DIR}/dist/cpms-host-linux-x64"
PAYLOAD_DIR="${OUT_DIR}/payload"

echo "[1/4] Build backend release binary"
cargo build --release --manifest-path "${ROOT_DIR}/backend/Cargo.toml"

echo "[2/4] Prepare installer layout"
rm -rf "${OUT_DIR}"
mkdir -p "${PAYLOAD_DIR}/bin" "${PAYLOAD_DIR}/db" "${PAYLOAD_DIR}/systemd"

echo "[3/4] Copy payload files"
cp "${ROOT_DIR}/backend/target/release/cpms-backend" "${PAYLOAD_DIR}/bin/cpms-backend"
cp "${ROOT_DIR}/deploy/linux/backup_to_storage.sh" "${PAYLOAD_DIR}/bin/backup_to_storage.sh"
cp "${ROOT_DIR}/backend/db/schema.sql" "${PAYLOAD_DIR}/db/schema.sql"
cp "${ROOT_DIR}/backend/db/seed.sql" "${PAYLOAD_DIR}/db/seed.sql"
cp "${ROOT_DIR}/deploy/linux/systemd/cpms-backend.service" "${PAYLOAD_DIR}/systemd/cpms-backend.service"
cp "${ROOT_DIR}/deploy/linux/systemd/cpms-backup.service" "${PAYLOAD_DIR}/systemd/cpms-backup.service"
cp "${ROOT_DIR}/deploy/linux/systemd/cpms-backup.timer" "${PAYLOAD_DIR}/systemd/cpms-backup.timer"
cp "${ROOT_DIR}/deploy/linux/install_host.sh" "${OUT_DIR}/install_host.sh"
cp "${ROOT_DIR}/deploy/linux/uninstall_host.sh" "${OUT_DIR}/uninstall_host.sh"
cp "${ROOT_DIR}/deploy/linux/install_backup_timer.sh" "${OUT_DIR}/install_backup_timer.sh"

chmod +x \
  "${OUT_DIR}/install_host.sh" \
  "${OUT_DIR}/uninstall_host.sh" \
  "${OUT_DIR}/install_backup_timer.sh" \
  "${PAYLOAD_DIR}/bin/cpms-backend" \
  "${PAYLOAD_DIR}/bin/backup_to_storage.sh"

echo "[4/4] Create archive"
tar -C "${ROOT_DIR}/dist" -czf "${ROOT_DIR}/dist/cpms-host-linux-x64.tar.gz" "cpms-host-linux-x64"

echo
echo "Done."
echo "Installer directory: ${OUT_DIR}"
echo "Installer archive: ${ROOT_DIR}/dist/cpms-host-linux-x64.tar.gz"
