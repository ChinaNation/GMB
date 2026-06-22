#!/usr/bin/env bash
set -euo pipefail

# 用法:
#   sudo bash install_citizencode_app.sh /opt/citizencode /path/to/citizencode-backend-binary
# 示例:
#   sudo bash install_citizencode_app.sh /opt/citizencode /tmp/citizencode-backend

APP_HOME="${1:-/opt/citizencode}"
BINARY_SRC="${2:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CHINA_DB_SRC="${CID_CHINA_DB_SRC:-${DEPLOY_ROOT}/../../backend/china/china.sqlite}"

if [[ -z "${BINARY_SRC}" ]]; then
  echo "缺少后端二进制路径。用法: sudo bash install_citizencode_app.sh /opt/citizencode /path/to/citizencode-backend-binary"
  exit 1
fi

if [[ ! -f "${BINARY_SRC}" ]]; then
  echo "二进制不存在: ${BINARY_SRC}"
  exit 1
fi

if [[ ! -f "${CHINA_DB_SRC}" ]]; then
  echo "行政区 SQLite 不存在: ${CHINA_DB_SRC}"
  exit 1
fi

if ! id -u cid >/dev/null 2>&1; then
  useradd --system --home /opt/citizencode --shell /usr/sbin/nologin cid
fi

mkdir -p "${APP_HOME}/bin" "${APP_HOME}/scripts" "${APP_HOME}/china" /var/lib/citizencode /etc/citizencode
install -m 755 "${BINARY_SRC}" "${APP_HOME}/bin/citizencode-backend"
install -m 644 "${CHINA_DB_SRC}" "${APP_HOME}/china/china.sqlite"
chown -R cid:cid "${APP_HOME}"
chown -R cid:cid /var/lib/citizencode

install -m 755 "${SCRIPT_DIR}/backup_to_standby.sh" "${APP_HOME}/scripts/backup_to_standby.sh"
install -m 755 "${SCRIPT_DIR}/update_citizencode_app.sh" "${APP_HOME}/scripts/update_citizencode_app.sh"

# 生成环境变量模板（首次创建）
if [[ ! -f /etc/citizencode/citizencode.env ]]; then
  cat > /etc/citizencode/citizencode.env <<'ENVEOF'
# CID 后端监听
CID_BIND_ADDR=127.0.0.1:8899
CID_ENV=production
CID_PASSKEY_RP_ID=cid.crcfrcn.com
CID_PASSKEY_ORIGIN=https://cid.crcfrcn.com

# 生产请指向主库（不要写备库），并使用 verify-full
DATABASE_URL=postgres://cid_app:CHANGE_ME_APP_PASSWORD@PRIMARY_DB_FQDN:5432/cid_prod?sslmode=verify-full

# Redis（限流 / 防重放 / 短缓存）
CID_REDIS_URL=redis://127.0.0.1:6379/0

# 行政区随包只读源；唯一来自开发库 citizencode/backend/china/china.sqlite
CID_CHINA_DB=/opt/citizencode/china/china.sqlite

# 链对接相关（CID 基础站点部署可先留空，调用链交互接口前必须补齐）
CID_CHAIN_TOKEN=
CID_CHAIN_SIGNING_SECRET=
CID_CHAIN_WS_URL=
CID_CHAIN_GENESIS_HASH=
CID_RUNTIME_ISSUER_CID_NUMBER=
CID_RUNTIME_ISSUER_MAIN_ACCOUNT=
CID_RUNTIME_SIGNER_PUBKEY=
CID_RUNTIME_SCOPE_PROVINCE_NAME=全国
CID_RUNTIME_SCOPE_CITY_NAME=

# 公共身份查询鉴权（基础站点部署可先留空，启用公开查询时再补）
CID_PUBLIC_SEARCH_TOKEN=

# 主签名密钥和 key id（必须替换）
CID_SIGNING_SEED_HEX=CHANGE_ME_SIGNING_SEED_HEX
CID_KEY_ID=cid-master-v1

ENVEOF
  chmod 600 /etc/citizencode/citizencode.env
  echo "已创建 /etc/citizencode/citizencode.env，请先修改后再启动服务。"
fi

# 执行运行时配置检查
set -a
source /etc/citizencode/citizencode.env
set +a

required_vars=(
  DATABASE_URL
  CID_REDIS_URL
  CID_CHINA_DB
  CID_SIGNING_SEED_HEX
  CID_KEY_ID
)

for key in "${required_vars[@]}"; do
  value="${!key:-}"
  if [[ -z "${value}" ]]; then
    echo "${key} 未配置，停止执行。"
    exit 1
  fi
  if [[ "${value}" == *"CHANGE_ME"* ]]; then
    echo "${key} 仍是占位值(CHANGE_ME)，停止执行。"
    exit 1
  fi
done

# 安装 systemd 单元
install -m 644 "${DEPLOY_ROOT}/systemd/citizencode-backend.service" /etc/systemd/system/citizencode-backend.service
install -m 644 "${DEPLOY_ROOT}/systemd/cid-backup.service" /etc/systemd/system/cid-backup.service
install -m 644 "${DEPLOY_ROOT}/systemd/cid-backup.timer" /etc/systemd/system/cid-backup.timer

# 中文注释：行政区 SQLite 是公权机构确定性目录的输入。每次安装新版应用/行政区后，
# 先把 Postgres 运行库中的 GENERATED 公权机构对账到当前 china.sqlite，再允许服务启动。
"${APP_HOME}/bin/citizencode-backend" reconcile-gov --changed-only
"${APP_HOME}/bin/citizencode-backend" check-gov --strict

systemctl daemon-reload
systemctl enable citizencode-backend
systemctl restart citizencode-backend
systemctl enable cid-backup.timer
systemctl restart cid-backup.timer

echo "部署完成。健康检查: curl http://127.0.0.1:8899/api/v1/health"
