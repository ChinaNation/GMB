#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QR_GEN="$SCRIPT_DIR/gen_cpms_qr_dev.py"

BASE_URL="${BASE_URL:-http://127.0.0.1:8899}"
CHAIN_TOKEN="${CHAIN_TOKEN:-chain-dev-token}"
ADMIN_PUBKEY="${ADMIN_PUBKEY:-DEMO_SUPER_ADMIN_01}"

SITE_SFID="${SITE_SFID:-SFID-SITE-001}"
CPMS_PUBKEY_1="${CPMS_PUBKEY_1:-CPMS-PUBKEY-1}"
CPMS_PUBKEY_2="${CPMS_PUBKEY_2:-CPMS-PUBKEY-2}"
CPMS_PUBKEY_3="${CPMS_PUBKEY_3:-CPMS-PUBKEY-3}"
CPMS_SIGN_PUBKEY="${CPMS_SIGN_PUBKEY:-$CPMS_PUBKEY_1}"

PUBKEY="${PUBKEY:-0xabc123pubkey}"
ARCHIVE_NO="${ARCHIVE_NO:-11001M112345678920000101}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 2
  }
}

need_cmd curl
need_cmd jq
need_cmd python3

json_post() {
  local url="$1"
  local data="$2"
  shift 2 || true
  curl -sS -X POST "$url" -H 'content-type: application/json' "$@" -d "$data"
}

json_get() {
  local url="$1"
  shift || true
  curl -sS "$url" "$@"
}

chain_post() {
  local url="$1"
  local data="$2"
  local rid="smoke-req-$(date +%s)-$RANDOM"
  local nonce="smoke-nonce-$(date +%s)-$RANDOM"
  local ts
  ts="$(date +%s)"
  json_post "$url" "$data" \
    -H "x-chain-token: $CHAIN_TOKEN" \
    -H "x-chain-request-id: $rid" \
    -H "x-chain-nonce: $nonce" \
    -H "x-chain-timestamp: $ts"
}

chain_get() {
  local url="$1"
  local rid="smoke-req-$(date +%s)-$RANDOM"
  local nonce="smoke-nonce-$(date +%s)-$RANDOM"
  local ts
  ts="$(date +%s)"
  json_get "$url" \
    -H "x-chain-token: $CHAIN_TOKEN" \
    -H "x-chain-request-id: $rid" \
    -H "x-chain-nonce: $nonce" \
    -H "x-chain-timestamp: $ts"
}

chain_post_with_headers() {
  local url="$1"
  local data="$2"
  local rid="$3"
  local nonce="$4"
  local ts="$5"
  json_post "$url" "$data" \
    -H "x-chain-token: $CHAIN_TOKEN" \
    -H "x-chain-request-id: $rid" \
    -H "x-chain-nonce: $nonce" \
    -H "x-chain-timestamp: $ts"
}

chain_get_with_headers() {
  local url="$1"
  local rid="$2"
  local nonce="$3"
  local ts="$4"
  json_get "$url" \
    -H "x-chain-token: $CHAIN_TOKEN" \
    -H "x-chain-request-id: $rid" \
    -H "x-chain-nonce: $nonce" \
    -H "x-chain-timestamp: $ts"
}

if [[ -z "${ADMIN_TOKEN:-}" ]]; then
  echo "ADMIN_TOKEN is required for real smoke testing. Please log in with a real admin wallet and export ADMIN_TOKEN." >&2
  exit 2
fi
echo "0) use provided ADMIN_TOKEN for real admin smoke testing"
admin_headers=(
  -H "authorization: Bearer $ADMIN_TOKEN"
)

echo
echo "1) register cpms site keys"
REGISTER_QR="$(python3 "$QR_GEN" register \
  --site-sfid "$SITE_SFID" \
  --pubkey-1 "$CPMS_PUBKEY_1" \
  --pubkey-2 "$CPMS_PUBKEY_2" \
  --pubkey-3 "$CPMS_PUBKEY_3" | head -n1)"
REGISTER_REQ="$(jq -cn --arg qr "$REGISTER_QR" '{qr_payload:$qr}')"
json_post "$BASE_URL/api/v1/admin/cpms-keys/register-scan" "$REGISTER_REQ" "${admin_headers[@]}" | jq .

echo
echo "2) chain create bind request"
BIND_REQ="$(jq -cn --arg p "$PUBKEY" '{account_pubkey:$p}')"
chain_post "$BASE_URL/api/v1/bind/request" "$BIND_REQ" | jq .

echo
echo "2.1) replay protection: duplicate nonce should fail"
RID_FIX="smoke-fixed-rid-$(date +%s)"
NONCE_FIX="smoke-fixed-nonce-$(date +%s)"
TS_FIX="$(date +%s)"
REQ_ONCE="$(chain_get_with_headers "$BASE_URL/api/v1/bind/result?account_pubkey=$PUBKEY" "$RID_FIX" "$NONCE_FIX" "$TS_FIX" | jq -r '.code' 2>/dev/null || true)"
REQ_DUP="$(chain_get_with_headers "$BASE_URL/api/v1/bind/result?account_pubkey=$PUBKEY" "smoke-fixed-rid-2-$(date +%s)" "$NONCE_FIX" "$TS_FIX")"
echo "$REQ_DUP" | jq .
DUP_CODE="$(echo "$REQ_DUP" | jq -r '.code')"
if [[ "$DUP_CODE" == "0" ]]; then
  echo "expected duplicate nonce request to fail" >&2
  exit 1
fi

echo
echo "2.2) replay protection: expired timestamp should fail"
TS_OLD="$(( $(date +%s) - 1000 ))"
REQ_OLD="$(chain_post_with_headers "$BASE_URL/api/v1/bind/request" "$BIND_REQ" "smoke-old-rid-$(date +%s)" "smoke-old-nonce-$(date +%s)" "$TS_OLD")"
echo "$REQ_OLD" | jq .
OLD_CODE="$(echo "$REQ_OLD" | jq -r '.code')"
if [[ "$OLD_CODE" == "0" ]]; then
  echo "expected old timestamp request to fail" >&2
  exit 1
fi

echo
echo "3) negative check: confirm without scanned qr_id should fail"
BAD_CONFIRM="$(jq -cn --arg p "$PUBKEY" --arg a "$ARCHIVE_NO" --arg q "not-scanned-qr-id" \
  '{account_pubkey:$p,archive_index:$a,qr_id:$q}')"
NEG_RESP="$(json_post "$BASE_URL/api/v1/admin/bind/confirm" "$BAD_CONFIRM" "${admin_headers[@]}")"
echo "$NEG_RESP" | jq .
NEG_CODE="$(echo "$NEG_RESP" | jq -r '.code')"
if [[ "$NEG_CODE" == "0" ]]; then
  echo "expected confirm without scanned qr_id to fail, but it succeeded" >&2
  exit 1
fi

echo
echo "4) admin scan citizen bind qr"
CITIZEN_QR="$(python3 "$QR_GEN" citizen \
  --site-sfid "$SITE_SFID" \
  --archive-no "$ARCHIVE_NO" \
  --status NORMAL \
  --sign-pubkey "$CPMS_SIGN_PUBKEY" | head -n1)"
SCAN_REQ="$(jq -cn --arg qr "$CITIZEN_QR" '{qr_payload:$qr}')"
SCAN_RESP="$(json_post "$BASE_URL/api/v1/admin/bind/scan" "$SCAN_REQ" "${admin_headers[@]}")"
echo "$SCAN_RESP" | jq .
QR_ID="$(echo "$SCAN_RESP" | jq -r '.data.qr_id')"

echo
echo "5) admin confirm bind (must carry scanned qr_id)"
CONFIRM_REQ="$(jq -cn --arg p "$PUBKEY" --arg a "$ARCHIVE_NO" --arg q "$QR_ID" \
  '{account_pubkey:$p,archive_index:$a,qr_id:$q}')"
json_post "$BASE_URL/api/v1/admin/bind/confirm" "$CONFIRM_REQ" "${admin_headers[@]}" | jq .

echo
echo "6) chain vote verify should be eligible (NORMAL)"
VERIFY_REQ="$(jq -cn --arg p "$PUBKEY" '{account_pubkey:$p,proposal_id:1}')"
VERIFY_NORMAL="$(chain_post "$BASE_URL/api/v1/vote/verify" "$VERIFY_REQ")"
echo "$VERIFY_NORMAL" | jq .
NORMAL_ELIGIBLE="$(echo "$VERIFY_NORMAL" | jq -r '.data.has_vote_eligibility')"
if [[ "$NORMAL_ELIGIBLE" != "true" ]]; then
  echo "expected NORMAL status voter to be eligible" >&2
  exit 1
fi

echo
echo "7) super admin scan status qr -> ABNORMAL"
STATUS_QR="$(python3 "$QR_GEN" status \
  --site-sfid "$SITE_SFID" \
  --archive-no "$ARCHIVE_NO" \
  --status ABNORMAL \
  --sign-pubkey "$CPMS_SIGN_PUBKEY" | head -n1)"
STATUS_REQ="$(jq -cn --arg qr "$STATUS_QR" '{qr_payload:$qr}')"
json_post "$BASE_URL/api/v1/admin/cpms-status/scan" "$STATUS_REQ" "${admin_headers[@]}" | jq .

echo
echo "8) chain vote verify should be ineligible (ABNORMAL)"
VERIFY_ABNORMAL="$(chain_post "$BASE_URL/api/v1/vote/verify" "$VERIFY_REQ")"
echo "$VERIFY_ABNORMAL" | jq .
ABNORMAL_ELIGIBLE="$(echo "$VERIFY_ABNORMAL" | jq -r '.data.has_vote_eligibility')"
if [[ "$ABNORMAL_ELIGIBLE" != "false" ]]; then
  echo "expected ABNORMAL status voter to be ineligible" >&2
  exit 1
fi

echo
echo "9) chain voters count should be returned"
VOTERS_COUNT="$(chain_get "$BASE_URL/api/v1/chain/voters/count?account_pubkey=$PUBKEY")"
echo "$VOTERS_COUNT" | jq .
ELIGIBLE_TOTAL="$(echo "$VOTERS_COUNT" | jq -r '.data.eligible_total')"
if [[ "$ELIGIBLE_TOTAL" == "null" ]]; then
  echo "expected eligible_total in chain voters count response" >&2
  exit 1
fi

echo
echo "10) reward state machine: query -> ack failed -> ack success"
REWARD_STATE_1="$(chain_get "$BASE_URL/api/v1/chain/reward/state?account_pubkey=$PUBKEY")"
echo "$REWARD_STATE_1" | jq .
CALLBACK_ID="$(echo "$REWARD_STATE_1" | jq -r '.data.callback_id')"
ACK_FAIL_REQ="$(jq -cn --arg p "$PUBKEY" --arg c "$CALLBACK_ID" '{account_pubkey:$p,callback_id:$c,status:"FAILED",error_message:"chain busy",retry_after_seconds:1}')"
ACK_FAIL_RESP="$(chain_post "$BASE_URL/api/v1/chain/reward/ack" "$ACK_FAIL_REQ")"
echo "$ACK_FAIL_RESP" | jq .
ACK_FAIL_STATUS="$(echo "$ACK_FAIL_RESP" | jq -r '.data.reward_status')"
if [[ "$ACK_FAIL_STATUS" != "RETRY_WAITING" && "$ACK_FAIL_STATUS" != "FAILED" ]]; then
  echo "expected reward status RETRY_WAITING or FAILED after failed ack" >&2
  exit 1
fi
ACK_OK_REQ="$(jq -cn --arg p "$PUBKEY" --arg c "$CALLBACK_ID" '{account_pubkey:$p,callback_id:$c,status:"SUCCESS",reward_tx_hash:"0xsmoketx"}')"
ACK_OK_RESP="$(chain_post "$BASE_URL/api/v1/chain/reward/ack" "$ACK_OK_REQ")"
echo "$ACK_OK_RESP" | jq .
ACK_OK_STATUS="$(echo "$ACK_OK_RESP" | jq -r '.data.reward_status')"
if [[ "$ACK_OK_STATUS" != "REWARDED" ]]; then
  echo "expected reward status REWARDED after success ack" >&2
  exit 1
fi

echo
echo "Done: business flow smoke passed."
