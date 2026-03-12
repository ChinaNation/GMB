#!/usr/bin/env bash
set -euo pipefail

# 中文注释：批量运行自定义 benchmark。
# - pallet 类型：调用 `node benchmark pallet`，写回各自的 weights.rs。
# - utility 类型：运行独立 bench harness，用于非 pallet 模块的专项性能验证。
# 用法：
#   ./scripts/run-benchmarks.sh
#   ./scripts/run-benchmarks.sh --check
#   ./scripts/run-benchmarks.sh --dry-run
#   ./scripts/run-benchmarks.sh --pallet sfid_code_auth --steps 20 --repeat 10

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NODE_BIN="$ROOT_DIR/target/release/node"

CHAIN="${CHAIN:-mainnet}"
STEPS="${STEPS:-50}"
REPEAT="${REPEAT:-20}"
HEAP_PAGES="${HEAP_PAGES:-4096}"
CHECK_MODE=0
DRY_RUN=0
BUILD_NODE=1
TEMPLATE_PATH="${TEMPLATE_PATH:-$ROOT_DIR/scripts/benchmark-weight-template.hbs}"

declare -a SELECTED_PALLETS=()

# 中文注释：benchmark 目标列表。
# 格式：kind:name:payload
# - pallet: payload=weights.rs 相对路径
# - utility: payload=说明性路径（当前仅用于日志展示）
declare -a TARGETS=(
  "pallet:shengbank_stake_interest:issuance/shengbank-stake-interest/src/weights.rs"
  "pallet:fullnode_pow_reward:issuance/fullnode-pow-reward/src/weights.rs"
  "pallet:citizen_lightnode_issuance:issuance/citizen-lightnode-issuance/src/weights.rs"
  "pallet:sfid_code_auth:otherpallet/sfid-code-auth/src/weights.rs"
  "pallet:pow_difficulty_module:otherpallet/pow-difficulty-module/src/weights.rs"
  "pallet:resolution_issuance_iss:issuance/resolution-issuance-iss/src/weights.rs"
  "pallet:resolution_issuance_gov:governance/resolution-issuance-gov/src/weights.rs"
  "pallet:voting_engine_system:governance/voting-engine-system/src/weights.rs"
  "pallet:admins_origin_gov:governance/admins-origin-gov/src/weights.rs"
  "pallet:grandpa_key_gov:governance/grandpa-key-gov/src/weights.rs"
  "pallet:runtime_root_upgrade:governance/runtime-root-upgrade/src/weights.rs"
  "pallet:resolution_destro_gov:governance/resolution-destro-gov/src/weights.rs"
  "pallet:duoqian_transaction_pow:transaction/duoqian-transaction-pow/src/weights.rs"
  "utility:onchain_transaction_pow:transaction/onchain-transaction-pow/benches/transaction_fee_paths.rs"
)

usage() {
  cat <<'EOF'
Usage: ./scripts/run-benchmarks.sh [options]

Options:
  --check                运行后检查 weights.rs 是否产生变更；有变更则返回非 0。
  --dry-run              仅打印将要执行的命令，不实际执行。
  --no-build             跳过 cargo build（要求本地已有 target/release/node）。
  --chain <name>         benchmark chain，默认 mainnet。
  --steps <N>            benchmark steps，默认 50。
  --repeat <N>           benchmark repeat，默认 20。
  --heap-pages <N>       wasm heap pages，默认 4096。
  --template <path>      可选 hbs 模板路径（未传则使用 CLI 默认模板）。
  --pallet <name>        仅运行指定 benchmark 目标（可重复传入多个）。
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

NEEDS_NODE=0
for target in "${TARGETS[@]}"; do
  IFS=':' read -r kind pallet _ <<< "$target"
  if [[ "$kind" == "pallet" ]] && contains_selected_pallet "$pallet"; then
    NEEDS_NODE=1
    break
  fi
done

if [[ "$BUILD_NODE" -eq 1 && "$NEEDS_NODE" -eq 1 ]]; then
  log "构建 benchmark 节点二进制（node, release, runtime-benchmarks）"
  cargo build -p node --release --features runtime-benchmarks --locked
fi

if [[ "$DRY_RUN" -eq 0 && "$NEEDS_NODE" -eq 1 && ! -x "$NODE_BIN" ]]; then
  echo "未找到可执行文件: $NODE_BIN" >&2
  echo "请先执行: cargo build -p node --release --features runtime-benchmarks" >&2
  exit 1
fi

cd "$ROOT_DIR"

declare -a TOUCHED_OUTPUTS=()
declare -i RUN_COUNT=0

for target in "${TARGETS[@]}"; do
  IFS=':' read -r kind pallet rel_output <<< "$target"

  if ! contains_selected_pallet "$pallet"; then
    continue
  fi

  RUN_COUNT+=1

  if [[ "$kind" == "pallet" ]]; then
    output="$ROOT_DIR/$rel_output"
    mkdir -p "$(dirname "$output")"
    TOUCHED_OUTPUTS+=("$rel_output")

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
      log "开始 pallet benchmark: pallet=$pallet output=$rel_output"
      "${cmd[@]}"
      log "完成 pallet benchmark: pallet=$pallet"
    fi
  elif [[ "$kind" == "utility" ]]; then
    cmd=(
      cargo bench
      -p onchain-transaction-pow
      --bench transaction_fee_paths
      --
      --warm-up-time 1
      --measurement-time 1
      --sample-size 10
    )

    if [[ "$DRY_RUN" -eq 1 ]]; then
      log "DRY-RUN: ${cmd[*]}"
    else
      log "开始 utility benchmark: target=$pallet harness=$rel_output"
      "${cmd[@]}"
      log "完成 utility benchmark: target=$pallet"
    fi
  else
    echo "未知 benchmark 类型: $kind" >&2
    exit 2
  fi
done

if [[ "${RUN_COUNT}" -eq 0 ]]; then
  echo "未匹配到任何 pallet，请检查 --pallet 参数。" >&2
  exit 2
fi

if [[ "$CHECK_MODE" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
  if [[ "${#TOUCHED_OUTPUTS[@]}" -eq 0 ]]; then
    log "本次未生成任何 weights.rs（可能只运行了 utility benchmark），跳过 weights 变更检查。"
    exit 0
  fi
  if ! git -C "$ROOT_DIR" diff --quiet -- "${TOUCHED_OUTPUTS[@]}"; then
    log "检测到 benchmark 生成的 weights 变更，请提交更新。"
    git -C "$ROOT_DIR" --no-pager diff -- "${TOUCHED_OUTPUTS[@]}"
    exit 1
  fi
  log "weights.rs 无变化，检查通过。"
fi

log "benchmark 批量执行结束（数量: ${RUN_COUNT}）"
