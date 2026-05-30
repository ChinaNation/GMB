#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export CPMS_DATABASE_URL="${CPMS_DATABASE_URL:-postgres://cpms:cpms@127.0.0.1:5432/cpms}"
if [[ -z "${CPMS_KEY_ENCRYPT_SECRET:-}" ]]; then
  CPMS_KEY_FILE="${ROOT_DIR}/.cpms_key"
  if [[ ! -f "${CPMS_KEY_FILE}" ]]; then
    if command -v openssl >/dev/null 2>&1; then
      openssl rand -hex 32 >"${CPMS_KEY_FILE}"
    else
      od -An -tx1 -N32 /dev/urandom | tr -d ' \n' >"${CPMS_KEY_FILE}"
    fi
    chmod 0600 "${CPMS_KEY_FILE}"
  fi
  export CPMS_KEY_ENCRYPT_SECRET="$(tr -d ' \n' <"${CPMS_KEY_FILE}")"
fi
CPMS_BIND="${CPMS_BIND:-0.0.0.0:8080}"
CPMS_PORT="${CPMS_BIND##*:}"
CPMS_HEALTHCHECK_URL="http://127.0.0.1:${CPMS_PORT}/api/v1/health"
CPMS_FRONTEND_PORT=5174
DB_ADMIN_URL="${CPMS_DATABASE_URL%/*}/postgres"
CPMS_RESET="${CPMS_RESET:-0}"

if [[ "${1:-}" == "--reset" ]]; then
  CPMS_RESET=1
fi

# ── 杀掉所有残留的 CPMS 进程（后端 + 前端 + 占用端口的） ──
echo "=== 清理残留 CPMS 进程 ==="
STALE_PIDS="$(pgrep -f 'cpms-back|target/debug/cpms|cpms/backend' || true)"
PORT_PIDS="$(lsof -ti "tcp:${CPMS_PORT}" 2>/dev/null || true)"
FRONT_PIDS="$(lsof -ti "tcp:${CPMS_FRONTEND_PORT}" 2>/dev/null || true)"
ALL_PIDS="$(echo -e "${STALE_PIDS}\n${PORT_PIDS}\n${FRONT_PIDS}" | sort -u)"
for pid in $ALL_PIDS; do
  [[ -z "$pid" ]] && continue
  kill "$pid" 2>/dev/null || true
done
[[ -n "$ALL_PIDS" ]] && sleep 1
echo "残留进程已清理"

# ── 默认保留数据库；仅显式重置时才重建 ──
if [[ "$CPMS_RESET" == "1" ]]; then
  echo "=== CPMS 显式重置：重建数据库 ==="
  psql "$DB_ADMIN_URL" -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'cpms' AND pid != pg_backend_pid();" >/dev/null 2>&1 || true
  psql "$DB_ADMIN_URL" -c "DROP DATABASE IF EXISTS cpms;"
  psql "$DB_ADMIN_URL" -c "CREATE DATABASE cpms OWNER cpms;"
  echo "数据库已重建"
else
  echo "=== CPMS 保留现有数据库（如需重置请执行：CPMS_RESET=1 ./cpms.sh 或 ./cpms.sh --reset）==="
  psql "$DB_ADMIN_URL" -tc "SELECT 1 FROM pg_database WHERE datname = 'cpms';" | grep -q 1 || {
    psql "$DB_ADMIN_URL" -c "CREATE DATABASE cpms OWNER cpms;"
    echo "数据库不存在，已创建空数据库"
  }
fi

if [[ ! -d "$ROOT_DIR/frontend/node_modules" ]]; then
  (cd "$ROOT_DIR/frontend" && npm install)
fi

BACKEND_PID=""
FRONTEND_PID=""

echo "=== 启动后端（自动运行 migrations）==="
(cd "$ROOT_DIR" && cargo run --manifest-path backend/Cargo.toml) &
BACKEND_PID="$!"

wait_backend_ready() {
  local retries=120
  local i
  for ((i=1; i<=retries; i++)); do
    if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
      echo "Backend exited before becoming ready."
      echo "如果日志里出现 VersionMismatch / VersionMissing / Dirty，说明开发库和当前 migration 不一致。"
      echo "CPMS 仍处于开发期，可执行：./cpms.sh --reset 重建开发库。"
      return 1
    fi
    if curl -fsS "$CPMS_HEALTHCHECK_URL" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "Backend did not become ready on ${CPMS_HEALTHCHECK_URL} within ${retries}s"
  return 1
}

wait_backend_ready
echo "=== 后端就绪 ==="

echo "=== 启动前端（端口 ${CPMS_FRONTEND_PORT}）==="
(cd "$ROOT_DIR/frontend" && npx vite --port "$CPMS_FRONTEND_PORT") &
FRONTEND_PID="$!"

echo ""
echo "============================================"
echo "  CPMS 系统已启动"
echo "  前端: http://localhost:${CPMS_FRONTEND_PORT}"
echo "  后端: http://127.0.0.1:${CPMS_PORT}"
echo ""
echo "  请打开浏览器访问 http://localhost:${CPMS_FRONTEND_PORT}"
echo "  如果页面显示未初始化，请按照页面指引完成初始化："
echo "    1. 扫码 SFID 安装授权二维码（INSTALL）"
echo "    2. 绑定超级管理员"
echo "    3. 登录后创建档案并签发 ARCHIVE 档案二维码"
echo "============================================"
echo ""

cleanup() {
  if [[ -n "$FRONTEND_PID" ]]; then
    kill "$FRONTEND_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$BACKEND_PID" ]]; then
    kill "$BACKEND_PID" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT INT TERM
wait
