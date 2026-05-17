#!/usr/bin/env bash
# 本地跑所有 pallet 的 benchmark，生成 weights.rs。
# 需要的时候手动跑，把生成的 weights.rs 提交到仓库。
#
# 用法：
#   ./scripts/benchmark.sh          # 跑所有 pallet
#   ./scripts/benchmark.sh pow_difficulty   # 只跑指定 pallet
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHAIN_ROOT="$(dirname "$SCRIPT_DIR")"

# benchmark 必须基于当前源码生成 weights，不从 GitHub CI 下载 wasm。
# runtime 正式升级走链上 setCode，CI wasm 只供链上升级流程显式使用。
unset WASM_FILE
echo "==> 使用本地源码构建 benchmark runtime，不下载 GitHub CI WASM..."

# ── 1. 清除 runtime 缓存，用当前源码编译 ──
echo "==> 清除 runtime 缓存..."
find "$CHAIN_ROOT/target" -maxdepth 3 -type d -name "citizenchain-*" -path "*/build/*" -exec rm -rf {} + 2>/dev/null || true
find "$CHAIN_ROOT/target" -maxdepth 2 -type d -name "citizenchain" -path "*/wbuild/*" -exec rm -rf {} + 2>/dev/null || true
echo "    已清除"

# ── 2. 编译带 benchmark feature 的 node ──
echo "==> 编译 benchmark node（release）..."
cd "$CHAIN_ROOT"
cargo build --release --features runtime-benchmarks
echo "    编译完成"

# ── 3. 跑 benchmark ──
PALLETS=(
    "shengbank_interest:runtime/issuance/shengbank-interest/src/weights.rs"
    "fullnode_issuance:runtime/issuance/fullnode-issuance/src/weights.rs"
    "citizen_issuance:runtime/issuance/citizen-issuance/src/weights.rs"
    "resolution_issuance:runtime/issuance/resolution-issuance/src/weights.rs"
    "sfid_system:runtime/otherpallet/sfid-system/src/weights.rs"
    "pow_difficulty:runtime/otherpallet/pow-difficulty/src/weights.rs"
    "admins_change:runtime/governance/admins-change/src/weights.rs"
    "resolution_destro:runtime/governance/resolution-destro/src/weights.rs"
    "grandpakey_change:runtime/governance/grandpakey-change/src/weights.rs"
    "duoqian_manage:runtime/transaction/duoqian-manage/src/weights.rs"
    "duoqian_transfer:runtime/transaction/duoqian-transfer/src/weights.rs"
    "voting_engine:runtime/governance/voting-engine/src/weights.rs"
    "runtime_upgrade:runtime/governance/runtime-upgrade/src/weights.rs"
)

FILTER="${1:-}"
FAILED=0

for entry in "${PALLETS[@]}"; do
    PALLET="${entry%%:*}"
    OUTPUT="${entry##*:}"

    # 如果指定了 pallet 名，只跑那一个
    if [ -n "$FILTER" ] && [ "$PALLET" != "$FILTER" ]; then
        continue
    fi

    echo ""
    echo "══════════════════════════════════════"
    echo "▶ $PALLET"
    echo "══════════════════════════════════════"
    if ./target/release/node benchmark pallet \
        --chain=citizenchain \
        --pallet="$PALLET" \
        --extrinsic='*' \
        --steps=50 \
        --repeat=20 \
        --output="$OUTPUT"; then
        echo "✓ $PALLET → $OUTPUT"
    else
        echo "✗ $PALLET 失败"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
if [ "$FAILED" -gt 0 ]; then
    echo "⚠ $FAILED 个 pallet benchmark 失败"
    exit 1
else
    echo "✓ 全部完成，weights.rs 已更新。记得提交到仓库。"
fi
