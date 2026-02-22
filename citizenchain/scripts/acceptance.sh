#!/usr/bin/env bash
set -euo pipefail

# 中文注释：区块链自动化验收脚本
# 用法：
#   ./scripts/acceptance.sh quick
#   ./scripts/acceptance.sh full
#   ./scripts/acceptance.sh coverage
#   ./scripts/acceptance.sh full --html --lcov

MODE="${1:-quick}"
shift || true

WITH_HTML=0
WITH_LCOV=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --html)
      WITH_HTML=1
      shift
      ;;
    --lcov)
      WITH_LCOV=1
      shift
      ;;
    *)
      echo "未知参数: $1"
      exit 2
      ;;
  esac
done

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="$ROOT_DIR/target/acceptance"
REPORT_FILE="$REPORT_DIR/report-$(date +%Y%m%d-%H%M%S).log"

mkdir -p "$REPORT_DIR"

log() {
  echo "[$(date '+%F %T')] $*" | tee -a "$REPORT_FILE"
}

run_step() {
  local name="$1"
  shift
  log "开始: $name"
  "$@" 2>&1 | tee -a "$REPORT_FILE"
  log "完成: $name"
}

cd "$ROOT_DIR"

log "自动化验收模式: $MODE"
log "项目目录: $ROOT_DIR"

run_step "cargo check workspace" cargo check --workspace
run_step "cargo test workspace" cargo test --workspace -q

case "$MODE" in
  quick)
    log "quick 模式结束（已完成编译与全量单元测试）"
    ;;

  full)
    run_step "cargo test runtime package" cargo test -p gmb-runtime -q
    run_step "cargo test voting-engine-system package" cargo test -p voting-engine-system -q
    run_step "cargo test sfid-code-auth package" cargo test -p sfid-code-auth -q
    run_step "cargo test onchain-transaction-fee package" cargo test -p onchain-transaction-fee -q
    run_step "cargo test offchain-transaction-fee package" cargo test -p offchain-transaction-fee -q
    log "full 模式结束（包含关键模块回归）"
    ;;

  coverage)
    run_step "cargo llvm-cov summary" env SKIP_WASM_BUILD=1 cargo llvm-cov --workspace --summary-only

    if [[ "$WITH_LCOV" -eq 1 ]]; then
      run_step "cargo llvm-cov lcov" env SKIP_WASM_BUILD=1 cargo llvm-cov --workspace --lcov --output-path coverage.lcov
      log "LCOV 输出: $ROOT_DIR/coverage.lcov"
    fi

    if [[ "$WITH_HTML" -eq 1 ]]; then
      run_step "cargo llvm-cov html" env SKIP_WASM_BUILD=1 cargo llvm-cov --workspace --html
      log "HTML 输出目录: $ROOT_DIR/target/llvm-cov/html"
    fi

    log "coverage 模式结束"
    ;;

  *)
    log "不支持的模式: $MODE"
    exit 2
    ;;
esac

log "验收完成，报告文件: $REPORT_FILE"
