#!/usr/bin/env bash
# 清链重新创世:杀进程 + 完全删除本机区块链数据 + 用【当前源码】现造创世启动。
#
# 与 run.sh 的区别(两脚本只此一别):
#   - run.sh        = 正常启动,用【冻结 SSOT】(node/chainspecs/citizenchain.raw.json)续跑现有链。
#   - clean-run.sh  = 清链 + 【不走 SSOT】,直接用当前 genesis_build 现造创世。
#     改了创世配置(宪法、立法院、创世账户…)无需重烤 SSOT 即时生效 —— 本地验证新创世的唯一入口。
#
# 机制:节点进程内启动读 CITIZENCHAIN_CHAIN_SPEC;设为 citizenchain-fresh 即走
#   chain_spec::fresh_genesis_config()(当前源码 genesis_config() 现造),不读冻结 JSON。
#   fresh genesis 需要 runtime WASM,故 WASM_BUILD_FROM_SOURCE=1 让 build.rs 从源码编 WASM。
#
# 代价:
#   ① 现造创世的 genesis :code = 本地构建的 WASM,与他人/现网不逐字节一致 → 这是一条独立本地链。
#      要做全网共识,需用同一份 WASM 导出 raw spec 分发(`cargo run -p node -- export-chain-spec
#      --chain citizenchain-fresh --raw`),不在本脚本职责内。
#   ② 首次需从源码编译 runtime WASM,较慢。
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/gmb.dev"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"

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

# ── 2. 完全删除区块链数据(清链)──
# 中文注释:只删 db/(区块 + 状态)= 完全清链;node-key(PeerId)、keystore(GRANDPA/powr 矿工密钥)、
#   tls/(WSS 证书)是与创世无关的节点身份,保留以免重新生成。
DB_DIR="$APP_DATA_DIR/chains/citizenchain/db"
echo "==> 完全删除区块链数据:$DB_DIR"
rm -rf "$DB_DIR"
echo "    已清链(node-key/keystore/tls 保留)"

# ── 3. 用当前源码现造创世启动(不走冻结 SSOT)──
unset WASM_FILE
export WASM_BUILD_FROM_SOURCE=1                   # build.rs 从源码编 runtime WASM → fresh genesis 可用
export CITIZENCHAIN_CHAIN_SPEC=citizenchain-fresh # 节点改用 fresh_genesis_config()(当前 genesis_build)
export CITIZENCHAIN_DATA_PROFILE=dev
cd "$CHAIN_ROOT/node"
echo "==> 用当前源码现造创世启动(genesis_build 现跑,宪法/立法院等创世改动即时生效)..."
cargo tauri dev
