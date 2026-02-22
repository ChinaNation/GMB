#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8899}"
PUBKEY="${PUBKEY:-0xabc123pubkey}"
ARCHIVE_INDEX="${ARCHIVE_INDEX:-CIV-TEST-0001}"

echo "1) bind request"
curl -sS -X POST "$BASE_URL/api/v1/bind/request" \
  -H 'content-type: application/json' \
  -d "{\"account_pubkey\":\"$PUBKEY\"}"

echo "\n2) admin query by pubkey"
curl -sS "$BASE_URL/api/v1/admin/bind/query?account_pubkey=$PUBKEY" \
  -H 'x-admin-user: admin' \
  -H 'x-admin-password: admin123'

echo "\n3) admin confirm bind"
curl -sS -X POST "$BASE_URL/api/v1/admin/bind/confirm" \
  -H 'content-type: application/json' \
  -H 'x-admin-user: admin' \
  -H 'x-admin-password: admin123' \
  -d "{\"account_pubkey\":\"$PUBKEY\",\"archive_index\":\"$ARCHIVE_INDEX\"}"

echo "\n4) wuminapp query bind result"
curl -sS "$BASE_URL/api/v1/bind/result?account_pubkey=$PUBKEY"

echo "\n5) wuminapp vote verify"
curl -sS -X POST "$BASE_URL/api/v1/vote/verify" \
  -H 'content-type: application/json' \
  -d "{\"account_pubkey\":\"$PUBKEY\"}"

echo "\nDone"
