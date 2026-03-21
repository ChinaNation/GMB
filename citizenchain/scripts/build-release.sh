#!/usr/bin/env bash
# 【正式模式】编译正式链节点（6分钟出块，标准难度）
# node 二进制由 nodeui 的 build.rs 自动编译+复制（不加 dev-chain feature）
# 仅编译，不启动。用于正式链发布。
set -euo pipefail
cd "$(dirname "$0")/../nodeui"
echo "==> 编译 nodeui + 链节点（正式链）..."
cargo build --release
echo "✅ 正式链编译完成"
