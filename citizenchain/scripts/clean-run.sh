#!/usr/bin/env bash
# 清链重新创世:杀进程 + 完全删除本机区块链数据 + 用【当前源码】现造创世启动。
#
# 与 run.sh 的区别(两脚本只此一别):
#   - run.sh        = 正常启动,用【冻结 SSOT】(node/chainspecs/citizenchain.raw.json)续跑现有链。
#   - clean-run.sh  = 清链 + 【不走 SSOT】,直接用当前 genesis_build 现造创世。
#     改了创世配置(宪法、立法院、创世账户…)无需重烤 SSOT 即时生效 —— 本地验证新创世的唯一入口。
#     宪法/机构等创世改动(如护照颁发改注册局)就靠它重新创世生效。
#
# 机制:节点进程内启动读 CITIZENCHAIN_CHAIN_SPEC;设为 citizenchain-fresh 即走
#   chain_spec::fresh_genesis_config()(当前源码 genesis_config() 现造),不读冻结 JSON。
#   fresh genesis 需要 runtime WASM,故 WASM_BUILD_FROM_SOURCE=1 让 build.rs 从源码编 WASM。
#
# 启动后:节点自动挖矿;链上中国平台需在节点设置页手动启动,统一入口 https://onchina.local:8964。
#   平台登录与节点启动解耦,本机构管理员冷钱包扫码、对链上 Active 管理员集合鉴权(3b)即可登录。
#
# 代价:
#   ① 现造创世的 genesis :code = 本地构建的 WASM,与他人/现网不逐字节一致 → 这是一条独立本地链。
#      要做全网共识,需用同一份 WASM 导出 raw spec 分发(`cargo run -p node -- export-chain-spec
#      --chain citizenchain-fresh --raw`),不在本脚本职责内。
#   ② 首次需从源码编译 runtime WASM,较慢。
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/gmb.dev"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"   # citizenchain/

cleanup() {
    echo ""
    echo "==> 正在关闭节点 + 链上中国平台 + 内嵌 PG..."
    pkill -f "citizenchain" 2>/dev/null || true
    pkill -f "target/debug/onchina" 2>/dev/null || true
    lsof -ti:5173 2>/dev/null | xargs kill -9 2>/dev/null || true
    if [ -n "${ONCHINA_PG_BIN_DIR:-}" ] && [ -n "${ONCHINA_PG_DATA_DIR:-}" ] && [ -d "${ONCHINA_PG_DATA_DIR:-}" ]; then
        "$ONCHINA_PG_BIN_DIR/pg_ctl" stop -D "$ONCHINA_PG_DATA_DIR" -m fast >/dev/null 2>&1 || true
    fi
    sleep 1
    echo "    已关闭"
}
trap cleanup EXIT INT TERM HUP

# ── 1. 杀进程 ──
echo "==> 杀掉所有节点/链上中国平台进程..."
pkill -9 -f "citizenchain" 2>/dev/null || true
pkill -9 -f "target/debug/onchina" 2>/dev/null || true
lsof -ti:5173 2>/dev/null | xargs kill -9 2>/dev/null || true
sleep 1
echo "    已清理"

# ── 2. 完全删除区块链数据(清链)+ 链上中国平台内嵌 PG 数据(与新创世一致地全新)──
# 中文注释:只删 db/(区块 + 状态)= 完全清链;node-key(PeerId)、keystore(矿工密钥)、
#   tls/(WSS 证书)是与创世无关的节点身份,保留以免重新生成。
DB_DIR="$APP_DATA_DIR/chains/citizenchain/db"
ONCHINA_PGDATA="$APP_DATA_DIR/onchina-pgdata"
echo "==> 完全删除区块链数据:$DB_DIR"
rm -rf "$DB_DIR"
echo "==> 删除链上中国平台内嵌 PG 数据(随新创世全新 initdb):$ONCHINA_PGDATA"
rm -rf "$ONCHINA_PGDATA"
echo "    已清链(node-key/keystore/tls 保留)"

# ── 3. onchina 控制台 dev 配置 ──
# 启动节点不需要任何机构鉴权/身份;此处仅准备链上中国平台手动启动所需资源(内嵌 PG + 前端 + china.sqlite)。
echo "==> 构建 onchina 二进制 + 前端..."
( cd "$CHAIN_ROOT" && cargo build -p onchina )
echo "==> 构建链上中国平台前端产物..."
( cd "$CHAIN_ROOT/onchina/frontend" && if [ ! -d node_modules ]; then npm ci; fi && npm run build )
PG_PREFIX=""
for v in postgresql@17 postgresql@16 postgresql@15 postgresql; do
    if p="$(brew --prefix "$v" 2>/dev/null)" && [ -x "$p/bin/initdb" ]; then PG_PREFIX="$p"; break; fi
done
if [ -n "$PG_PREFIX" ]; then
    export ONCHINA_EMBEDDED_PG=1
    export ONCHINA_PG_BIN_DIR="$PG_PREFIX/bin"
    export ONCHINA_PG_PORT="${ONCHINA_PG_PORT:-5433}"
    export ONCHINA_PG_DATA_DIR="$ONCHINA_PGDATA"
    echo "    内嵌私有 PG:$ONCHINA_PG_BIN_DIR(端口 $ONCHINA_PG_PORT)"
else
    echo "    [warn] 未找到本机 PostgreSQL(brew install postgresql@16);链上中国平台仍可起但缺 DB,功能受限。"
fi
export ONCHINA_CHINA_DB="$CHAIN_ROOT/onchina/src/cid/china/china.sqlite"
export ONCHINA_FRONTEND_DIST="$CHAIN_ROOT/onchina/frontend/dist"
export ONCHINA_ENABLE_TLS=1
export ONCHINA_TLS_DIR="$APP_DATA_DIR/onchina-tls"
# 中文注释:本地开发让链上中国平台启动时自动对账公权机构目录(全新内嵌 PG 是空库,
#   首启需把 40 万+ 公权机构从 china.sqlite 生成进库;首次较慢,之后增量对账很快),
#   否则启动期"目录落后"守卫会 panic、平台起不来。
export ONCHINA_GOV_AUTO_RECONCILE=1

# ── dev 平台签名与链上凭证签发配置(本地测试值)──
# 中文注释:这些变量只让本地能签登录 QR 挑战和链上凭证;节点启动、平台启动、
#   以及管理员所属机构判断都不依赖它们。机构归属由管理员冷钱包登录后链上反查并绑定。
export ONCHINA_SIGNING_SEED_HEX="${ONCHINA_SIGNING_SEED_HEX:-dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd}"
export ONCHAIN_CREDENTIAL_ISSUER_CID_NUMBER="${ONCHAIN_CREDENTIAL_ISSUER_CID_NUMBER:-ZS001-FRG07-249474503-2026}"
export ONCHAIN_CREDENTIAL_ISSUER_MAIN_ACCOUNT="${ONCHAIN_CREDENTIAL_ISSUER_MAIN_ACCOUNT:-0x406246b466028ae3cb89f36b70457478eca4ec224b2ad3f2122e5a0a407e642e}"
echo "==> 联邦注册局管理员省映射:全走链读,不再执行本地 seed"
echo "==> 链上中国平台:节点设置页点击“启动”后访问 https://onchina.local:8964"

# ── 4. 用当前源码现造创世启动(不走冻结 SSOT)──
unset WASM_FILE
export WASM_BUILD_FROM_SOURCE=1                   # build.rs 从源码编 runtime WASM → fresh genesis 可用
export CITIZENCHAIN_CHAIN_SPEC=citizenchain-fresh # 节点改用 fresh_genesis_config()(当前 genesis_build)
export CITIZENCHAIN_DATA_PROFILE=dev
cd "$CHAIN_ROOT/node"
echo "==> 用当前源码现造创世启动(genesis_build 现跑,宪法/立法院等创世改动即时生效)..."
cargo tauri dev
