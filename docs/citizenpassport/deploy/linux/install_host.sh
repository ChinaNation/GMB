#!/usr/bin/env bash
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: please run as root"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PAYLOAD_DIR="${SCRIPT_DIR}/payload"
BIN_SRC="${PAYLOAD_DIR}/bin/citizenpassport-backend"
CHINA_DB_SRC="${PAYLOAD_DIR}/data/china.sqlite"
FRONTEND_SRC="${PAYLOAD_DIR}/frontend"
SERVICE_SRC="${PAYLOAD_DIR}/systemd/citizenpassport-backend.service"
BACKUP_SCRIPT_SRC="${PAYLOAD_DIR}/bin/backup_to_storage.sh"
BACKUP_SERVICE_SRC="${PAYLOAD_DIR}/systemd/citizenpassport-backup.service"
BACKUP_TIMER_SRC="${PAYLOAD_DIR}/systemd/citizenpassport-backup.timer"
NGINX_SRC="${PAYLOAD_DIR}/nginx/citizenpassport.conf"
CERT_SCRIPT_SRC="${PAYLOAD_DIR}/certs/generate_citizenpassport_certs.sh"
INSTALL_GUIDE_SRC="${PAYLOAD_DIR}/docs/CitizenPassport安装配置手册.md"
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
  if [[ ! -f "${CHINA_DB_SRC}" ]]; then
    echo "ERROR: missing china administrative-area source at ${CHINA_DB_SRC}"
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
  if [[ ! -f "${INSTALL_GUIDE_SRC}" ]]; then
    echo "ERROR: missing install guide Markdown at ${INSTALL_GUIDE_SRC}"
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
    echo "ERROR: ${CPMS_PACKAGE_NAME:-citizenpassport-ubuntu24-${CPMS_PACKAGE_ARCH}.run} only supports ${CPMS_PACKAGE_ARCH}; current architecture is ${arch}"
    exit 1
  fi
  if [[ ! -f /etc/os-release ]]; then
    echo "ERROR: unsupported Linux distribution"
    exit 1
  fi
  # 安装包只面向 Ubuntu Server 24.04 LTS，避免在未知系统上半安装。
  source /etc/os-release
  if [[ "${ID:-}" != "ubuntu" || "${VERSION_ID:-}" != "24.04" ]]; then
    echo "ERROR: this installer requires Ubuntu 24.04 ${CPMS_PACKAGE_ARCH}"
    exit 1
  fi
}

install_offline_deps() {
  export DEBIAN_FRONTEND=noninteractive
  install -d -m 0755 /opt/citizenpassport/offline-debs
  cp -f "${DEBS_DIR}"/*.deb /opt/citizenpassport/offline-debs/

  # 这里只使用安装包内置 deb，不执行 apt-get update，不读取外部网络源。
  if ! dpkg -i /opt/citizenpassport/offline-debs/*.deb; then
    if ! dpkg --configure -a; then
      echo "ERROR: offline deb dependency closure is incomplete"
      echo "Please rebuild ${CPMS_PACKAGE_NAME:-citizenpassport-ubuntu24-${CPMS_PACKAGE_ARCH}.run} with the full Ubuntu 24.04 ${CPMS_PACKAGE_ARCH} dependency closure."
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

stop_existing_backend() {
  systemctl stop citizenpassport-backend >/dev/null 2>&1 || true
}

ensure_cpms_user() {
  if ! id -u cpms >/dev/null 2>&1; then
    useradd --system --home-dir /var/lib/citizenpassport --create-home --shell /usr/sbin/nologin cpms
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

  install -d -m 0750 /etc/citizenpassport
  install -d -m 0750 -o cpms -g cpms /var/lib/citizenpassport/runtime
  install -d -m 0750 -o cpms -g cpms /var/lib/citizenpassport/materials
  install -d -m 0750 /var/backups/citizenpassport

  if [[ -f /etc/citizenpassport/citizenpassport-backend.env ]]; then
    set -a
    source /etc/citizenpassport/citizenpassport-backend.env
    set +a
    if [[ -z "${CPMS_DATABASE_URL:-}" ]]; then
      echo "ERROR: existing /etc/citizenpassport/citizenpassport-backend.env is missing CPMS_DATABASE_URL"
      exit 1
    fi
    db_password="${CPMS_DATABASE_URL#postgresql://${db_user}:}"
    db_password="${db_password%@127.0.0.1:5432/${db_name}}"
    key_encrypt_secret="${CPMS_KEY_ENCRYPT_SECRET:-}"
    if [[ "${db_password}" == "${CPMS_DATABASE_URL:-}" || -z "${key_encrypt_secret}" ]]; then
      echo "ERROR: existing /etc/citizenpassport/citizenpassport-backend.env is not a valid CPMS installer env file"
      exit 1
    fi
  else
    db_password="$(generate_password)"
    key_encrypt_secret="$(generate_secret_hex32)"
    cat >/etc/citizenpassport/citizenpassport-backend.env <<EOF
CPMS_BIND=127.0.0.1:8080
CPMS_DATABASE_URL=postgresql://${db_user}:${db_password}@127.0.0.1:5432/${db_name}
CPMS_KEY_ENCRYPT_SECRET=${key_encrypt_secret}
CPMS_FRONTEND_DIR=/opt/citizenpassport/frontend
CPMS_CHINA_DB=/opt/citizenpassport/data/china.sqlite
CPMS_MATERIALS_DIR=/var/lib/citizenpassport/materials
CPMS_COOKIE_SECURE=true
EOF
  fi
  # 行政区唯一源 china.sqlite 的路径，老安装升级时补齐缺失项。
  if ! grep -q '^CPMS_CHINA_DB=' /etc/citizenpassport/citizenpassport-backend.env; then
    echo 'CPMS_CHINA_DB=/opt/citizenpassport/data/china.sqlite' >>/etc/citizenpassport/citizenpassport-backend.env
  fi
  chmod 0600 /etc/citizenpassport/citizenpassport-backend.env
  chown root:cpms /etc/citizenpassport/citizenpassport-backend.env

  if su - postgres -c "psql -tc \"SELECT 1 FROM pg_roles WHERE rolname='${db_user}'\" | grep -q 1"; then
    su - postgres -c "psql -c \"ALTER ROLE ${db_user} WITH LOGIN PASSWORD '${db_password}';\""
  else
    su - postgres -c "psql -c \"CREATE ROLE ${db_user} WITH LOGIN PASSWORD '${db_password}';\""
  fi
  su - postgres -c "psql -tc \"SELECT 1 FROM pg_database WHERE datname='${db_name}'\" | grep -q 1" \
    || su - postgres -c "psql -c \"CREATE DATABASE ${db_name} OWNER ${db_user};\""

  # 正式安装不导入 schema.sql；数据库结构唯一入口是后端启动时的 MIGRATOR.run()。
  # 这里仅创建/修正库、schema 和旧安装残留对象的归属，避免 cpms 用户执行 migration 时无权限。
  su - postgres -c "psql -v ON_ERROR_STOP=1 -c \"ALTER DATABASE ${db_name} OWNER TO ${db_user};\""
  su - postgres -c "psql -d ${db_name} -v ON_ERROR_STOP=1" <<SQL
ALTER SCHEMA public OWNER TO ${db_user};
GRANT ALL PRIVILEGES ON DATABASE ${db_name} TO ${db_user};
GRANT USAGE, CREATE ON SCHEMA public TO ${db_user};

DO \$\$
DECLARE r record;
BEGIN
  FOR r IN SELECT tablename FROM pg_tables WHERE schemaname = 'public' LOOP
    EXECUTE format('ALTER TABLE public.%I OWNER TO ${db_user}', r.tablename);
    EXECUTE format('GRANT ALL PRIVILEGES ON TABLE public.%I TO ${db_user}', r.tablename);
  END LOOP;

  FOR r IN SELECT sequencename FROM pg_sequences WHERE schemaname = 'public' LOOP
    EXECUTE format('ALTER SEQUENCE public.%I OWNER TO ${db_user}', r.sequencename);
    EXECUTE format('GRANT ALL PRIVILEGES ON SEQUENCE public.%I TO ${db_user}', r.sequencename);
  END LOOP;
END \$\$;
SQL
}

install_backend() {
  install -d -m 0755 /opt/citizenpassport/bin
  install -d -m 0755 /opt/citizenpassport/data
  install -d -m 0755 /opt/citizenpassport/docs
  install -d -m 0755 /opt/citizenpassport/frontend
  install -m 0755 "${BIN_SRC}" /opt/citizenpassport/bin/citizenpassport-backend
  # 行政区唯一源只读拷贝，运行时由 CPMS_CHINA_DB 指向此路径。
  install -m 0644 "${CHINA_DB_SRC}" /opt/citizenpassport/data/china.sqlite
  install -m 0755 "${CERT_SCRIPT_SRC}" /opt/citizenpassport/bin/generate_citizenpassport_certs.sh
  install -m 0644 "${INSTALL_GUIDE_SRC}" /opt/citizenpassport/docs/CitizenPassport安装配置手册.md
  rm -rf /opt/citizenpassport/frontend/*
  cp -R "${FRONTEND_SRC}/." /opt/citizenpassport/frontend/
  if [[ -f "${BACKUP_SCRIPT_SRC}" ]]; then
    install -m 0755 "${BACKUP_SCRIPT_SRC}" /opt/citizenpassport/bin/backup_to_storage.sh
  fi
  chown -R root:root /opt/citizenpassport/frontend
  chown -R cpms:cpms /var/lib/citizenpassport
}

install_certs() {
  /opt/citizenpassport/bin/generate_citizenpassport_certs.sh
}

trust_local_root_ca() {
  if [[ -f /etc/citizenpassport/certs/citizenpassport-root-ca.crt ]]; then
    install -m 0644 /etc/citizenpassport/certs/citizenpassport-root-ca.crt /usr/local/share/ca-certificates/citizenpassport-root-ca.crt
    update-ca-certificates
  fi
}

configure_nginx() {
  install -d -m 0755 /etc/nginx/sites-available /etc/nginx/sites-enabled
  install -m 0644 "${NGINX_SRC}" /etc/nginx/sites-available/citizenpassport.conf
  rm -f /etc/nginx/sites-enabled/default
  ln -sfn /etc/nginx/sites-available/citizenpassport.conf /etc/nginx/sites-enabled/citizenpassport.conf
  nginx -t
  systemctl enable --now nginx
  systemctl reload nginx
}

install_service() {
  install -m 0644 "${SERVICE_SRC}" /etc/systemd/system/citizenpassport-backend.service
  if [[ -f "${BACKUP_SERVICE_SRC}" ]]; then
    install -m 0644 "${BACKUP_SERVICE_SRC}" /etc/systemd/system/citizenpassport-backup.service
  fi
  if [[ -f "${BACKUP_TIMER_SRC}" ]]; then
    install -m 0644 "${BACKUP_TIMER_SRC}" /etc/systemd/system/citizenpassport-backup.timer
  fi
  systemctl daemon-reload
  systemctl enable --now citizenpassport-backend
}

prepare_backup_env() {
  install -d -m 0750 /etc/citizenpassport
  if [[ ! -f /etc/citizenpassport/backup.env ]]; then
    cat >/etc/citizenpassport/backup.env <<'EOF'
# 备份存储机 SSH 地址
STORAGE_HOST=CHANGE_ME
STORAGE_PORT=22
STORAGE_USER=CHANGE_ME

# 备份存储机上的绝对目录，例如 /data/citizenpassport-backups
STORAGE_PATH=/data/citizenpassport-backups

# 远端保留天数；0 表示永久保留
RETENTION_DAYS=0

# 本机保留天数；0 表示永久保留
LOCAL_RETENTION_DAYS=0
EOF
    chmod 0600 /etc/citizenpassport/backup.env
  fi
}

main() {
  check_payload
  load_manifest
  check_os
  install_offline_deps
  ensure_postgres_service
  stop_existing_backend
ensure_cpms_user
  setup_database
  install_backend
  install_certs
  trust_local_root_ca
  install_service
  configure_nginx
  prepare_backup_env

  echo "CitizenPassport host install complete."
  echo "Login page: https://www.citizenpassport.com/login"
  echo "Root CA certificate: /etc/citizenpassport/certs/citizenpassport-root-ca.crt"
  echo "Install guide: /opt/citizenpassport/docs/CitizenPassport安装配置手册.md"
  echo "Service status: systemctl status citizenpassport-backend"
  echo "Nginx status: systemctl status nginx"
  echo "Backup config: /etc/citizenpassport/backup.env"
  echo "Enable backup timer after config: systemctl enable --now citizenpassport-backup.timer"
}

main "$@"
