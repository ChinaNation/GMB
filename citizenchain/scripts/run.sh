#!/usr/bin/env bash
# 正常启动:不清库,用【冻结 SSOT】(node/chainspecs/citizenchain.raw.json)续跑现有链。
# 要清链并用当前源码现造创世,改用 clean-run.sh。
#
# 启动后:节点自动挖矿;链上中国平台需在节点设置页手动启动,统一入口 https://onchina.local:8964。
# 平台登录与节点启动**解耦**:本机构管理员用冷钱包扫码、对链上 Active 管理员集合
# 鉴权(3b)即可登录;不是本机构管理员就不用管,也没有任何机构权限。
set -euo pipefail

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

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"   # citizenchain/
TARGET_DIR="$REPO_ROOT/target"

# 本地启动脚本只使用当前源码构建 runtime WASM。
# runtime 正式升级走链上 setCode，桌面端启动不再从 GitHub CI 下载 wasm 产物。
unset WASM_FILE
# 开发启动固定使用 gmb.dev，避免和正式安装版争用同一份 RocksDB。
export CITIZENCHAIN_DATA_PROFILE=dev
mkdir -p "$TARGET_DIR"

# ── onchina 控制台dev 配置 ──
# 启动节点不需要任何机构鉴权/身份。这里只让本机能跑起链上中国平台服务:
#   ① 构建 onchina 二进制(节点同目录,设置页手动启动时由 onchina_proc 拉起)+ 前端产物;
#   ② DB 用内嵌私有 PG(方案 A):借本机 PostgreSQL 二进制起一个 onchina 专属实例(127.0.0.1)。
# 本机构的"系统签名钥 / 机构身份"是可选配置(签登录 QR / 签发凭证才需要),非启动前提。
echo "==> 构建 onchina 二进制 + 前端..."
( cd "$REPO_ROOT" && cargo build -p onchina )
echo "==> 构建链上中国平台前端产物..."
( cd "$REPO_ROOT/onchina/frontend" && if [ ! -d node_modules ]; then npm ci; fi && npm run build )
PG_PREFIX=""
for v in postgresql@17 postgresql@16 postgresql@15 postgresql; do
    if p="$(brew --prefix "$v" 2>/dev/null)" && [ -x "$p/bin/initdb" ]; then PG_PREFIX="$p"; break; fi
done
if [ -n "$PG_PREFIX" ]; then
    export ONCHINA_EMBEDDED_PG=1
    export ONCHINA_PG_BIN_DIR="$PG_PREFIX/bin"
    export ONCHINA_PG_PORT="${ONCHINA_PG_PORT:-5433}"
    export ONCHINA_PG_DATA_DIR="$HOME/Library/Application Support/gmb.dev/onchina-pgdata"
    echo "    内嵌私有 PG:$ONCHINA_PG_BIN_DIR(端口 $ONCHINA_PG_PORT)"
else
    echo "    [warn] 未找到本机 PostgreSQL(brew install postgresql@16);链上中国平台仍可起但缺 DB,功能受限。"
fi
export ONCHINA_CHINA_DB="$REPO_ROOT/onchina/src/cid/china/china.sqlite"
export ONCHINA_FRONTEND_DIST="$REPO_ROOT/onchina/frontend/dist"
export ONCHINA_ENABLE_TLS=1
export ONCHINA_TLS_DIR="$HOME/Library/Application Support/gmb.dev/onchina-tls"
# 本地开发让链上中国平台启动时自动对账公权机构目录(全新内嵌 PG 是空库,
#   首启需把 40 万+ 公权机构从 china.sqlite 生成进库;首次较慢,之后增量对账很快),
#   否则启动期"目录落后"守卫会 panic、平台起不来。
export ONCHINA_GOV_AUTO_RECONCILE=1

# ── dev 平台签名与链上凭证签发配置(本地测试值)──
# 这些变量只让本地能签登录 QR 挑战和链上凭证;节点启动、平台启动、
#   以及管理员所属机构判断都不依赖它们。机构归属由管理员冷钱包登录后链上反查并绑定。
export ONCHINA_SIGNING_SEED_HEX="${ONCHINA_SIGNING_SEED_HEX:-dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd}"
export ONCHAIN_CREDENTIAL_ISSUER_CID_NUMBER="${ONCHAIN_CREDENTIAL_ISSUER_CID_NUMBER:-ZS001-FRG07-249474503-2026}"
export ONCHAIN_CREDENTIAL_ISSUER_MAIN_ACCOUNT="${ONCHAIN_CREDENTIAL_ISSUER_MAIN_ACCOUNT:-0x406246b466028ae3cb89f36b70457478eca4ec224b2ad3f2122e5a0a407e642e}"

echo "==> 使用本地源码构建 runtime WASM，不下载 GitHub CI WASM..."
echo "    节点启动产物目录: $TARGET_DIR"
echo "    开发数据目录: $HOME/Library/Application Support/gmb.dev"
echo "==> 链上中国平台:节点设置页点击“启动”后访问 https://onchina.local:8964"

# ── 启动 ──
cd "$REPO_ROOT/node"
echo "==> 启动公民链..."
cargo tauri dev
