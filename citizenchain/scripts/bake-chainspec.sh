#!/usr/bin/env bash
# 烘焙 CitizenChain raw chainspec。
#
# 默认模式只生成预览文件到 target/chainspec,不覆盖冻结 SSOT。
# 正式创世必须在 GitHub WASM CI 成功后执行:
#   citizenchain/scripts/bake-chainspec.sh --finalize --wasm /path/to/citizenchain.compact.compressed.wasm
#
# 正式模式会把同一份 raw spec 同步到:
#   1. citizenchain/node/chainspecs/citizenchain.raw.json
#   2. citizenapp/assets/chainspec.json
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"
REPO_ROOT="$(dirname "$CHAIN_ROOT")"
OUT="$CHAIN_ROOT/target/chainspec/citizenchain.raw.json"
FINALIZE=0
SKIP_CHECK=0
WASM_FILE_ARG=""

usage() {
    cat <<'EOF'
Usage:
  citizenchain/scripts/bake-chainspec.sh [--out FILE] [--skip-check]
  citizenchain/scripts/bake-chainspec.sh --finalize --wasm FILE [--out FILE]

Options:
  --out FILE       生成 raw chainspec 的输出路径。默认 citizenchain/target/chainspec/citizenchain.raw.json
  --wasm FILE      GitHub WASM CI 产出的 runtime wasm。正式创世必须提供
  --finalize       覆盖冻结 SSOT: node/chainspecs/citizenchain.raw.json 与 citizenapp/assets/chainspec.json
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
TMP="$(mktemp "$CHAIN_ROOT/target/chainspec/.citizenchain.raw.XXXXXX.json")"
trap 'rm -f "$TMP"' EXIT

echo "==> 导出 fresh raw chainspec..."
(
    cd "$CHAIN_ROOT"
    cargo run -p node -- export-chain-spec --chain citizenchain-fresh --raw > "$TMP"
)

if [[ "$SKIP_CHECK" != "1" ]]; then
    echo "==> 检查宪法创世与冻结条件..."
    CHECK_ARGS=("$SCRIPT_DIR/check-constitution-genesis.py" "$TMP")
    if [[ -n "$WASM_FILE_ARG" ]]; then
        CHECK_ARGS+=(--expect-code-file "$WASM_FILE")
    fi
    python3 "${CHECK_ARGS[@]}"
else
    echo "==> 已跳过宪法创世检查(--skip-check)"
fi

mv "$TMP" "$OUT"
trap - EXIT
echo "==> 已生成: $OUT"

if [[ "$FINALIZE" == "1" ]]; then
    NODE_SPEC="$CHAIN_ROOT/node/chainspecs/citizenchain.raw.json"
    APP_SPEC="$REPO_ROOT/citizenapp/assets/chainspec.json"
    install -m 0644 "$OUT" "$NODE_SPEC"
    install -m 0644 "$OUT" "$APP_SPEC"
    echo "==> 已同步冻结 SSOT:"
    echo "    $NODE_SPEC"
    echo "    $APP_SPEC"
else
    echo "==> 预览模式完成,未覆盖冻结 SSOT。正式创世请加 --finalize --wasm <CI_WASM>。"
fi
