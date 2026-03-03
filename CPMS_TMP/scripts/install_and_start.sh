#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${ROOT_DIR}/deploy/docker-compose.yml"

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker not found. Please install Docker Engine/Desktop first."
  exit 1
fi

if docker compose version >/dev/null 2>&1; then
  COMPOSE_CMD=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE_CMD=(docker-compose)
else
  echo "ERROR: docker compose not found."
  exit 1
fi

echo "[CPMS] Starting postgres + backend ..."
"${COMPOSE_CMD[@]}" -f "${COMPOSE_FILE}" up -d --build

echo
echo "[CPMS] Services are up."
echo "[CPMS] Open login page on LAN: http://<host-lan-ip>:${CPMS_HTTP_PORT:-8080}/login"
echo "[CPMS] Check status: ${COMPOSE_CMD[*]} -f ${COMPOSE_FILE} ps"
