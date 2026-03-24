#!/usr/bin/env bash
# 从 citizenchain 导出 chainspec JSON 文件供 smoldot 轻节点使用。
#
# 用法：
#   ./scripts/generate-chainspec.sh                          # 从本机运行中的节点导出（自动修正 bootnode）
#   ./scripts/generate-chainspec.sh build-spec [dev|mainnet] # 用 citizenchain 二进制导出
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WUMINAPP_DIR="$(dirname "$SCRIPT_DIR")"
GMB_DIR="$(dirname "$WUMINAPP_DIR")"
OUTPUT="$WUMINAPP_DIR/assets/chainspec.json"

MODE="${1:-auto}"

case "$MODE" in
  auto)
    # 自动模式：用 build-spec 生成 chainspec，然后用运行中节点的真实 Peer ID 和 IP 修正 bootnode。
    NODE_RPC="${2:-http://127.0.0.1:9944}"
    echo "=== 自动生成 chainspec ==="

    # 1. 找到节点二进制并导出基础 chainspec
    NODE_PID=$(lsof -ti :9944 -s TCP:LISTEN 2>/dev/null | head -1 || true)
    if [ -z "$NODE_PID" ]; then
      echo "错误: 未找到监听 9944 端口的节点进程"
      exit 1
    fi
    # 节点二进制路径可能含空格（macOS Application Support），用 --chain 参数位置截取
    FULL_CMD=$(ps -p "$NODE_PID" -o command=)
    NODE_BIN=$(echo "$FULL_CMD" | sed 's/ --chain .*//')
    echo "节点二进制: $NODE_BIN"

    # 获取节点的 --chain 参数
    CHAIN_ARG=$(echo "$FULL_CMD" | grep -o '\-\-chain [^ ]*' | awk '{print $2}' || echo "dev")
    echo "链类型: $CHAIN_ARG"
    "$NODE_BIN" build-spec --chain="$CHAIN_ARG" --raw 2>/dev/null > "$OUTPUT"

    # 2. 获取运行中节点的真实 Peer ID
    REAL_PEER_ID=$(curl -s -H "Content-Type: application/json" \
      -d '{"id":1,"jsonrpc":"2.0","method":"system_localPeerId","params":[]}' \
      "$NODE_RPC" | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])")
    echo "节点 Peer ID: $REAL_PEER_ID"

    # 3. 获取本机 WiFi IP（en0）
    LOCAL_IP=$(ifconfig en0 2>/dev/null | grep "inet " | awk '{print $2}' || true)
    if [ -z "$LOCAL_IP" ]; then
      LOCAL_IP="127.0.0.1"
      echo "警告: 未检测到 en0 IP，使用 127.0.0.1（仅本机可用）"
    fi
    echo "本机 IP: $LOCAL_IP"

    # 4. 获取 lightSyncState（smoldot 用来跳过历史区块验证）
    echo "获取 lightSyncState..."
    LIGHT_SYNC_STATE=$(curl -s -H "Content-Type: application/json" \
      -d '{"id":1,"jsonrpc":"2.0","method":"sync_state_genSyncSpec","params":[true]}' \
      "$NODE_RPC" | python3 -c "import sys,json; r=json.load(sys.stdin).get('result',{}); print(json.dumps(r.get('lightSyncState')) if 'lightSyncState' in r else '')" 2>/dev/null || true)

    # 5. 替换 chainspec 中的 bootnode，并注入 lightSyncState
    python3 -c "
import json
with open('$OUTPUT') as f:
    spec = json.load(f)
spec['bootNodes'] = ['/ip4/$LOCAL_IP/tcp/30333/ws/p2p/$REAL_PEER_ID']
light_sync = '$LIGHT_SYNC_STATE'
if light_sync:
    try:
        spec['lightSyncState'] = json.loads(light_sync)
        print('lightSyncState: 已注入（smoldot 将跳过历史区块验证）')
    except json.JSONDecodeError:
        print('lightSyncState: 解析失败，跳过')
else:
    print('lightSyncState: 节点未返回，跳过（首次同步将从创世块开始）')
print('bootNodes:', spec['bootNodes'])
with open('$OUTPUT', 'w') as f:
    json.dump(spec, f, indent=2)
"
    ;;

  build-spec)
    # 用 citizenchain 二进制的 build-spec 命令导出（不修正 bootnode）
    CHAIN_BINARY="$GMB_DIR/citizenchain/target/release/citizenchain"
    if [ ! -f "$CHAIN_BINARY" ]; then
      echo "错误：citizenchain 二进制不存在: $CHAIN_BINARY"
      echo "请先编译: cd $GMB_DIR/citizenchain && cargo build --release"
      exit 1
    fi

    CHAIN="${2:-dev}"
    echo "=== 用 build-spec 导出 $CHAIN chainspec ==="
    "$CHAIN_BINARY" build-spec --chain="$CHAIN" --raw > "$OUTPUT"
    ;;

  *)
    echo "用法:"
    echo "  $0                          # 自动导出并修正 bootnode（推荐）"
    echo "  $0 build-spec [dev|mainnet] # 仅用二进制导出（不修正 bootnode）"
    exit 1
    ;;
esac

echo ""
echo "chainspec 已导出到: $OUTPUT"
echo "文件大小: $(wc -c < "$OUTPUT" | tr -d ' ') bytes"
