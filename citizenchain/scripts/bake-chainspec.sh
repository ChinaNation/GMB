#!/usr/bin/env bash
# 烘焙 CitizenChain 冻结 chainspec(plain 形态,ADR-031 D5)。
#
# 59.7 万公权机构全量创世直铸下 raw 形态会到 GB 级,不再入库。冻结 SSOT 改为
# plain JSON(runtime WASM + genesis patch + bootnodes),节点首启经 runtime
# `GenesisBuilder` 本地物化创世 state;CitizenApp/smoldot 用 stateRootHash 轻形态。
#
# 默认模式只生成预览文件到 target/chainspec,不覆盖冻结 SSOT。
# 正式创世必须在 GitHub WASM CI 成功后执行:
#   citizenchain/scripts/bake-chainspec.sh --finalize --wasm /path/to/citizenchain.compact.compressed.wasm
#
# 正式模式会同步:
#   1. citizenchain/node/chainspecs/citizenchain.plain.json   (节点冻结 SSOT)
#   2. citizenapp/assets/chainspec.json                        (smoldot 轻形态:stateRootHash)
#
# 流程:导出 plain spec → 临时节点物化创世(记录耗时)→ RPC 宪法创世检查
#       → 读块 0 头生成轻形态 → finalize 同步。
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"
REPO_ROOT="$(dirname "$CHAIN_ROOT")"
OUT="$CHAIN_ROOT/target/chainspec/citizenchain.plain.json"
APP_OUT="$CHAIN_ROOT/target/chainspec/chainspec.app.json"
FINALIZE=0
SKIP_CHECK=0
WASM_FILE_ARG=""
RPC_PORT=19944

usage() {
    cat <<'EOF'
Usage:
  citizenchain/scripts/bake-chainspec.sh [--out FILE] [--skip-check]
  citizenchain/scripts/bake-chainspec.sh --finalize --wasm FILE [--out FILE]

Options:
  --out FILE       生成 plain chainspec 的输出路径。默认 citizenchain/target/chainspec/citizenchain.plain.json
  --wasm FILE      GitHub WASM CI 产出的 runtime wasm。正式创世必须提供
  --finalize       覆盖冻结 SSOT: node/chainspecs/citizenchain.plain.json 与 citizenapp/assets/chainspec.json
  --skip-check     跳过宪法创世检查。只用于排障,正式创世不得使用
  -h, --help       显示帮助
EOF
}

while (($#)); do
    case "$1" in
        --out)
            OUT="${2:?--out 需要文件路径}"
            shift 2
            ;;
        --wasm)
            WASM_FILE_ARG="${2:?--wasm 需要 wasm 文件路径}"
            shift 2
            ;;
        --finalize)
            FINALIZE=1
            shift
            ;;
        --skip-check)
            SKIP_CHECK=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "未知参数: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ "$FINALIZE" == "1" && -z "$WASM_FILE_ARG" ]]; then
    echo "错误: --finalize 必须同时提供 --wasm FILE,确保 :code 来自已通过 CI 的 WASM。" >&2
    exit 2
fi

if [[ -n "$WASM_FILE_ARG" ]]; then
    if [[ ! -s "$WASM_FILE_ARG" ]]; then
        echo "错误: WASM 文件不存在或为空: $WASM_FILE_ARG" >&2
        exit 2
    fi
    export WASM_FILE="$(cd "$(dirname "$WASM_FILE_ARG")" && pwd)/$(basename "$WASM_FILE_ARG")"
    unset WASM_BUILD_FROM_SOURCE
    echo "==> 使用指定 WASM_FILE: $WASM_FILE"
else
    export WASM_BUILD_FROM_SOURCE=1
    unset WASM_FILE
    echo "==> 未指定 --wasm,仅做本地预览:从源码构建 runtime WASM"
fi

mkdir -p "$(dirname "$OUT")"
TMP="$(mktemp "$CHAIN_ROOT/target/chainspec/.citizenchain.plain.XXXXXX.json")"
NODE_TMP_DIR="$(mktemp -d "$CHAIN_ROOT/target/chainspec/.bakenode.XXXXXX")"
NODE_PID=""
cleanup() {
    [[ -n "$NODE_PID" ]] && kill "$NODE_PID" 2>/dev/null || true
    rm -f "$TMP"
    rm -rf "$NODE_TMP_DIR"
}
trap cleanup EXIT

echo "==> 导出 fresh plain chainspec..."
(
    cd "$CHAIN_ROOT"
    cargo run -p node -- export-chain-spec --chain citizenchain-fresh > "$TMP"
)

rpc() {
    curl -s -H 'content-type: application/json' \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":$2}" \
        "http://127.0.0.1:$RPC_PORT" | python3 -c 'import sys,json;print(json.dumps(json.load(sys.stdin).get("result")))'
}

echo "==> 启动临时节点物化创世(59.7 万机构,分钟级,记录耗时)..."
GENESIS_T0=$(date +%s)
(
    cd "$CHAIN_ROOT"
    CITIZENCHAIN_HEADLESS=1 ./target/debug/citizenchain --tmp --chain "$TMP" \
        --base-path "$NODE_TMP_DIR" --rpc-port "$RPC_PORT" \
        --no-mdns --no-prometheus --no-telemetry \
        >"$NODE_TMP_DIR/node.log" 2>&1
) &
NODE_PID=$!

GENESIS_HASH="null"
for _ in $(seq 1 120); do
    sleep 5
    if ! kill -0 "$NODE_PID" 2>/dev/null; then
        echo "错误: 临时节点提前退出,日志尾部:" >&2
        tail -20 "$NODE_TMP_DIR/node.log" >&2
        exit 1
    fi
    GENESIS_HASH=$(rpc chain_getBlockHash '[0]' 2>/dev/null || echo null)
    [[ "$GENESIS_HASH" != "null" && -n "$GENESIS_HASH" ]] && break
done
if [[ "$GENESIS_HASH" == "null" || -z "$GENESIS_HASH" ]]; then
    echo "错误: 10 分钟内未完成创世物化,日志尾部:" >&2
    tail -20 "$NODE_TMP_DIR/node.log" >&2
    exit 1
fi
GENESIS_SECS=$(( $(date +%s) - GENESIS_T0 ))
GENESIS_HASH_STR=$(echo "$GENESIS_HASH" | tr -d '"')
STATE_ROOT=$(rpc chain_getHeader "[$GENESIS_HASH]" | python3 -c 'import sys,json;print(json.loads(sys.stdin.read())["stateRoot"])')
echo "==> 创世物化完成: 耗时 ${GENESIS_SECS}s, genesis=$GENESIS_HASH_STR, stateRoot=$STATE_ROOT"

if [[ "$SKIP_CHECK" != "1" ]]; then
    echo "==> 检查宪法创世与冻结条件(RPC 模式)..."
    CHECK_ARGS=("$SCRIPT_DIR/check-constitution-genesis.py" --rpc "http://127.0.0.1:$RPC_PORT" --at "$GENESIS_HASH_STR")
    if [[ -n "$WASM_FILE_ARG" ]]; then
        CHECK_ARGS+=(--expect-code-file "$WASM_FILE")
    fi
    python3 "${CHECK_ARGS[@]}"
else
    echo "==> 已跳过宪法创世检查(--skip-check)"
fi

echo "==> 生成 CitizenApp 轻形态 chainspec(stateRootHash)..."
python3 - "$TMP" "$APP_OUT" "$STATE_ROOT" <<'PYEOF'
import json, sys
plain_path, app_path, state_root = sys.argv[1], sys.argv[2], sys.argv[3]
plain = json.load(open(plain_path))
# 轻形态:去掉 runtimeGenesis(59.7 万 state 不进 App),只留 stateRootHash;
# smoldot 据此自建创世头,校验后续区块。
app = {k: plain[k] for k in
       ("name", "id", "chainType", "bootNodes", "telemetryEndpoints",
        "protocolId", "properties", "codeSubstitutes") if k in plain}
app["genesis"] = {"stateRootHash": state_root}
json.dump(app, open(app_path, "w"), ensure_ascii=False, indent=2)
print(f"    {app_path}")
PYEOF

kill "$NODE_PID" 2>/dev/null || true
NODE_PID=""

mv "$TMP" "$OUT"
trap - EXIT
rm -rf "$NODE_TMP_DIR"
echo "==> 已生成: $OUT"
echo "==> 首启物化耗时 ${GENESIS_SECS}s(验收记录);创世哈希 $GENESIS_HASH_STR"

if [[ "$FINALIZE" == "1" ]]; then
    NODE_SPEC="$CHAIN_ROOT/node/chainspecs/citizenchain.plain.json"
    APP_SPEC="$REPO_ROOT/citizenapp/assets/chainspec.json"
    install -m 0644 "$OUT" "$NODE_SPEC"
    install -m 0644 "$APP_OUT" "$APP_SPEC"
    echo "==> 已同步冻结 SSOT:"
    echo "    $NODE_SPEC"
    echo "    $APP_SPEC (轻形态 stateRootHash)"
else
    echo "==> 预览模式完成,未覆盖冻结 SSOT。正式创世请加 --finalize --wasm <CI_WASM>。"
fi
