#!/usr/bin/env bash
# 从 GitHub CI 下载最新编译好的 WASM binary
#
# 用法:
#   ./scripts/fetch-wasm.sh              # 下载正式 WASM（部署用）
#   ./scripts/fetch-wasm.sh benchmarks   # 下载 benchmarks WASM（跑权重用）
#
# 下载后本地编译自动跳过 WASM 构建:
#   export WASM_FILE=$(pwd)/target/ci-wasm/citizenchain.compact.compressed.wasm
#   cargo build --release

set -euo pipefail

REPO="ChinaNation/GMB"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

MODE="${1:-release}"

if [ "$MODE" = "benchmarks" ]; then
  ARTIFACT_NAME="citizenchain-wasm-benchmarks"
  OUT_DIR="${ROOT_DIR}/target/ci-wasm-benchmarks"
  echo "正在从 GitHub CI 获取最新 benchmarks WASM..."
else
  ARTIFACT_NAME="citizenchain-wasm"
  OUT_DIR="${ROOT_DIR}/target/ci-wasm"
  echo "正在从 GitHub CI 获取最新 WASM..."
fi

# 获取最新 artifact ID
ARTIFACT_ID=$(gh api "repos/${REPO}/actions/artifacts" \
  --jq "[.artifacts[] | select(.name==\"${ARTIFACT_NAME}\" and .expired==false)] | .[0].id")

if [ -z "$ARTIFACT_ID" ] || [ "$ARTIFACT_ID" = "null" ]; then
  echo "错误: 未找到 artifact '${ARTIFACT_NAME}'"
  echo "请确认 citizenchain-wasm CI 已运行成功"
  exit 1
fi

CREATED_AT=$(gh api "repos/${REPO}/actions/artifacts/${ARTIFACT_ID}" --jq '.created_at')
echo "artifact ID: ${ARTIFACT_ID} (${CREATED_AT})"

# 下载并解压
mkdir -p "$OUT_DIR"
TMP_ZIP=$(mktemp)
gh api "repos/${REPO}/actions/artifacts/${ARTIFACT_ID}/zip" > "$TMP_ZIP"
unzip -o "$TMP_ZIP" -d "$OUT_DIR"
rm -f "$TMP_ZIP"

WASM_PATH="${OUT_DIR}/citizenchain.compact.compressed.wasm"
if [ ! -f "$WASM_PATH" ]; then
  FOUND=$(find "$OUT_DIR" -name "citizenchain.compact.compressed.wasm" | head -1)
  if [ -n "$FOUND" ]; then
    mv "$FOUND" "$WASM_PATH"
  else
    echo "错误: 解压后未找到 citizenchain.compact.compressed.wasm"
    ls -la "$OUT_DIR"
    exit 1
  fi
fi

SIZE=$(du -h "$WASM_PATH" | cut -f1)
echo ""
echo "下载完成: ${WASM_PATH} (${SIZE})"
echo ""
echo "使用方法:"
echo "  export WASM_FILE=${WASM_PATH}"
if [ "$MODE" = "benchmarks" ]; then
  echo "  cargo build --features runtime-benchmarks --release"
  echo "  ./target/release/node benchmark pallet --chain=citizenchain --pallet=<name> --extrinsic='*' --steps=50 --repeat=20 --output=./runtime/<path>/weights.rs"
else
  echo "  cargo build --release"
fi
