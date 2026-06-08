#!/usr/bin/env bash
# 杀进程 + 只清本机区块数据库 + 用冻结 SSOT 创世重新接入网络
#
# 关键:本机节点的创世 :code 必须 == 线上 CI WASM,所以直接用内嵌的冻结 SSOT
#   (citizenchain/node/chainspecs/citizenchain.raw.json),**不再本地现造创世**。
#   旧版用本地源码 export 一份 fresh genesis,其 :code 是本地构建的 WASM,与线上
#   CI WASM 不逐字节一致 → genesis hash 分叉 → 连不进全网。现已改为直接用 SSOT。
#
# runtime 升级走链上 system.setCode;要改创世只能跑 bake-chainspec.sh 重新烘焙 SSOT。
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/gmb.dev"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"
SSOT="$CHAIN_ROOT/node/chainspecs/citizenchain.raw.json"

cleanup() {
    echo ""
    echo "==> 正在关闭节点进程..."
    pkill -f "citizenchain" 2>/dev/null || true
    lsof -ti:5173 2>/dev/null | xargs kill -9 2>/dev/null || true
    sleep 1
    echo "    节点已关闭"
}
trap cleanup EXIT INT TERM HUP

# ── 1. 杀进程 ──
echo "==> 杀掉所有节点进程..."
pkill -9 -f "citizenchain" 2>/dev/null || true
lsof -ti:5173 2>/dev/null | xargs kill -9 2>/dev/null || true
sleep 1
echo "    已清理"

# ── 2. 打印 SSOT 创世指纹(便于核对与线上一致)──
if [[ -s "$SSOT" ]]; then
  python3 - "$SSOT" <<'PY'
import json, hashlib, sys
top = json.load(open(sys.argv[1]))["genesis"]["raw"]["top"]
code = bytes.fromhex(top["0x3a636f6465"][2:])
print("==> 使用冻结 SSOT 创世:")
print(f"    {sys.argv[1]}")
print(f"    genesis :code blake2 = 0x{hashlib.blake2b(code, digest_size=32).hexdigest()}")
PY
else
  echo "错误:SSOT 不存在:$SSOT(请从 git 恢复)"; exit 1
fi

# ── 3. 只清区块数据库,保留节点身份/keystore/TLS ──
# 中文注释:不能删 node-key/secret_ed25519(PeerId 真源)、chains/*/keystore/
#   (GRANDPA + powr 矿工密钥)、tls/(WSS 证书)。只删 db/ 让区块从 #0 重挖。
DB_DIR="$APP_DATA_DIR/chains/citizenchain/db"
echo "==> 清除区块数据库:$DB_DIR"
rm -rf "$DB_DIR"
echo "    已清除(node-key/keystore/tls 全部保留)"

# ── 4. 用默认(内嵌 SSOT)创世启动 ──
# 本地只用当前源码构建 native runtime;创世 :code 来自 SSOT,不下载 GitHub CI WASM。
unset WASM_FILE
export CITIZENCHAIN_DATA_PROFILE=dev
cd "$CHAIN_ROOT/node"
echo "==> 启动公民链(冻结 SSOT 创世,清库后从 #0 重新接入)..."
cargo tauri dev
