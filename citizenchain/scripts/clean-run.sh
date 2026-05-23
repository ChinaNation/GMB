#!/usr/bin/env bash
# 杀进程 + 生成 fresh genesis + 清链数据 + 启动本机新链
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/gmb.dev"

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

# ── 2. 准备本地源码 fresh genesis ──
CHAIN_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FRESH_SPEC_DIR="$CHAIN_ROOT/target/fresh-genesis"
FRESH_SPEC="$FRESH_SPEC_DIR/citizenchain.fresh.raw.json"
# 本地 clean-run 只使用当前源码生成创世 runtime code。
# runtime 正式升级走链上 setCode，不从 GitHub CI 下载 wasm 产物。
unset WASM_FILE
# 中文注释：clean-run 是开发工具，固定清理 gmb.dev，不能碰正式版 gmb 数据目录。
export CITIZENCHAIN_DATA_PROFILE=dev
echo "==> 使用本地源码生成 fresh genesis，不下载 GitHub CI WASM..."

# ── 3. 彻底清除所有编译缓存 ──
echo "==> 清除编译缓存..."
find "$CHAIN_ROOT/target" -maxdepth 3 -type d -name "citizenchain-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
find "$CHAIN_ROOT/target" -maxdepth 2 -type d -name "citizenchain" -path "*/wbuild/*" -exec rm -rf {} + 2>/dev/null || true
find "$CHAIN_ROOT/target" -name "libcitizenchain*" -delete 2>/dev/null || true
find "$CHAIN_ROOT/target" -name "libnode*" -delete 2>/dev/null || true
find "$CHAIN_ROOT/target" -maxdepth 3 -type d -name "node-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
echo "    已清除"

# ── 4. 用本地源码生成 fresh raw chainspec ──
cd "$CHAIN_ROOT/node"
echo "==> 生成 fresh genesis raw chainspec..."
mkdir -p "$FRESH_SPEC_DIR"
FROZEN_SPEC="$CHAIN_ROOT/node/chainspecs/citizenchain.raw.json"
cargo run -- export-chain-spec --chain citizenchain-fresh --raw > "$FRESH_SPEC.tmp"
python3 - "$FRESH_SPEC.tmp" "$FROZEN_SPEC" <<'PY'
import hashlib
import json
import sys

spec_file, frozen_file = sys.argv[1], sys.argv[2]
with open(spec_file, "r", encoding="utf-8") as fh:
    spec = json.load(fh)
with open(frozen_file, "r", encoding="utf-8") as fh:
    frozen = json.load(fh)

# 中文注释:fresh 创世必须沿用冻结主网那 44 个 bootnode,所有清链后的节点
# 通过同一组 DNS/PeerId 互联;同时 protocolId/chainType/properties/name/id
# 必须与冻结 spec 完全一致,确保全网 genesis_hash 收敛。
for field in ("name", "id", "chainType", "protocolId", "properties", "bootNodes"):
    expected = frozen.get(field)
    actual = spec.get(field)
    if expected != actual:
        raise SystemExit(
            f"错误：fresh chainspec {field} 与冻结 spec 不一致\n"
            f"  expected={expected!r}\n"
            f"  actual  ={actual!r}"
        )

bootnodes = spec.get("bootNodes") or []
if len(bootnodes) != 44:
    raise SystemExit(f"错误：bootNodes 数量必须为 44,实际 {len(bootnodes)}")

top = spec.get("genesis", {}).get("raw", {}).get("top", {})
code_hex = top.get("0x3a636f6465")
if not code_hex:
    raise SystemExit("错误：fresh chainspec 缺少 genesis :code")
code = bytes.fromhex(code_hex[2:] if code_hex.startswith("0x") else code_hex)

print(f"    bootNodes={len(bootnodes)} (沿用冻结 spec)")
print(f"    runtime code size={len(code)}")
print(f"    runtime code blake2_256=0x{hashlib.blake2b(code, digest_size=32).hexdigest()}")
PY
mv "$FRESH_SPEC.tmp" "$FRESH_SPEC"
echo "    fresh chainspec: $FRESH_SPEC"

# ── 5. 只清区块数据库,保留节点身份/keystore/TLS 证书 ──
# 中文注释:不能删 node-key/secret_ed25519(PeerId 真源,删了 chainspec 里 44 个
# /p2p/12D3... 全失效)、不能删 chains/*/keystore/(GRANDPA 权威 + powr 矿工密钥)、
# 不能删 tls/(WSS 证书)。只删 chains/citizenchain/db/ 让区块从 #0 重挖即可。
DB_DIR="$APP_DATA_DIR/chains/citizenchain/db"
echo "==> 清除区块数据库：$DB_DIR"
rm -rf "$DB_DIR"
echo "    已清除(node-key/keystore/tls 全部保留)"

# ── 6. 启动本机 fresh genesis ──
export CITIZENCHAIN_CHAIN_SPEC="$FRESH_SPEC"
echo "==> 启动公民链（本机 fresh genesis，不连接旧 bootnodes）..."
cargo tauri dev
