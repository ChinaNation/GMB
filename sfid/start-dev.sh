#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$ROOT_DIR/.env.dev.local"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing env file: $ENV_FILE"
  exit 1
fi

set -a
source "$ENV_FILE"
set +a

if [[ ! -d "$ROOT_DIR/frontend/node_modules" ]]; then
  (cd "$ROOT_DIR/frontend" && npm install)
fi

BACKEND_PID=""
if ! lsof -ti tcp:8899 >/dev/null 2>&1; then
  (cd "$ROOT_DIR" && cargo run --manifest-path backend/Cargo.toml) &
  BACKEND_PID="$!"
fi

wait_backend_ready() {
  local retries=120
  local i
  for ((i=1; i<=retries; i++)); do
    if curl -fsS "http://127.0.0.1:8899/api/v1/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "Backend did not become ready on http://127.0.0.1:8899 within ${retries}s"
  return 1
}

wait_backend_ready

(cd "$ROOT_DIR/frontend" && npm run dev) &
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
