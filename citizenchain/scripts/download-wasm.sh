#!/usr/bin/env bash
# 下载最新成功的 CitizenChain WASM CI artifact 到 citizenchain/target/wasm-ci/
# 依赖 gh CLI，不会触发新的 CI 构建。

set -euo pipefail

# 补充常见的 PATH，确保 gh CLI 可用
export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"

REPO="ChinaNation/GMB"
WORKFLOW="CitizenChain WASM"
ARTIFACT_NAME="citizenchain-wasm"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CITIZENCHAIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="$CITIZENCHAIN_DIR/target/wasm-ci"

if ! command -v gh >/dev/null 2>&1; then
  echo "错误：未找到 gh CLI，请先安装并执行 gh auth login" >&2
  exit 1
fi

echo "查询最新成功的 $WORKFLOW 运行..."
RUN_ID=$(gh run list \
  --repo "$REPO" \
  --workflow "$WORKFLOW" \
  --status success \
  --limit 1 \
  --json databaseId \
  --jq '.[0].databaseId')

if [[ -z "${RUN_ID:-}" || "$RUN_ID" == "null" ]]; then
  echo "错误：未找到成功的 $WORKFLOW 运行" >&2
  exit 1
fi

echo "最新成功 run: $RUN_ID"
mkdir -p "$OUT_DIR"

# 清空旧文件，只保留本次下载的
find "$OUT_DIR" -type f -name "*.wasm" -delete

echo "下载 artifact 到 $OUT_DIR ..."
gh run download "$RUN_ID" \
  --repo "$REPO" \
  --name "$ARTIFACT_NAME" \
  --dir "$OUT_DIR"

echo ""
echo "完成。文件列表："
ls -la "$OUT_DIR"/*.wasm

echo ""
echo "Blake2_256 摘要："
for f in "$OUT_DIR"/*.wasm; do
  python3 -c "
import hashlib, sys
with open('$f','rb') as fh:
    data = fh.read()
h = hashlib.blake2b(data, digest_size=32).hexdigest()
print(f'{\"$f\".split(\"/\")[-1]:50s} size={len(data):>10d}  blake2_256=0x{h}')
"
done
