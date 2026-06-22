#!/usr/bin/env bash
# IM 双节点真实运行态 smoke：启动两个临时 headless 节点，验证 /gmb/im/1 直连投递。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
NODE_DIR="$REPO_ROOT/node"
NODE_BIN="$REPO_ROOT/target/debug/citizenchain"
WORK_DIR="${TMPDIR:-/tmp}/gmb-im-two-node-smoke-$$"

A_RPC=19944
B_RPC=19945
A_P2P=31333
B_P2P=31334

mkdir -p "$WORK_DIR"

cleanup() {
    local code=$?
    if [[ -n "${A_PID:-}" ]]; then kill "$A_PID" 2>/dev/null || true; fi
    if [[ -n "${B_PID:-}" ]]; then kill "$B_PID" 2>/dev/null || true; fi
    wait "${A_PID:-}" 2>/dev/null || true
    wait "${B_PID:-}" 2>/dev/null || true
    if [[ "${KEEP_IM_SMOKE_WORKDIR:-0}" != "1" ]]; then
        rm -rf "$WORK_DIR"
    else
        echo "保留 IM smoke 工作目录: $WORK_DIR"
    fi
    exit "$code"
}
trap cleanup EXIT INT TERM HUP

rpc() {
    local port="$1"
    local method="$2"
    local params_json="$3"
    python3 - "$port" "$method" "$params_json" <<'PY'
import json
import sys
import urllib.error
import urllib.request

port, method, params_json = sys.argv[1], sys.argv[2], sys.argv[3]
payload = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": method,
    "params": json.loads(params_json),
}
request = urllib.request.Request(
    f"http://127.0.0.1:{port}/",
    data=json.dumps(payload).encode("utf-8"),
    headers={"Content-Type": "application/json"},
    method="POST",
)
try:
    with urllib.request.urlopen(request, timeout=12) as response:
        data = json.load(response)
except urllib.error.URLError as exc:
    print(f"RPC {method} 调用失败: {exc}", file=sys.stderr)
    sys.exit(2)

if "error" in data:
    print(json.dumps(data["error"], ensure_ascii=False), file=sys.stderr)
    sys.exit(3)
print(json.dumps(data.get("result"), ensure_ascii=False))
PY
}

wait_rpc() {
    local port="$1"
    local label="$2"
    for _ in $(seq 1 90); do
        if rpc "$port" "system_health" "[]" >/dev/null 2>&1; then
            echo "$label RPC 已就绪: $port"
            return 0
        fi
        sleep 1
    done
    echo "$label RPC 等待超时，日志如下:" >&2
    cat "$WORK_DIR/$label.log" >&2 || true
    return 1
}

make_binding() {
    local wallet="$1"
    local device="$2"
    local peer_id="$3"
    local endpoint="$4"
    python3 - "$wallet" "$device" "$peer_id" "$endpoint" <<'PY'
import json
import sys

wallet, device, peer_id, endpoint = sys.argv[1:5]
print(json.dumps([{
    "wallet_account": wallet,
    "im_device_id": device,
    "im_device_pubkey": "0x" + device.encode().hex(),
    "node_peer_id": peer_id,
    "node_endpoints": [{
        "peer_id": peer_id,
        "multiaddr": endpoint,
        "kind": "ip4",
    }],
    "expires_at_millis": 4102444800000,
    "nonce": f"smoke-{device}",
    "wallet_signature": "0x736d6f6b655f736967",
}], ensure_ascii=False))
PY
}

make_keypackage_publish() {
    local wallet="$1"
    local device="$2"
    local key_package_id="$3"
    python3 - "$wallet" "$device" "$key_package_id" <<'PY'
import json
import sys

wallet, device, key_package_id = sys.argv[1:4]
print(json.dumps([{
    "owner_wallet_account": wallet,
    "device_id": device,
    "device_public_key_hex": "aabbccdd",
    "key_package_id": key_package_id,
    "key_package_hex": "b0b0cafe",
    "cipher_suite": "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519",
    "created_at_millis": 1800000000000,
    "expires_at_millis": 4102444800000,
}], ensure_ascii=False))
PY
}

make_direct_keypackage_fetch() {
    local remote_peer="$1"
    local remote_endpoint="$2"
    local owner="$3"
    local requester="$4"
    python3 - "$remote_peer" "$remote_endpoint" "$owner" "$requester" <<'PY'
import json
import sys

remote_peer, remote_endpoint, owner, requester = sys.argv[1:5]
print(json.dumps([{
    "remote_endpoint": {
        "peer_id": remote_peer,
        "multiaddr": remote_endpoint,
        "kind": "ip4",
    },
    "fetch": {
        "owner_wallet_account": owner,
        "requester_chat_account": requester,
        "limit": 1,
    },
}], ensure_ascii=False))
PY
}

make_direct_keypackage_consume() {
    local remote_peer="$1"
    local remote_endpoint="$2"
    local owner="$3"
    local requester="$4"
    local key_package_id="$5"
    python3 - "$remote_peer" "$remote_endpoint" "$owner" "$requester" "$key_package_id" <<'PY'
import json
import sys

remote_peer, remote_endpoint, owner, requester, key_package_id = sys.argv[1:6]
print(json.dumps([{
    "remote_endpoint": {
        "peer_id": remote_peer,
        "multiaddr": remote_endpoint,
        "kind": "ip4",
    },
    "consume": {
        "owner_wallet_account": owner,
        "key_package_id": key_package_id,
        "requester_chat_account": requester,
    },
}], ensure_ascii=False))
PY
}

make_direct_submit() {
    local remote_peer="$1"
    local remote_endpoint="$2"
    local mailbox_owner="$3"
    local envelope_id="$4"
    local recipient="$5"
    python3 - "$remote_peer" "$remote_endpoint" "$mailbox_owner" "$envelope_id" "$recipient" <<'PY'
import json
import sys

remote_peer, remote_endpoint, mailbox_owner, envelope_id, recipient = sys.argv[1:6]
print(json.dumps([{
    "remote_endpoint": {
        "peer_id": remote_peer,
        "multiaddr": remote_endpoint,
        "kind": "ip4",
    },
    "submit": {
        "mailbox_owner_chat_account": mailbox_owner,
        "envelope": {
            "protocol_version": 1,
            "envelope_id": envelope_id,
            "conversation_id": "conv-smoke-a-b",
            "sender_chat_account": "alice-wallet",
            "recipient_chat_account": recipient,
            "sender_device_id": "alice-phone",
            "encrypted_payload_hex": "aabbccddeeff",
            "created_at_millis": 1800000000000,
            "ttl_millis": 600000,
        },
    },
}], ensure_ascii=False))
PY
}

json_get_string() {
    python3 - "$1" <<'PY'
import json
import sys
print(json.loads(sys.argv[1]))
PY
}

assert_python() {
    local value_json="$1"
    local code="$2"
    python3 - "$value_json" "$code" <<'PY'
import json
import sys

value = json.loads(sys.argv[1])
code = sys.argv[2]
namespace = {"value": value}
exec(code, {}, namespace)
PY
}

echo "==> 构建 citizenchain 节点二进制"
cd "$NODE_DIR"
cargo build -p node >/dev/null

echo "==> 启动 A/B 两个临时 headless 节点"
GMB_IM_OWNER_RPC=1 CITIZENCHAIN_HEADLESS=1 "$NODE_BIN" \
    --base-path "$WORK_DIR/a" \
    --chain citizenchain \
    --listen-addr "/ip4/127.0.0.1/tcp/$A_P2P/wss" \
    --rpc-port "$A_RPC" \
    --rpc-methods Unsafe \
    --rpc-cors all \
    --no-prometheus \
    --mining-threads 0 \
    --name gmb-im-smoke-a \
    >"$WORK_DIR/A.log" 2>&1 &
A_PID=$!

start_b() {
    GMB_IM_OWNER_RPC=1 CITIZENCHAIN_HEADLESS=1 "$NODE_BIN" \
        --base-path "$WORK_DIR/b" \
        --chain citizenchain \
        --listen-addr "/ip4/127.0.0.1/tcp/$B_P2P/wss" \
        --rpc-port "$B_RPC" \
        --rpc-methods Unsafe \
        --rpc-cors all \
        --no-prometheus \
        --mining-threads 0 \
        --name gmb-im-smoke-b \
        >"$WORK_DIR/B.log" 2>&1 &
    B_PID=$!
}

stop_b() {
    if [[ -n "${B_PID:-}" ]]; then
        kill "$B_PID" 2>/dev/null || true
        wait "$B_PID" 2>/dev/null || true
        B_PID=""
    fi
}

start_b

wait_rpc "$A_RPC" "A"
wait_rpc "$B_RPC" "B"

A_PEER="$(json_get_string "$(rpc "$A_RPC" "system_localPeerId" "[]")")"
B_PEER="$(json_get_string "$(rpc "$B_RPC" "system_localPeerId" "[]")")"
A_ENDPOINT="/ip4/127.0.0.1/tcp/$A_P2P/wss/p2p/$A_PEER"
B_ENDPOINT="/ip4/127.0.0.1/tcp/$B_P2P/wss/p2p/$B_PEER"

echo "==> A PeerId: $A_PEER"
echo "==> B PeerId: $B_PEER"

echo "==> 登记 A/B 已授权手机设备"
rpc "$A_RPC" "im_registerOwnerDevice" "$(make_binding "alice-wallet" "alice-phone" "$A_PEER" "$A_ENDPOINT")" >/dev/null
rpc "$B_RPC" "im_registerOwnerDevice" "$(make_binding "bob-wallet" "bob-phone" "$B_PEER" "$B_ENDPOINT")" >/dev/null

echo "==> B 已授权手机发布 OpenMLS KeyPackage"
PUBLISHED_KP_JSON="$(rpc "$B_RPC" "im_publishKeyPackage" "$(make_keypackage_publish "bob-wallet" "bob-phone" "kp-smoke-bob-1")")"
assert_python "$PUBLISHED_KP_JSON" 'assert value["key_package_id"] == "kp-smoke-bob-1", value; assert value["key_package_hex"] == "b0b0cafe", value'

echo "==> 重启 B 节点，验证 KeyPackage 池落盘"
stop_b
start_b
wait_rpc "$B_RPC" "B"

echo "==> A 通过 /gmb/im/1 从 B 私人节点拉取 KeyPackage"
FETCH_KP_JSON="$(rpc "$A_RPC" "im_fetchDirectKeyPackages" "$(make_direct_keypackage_fetch "$B_PEER" "$B_ENDPOINT" "bob-wallet" "alice-wallet")")"
assert_python "$FETCH_KP_JSON" 'assert value["kind"] == "KeyPackages", value; assert len(value["body"]) == 1, value; assert value["body"][0]["key_package_id"] == "kp-smoke-bob-1", value'

echo "==> A 通过 /gmb/im/1 声明消费 B 的一次性 KeyPackage"
CONSUMED_KP_JSON="$(rpc "$A_RPC" "im_consumeDirectKeyPackage" "$(make_direct_keypackage_consume "$B_PEER" "$B_ENDPOINT" "bob-wallet" "alice-wallet" "kp-smoke-bob-1")")"
assert_python "$CONSUMED_KP_JSON" 'assert value["kind"] == "KeyPackageConsumed", value; assert value["body"]["key_package_id"] == "kp-smoke-bob-1", value; assert value["body"]["consumed_at_millis"] is not None, value'

echo "==> 已消费 KeyPackage 不再对外返回"
EMPTY_KP_JSON="$(rpc "$A_RPC" "im_fetchDirectKeyPackages" "$(make_direct_keypackage_fetch "$B_PEER" "$B_ENDPOINT" "bob-wallet" "alice-wallet")")"
assert_python "$EMPTY_KP_JSON" 'assert value["kind"] == "KeyPackages", value; assert value["body"] == [], value'

echo "==> A 通过 /gmb/im/1 向 B 私人 mailbox 投递密文信封"
ACK_JSON="$(rpc "$A_RPC" "im_submitDirectEnvelope" "$(make_direct_submit "$B_PEER" "$B_ENDPOINT" "bob-wallet" "env-smoke-1" "bob-wallet")")"
assert_python "$ACK_JSON" 'assert value["kind"] == "EnvelopeAck", value; assert value["body"]["envelope_id"] == "env-smoke-1", value'

echo "==> 重启 B 节点，验证 pending 密文落盘不丢"
stop_b
start_b
wait_rpc "$B_RPC" "B"

echo "==> B 已授权手机设备拉取待收密文"
PENDING_JSON="$(rpc "$B_RPC" "im_fetchPending" '["bob-wallet","bob-phone"]')"
assert_python "$PENDING_JSON" 'assert len(value) == 1, value; assert value[0]["envelope_id"] == "env-smoke-1", value; assert value[0]["encrypted_payload_hex"] == "aabbccddeeff", value'

echo "==> B 已授权手机设备 ack 信封"
ACKED_JSON="$(rpc "$B_RPC" "im_ackEnvelope" '["bob-wallet","bob-phone","env-smoke-1"]')"
assert_python "$ACKED_JSON" 'assert value["envelope_id"] == "env-smoke-1", value; assert value["state"] == "AcknowledgedByOwner", value'

echo "==> 再次重启 B 节点，验证 ack 状态落盘"
stop_b
start_b
wait_rpc "$B_RPC" "B"

EMPTY_JSON="$(rpc "$B_RPC" "im_fetchPending" '["bob-wallet","bob-phone"]')"
assert_python "$EMPTY_JSON" 'assert value == [], value'

echo "==> 验证 B 拒绝第三方 mailbox 投递"
REJECT_JSON="$(rpc "$A_RPC" "im_submitDirectEnvelope" "$(make_direct_submit "$B_PEER" "$B_ENDPOINT" "carol-wallet" "env-smoke-third-party" "carol-wallet")")"
assert_python "$REJECT_JSON" 'assert value["kind"] == "Error", value; assert "尚未授权该钱包账户" in value["body"], value'

echo "==> 验证已 ack 的 envelope 重复投递不会重新入队"
DUP_JSON="$(rpc "$A_RPC" "im_submitDirectEnvelope" "$(make_direct_submit "$B_PEER" "$B_ENDPOINT" "bob-wallet" "env-smoke-1" "bob-wallet")")"
assert_python "$DUP_JSON" 'assert value["kind"] == "EnvelopeAck", value; assert value["body"]["state"] == "AcknowledgedByOwner", value'
STILL_EMPTY_JSON="$(rpc "$B_RPC" "im_fetchPending" '["bob-wallet","bob-phone"]')"
assert_python "$STILL_EMPTY_JSON" 'assert value == [], value'

echo "==> IM 双节点真实运行态 smoke 通过"
