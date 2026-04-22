#!/usr/bin/env bash
# 一键部署公民链节点到服务器（用 deb 包安装，和桌面端完全同一个程序）。
#
# 用法：
#   ./scripts/fuwuqi.sh <q|b> <server_ip> [ssh_user]
#      q = 清链部署（清除旧数据，全新创世）
#      b = 不清链部署（只替换程序，保留链数据）
#   例：./scripts/fuwuqi.sh q 147.224.14.117 ubuntu
set -euo pipefail

# ╔══════════════════════════════════════════════════════════════╗
# ║                     【密钥配置（可选）】                       ║
# ╚══════════════════════════════════════════════════════════════╝
# 用法：引导节点不带0x；

NODE_KEY="83e5af5b66ace1501e7bc2379a76873382883dd37ccdda791578ae50f8c72587"
GRANDPA_KEY=""
MINER_REWARD_ADDRESS="w5D8NC99pbhhvq1znhu63XSUnjukm5ozqXnqg6jxP5Ged9ZiP"

# ══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

DEPLOY_MODE="${1:?用法: $0 <q|b> <server_ip> [ssh_user]}"
SERVER_IP="${2:?用法: $0 <q|b> <server_ip> [ssh_user]}"
SSH_USER="${3:-ubuntu}"

if [[ "$DEPLOY_MODE" != "q" && "$DEPLOY_MODE" != "b" ]]; then
    echo "错误：第一个参数必须是 q（清链）或 b（不清链）"
    exit 1
fi

SSH_TARGET="$SSH_USER@$SERVER_IP"
REMOTE_DATA="/opt/citizenchain/data"
REMOTE_KEYSTORE="$REMOTE_DATA/chains/citizenchain/keystore"
SERVICE_NAME="citizenchain-node"
SSH_KEY="$HOME/.ssh/ed25519"
# 中文注释:
# 用 ControlMaster 复用同一条 TCP 连接跑完所有步骤,避免链路抖动导致
# 每步独立 SSH 连接时随机超时(实测中间链路丢包 30-50% 会让 scp/deb 上传失败)。
# ControlPersist=15m 让 master 在脚本跑完后再保留一段时间,期间多跑一次也快。
SSH_CTRL_DIR="$HOME/.ssh/sockets"
mkdir -p "$SSH_CTRL_DIR"
chmod 700 "$SSH_CTRL_DIR"
SSH_OPTS="-i $SSH_KEY -o StrictHostKeyChecking=accept-new \
  -o ControlMaster=auto \
  -o ControlPath=$SSH_CTRL_DIR/%r@%h-%p \
  -o ControlPersist=15m \
  -o ConnectTimeout=30 \
  -o ServerAliveInterval=10 \
  -o ServerAliveCountMax=6"

# ─── 校验密钥格式 ───

validate_hex_key() {
  local name="$1" value="$2"
  # 去掉可选的 0x 前缀后校验
  local stripped="${value#0x}"
  if [ -n "$stripped" ] && ! echo "$stripped" | grep -qE '^[0-9a-fA-F]{64}$'; then
    echo "错误：$name 格式无效，应为 64 位十六进制字符串（可带 0x 前缀）"
    exit 1
  fi
}
validate_hex_key "NODE_KEY" "$NODE_KEY"
validate_hex_key "GRANDPA_KEY" "$GRANDPA_KEY"

echo "=== 部署公民链到 $SSH_TARGET ==="

# ─── 1. 下载最新 deb 包（和桌面端完全同一个程序）───

echo ""
echo ">>> 下载最新安装包..."
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

gh run download --name "公民链-linux" --dir "$TMPDIR" -R ChinaNation/GMB 2>/dev/null || {
  echo "错误：下载失败。请确认 gh auth login 且 Linux CI 已成功。"
  exit 1
}

DEB_FILE=$(find "$TMPDIR" -name "*.deb" | head -1)
if [ -z "$DEB_FILE" ]; then
  echo "错误：未找到 deb 安装包"
  ls -la "$TMPDIR"
  exit 1
fi
echo "已下载：$(du -h "$DEB_FILE" | cut -f1)"

# ─── 2. 推导 GRANDPA 公钥 ───

GRANDPA_PUBKEY=""
if [ -n "$GRANDPA_KEY" ]; then
  echo ""
  echo ">>> 推导 GRANDPA 公钥..."
  GRANDPA_KEY_STRIPPED="${GRANDPA_KEY#0x}"
  GRANDPA_PUBKEY=$(python3 -c "
from nacl.signing import SigningKey
sk = SigningKey(bytes.fromhex('$GRANDPA_KEY_STRIPPED'))
print(sk.verify_key.encode().hex())
")
  echo "GRANDPA 公钥: $GRANDPA_PUBKEY"
fi

# ─── 3. 首次初始化 ───

NEED_SETUP=false
ssh $SSH_OPTS "$SSH_TARGET" "test -f /etc/systemd/system/$SERVICE_NAME.service" 2>/dev/null || NEED_SETUP=true

if [ "$NEED_SETUP" = true ]; then
  echo ""
  echo ">>> 首次部署，初始化服务器..."
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin citizenchain 2>/dev/null || true
    sudo mkdir -p $REMOTE_DATA
    sudo chown -R citizenchain:citizenchain /opt/citizenchain
    sudo iptables -C INPUT -p tcp --dport 30333 -j ACCEPT 2>/dev/null || \
    sudo iptables -I INPUT -p tcp --dport 30333 -j ACCEPT
    sudo sh -c 'iptables-save > /etc/iptables/rules.v4' 2>/dev/null || true
  "
  echo "初始化完成"
fi

# ─── 4. 同步 systemd 服务文件 ───

echo ""
echo ">>> 同步 systemd 服务文件..."
scp $SSH_OPTS "$SCRIPT_DIR/citizenchain-node.service" "$SSH_TARGET:/tmp/$SERVICE_NAME.service"
ssh $SSH_OPTS "$SSH_TARGET" "
  sudo mv /tmp/$SERVICE_NAME.service /etc/systemd/system/
  sudo systemctl daemon-reload
  sudo systemctl enable $SERVICE_NAME
"

# ─── 5. 停止服务 + 清链（如果 q 模式）───

echo ""
echo ">>> 停止节点..."
ssh -o ConnectTimeout=10 -o ServerAliveInterval=5 $SSH_OPTS "$SSH_TARGET" "sudo systemctl stop $SERVICE_NAME 2>/dev/null || true"

if [[ "$DEPLOY_MODE" == "q" ]]; then
  echo ">>> 清除旧链数据..."
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo rm -rf $REMOTE_DATA/chains
    sudo rm -rf $REMOTE_DATA/tls
  "
  echo "    已清除"
else
  echo ">>> 跳过清链（保留链数据）"
fi

# ─── 6. 上传并安装 deb 包 ───

echo ""
echo ">>> 上传并安装 deb 包..."
scp $SSH_OPTS "$DEB_FILE" "$SSH_TARGET:/tmp/citizenchain.deb"
ssh $SSH_OPTS "$SSH_TARGET" "
  sudo dpkg -i /tmp/citizenchain.deb || sudo apt-get install -f -y
  rm -f /tmp/citizenchain.deb
"

# ─── 7. 写入密钥 ───

echo ""
echo ">>> 配置密钥..."

if [ -n "$NODE_KEY" ]; then
  NODE_KEY_STRIPPED="${NODE_KEY#0x}"
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo mkdir -p $REMOTE_DATA/node-key
    printf '%s' '$NODE_KEY_STRIPPED' | sudo tee $REMOTE_DATA/node-key/secret_ed25519 > /dev/null
    sudo chmod 600 $REMOTE_DATA/node-key/secret_ed25519
    sudo chown citizenchain:citizenchain $REMOTE_DATA/node-key/secret_ed25519
  "
  echo "  P2P 身份密钥 -> 已写入"
else
  echo "  P2P 身份密钥 -> 跳过（未配置）"
fi

if [ -n "$GRANDPA_KEY" ] && [ -n "$GRANDPA_PUBKEY" ]; then
  GRAN_FILENAME="6772616e${GRANDPA_PUBKEY}"
  GRAN_CONTENT="\"${GRANDPA_KEY}\""
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo mkdir -p $REMOTE_KEYSTORE
    echo -n '$GRAN_CONTENT' | sudo tee $REMOTE_KEYSTORE/$GRAN_FILENAME > /dev/null
    sudo chmod 600 $REMOTE_KEYSTORE/$GRAN_FILENAME
    sudo chown -R citizenchain:citizenchain $REMOTE_DATA
  "
  echo "  GRANDPA 投票密钥 -> 已写入"
else
  echo "  GRANDPA 投票密钥 -> 跳过（未配置）"
fi

# ─── 8. 启动服务 ───

echo ""
echo ">>> 启动节点..."
ssh $SSH_OPTS "$SSH_TARGET" "sudo systemctl start $SERVICE_NAME"
sleep 5

if ssh $SSH_OPTS "$SSH_TARGET" "sudo systemctl is-active --quiet $SERVICE_NAME"; then
  echo "节点运行中"
else
  echo "节点启动失败："
  ssh $SSH_OPTS "$SSH_TARGET" "sudo journalctl -u $SERVICE_NAME --no-pager -n 20"
  exit 1
fi

# ─── 9. 绑定矿工收款地址 ───

if [ -n "$MINER_REWARD_ADDRESS" ]; then
  echo ""
  echo ">>> 绑定矿工收款地址: $MINER_REWARD_ADDRESS"
  ssh $SSH_OPTS "$SSH_TARGET" "
    curl -s -H 'Content-Type: application/json' \
      -d '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"reward_bindWallet\",\"params\":[\"$MINER_REWARD_ADDRESS\"]}' \
      http://127.0.0.1:9944
  " 2>/dev/null || true
fi

# ─── 10. 显示节点信息 ───

PEER_ID=$(ssh $SSH_OPTS "$SSH_TARGET" "
  curl -s -H 'Content-Type: application/json' \
    -d '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_localPeerId\",\"params\":[]}' \
    http://127.0.0.1:9944 2>/dev/null | python3 -c 'import sys,json; print(json.load(sys.stdin)[\"result\"])' 2>/dev/null
" || echo "")

echo ""
echo "══════════════════════════════════════════════════════"
echo "  部署完成"
echo "──────────────────────────────────────────────────────"
if [ -n "$PEER_ID" ]; then
  echo "  Peer ID:  $PEER_ID"
  echo "  bootNode: /ip4/$SERVER_IP/tcp/30333/wss/p2p/$PEER_ID"
fi
echo "══════════════════════════════════════════════════════"
