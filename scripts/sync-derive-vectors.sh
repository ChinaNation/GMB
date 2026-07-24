#!/usr/bin/env bash
# ADR-024 账户派生金标向量本机同步守卫。
#
# 作用:
#   1. 用 account_derive 唯一真源重生 canonical fixture(ACCOUNT_DERIVE_UPDATE=1 跑 Rust 导出测试)。
#   2. 复制 canonical → citizenapp Dart 副本。
#   3. `git diff --exit-code` 两份文件:若与提交版有任何差异则失败(=有人改了派生算法/常量却没提交刷新后的金标,或 Dart 副本漂移)。
#
# 用法:
#   scripts/sync_account_derive_vectors.sh           # 默认 check 模式:重生后必须无 diff,否则退出码 1
#   scripts/sync_account_derive_vectors.sh --write    # 本地刷新:重生 + 复制,允许并保留 diff(供 commit)
#
# 行为中性铁律:Tier 1/2 期间不应产生 diff(地址不变)。Tier 3(域 DUOQIAN→GMB)
# 创世时本脚本用 --write 刷新金标,并随该次创世一起提交。

set -euo pipefail

# 仓库根(本脚本位于 <repo>/scripts/)。
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

PRIMITIVES_MANIFEST="${REPO_ROOT}/citizenchain/runtime/primitives/Cargo.toml"
CANONICAL="${REPO_ROOT}/citizenchain/runtime/primitives/tests/fixtures/account_derive_vectors.json"
DART_COPY="${REPO_ROOT}/citizenapp/test/governance/shared/fixtures/account_derive_vectors.json"

MODE="check"
if [[ "${1:-}" == "--write" ]]; then
  MODE="write"
fi

echo "[sync] 1/3 用 account_derive 重生 canonical 金标 fixture ..."
ACCOUNT_DERIVE_UPDATE=1 cargo test \
  --manifest-path "${PRIMITIVES_MANIFEST}" \
  --test account_derive_golden \
  -- --nocapture

echo "[sync] 2/3 复制 canonical → Dart 副本 ..."
cp "${CANONICAL}" "${DART_COPY}"

echo "[sync] 3/3 校验两份金标与提交版一致 ..."
if [[ "${MODE}" == "write" ]]; then
  echo "[sync] --write 模式:保留改动,跳过 diff 守卫。请检查并提交:"
  echo "         ${CANONICAL}"
  echo "         ${DART_COPY}"
  git -C "${REPO_ROOT}" --no-pager diff -- "${CANONICAL}" "${DART_COPY}" || true
  exit 0
fi

# check 模式:重生后必须与提交版逐字节一致。
if ! git -C "${REPO_ROOT}" diff --exit-code -- "${CANONICAL}" "${DART_COPY}"; then
  echo "" >&2
  echo "[sync] ✗ 金标向量与提交版不一致!" >&2
  echo "[sync]   原因:account_derive 算法/常量变了却没刷新金标,或 Dart 副本漂移。" >&2
  echo "[sync]   修复:本地跑 'scripts/sync_account_derive_vectors.sh --write' 后提交两份文件。" >&2
  echo "[sync]   注意:Tier 1/2 期间出现 diff = 行为非中性(地址变了),需排查回归。" >&2
  exit 1
fi

echo "[sync] ✓ 金标向量一致(canonical + Dart 副本)。"
