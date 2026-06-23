#!/usr/bin/env bash
# 重新创世专用:用 CI WASM 烘焙唯一权威创世 SSOT,并同步 CitizenApp 轻节点 chainspec。
#
# ⚠️ 上线后链冻结,绝不再跑此脚本;runtime 升级一律走链上 system.setCode。
#    本脚本只在「预上线重新创世」时使用。
#
# 流程(固化本轮手动步骤):
#   1. 下载 CitizenChain WASM CI 的 citizenchain-wasm 产物(compact.compressed)
#   2. WASM_FILE=<该 wasm> export-chain-spec --chain citizenchain-fresh --raw
#   3. 断言 genesis :code blake2 == CI wasm + bootNodes=44 + 网络身份字段与现 SSOT 一致
#   4. 写入 SSOT(citizenchain/node/chainspecs/citizenchain.raw.json)+ 同步 CitizenApp 副本
#
# 用法: ./bake-chainspec.sh [<CitizenChain-WASM-run-id>]   (缺省取最新成功 run)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"            # citizenchain/
REPO_ROOT="$(dirname "$CHAIN_ROOT")"             # GMB/
SSOT="$CHAIN_ROOT/node/chainspecs/citizenchain.raw.json"
CITIZENAPP="$REPO_ROOT/citizenapp/assets/chainspec.json"
RUN_ID="${1:-}"

command -v gh >/dev/null || { echo "错误:需要 gh CLI 且已 gh auth login"; exit 1; }
TMPDIR="$(mktemp -d)"; trap 'rm -rf "$TMPDIR"' EXIT

if [[ -z "$RUN_ID" ]]; then
  RUN_ID="$(gh run list -R ChinaNation/GMB --workflow "CitizenChain WASM" \
            --status success --limit 1 --json databaseId --jq '.[0].databaseId')"
fi
echo ">>> 下载 citizenchain-wasm 产物 (run $RUN_ID)..."
gh run download "$RUN_ID" -R ChinaNation/GMB --name citizenchain-wasm --dir "$TMPDIR"
WASM="$TMPDIR/citizenchain.compact.compressed.wasm"
[[ -s "$WASM" ]] || { echo "错误:未找到 citizenchain.compact.compressed.wasm"; ls -la "$TMPDIR"; exit 1; }
CI_HASH="$(python3 -c "import hashlib,sys;print(hashlib.blake2b(open(sys.argv[1],'rb').read(),digest_size=32).hexdigest())" "$WASM")"
echo "    CI WASM blake2 = $CI_HASH"

echo ">>> 用 CI WASM 导出 fresh raw chainspec..."
(
  cd "$CHAIN_ROOT"
  WASM_FILE="$WASM" \
  LIBCLANG_PATH="${LIBCLANG_PATH:-$( (command -v llvm-config >/dev/null && llvm-config --libdir) || echo /opt/homebrew/opt/llvm/lib)}" \
  PROTOC="${PROTOC:-$(command -v protoc || true)}" \
  cargo run -p node -- export-chain-spec --chain citizenchain-fresh --raw
) > "$TMPDIR/new.json"

echo ">>> 断言 :code == CI WASM + 网络身份字段一致..."
python3 - "$TMPDIR/new.json" "$SSOT" "$CI_HASH" <<'PY'
import json, hashlib, sys
new = json.load(open(sys.argv[1])); ci = sys.argv[3]
try:
    old = json.load(open(sys.argv[2]))
except FileNotFoundError:
    old = None
code = bytes.fromhex(new["genesis"]["raw"]["top"]["0x3a636f6465"][2:])
h = hashlib.blake2b(code, digest_size=32).hexdigest()
assert h == ci, f":code({h}) != CI wasm({ci}) —— 创世 :code 必须是 CI 产物"
bn = new.get("bootNodes", [])
assert len(bn) == 44, f"bootNodes={len(bn)} != 44"
if old:
    for f in ("name", "id", "chainType", "protocolId", "properties", "bootNodes"):
        assert new.get(f) == old.get(f), f"{f} 与现 SSOT 不一致(网络身份必须延续)"
print("    ✅ :code=CI wasm, bootNodes=44, 网络身份字段一致")
PY

cp "$TMPDIR/new.json" "$SSOT"
cp "$TMPDIR/new.json" "$CITIZENAPP"
echo ">>> 已写入并同步:"
echo "    SSOT     : $SSOT"
echo "    CitizenApp: $CITIZENAPP"
echo ""
echo "下一步:git add 两份 chainspec → 提交 → 推送(触发 CitizenChain 节点 CI + CitizenApp CI)。"
echo "       服务器部署:scripts/fuwuqi.sh q <ip> ubuntu"
