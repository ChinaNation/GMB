#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAYLOAD_DIR="${SCRIPT_DIR}/payload"
BIN_SRC="${PAYLOAD_DIR}/bin/cpms-backend"
FRONTEND_SRC="${PAYLOAD_DIR}/frontend"
SCHEMA_SQL="${PAYLOAD_DIR}/db/schema.sql"
SEED_SQL="${PAYLOAD_DIR}/db/seed.sql"
SERVICE_SRC="${PAYLOAD_DIR}/systemd/cpms-backend.service"
BACKUP_SCRIPT_SRC="${PAYLOAD_DIR}/bin/backup_to_storage.sh"
BACKUP_SERVICE_SRC="${PAYLOAD_DIR}/systemd/cpms-backup.service"
BACKUP_TIMER_SRC="${PAYLOAD_DIR}/systemd/cpms-backup.timer"
NGINX_SRC="${PAYLOAD_DIR}/nginx/cpms.conf"
CERT_SCRIPT_SRC="${PAYLOAD_DIR}/certs/generate_cpms_certs.sh"
DEBS_DIR="${PAYLOAD_DIR}/debs"
MANIFEST_SRC="${PAYLOAD_DIR}/manifest.env"

check_payload() {
  if [[ ! -x "${BIN_SRC}" ]]; then
    echo "ERROR: missing backend binary at ${BIN_SRC}"
    exit 1
  fi
  if [[ ! -f "${FRONTEND_SRC}/index.html" ]]; then
    echo "ERROR: missing frontend static payload at ${FRONTEND_SRC}"
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
  if [[ ! -f "${MANIFEST_SRC}" ]]; then
    echo "ERROR: missing package manifest at ${MANIFEST_SRC}"
    exit 1
  fi
  if [[ ! -f "${NGINX_SRC}" || ! -f "${CERT_SCRIPT_SRC}" ]]; then
    echo "ERROR: missing nginx or certificate payload"
    exit 1
  fi
  if ! compgen -G "${DEBS_DIR}/*.deb" >/dev/null; then
    echo "ERROR: missing offline deb payload at ${DEBS_DIR}"
    exit 1
  fi
}

load_manifest() {
  source "${MANIFEST_SRC}"
  if [[ "${CPMS_PACKAGE_OS:-}" != "ubuntu24" ]]; then
    echo "ERROR: unsupported package OS: ${CPMS_PACKAGE_OS:-missing}"
    exit 1
  fi
  case "${CPMS_PACKAGE_ARCH:-}" in
    amd64|arm64)
      ;;
    *)
      echo "ERROR: unsupported package arch: ${CPMS_PACKAGE_ARCH:-missing}"
      exit 1
      ;;
  esac
}

current_debian_arch() {
  if command -v dpkg >/dev/null 2>&1; then
    dpkg --print-architecture
    return 0
  fi
  case "$(uname -m)" in
    x86_64|amd64) echo "amd64" ;;
    aarch64|arm64) echo "arm64" ;;
    *) uname -m ;;
  esac
}

check_os() {
  local arch
  arch="$(current_debian_arch)"
  if [[ "${arch}" != "${CPMS_PACKAGE_ARCH}" ]]; then
    echo "ERROR: ${CPMS_PACKAGE_NAME:-cpms-ubuntu24-${CPMS_PACKAGE_ARCH}.run} only supports ${CPMS_PACKAGE_ARCH}; current architecture is ${arch}"
    exit 1
  fi
  if [[ ! -f /etc/os-release ]]; then
    echo "ERROR: unsupported Linux distribution"
    exit 1
  fi
  # 中文注释：安装包只面向 Ubuntu Server 24.04 LTS，避免在未知系统上半安装。
  source /etc/os-release
  if [[ "${ID:-}" != "ubuntu" || "${VERSION_ID:-}" != "24.04" ]]; then
    echo "ERROR: this installer requires Ubuntu 24.04 ${CPMS_PACKAGE_ARCH}"
    exit 1
  fi
}

install_offline_deps() {
  export DEBIAN_FRONTEND=noninteractive
  install -d -m 0755 /opt/cpms/offline-debs
  cp -f "${DEBS_DIR}"/*.deb /opt/cpms/offline-debs/

  # 中文注释：这里只使用安装包内置 deb，不执行 apt-get update，不读取外部网络源。
  if ! dpkg -i /opt/cpms/offline-debs/*.deb; then
    if ! dpkg --configure -a; then
      echo "ERROR: offline deb dependency closure is incomplete"
      echo "Please rebuild ${CPMS_PACKAGE_NAME:-cpms-ubuntu24-${CPMS_PACKAGE_ARCH}.run} with the full Ubuntu 24.04 ${CPMS_PACKAGE_ARCH} dependency closure."
      exit 1
    fi
  fi

  for cmd in psql pg_dump nginx openssl rsync ssh; do
    if ! command -v "${cmd}" >/dev/null 2>&1; then
      echo "ERROR: missing runtime command after offline install: ${cmd}"
      exit 1
    fi
  done
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
  openssl rand -hex 24
}

generate_secret_hex32() {
  openssl rand -hex 32
}

setup_database() {
  local db_name="cpms"
  local db_user="cpms"
  local db_password
  local key_encrypt_secret

  install -d -m 0750 /etc/cpms
  install -d -m 0750 -o cpms -g cpms /var/lib/cpms/runtime
  install -d -m 0750 -o cpms -g cpms /var/lib/cpms/materials
  install -d -m 0750 /var/backups/cpms

  if [[ -f /etc/cpms/cpms-backend.env ]]; then
    set -a
    source /etc/cpms/cpms-backend.env
    set +a
    if [[ -z "${CPMS_DATABASE_URL:-}" ]]; then
      echo "ERROR: existing /etc/cpms/cpms-backend.env is missing CPMS_DATABASE_URL"
      exit 1
    fi
    db_password="${CPMS_DATABASE_URL#postgresql://${db_user}:}"
    db_password="${db_password%@127.0.0.1:5432/${db_name}}"
    key_encrypt_secret="${CPMS_KEY_ENCRYPT_SECRET:-}"
    if [[ "${db_password}" == "${CPMS_DATABASE_URL:-}" || -z "${key_encrypt_secret}" ]]; then
      echo "ERROR: existing /etc/cpms/cpms-backend.env is not a valid CPMS installer env file"
      exit 1
    fi
  else
    db_password="$(generate_password)"
    key_encrypt_secret="$(generate_secret_hex32)"
    cat >/etc/cpms/cpms-backend.env <<EOF
CPMS_BIND=127.0.0.1:8080
CPMS_DATABASE_URL=postgresql://${db_user}:${db_password}@127.0.0.1:5432/${db_name}
CPMS_KEY_ENCRYPT_SECRET=${key_encrypt_secret}
CPMS_FRONTEND_DIR=/opt/cpms/frontend
CPMS_MATERIALS_DIR=/var/lib/cpms/materials
CPMS_COOKIE_SECURE=true
EOF
  fi
  chmod 0600 /etc/cpms/cpms-backend.env
  chown root:cpms /etc/cpms/cpms-backend.env

  if su - postgres -c "psql -tc \"SELECT 1 FROM pg_roles WHERE rolname='${db_user}'\" | grep -q 1"; then
    su - postgres -c "psql -c \"ALTER ROLE ${db_user} WITH LOGIN PASSWORD '${db_password}';\""
  else
    su - postgres -c "psql -c \"CREATE ROLE ${db_user} WITH LOGIN PASSWORD '${db_password}';\""
  fi
  su - postgres -c "psql -tc \"SELECT 1 FROM pg_database WHERE datname='${db_name}'\" | grep -q 1" \
    || su - postgres -c "psql -c \"CREATE DATABASE ${db_name} OWNER ${db_user};\""

  su - postgres -c "psql -d ${db_name} -v ON_ERROR_STOP=1 -f '${SCHEMA_SQL}'"
  su - postgres -c "psql -d ${db_name} -v ON_ERROR_STOP=1 -f '${SEED_SQL}'"
}

install_backend() {
  install -d -m 0755 /opt/cpms/bin
  install -d -m 0755 /opt/cpms/frontend
  install -m 0755 "${BIN_SRC}" /opt/cpms/bin/cpms-backend
  install -m 0755 "${CERT_SCRIPT_SRC}" /opt/cpms/bin/generate_cpms_certs.sh
  rm -rf /opt/cpms/frontend/*
  cp -R "${FRONTEND_SRC}/." /opt/cpms/frontend/
  if [[ -f "${BACKUP_SCRIPT_SRC}" ]]; then
    install -m 0755 "${BACKUP_SCRIPT_SRC}" /opt/cpms/bin/backup_to_storage.sh
  fi
  chown -R root:root /opt/cpms/frontend
  chown -R cpms:cpms /var/lib/cpms
}

install_certs() {
  /opt/cpms/bin/generate_cpms_certs.sh
}

configure_nginx() {
  install -d -m 0755 /etc/nginx/sites-available /etc/nginx/sites-enabled
  install -m 0644 "${NGINX_SRC}" /etc/nginx/sites-available/cpms.conf
  rm -f /etc/nginx/sites-enabled/default
  ln -sfn /etc/nginx/sites-available/cpms.conf /etc/nginx/sites-enabled/cpms.conf
  nginx -t
  systemctl enable --now nginx
  systemctl reload nginx
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
}

main() {
  check_payload
  load_manifest
  check_os
  install_offline_deps
  ensure_postgres_service
  ensure_cpms_user
  setup_database
  install_backend
  install_certs
  install_service
  configure_nginx
  prepare_backup_env

  echo "CPMS host install complete."
  echo "Login page: https://www.cpms.com/login"
  echo "Root CA certificate: /etc/cpms/certs/cpms-root-ca.crt"
  echo "Service status: systemctl status cpms-backend"
  echo "Nginx status: systemctl status nginx"
  echo "Backup config: /etc/cpms/backup.env"
  echo "Enable backup timer after config: systemctl enable --now cpms-backup.timer"
}

main "$@"
