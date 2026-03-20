#!/usr/bin/env bash
# 【正式模式】编译正式链节点（6分钟出块，标准难度）
# 当前仍由旧版 Tauri 节点壳 `nodeuitauri` 的 build.rs 自动编译+复制节点二进制。
# 新版 Flutter `nodeui` 完成迁移前，正式链桌面打包仍以 `nodeuitauri` 为准。
set -euo pipefail
cd "$(dirname "$0")/../citizenchain/nodeuitauri"
echo "==> 编译 nodeuitauri + 链节点（正式链）..."
cargo build --release
echo "✅ 正式链编译完成"
