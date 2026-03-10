#!/usr/bin/env bash
set -euo pipefail

# 中文注释：批量运行自定义 pallet benchmark，并将结果写回各自的 weights.rs。
# 用法：
#   ./scripts/run-benchmarks.sh
#   ./scripts/run-benchmarks.sh --check
#   ./scripts/run-benchmarks.sh --dry-run
#   ./scripts/run-benchmarks.sh --pallet sfid_code_auth --steps 20 --repeat 10

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NODE_BIN="$ROOT_DIR/target/release/node"

CHAIN="${CHAIN:-dev}"
STEPS="${STEPS:-50}"
REPEAT="${REPEAT:-20}"
HEAP_PAGES="${HEAP_PAGES:-4096}"
CHECK_MODE=0
DRY_RUN=0
BUILD_NODE=1
TEMPLATE_PATH="${TEMPLATE_PATH:-}"

declare -a SELECTED_PALLETS=()

# 中文注释：需要 benchmark 的自定义 pallet 列表（pallet_name:weights_path）。
declare -a TARGETS=(
  "shengbank_stake_interest:issuance/shengbank-stake-interest/src/weights.rs"
  "fullnode_pow_reward:issuance/fullnode-pow-reward/src/weights.rs"
  "sfid_code_auth:otherpallet/sfid-code-auth/src/weights.rs"
  "resolution_destro_gov:governance/resolution-destro-gov/src/weights.rs"
  "finality_key_gov:governance/finality-key-gov/src/weights.rs"
  "resolution_issuance_iss:issuance/resolution-issuance-iss/src/weights.rs"
  "duoqian_transaction_pow:transaction/duoqian-transaction-pow/src/weights.rs"
  "admins_origin_gov:governance/admins-origin-gov/src/weights.rs"
  "resolution_issuance_gov:governance/resolution-issuance-gov/src/weights.rs"
  "offchain_transaction_fee:transaction/offchain-transaction-pos/src/weights.rs"
)

usage() {
  cat <<'EOF'
Usage: ./scripts/run-benchmarks.sh [options]

Options:
  --check                运行后检查 weights.rs 是否产生变更；有变更则返回非 0。
  --dry-run              仅打印将要执行的命令，不实际执行。
  --no-build             跳过 cargo build（要求本地已有 target/release/node）。
  --chain <name>         benchmark chain，默认 dev。
  --steps <N>            benchmark steps，默认 50。
  --repeat <N>           benchmark repeat，默认 20。
  --heap-pages <N>       wasm heap pages，默认 4096。
  --template <path>      可选 hbs 模板路径（未传则使用 CLI 默认模板）。
  --pallet <name>        仅运行指定 pallet（可重复传入多个）。
  -h, --help             显示帮助。
EOF
}

log() {
  echo "[$(date '+%F %T')] $*"
}

contains_selected_pallet() {
  local pallet="$1"
  if [[ "${#SELECTED_PALLETS[@]}" -eq 0 ]]; then
    return 0
  fi
  local selected
  for selected in "${SELECTED_PALLETS[@]}"; do
    if [[ "$selected" == "$pallet" ]]; then
      return 0
    fi
  done
  return 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check)
      CHECK_MODE=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --no-build)
      BUILD_NODE=0
      shift
      ;;
    --chain)
      CHAIN="$2"
      shift 2
      ;;
    --steps)
      STEPS="$2"
      shift 2
      ;;
    --repeat)
      REPEAT="$2"
      shift 2
      ;;
    --heap-pages)
      HEAP_PAGES="$2"
      shift 2
      ;;
    --template)
      TEMPLATE_PATH="$2"
      shift 2
      ;;
    --pallet)
      SELECTED_PALLETS+=("$2")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "未知参数: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ "$DRY_RUN" -eq 1 ]]; then
  BUILD_NODE=0
fi

if [[ "$BUILD_NODE" -eq 1 ]]; then
  log "构建 benchmark 节点二进制（node, release, runtime-benchmarks）"
  cargo build -p node --release --features runtime-benchmarks --locked
fi

if [[ "$DRY_RUN" -eq 0 && ! -x "$NODE_BIN" ]]; then
  echo "未找到可执行文件: $NODE_BIN" >&2
  echo "请先执行: cargo build -p node --release --features runtime-benchmarks" >&2
  exit 1
fi

cd "$ROOT_DIR"

declare -a TOUCHED_OUTPUTS=()
declare -i RUN_COUNT=0

for target in "${TARGETS[@]}"; do
  pallet="${target%%:*}"
  rel_output="${target#*:}"

  if ! contains_selected_pallet "$pallet"; then
    continue
  fi

  output="$ROOT_DIR/$rel_output"
  mkdir -p "$(dirname "$output")"
  TOUCHED_OUTPUTS+=("$rel_output")
  RUN_COUNT+=1

  cmd=(
    "$NODE_BIN"
    benchmark pallet
    --chain "$CHAIN"
    --pallet "$pallet"
    --extrinsic "*"
    --steps "$STEPS"
    --repeat "$REPEAT"
    --wasm-execution compiled
    --heap-pages "$HEAP_PAGES"
    --output "$output"
  )

  if [[ -n "$TEMPLATE_PATH" ]]; then
    cmd+=(--template "$TEMPLATE_PATH")
  fi

  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "DRY-RUN: ${cmd[*]}"
  else
    log "开始 benchmark: pallet=$pallet output=$rel_output"
    "${cmd[@]}"
    log "完成 benchmark: pallet=$pallet"
  fi
done

if [[ "${RUN_COUNT}" -eq 0 ]]; then
  echo "未匹配到任何 pallet，请检查 --pallet 参数。" >&2
  exit 2
fi

if [[ "$CHECK_MODE" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
  if ! git -C "$ROOT_DIR" diff --quiet -- "${TOUCHED_OUTPUTS[@]}"; then
    log "检测到 benchmark 生成的 weights 变更，请提交更新。"
    git -C "$ROOT_DIR" --no-pager diff -- "${TOUCHED_OUTPUTS[@]}"
    exit 1
  fi
  log "weights.rs 无变化，检查通过。"
fi

log "benchmark 批量执行结束（数量: ${RUN_COUNT}）"
