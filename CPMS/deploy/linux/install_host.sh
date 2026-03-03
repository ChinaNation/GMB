#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAYLOAD_DIR="${SCRIPT_DIR}/payload"
BIN_SRC="${PAYLOAD_DIR}/bin/cpms-backend"
SCHEMA_SQL="${PAYLOAD_DIR}/db/schema.sql"
SEED_SQL="${PAYLOAD_DIR}/db/seed.sql"
SERVICE_SRC="${PAYLOAD_DIR}/systemd/cpms-backend.service"
BACKUP_SCRIPT_SRC="${PAYLOAD_DIR}/bin/backup_to_storage.sh"
BACKUP_SERVICE_SRC="${PAYLOAD_DIR}/systemd/cpms-backup.service"
BACKUP_TIMER_SRC="${PAYLOAD_DIR}/systemd/cpms-backup.timer"

if [[ ! -x "${BIN_SRC}" ]]; then
  echo "ERROR: missing backend binary at ${BIN_SRC}"
  exit 1
fi
if [[ ! -f "${SCHEMA_SQL}" || ! -f "${SEED_SQL}" ]]; then
  echo "ERROR: missing database sql payload"
  exit 1
fi
if [[ ! -f "${SERVICE_SRC}" ]]; then
  echo "ERROR: missing systemd service file"
  exit 1
fi

install_postgres() {
  if command -v psql >/dev/null 2>&1; then
    return 0
  fi
  if command -v apt-get >/dev/null 2>&1; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y postgresql postgresql-client
  else
    echo "ERROR: only Debian/Ubuntu are supported by this installer."
    echo "Install PostgreSQL manually, then rerun installer."
    exit 1
  fi
}

ensure_postgres_service() {
  systemctl enable --now postgresql
}

ensure_cpms_user() {
  if ! id -u cpms >/dev/null 2>&1; then
    useradd --system --home-dir /var/lib/cpms --create-home --shell /usr/sbin/nologin cpms
  fi
}

generate_password() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex 24
  else
    tr -dc 'a-zA-Z0-9' </dev/urandom | head -c 32
  fi
}

setup_database() {
  local db_name="cpms"
  local db_user="cpms"
  local db_password
  db_password="$(generate_password)"

  install -d -m 0750 /etc/cpms
  cat >/etc/cpms/cpms-backend.env <<EOF
CPMS_BIND=0.0.0.0:8080
CPMS_DATABASE_URL=postgresql://${db_user}:${db_password}@127.0.0.1:5432/${db_name}
CPMS_INSTALL_FILE=/var/lib/cpms/runtime/cpms_install_init.json
EOF
  chmod 0600 /etc/cpms/cpms-backend.env
  chown root:cpms /etc/cpms/cpms-backend.env

  su - postgres -c "psql -tc \"SELECT 1 FROM pg_roles WHERE rolname='${db_user}'\" | grep -q 1" \
    || su - postgres -c "psql -c \"CREATE ROLE ${db_user} WITH LOGIN PASSWORD '${db_password}';\""
  su - postgres -c "psql -tc \"SELECT 1 FROM pg_database WHERE datname='${db_name}'\" | grep -q 1" \
    || su - postgres -c "psql -c \"CREATE DATABASE ${db_name} OWNER ${db_user};\""

  su - postgres -c "psql -d ${db_name} -v ON_ERROR_STOP=1 -f '${SCHEMA_SQL}'"
  su - postgres -c "psql -d ${db_name} -v ON_ERROR_STOP=1 -f '${SEED_SQL}'"
}

install_backend() {
  install -d -m 0755 /opt/cpms/bin
  install -d -m 0750 /var/lib/cpms/runtime
  install -m 0755 "${BIN_SRC}" /opt/cpms/bin/cpms-backend
  if [[ -f "${BACKUP_SCRIPT_SRC}" ]]; then
    install -m 0755 "${BACKUP_SCRIPT_SRC}" /opt/cpms/bin/backup_to_storage.sh
  fi
  chown -R cpms:cpms /var/lib/cpms
}

install_service() {
  install -m 0644 "${SERVICE_SRC}" /etc/systemd/system/cpms-backend.service
  if [[ -f "${BACKUP_SERVICE_SRC}" ]]; then
    install -m 0644 "${BACKUP_SERVICE_SRC}" /etc/systemd/system/cpms-backup.service
  fi
  if [[ -f "${BACKUP_TIMER_SRC}" ]]; then
    install -m 0644 "${BACKUP_TIMER_SRC}" /etc/systemd/system/cpms-backup.timer
  fi
  systemctl daemon-reload
  systemctl enable --now cpms-backend
}

prepare_backup_env() {
  install -d -m 0750 /etc/cpms
  if [[ ! -f /etc/cpms/backup.env ]]; then
    cat >/etc/cpms/backup.env <<'EOF'
# Storage computer SSH address
STORAGE_HOST=CHANGE_ME
STORAGE_PORT=22
STORAGE_USER=CHANGE_ME

# Absolute remote directory path, e.g. /data/cpms-backups
STORAGE_PATH=/data/cpms-backups

# Keep remote backups forever when set to 0
RETENTION_DAYS=0

# Keep local backups forever when set to 0
LOCAL_RETENTION_DAYS=0
EOF
    chmod 0600 /etc/cpms/backup.env
  fi
}

main() {
  install_postgres
  ensure_postgres_service
  ensure_cpms_user
  setup_database
  install_backend
  install_service
  prepare_backup_env

  echo "CPMS host install complete."
  echo "Login page: http://<host-lan-ip>:8080/login"
  echo "Service status: systemctl status cpms-backend"
  echo "Backup config: /etc/cpms/backup.env"
  echo "Enable backup timer after config: systemctl enable --now cpms-backup.timer"
}

main "$@"
