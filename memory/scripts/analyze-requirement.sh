#!/usr/bin/env bash
set -euo pipefail

# 中文注释：纯需求分析入口，只做分析，不创建任务卡。

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec bash "$SCRIPT_DIR/architect-entry.sh" "$@"

