#!/usr/bin/env bash
# 杀进程 + 下载最新 CI WASM + 生成 fresh genesis + 清链数据 + 启动本机新链
set -euo pipefail

APP_DATA_DIR="$HOME/Library/Application Support/org.chinanation.citizenchain.desktop"
REPO="ChinaNation/GMB"
WORKFLOW="CitizenChain WASM"
ARTIFACT_NAME="citizenchain-wasm"

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

# ── 2. 下载最新 CI WASM（必须成功）──
CHAIN_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WASM_DIR="$CHAIN_ROOT/target/ci-wasm"
FRESH_SPEC_DIR="$CHAIN_ROOT/target/fresh-genesis"
FRESH_SPEC="$FRESH_SPEC_DIR/citizenchain.fresh.raw.json"
echo "==> 查询最新成功的 WASM CI..."
RUN_ID=$(gh run list \
    --repo "$REPO" \
    --workflow "$WORKFLOW" \
    --status success \
    --limit 1 \
    --json databaseId \
    --jq '.[0].databaseId')
if [[ -z "${RUN_ID:-}" || "$RUN_ID" == "null" ]]; then
    echo "错误：未找到成功的 $WORKFLOW 运行"
    exit 1
fi
echo "    最新成功 run: $RUN_ID"

echo "==> 下载最新 WASM..."
rm -rf "$WASM_DIR"
mkdir -p "$WASM_DIR"
if ! gh run download "$RUN_ID" --name "$ARTIFACT_NAME" --dir "$WASM_DIR" -R "$REPO"; then
    echo "错误：无法下载 WASM。gh auth login 后重试。"
    exit 1
fi
export WASM_FILE="$WASM_DIR/citizenchain.compact.compressed.wasm"
[ -f "$WASM_FILE" ] || { echo "错误：WASM 文件不存在"; exit 1; }
echo "    WASM: $WASM_FILE"

# ── 3. 彻底清除所有编译缓存 ──
echo "==> 清除编译缓存..."
find "$CHAIN_ROOT/target" -maxdepth 3 -type d -name "citizenchain-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
find "$CHAIN_ROOT/target" -maxdepth 2 -type d -name "citizenchain" -path "*/wbuild/*" -exec rm -rf {} + 2>/dev/null || true
find "$CHAIN_ROOT/target" -name "libcitizenchain*" -delete 2>/dev/null || true
find "$CHAIN_ROOT/target" -name "libnode*" -delete 2>/dev/null || true
find "$CHAIN_ROOT/target" -maxdepth 3 -type d -name "node-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
echo "    已清除"

# ── 4. 用最新 CI WASM 生成 fresh raw chainspec ──
cd "$CHAIN_ROOT/node"
echo "==> 生成 fresh genesis raw chainspec..."
mkdir -p "$FRESH_SPEC_DIR"
FROZEN_SPEC="$CHAIN_ROOT/node/chainspecs/citizenchain.raw.json"
cargo run -- export-chain-spec --chain citizenchain-fresh --raw > "$FRESH_SPEC.tmp"
python3 - "$WASM_FILE" "$FRESH_SPEC.tmp" "$FROZEN_SPEC" <<'PY'
import hashlib
import json
import sys

wasm_file, spec_file, frozen_file = sys.argv[1], sys.argv[2], sys.argv[3]
with open(wasm_file, "rb") as fh:
    wasm = fh.read()
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
if code != wasm:
    raise SystemExit(
        "错误：fresh chainspec 的 genesis :code 与最新 CI WASM 不一致\n"
        f"  wasm blake2_256=0x{hashlib.blake2b(wasm, digest_size=32).hexdigest()}\n"
        f"  code blake2_256=0x{hashlib.blake2b(code, digest_size=32).hexdigest()}"
    )

print(f"    bootNodes={len(bootnodes)} (沿用冻结 spec)")
print(f"    wasm/code size={len(code)}")
print(f"    wasm/code blake2_256=0x{hashlib.blake2b(code, digest_size=32).hexdigest()}")
PY
mv "$FRESH_SPEC.tmp" "$FRESH_SPEC"
echo "    fresh chainspec: $FRESH_SPEC"

# ── 5. 只清区块数据库,保留节点身份/keystore/TLS 证书 ──
# 中文注释:不能删 network/secret_ed25519(PeerId 真源,删了 chainspec 里 44 个
# /p2p/12D3... 全失效)、不能删 keystore/(GRANDPA 权威 + powr 矿工密钥)、
# 不能删 tls/(WSS 证书)。只删 db/ 让区块从 #0 重挖即可。
DB_DIR="$APP_DATA_DIR/node-data/chains/citizenchain/db"
echo "==> 清除区块数据库：$DB_DIR"
rm -rf "$DB_DIR"
echo "    已清除(node-key/keystore/tls 全部保留)"

# ── 6. 启动本机 fresh genesis ──
export CITIZENCHAIN_CHAIN_SPEC="$FRESH_SPEC"
echo "==> 启动公民链（本机 fresh genesis，不连接旧 bootnodes）..."
cargo tauri dev
