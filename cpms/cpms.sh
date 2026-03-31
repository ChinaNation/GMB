#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export CPMS_DATABASE_URL="${CPMS_DATABASE_URL:-postgres://cpms:cpms@127.0.0.1:5432/cpms}"
CPMS_BIND="${CPMS_BIND:-0.0.0.0:8080}"
CPMS_PORT="${CPMS_BIND##*:}"
CPMS_HEALTHCHECK_URL="http://127.0.0.1:${CPMS_PORT}/api/v1/health"
CPMS_FRONTEND_PORT=5174
DB_ADMIN_URL="${CPMS_DATABASE_URL%/*}/postgres"

# ── 每次运行等于全新初始化：重建数据库 ──
echo "=== CPMS 全新初始化：重建数据库 ==="
psql "$DB_ADMIN_URL" -c "DROP DATABASE IF EXISTS cpms;"
psql "$DB_ADMIN_URL" -c "CREATE DATABASE cpms OWNER cpms;"
echo "数据库已重建"

if [[ ! -d "$ROOT_DIR/frontend/web/node_modules" ]]; then
  (cd "$ROOT_DIR/frontend/web" && npm install)
fi

BACKEND_PID=""
FRONTEND_PID=""

EXISTING_BACKEND_PIDS="$(lsof -ti "tcp:${CPMS_PORT}" || true)"
if [[ -n "$EXISTING_BACKEND_PIDS" ]]; then
  echo "Stopping existing backend on tcp:${CPMS_PORT}..."
  while IFS= read -r pid; do
    [[ -z "$pid" ]] && continue
    kill "$pid" >/dev/null 2>&1 || true
  done <<< "$EXISTING_BACKEND_PIDS"
  sleep 1
fi

echo "=== 启动后端（自动运行 migrations）==="
(cd "$ROOT_DIR" && cargo run --manifest-path backend/Cargo.toml) &
BACKEND_PID="$!"

wait_backend_ready() {
  local retries=120
  local i
  for ((i=1; i<=retries; i++)); do
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
(cd "$ROOT_DIR/frontend/web" && npx vite --port "$CPMS_FRONTEND_PORT") &
FRONTEND_PID="$!"

echo ""
echo "============================================"
echo "  CPMS 系统已启动（全新未初始化状态）"
echo "  前端: http://localhost:${CPMS_FRONTEND_PORT}"
echo "  后端: http://127.0.0.1:${CPMS_PORT}"
echo ""
echo "  请打开浏览器访问 http://localhost:${CPMS_FRONTEND_PORT}"
echo "  按照页面指引完成初始化："
echo "    1. 扫码 SFID 安装授权二维码（QR1）"
echo "    2. 绑定超级管理员"
echo "    3. 生成 QR2 并拿给 SFID 扫码注册"
echo "    4. 登录后扫码 QR3 完成匿名证书注册"
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
