#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export CPMS_DATABASE_URL="${CPMS_DATABASE_URL:-postgres://cpms:cpms@127.0.0.1:5432/cpms}"
CPMS_BIND="${CPMS_BIND:-0.0.0.0:8080}"
CPMS_PORT="${CPMS_BIND##*:}"
CPMS_HEALTHCHECK_URL="http://127.0.0.1:${CPMS_PORT}/api/v1/health"

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

(cd "$ROOT_DIR/frontend/web" && npm run dev) &
FRONTEND_PID="$!"

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
