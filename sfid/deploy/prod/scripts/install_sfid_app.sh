#!/usr/bin/env bash
set -euo pipefail

# 用法:
#   sudo bash install_sfid_app.sh /opt/sfid /path/to/sfid-backend-binary
# 示例:
#   sudo bash install_sfid_app.sh /opt/sfid /tmp/sfid-backend

APP_HOME="${1:-/opt/sfid}"
BINARY_SRC="${2:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROJECT_ROOT="$(cd "${DEPLOY_ROOT}/../.." && pwd)"

if [[ -z "${BINARY_SRC}" ]]; then
  echo "缺少后端二进制路径。用法: sudo bash install_sfid_app.sh /opt/sfid /path/to/sfid-backend-binary"
  exit 1
fi

if [[ ! -f "${BINARY_SRC}" ]]; then
  echo "二进制不存在: ${BINARY_SRC}"
  exit 1
fi

if ! command -v psql >/dev/null 2>&1; then
  echo "未检测到 psql，请先安装 PostgreSQL 客户端。"
  exit 1
fi

if ! id -u sfid >/dev/null 2>&1; then
  useradd --system --home /opt/sfid --shell /usr/sbin/nologin sfid
fi

mkdir -p "${APP_HOME}/bin" "${APP_HOME}/backend/db/migrations" "${APP_HOME}/scripts" /etc/sfid
install -m 755 "${BINARY_SRC}" "${APP_HOME}/bin/sfid-backend"
chown -R sfid:sfid "${APP_HOME}"

# 同步迁移脚本到部署目录
cp -f "${PROJECT_ROOT}/backend/db/migrations/"*.sql "${APP_HOME}/backend/db/migrations/"
install -m 755 "${SCRIPT_DIR}/backup_to_standby.sh" "${APP_HOME}/scripts/backup_to_standby.sh"
install -m 755 "${SCRIPT_DIR}/apply_sfid_migrations.sh" "${APP_HOME}/scripts/apply_sfid_migrations.sh"
install -m 755 "${SCRIPT_DIR}/update_sfid_app.sh" "${APP_HOME}/scripts/update_sfid_app.sh"

# 生成环境变量模板（首次创建）
if [[ ! -f /etc/sfid/sfid.env ]]; then
  cat > /etc/sfid/sfid.env <<'ENVEOF'
# SFID 后端监听
SFID_BIND_ADDR=127.0.0.1:8899

# 生产请指向主库（不要写备库），并使用 verify-full
DATABASE_URL=postgres://sfid_app:CHANGE_ME_APP_PASSWORD@PRIMARY_DB_FQDN:5432/sfid_prod?sslmode=verify-full

# Redis（限流 / 防重放 / 短缓存）
SFID_REDIS_URL=redis://127.0.0.1:6379/0

# 链对接相关（基础站点部署可先留空，后续接链时再补）
SFID_CHAIN_TOKEN=
SFID_CHAIN_SIGNING_SECRET=
SFID_CHAIN_WS_URL=
SFID_CHAIN_GENESIS_HASH=

# 公共身份查询鉴权（基础站点部署可先留空，启用公开查询时再补）
SFID_PUBLIC_SEARCH_TOKEN=

# 主签名密钥和 key id（必须替换）
SFID_SIGNING_SEED_HEX=CHANGE_ME_SIGNING_SEED_HEX
SFID_KEY_ID=sfid-master-v1

# PII 列加密密钥（兼容保留，基础站点部署可先留空）
SFID_PII_KEY=
ENVEOF
  chmod 600 /etc/sfid/sfid.env
  echo "已创建 /etc/sfid/sfid.env，请先修改后再启动服务。"
fi

# 执行运行时配置检查
set -a
source /etc/sfid/sfid.env
set +a

required_vars=(
  DATABASE_URL
  SFID_REDIS_URL
  SFID_SIGNING_SEED_HEX
  SFID_KEY_ID
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
install -m 644 "${DEPLOY_ROOT}/systemd/sfid-backend.service" /etc/systemd/system/sfid-backend.service
install -m 644 "${DEPLOY_ROOT}/systemd/sfid-backup.service" /etc/systemd/system/sfid-backup.service
install -m 644 "${DEPLOY_ROOT}/systemd/sfid-backup.timer" /etc/systemd/system/sfid-backup.timer

# 执行数据库迁移（自动跳过已执行项）
"${APP_HOME}/scripts/apply_sfid_migrations.sh" "${DATABASE_URL}" "${APP_HOME}/backend/db/migrations"

# 收敛应用角色 DELETE 权限：仅允许必要运行时表，禁止删除审计日志
psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 <<'SQL'
GRANT DELETE ON TABLE binding_unique_locks, bind_reward_states, runtime_cache_entries, runtime_misc TO sfid_app;
REVOKE DELETE ON TABLE audit_logs FROM sfid_app;
SQL

systemctl daemon-reload
systemctl enable sfid-backend
systemctl restart sfid-backend
systemctl enable sfid-backup.timer
systemctl restart sfid-backup.timer

echo "部署完成。健康检查: curl http://127.0.0.1:8899/api/v1/health"
