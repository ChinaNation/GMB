#!/usr/bin/env bash
# 一键部署/更新 citizenchain 节点到甲骨文云服务器。
#
# 用法：
#   ./scripts/deploy-oracle.sh <server_ip> [ssh_user]
#   例：./scripts/deploy-oracle.sh 140.238.1.2 ubuntu
#
# 前置条件：
#   - 本地已安装 gh CLI（brew install gh）并已登录（gh auth login）
#   - 本地 SSH 能连到服务器（ssh ubuntu@<ip>）
#   - GitHub Actions 已成功编译出 citizenchain-node-linux-amd64 Artifact
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

SERVER_IP="${1:?用法: $0 <server_ip> [ssh_user]}"
SSH_USER="${2:-ubuntu}"
SSH_TARGET="$SSH_USER@$SERVER_IP"

REMOTE_DIR="/opt/citizenchain"
SERVICE_NAME="citizenchain-node"
ARTIFACT_NAME="citizenchain-node-linux-amd64"
SSH_KEY="$HOME/.ssh/ed25519"
SSH_OPTS="-i $SSH_KEY -o StrictHostKeyChecking=accept-new"

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

# ─── 2. 检测服务器是否需要首次初始化 ───

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

  # 上传 systemd 服务文件
  scp $SSH_OPTS "$SCRIPT_DIR/citizenchain-node.service" "$SSH_TARGET:/tmp/$SERVICE_NAME.service"
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo mv /tmp/$SERVICE_NAME.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable $SERVICE_NAME
  "

  # 开放 P2P 端口
  ssh $SSH_OPTS "$SSH_TARGET" "
    sudo iptables -C INPUT -p tcp --dport 30333 -j ACCEPT 2>/dev/null || \
    sudo iptables -I INPUT -p tcp --dport 30333 -j ACCEPT
    sudo sh -c 'iptables-save > /etc/iptables/rules.v4' 2>/dev/null || true
  "

  echo "服务器初始化完成"
fi

# ─── 3. 上传二进制并重启服务 ───

echo ""
echo ">>> 上传 node 二进制..."
scp $SSH_OPTS "$NODE_BIN" "$SSH_TARGET:/tmp/node-new"

echo ">>> 替换并重启服务..."
ssh $SSH_OPTS "$SSH_TARGET" "
  sudo systemctl stop $SERVICE_NAME 2>/dev/null || true
  sudo mv /tmp/node-new $REMOTE_DIR/node
  sudo chmod +x $REMOTE_DIR/node
  sudo chown citizenchain:citizenchain $REMOTE_DIR/node
  sudo systemctl start $SERVICE_NAME
"

# ─── 4. 等待节点启动并获取信息 ───

echo ""
echo ">>> 等待节点启动..."
sleep 5

# 检查服务状态
if ssh $SSH_OPTS "$SSH_TARGET" "sudo systemctl is-active --quiet $SERVICE_NAME"; then
  echo "✅ 节点运行中"
else
  echo "❌ 节点启动失败，查看日志："
  ssh $SSH_OPTS "$SSH_TARGET" "sudo journalctl -u $SERVICE_NAME --no-pager -n 20"
  exit 1
fi

# 获取 Peer ID
echo ""
echo ">>> 获取节点信息..."
PEER_ID=$(ssh $SSH_OPTS "$SSH_TARGET" "
  curl -s -H 'Content-Type: application/json' \
    -d '{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"system_localPeerId\",\"params\":[]}' \
    http://127.0.0.1:9944 2>/dev/null | python3 -c 'import sys,json; print(json.load(sys.stdin)[\"result\"])' 2>/dev/null
" || echo "")

if [ -n "$PEER_ID" ]; then
  echo ""
  echo "╔══════════════════════════════════════════════════════╗"
  echo "║ 部署成功！                                          ║"
  echo "╟──────────────────────────────────────────────────────╢"
  echo "║ Peer ID: $PEER_ID"
  echo "║ bootNode: /ip4/$SERVER_IP/tcp/30333/ws/p2p/$PEER_ID"
  echo "╟──────────────────────────────────────────────────────╢"
  echo "║ 将上面的 bootNode 加入 chainspec 即可让轻节点发现此节点 ║"
  echo "╚══════════════════════════════════════════════════════╝"
else
  echo ""
  echo "✅ 节点已启动，但 RPC 尚未就绪（可能还在同步）。"
  echo "稍后通过以下命令获取 Peer ID："
  echo "  ssh $SSH_TARGET 'curl -s localhost:9944 -d \"{\\\"id\\\":1,\\\"jsonrpc\\\":\\\"2.0\\\",\\\"method\\\":\\\"system_localPeerId\\\",\\\"params\\\":[]}\"'"
fi
