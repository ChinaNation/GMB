#!/usr/bin/env bash
set -euo pipefail

# 中文注释：检查 Codex 新线程强制启动协议是否仍然成立。

MODE="${1:-}"

require_file() {
  local path="$1"

  if [[ ! -e "$path" ]]; then
    echo "缺少启动协议文件：$path" >&2
    exit 1
  fi
}

require_symlink_target() {
  local path="$1"
  local target="$2"
  local actual

  if [[ ! -L "$path" ]]; then
    echo "根目录入口必须保持为软链接：$path" >&2
    exit 1
  fi

  actual="$(readlink "$path")"
  if [[ "$actual" != "$target" ]]; then
    echo "根目录入口指向错误：$path -> $actual（期望 $target）" >&2
    exit 1
  fi
}

require_text() {
  local path="$1"
  local text="$2"

  if ! grep -Fq "$text" "$path"; then
    echo "启动协议缺少关键语句：$path -> $text" >&2
    exit 1
  fi
}

require_file "memory/AGENTS.md"
require_file "memory/CODEX.md"
require_file "memory/CLAUDE.md"
require_file "memory/07-ai/chat-protocol.md"
require_file "memory/07-ai/startup-acceptance.md"
require_file "memory/07-ai/document-boundaries.md"

require_symlink_target "AGENTS.md" "memory/AGENTS.md"
require_symlink_target "CODEX.md" "memory/CODEX.md"
require_symlink_target "CLAUDE.md" "memory/CLAUDE.md"

require_text "memory/AGENTS.md" "第一轮必须先做需求分析"
require_text "memory/AGENTS.md" "任务卡"
require_text "memory/CODEX.md" "第一轮必须输出需求分析"
require_text "memory/07-ai/chat-protocol.md" "需求分析"

if [[ "$MODE" == "--ci" ]]; then
  echo "启动协议检查通过。"
  exit 0
fi

cat <<'EOF'
启动协议检查通过。

手工验收步骤：
1. 在 GMB 工作区新开一个 Codex 线程。
2. 直接输入一段真实任务需求。
3. 第一轮回复必须以“需求分析”开头。
4. 回复中必须包含：任务需求、建议模块、影响范围、主要风险点、是否需要先沟通、建议下一步。
5. 确认继续执行后，真实开发任务必须创建任务卡。

详细标准见：memory/07-ai/startup-acceptance.md
EOF
