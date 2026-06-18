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

SFID_BIND_ADDR="${SFID_BIND_ADDR:-127.0.0.1:8899}"
SFID_HEALTHCHECK_URL="${SFID_HEALTHCHECK_URL:-http://${SFID_BIND_ADDR}/api/v1/health}"
SFID_PORT="${SFID_BIND_ADDR##*:}"
SFID_FRONTEND_PORT="${SFID_FRONTEND_PORT:-5179}"
SFID_LAUNCHD_LABEL="${SFID_LAUNCHD_LABEL:-com.gmb.sfid-backend}"

BACKEND_PID=""
FRONTEND_PID=""

port_listener_pids() {
  local port="$1"
  lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true
}

stop_port_listeners() {
  local port="$1"
  local label="$2"
  local pids
  pids="$(port_listener_pids "$port")"
  if [[ -z "$pids" ]]; then
    return 0
  fi

  echo "Stopping existing ${label} on tcp:${port}..."
  while IFS= read -r pid; do
    [[ -z "$pid" ]] && continue
    kill "$pid" >/dev/null 2>&1 || true
  done <<< "$pids"

  # 中文注释:macOS 上 node/cargo 子进程释放监听端口会慢半拍,这里等待端口真正空闲。
  for _ in {1..30}; do
    if [[ -z "$(port_listener_pids "$port")" ]]; then
      return 0
    fi
    sleep 0.2
  done

  pids="$(port_listener_pids "$port")"
  if [[ -n "$pids" ]]; then
    echo "Force stopping ${label} on tcp:${port}..."
    while IFS= read -r pid; do
      [[ -z "$pid" ]] && continue
      kill -9 "$pid" >/dev/null 2>&1 || true
    done <<< "$pids"
  fi

  for _ in {1..20}; do
    if [[ -z "$(port_listener_pids "$port")" ]]; then
      return 0
    fi
    sleep 0.2
  done

  echo "Port tcp:${port} is still occupied; aborting."
  return 1
}

stop_launchd_backend() {
  if ! command -v launchctl >/dev/null 2>&1; then
    return 0
  fi

  local domain="gui/$(id -u)/${SFID_LAUNCHD_LABEL}"
  if launchctl print "$domain" >/dev/null 2>&1; then
    echo "Stopping launchd service ${SFID_LAUNCHD_LABEL}..."
    # 中文注释:本机曾通过 launchctl submit 启动同端口后端,只 kill PID 会被 launchd 立刻拉起。
    launchctl bootout "$domain" >/dev/null 2>&1 \
      || launchctl remove "$SFID_LAUNCHD_LABEL" >/dev/null 2>&1 \
      || true
    sleep 1
  fi
}

check_database_ready() {
  if ! command -v pg_isready >/dev/null 2>&1; then
    echo "Missing pg_isready; install PostgreSQL client tools before starting SFID."
    return 1
  fi

  if pg_isready -d "$DATABASE_URL" >/dev/null 2>&1; then
    return 0
  fi

  local pgdata="${SFID_POSTGRES_DATA_DIR:-/opt/homebrew/var/postgresql@17}"
  local pid_file="$pgdata/postmaster.pid"
  if [[ -f "$pid_file" ]]; then
    local pid
    pid="$(head -1 "$pid_file" | tr -d '[:space:]')"
    if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      local args
      args="$(ps -p "$pid" -o args= 2>/dev/null || true)"
      if [[ "$args" != *postgres* && "$args" != *postmaster* ]]; then
        echo "PostgreSQL is not ready, and $pid_file is stale."
        echo "The PID in that file is $pid, but the running process is:"
        echo "$args"
        echo "Move the stale pid file away, then restart PostgreSQL:"
        echo "  mv \"$pid_file\" \"$pid_file.stale.\$(date +%Y%m%d%H%M%S)\""
        echo "  brew services restart postgresql@17"
        return 1
      fi
    fi
  fi

  echo "PostgreSQL is not ready for DATABASE_URL."
  echo "DATABASE_URL=${DATABASE_URL}"
  echo "Check it with:"
  echo "  pg_isready -d \"\$DATABASE_URL\""
  echo "  brew services list | grep postgresql"
  return 1
}

launch_backend() {
  local parent_pid="$$"
  (
    cd "$ROOT_DIR"
    local child_pid=""
    cleanup_child() {
      if [[ -n "$child_pid" ]]; then
        kill "$child_pid" >/dev/null 2>&1 || true
        wait "$child_pid" >/dev/null 2>&1 || true
      fi
      stop_port_listeners "$SFID_PORT" "backend" >/dev/null || true
    }
    trap '' HUP
    trap cleanup_child EXIT INT TERM
    "$ROOT_DIR/backend/target/debug/sfid-backend" &
    child_pid="$!"
    # 中文注释:后台 job 收不到前台 Ctrl-C 时,主动监控父脚本是否还活着。
    while kill -0 "$parent_pid" >/dev/null 2>&1; do
      if ! kill -0 "$child_pid" >/dev/null 2>&1; then
        wait "$child_pid"
        exit $?
      fi
      sleep 0.5
    done
  ) &
  BACKEND_PID="$!"
}

launch_frontend() {
  local parent_pid="$$"
  (
    cd "$ROOT_DIR/frontend"
    local child_pid=""
    cleanup_child() {
      if [[ -n "$child_pid" ]]; then
        kill "$child_pid" >/dev/null 2>&1 || true
        wait "$child_pid" >/dev/null 2>&1 || true
      fi
      stop_port_listeners "$SFID_FRONTEND_PORT" "frontend" >/dev/null || true
    }
    trap '' HUP
    trap cleanup_child EXIT INT TERM
    npm run dev &
    child_pid="$!"
    # 中文注释:npm/vite 也走父进程监控,避免 Ctrl-C 后留下 5179 遗留监听。
    while kill -0 "$parent_pid" >/dev/null 2>&1; do
      if ! kill -0 "$child_pid" >/dev/null 2>&1; then
        wait "$child_pid"
        exit $?
      fi
      sleep 0.5
    done
  ) &
  FRONTEND_PID="$!"
}

cleanup() {
  if [[ -n "$FRONTEND_PID" ]]; then
    kill "$FRONTEND_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$BACKEND_PID" ]]; then
    kill "$BACKEND_PID" >/dev/null 2>&1 || true
  fi
  # 中文注释:cargo/npm 退出时可能把真实监听进程短暂孤儿化,按端口多轮兜底清理。
  for _ in {1..3}; do
    stop_port_listeners "$SFID_FRONTEND_PORT" "frontend" >/dev/null || true
    stop_port_listeners "$SFID_PORT" "backend" >/dev/null || true
    sleep 0.2
  done
}

on_exit() {
  cleanup
}

on_signal() {
  cleanup
  trap - EXIT INT TERM HUP
  exit 130
}

trap on_exit EXIT
trap on_signal INT TERM HUP

if [[ ! -d "$ROOT_DIR/frontend/node_modules" ]]; then
  (cd "$ROOT_DIR/frontend" && npm install)
fi

stop_launchd_backend
stop_port_listeners "$SFID_FRONTEND_PORT" "frontend"
stop_port_listeners "$SFID_PORT" "backend"
check_database_ready

# 中文注释:不要用 `cargo run` 常驻服务,否则 Ctrl-C 后真实后端二进制可能被孤儿化。
(cd "$ROOT_DIR" && cargo build --manifest-path backend/Cargo.toml)
# 中文注释:本地启动前做公权机构目录守门;目录缺失、残缺或 china hash 变化时才对账。
"$ROOT_DIR/backend/target/debug/sfid-backend" ensure-gov
launch_backend

wait_backend_ready() {
  local retries=120
  local i
  for ((i=1; i<=retries; i++)); do
    if ! kill -0 "$BACKEND_PID" >/dev/null 2>&1; then
      wait "$BACKEND_PID" || true
      echo "Backend process exited before health check became ready."
      return 1
    fi
    if curl -fsS "$SFID_HEALTHCHECK_URL" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "Backend did not become ready on ${SFID_HEALTHCHECK_URL} within ${retries}s"
  return 1
}

wait_backend_ready

launch_frontend
wait
