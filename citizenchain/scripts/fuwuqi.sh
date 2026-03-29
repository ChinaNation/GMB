#!/usr/bin/env bash
# 一键部署 citizenchain 节点到服务器。
# 密钥配置为可选项，填写了哪项就设置哪项，未填写的保持节点默认状态。
#
# 用法：
#   1. 按需填写下方【密钥配置】
#   2. 执行：./scripts/fuwuqi.sh <q|b> <server_ip> [ssh_user]
#      q = 清链部署（清除旧数据，全新创世）
#      b = 不清链部署（只替换二进制，保留链数据）
#   例：./scripts/fuwuqi.sh b 147.224.14.117 ubuntu
#
# 前置条件：
#   - 本地已安装 gh CLI（brew install gh）并已登录（gh auth login）
#   - 本地 SSH 能连到服务器（ssh ubuntu@<ip>）
#   - GitHub Actions 已成功编译出 citizenchain-node-linux-amd64 Artifact
set -euo pipefail

# ╔══════════════════════════════════════════════════════════════╗
# ║                     【密钥配置（可选）】                       ║
# ║  按需填写，留空则跳过该项配置。                                ║
# ╚══════════════════════════════════════════════════════════════╝

# 引导节点P2P 节点身份私钥（ed25519，64 位十六进制，决定节点的 Peer ID / bootnode 地址）
NODE_KEY=""

# 投票节点 GRANDPA 最终性投票私钥（ed25519，64 位十六进制，须匹配 genesis 中的权威公钥）
GRANDPA_KEY=""

# 矿工收款地址（SS58 格式，挖矿奖励发到此地址）
MINER_REWARD_ADDRESS=""

# ══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEPLOY_MODE="${1:?用法: $0 <q|b> <server_ip> [ssh_user]  (q=清链, b=不清链)}"
SERVER_IP="${2:?用法: $0 <q|b> <server_ip> [ssh_user]}"
SSH_USER="${3:-ubuntu}"

if [[ "$DEPLOY_MODE" != "q" && "$DEPLOY_MODE" != "b" ]]; then
    echo "错误：第一个参数必须是 q（清链）或 b（不清链）"
    exit 1
fi
SSH_TARGET="$SSH_USER@$SERVER_IP"

REMOTE_DIR="/opt/citizenchain"
REMOTE_DATA="$REMOTE_DIR/data"
REMOTE_KEYSTORE="$REMOTE_DATA/chains/citizenchain/keystore"
SERVICE_NAME="citizenchain-node"
ARTIFACT_NAME="citizenchain-node-linux-amd64"
SSH_KEY="$HOME/.ssh/ed25519"
SSH_OPTS="-i $SSH_KEY -o StrictHostKeyChecking=accept-new"

# ─── 校验密钥格式（仅在非空时校验） ───

validate_hex_key() {
  local name="$1" value="$2"
  if [ -n "$value" ] && ! echo "$value" | grep -qE '^[0-9a-fA-F]{64}$'; then
    echo "错误：$name 格式无效，应为 64 位十六进制字符串"
    exit 1
  fi
}

validate_hex_key "NODE_KEY（P2P 身份私钥）" "$NODE_KEY"
validate_hex_key "GRANDPA_KEY（投票私钥）" "$GRANDPA_KEY"

echo "=== 部署 citizenchain 节点到 $SSH_TARGET ==="

# ─── 1. 从 GitHub Actions 下载最新编译好的二进制 ───

echo ""
echo ">>> 下载最新 Artifact..."
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

cd "$REPO_ROOT"
gh run download --name "$ARTIFACT_NAME" --dir "$TMPDIR" 2>/dev/null || {
  echo "错误：下载 Artifact 失败。请确认："
  echo "  1. gh CLI 已登录（gh auth login）"
  echo "  2. GitHub Actions 已成功编译（gh run list）"
  exit 1
}

NODE_BIN="$TMPDIR/node"
if [ ! -f "$NODE_BIN" ]; then
  echo "错误：Artifact 中未找到 node 二进制"
  ls -la "$TMPDIR"
  exit 1
fi

chmod +x "$NODE_BIN"
echo "已下载：$(ls -lh "$NODE_BIN" | awk '{print $5}')"

# ─── 2. 推导 GRANDPA 公钥（仅在配置了 GRANDPA_KEY 时） ───

GRANDPA_PUBKEY=""
if [ -n "$GRANDPA_KEY" ]; then
  echo ""
  echo ">>> 推导 GRANDPA 公钥..."
  GRANDPA_PUBKEY=$(python3 -c "
from nacl.signing import SigningKey
sk = SigningKey(bytes.fromhex('$GRANDPA_KEY'))
print(sk.verify_key.encode().hex())
")
  echo "GRANDPA 公钥: $GRANDPA_PUBKEY"
fi

# ─── 3. 检测服务器是否需要首次初始化 ───

NEED_SETUP=false
ssh $SSH_OPTS "$SSH_TARGET" "test -f /etc/systemd/system/$SERVICE_NAME.service" 2>/dev/null || NEED_SETUP=true

if [ "$NEED_SETUP" = true ]; then
  echo ""
  echo ">>> 首次部署，初始化服务器..."

  # 创建用户和目录
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin citizenchain 2>/dev/null || true
    sudo mkdir -p $REMOTE_DIR/data
    sudo chown -R citizenchain:citizenchain $REMOTE_DIR
  "

  # 开放 P2P 端口
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo iptables -C INPUT -p tcp --dport 30333 -j ACCEPT 2>/dev/null || \
    sudo iptables -I INPUT -p tcp --dport 30333 -j ACCEPT
    sudo sh -c 'iptables-save > /etc/iptables/rules.v4' 2>/dev/null || true
  "

  echo "服务器初始化完成"
fi

# 每次都同步 systemd 服务文件（确保配置最新）
echo ""
echo ">>> 同步 systemd 服务文件..."
scp $SSH_OPTS "$SCRIPT_DIR/citizenchain-node.service" "$SSH_TARGET:/tmp/$SERVICE_NAME.service"
ssh $SSH_OPTS "$SSH_TARGET" "
  sudo mv /tmp/$SERVICE_NAME.service /etc/systemd/system/
  sudo systemctl daemon-reload
  sudo systemctl enable $SERVICE_NAME
"

# ─── 4. 停止服务、清数据、上传二进制 ───

echo ""
echo ">>> 停止节点..."
ssh -o ConnectTimeout=10 -o ServerAliveInterval=5 $SSH_OPTS "$SSH_TARGET" "sudo systemctl stop $SERVICE_NAME 2>/dev/null; sudo killall -9 node 2>/dev/null || true"

if [[ "$DEPLOY_MODE" == "q" ]]; then
echo ">>> 清除旧链数据..."
ssh $SSH_OPTS "$SSH_TARGET" "
  sudo rm -rf $REMOTE_DATA/chains/citizenchain/db
  sudo rm -rf $REMOTE_DATA/chains/citizenchain/network
"
echo "    旧链数据已清除"
else
echo ">>> 跳过清链（保留链数据）"
fi

echo ">>> 强制上传 node 二进制..."
scp $SSH_OPTS "$NODE_BIN" "$SSH_TARGET:/tmp/node-new"
ssh $SSH_OPTS "$SSH_TARGET" "
  sudo mv /tmp/node-new $REMOTE_DIR/node
  sudo chmod +x $REMOTE_DIR/node
  sudo chown citizenchain:citizenchain $REMOTE_DIR/node
"

# ─── 5. 写入密钥（仅配置了的项） ───

echo ""
echo ">>> 配置密钥..."

# P2P 节点身份密钥
if [ -n "$NODE_KEY" ]; then
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo mkdir -p $REMOTE_DATA/node-key
    printf '%s' '$NODE_KEY' | sudo tee $REMOTE_DATA/node-key/secret_ed25519 > /dev/null
    sudo chmod 600 $REMOTE_DATA/node-key/secret_ed25519
    sudo chown citizenchain:citizenchain $REMOTE_DATA/node-key/secret_ed25519
  "
  echo "  P2P 身份密钥 -> $REMOTE_DATA/node-key/secret_ed25519"
else
  echo "  P2P 身份密钥 -> 跳过（未配置）"
fi

# GRANDPA 投票密钥（写入 keystore）
if [ -n "$GRANDPA_KEY" ] && [ -n "$GRANDPA_PUBKEY" ]; then
  GRAN_FILENAME="6772616e${GRANDPA_PUBKEY}"
  GRAN_CONTENT="\"0x${GRANDPA_KEY}\""
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo mkdir -p $REMOTE_KEYSTORE
    echo -n '$GRAN_CONTENT' | sudo tee $REMOTE_KEYSTORE/$GRAN_FILENAME > /dev/null
    sudo chmod 600 $REMOTE_KEYSTORE/$GRAN_FILENAME
    sudo chown -R citizenchain:citizenchain $REMOTE_DATA
  "
  echo "  GRANDPA 投票密钥 -> $REMOTE_KEYSTORE/6772616e..."
else
  echo "  GRANDPA 投票密钥 -> 跳过（未配置）"
fi

# ─── 6. 启动服务 ───

echo ""
echo ">>> 启动节点..."
ssh $SSH_OPTS "$SSH_TARGET" "sudo systemctl start $SERVICE_NAME"

sleep 5

# 检查服务状态
if ssh $SSH_OPTS "$SSH_TARGET" "sudo systemctl is-active --quiet $SERVICE_NAME"; then
  echo "节点运行中"
else
  echo "节点启动失败，查看日志："
  ssh $SSH_OPTS "$SSH_TARGET" "sudo journalctl -u $SERVICE_NAME --no-pager -n 20"
  exit 1
fi

# ─── 7. 绑定矿工收款地址（仅配置了时） ───

if [ -n "$MINER_REWARD_ADDRESS" ]; then
  echo ""
  echo ">>> 绑定矿工收款地址: $MINER_REWARD_ADDRESS"
  BIND_RESULT=$(ssh $SSH_OPTS "$SSH_TARGET" "
    curl -s -H 'Content-Type: application/json' \
      -d '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"reward_bindWallet\",\"params\":[\"$MINER_REWARD_ADDRESS\"]}' \
      http://127.0.0.1:9944
  " 2>/dev/null)

  if echo "$BIND_RESULT" | python3 -c 'import sys,json; d=json.load(sys.stdin); exit(0 if "result" in d else 1)' 2>/dev/null; then
    echo "  收款地址绑定成功"
  else
    echo "  警告：收款地址绑定失败（可稍后在 nodeui 中手动绑定）"
    echo "  返回: $BIND_RESULT"
  fi
fi

# ─── 8. 获取节点信息 ───

echo ""
echo ">>> 获取节点信息..."
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
if [ -n "$GRANDPA_PUBKEY" ]; then
  echo "  GRANDPA:  6772616e${GRANDPA_PUBKEY:0:16}..."
fi
if [ -n "$MINER_REWARD_ADDRESS" ]; then
  echo "  收款地址: $MINER_REWARD_ADDRESS"
fi
echo "══════════════════════════════════════════════════════"
