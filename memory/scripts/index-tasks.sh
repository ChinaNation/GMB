#!/usr/bin/env bash
set -euo pipefail

# 中文注释：刷新任务索引文件，方便在 memory 中查看 open/done 概览。

index_file="memory/08-tasks/index.md"
mkdir -p "memory/08-tasks"

list_task_files() {
  local dir="$1"
  local found=0
  while IFS= read -r file; do
    found=1
    printf -- '- `%s`\n' "$file"
  done < <(find "$dir" -maxdepth 1 -type f ! -name 'README.md' | sort)
  if [[ "$found" -eq 0 ]]; then
    printf -- '- 暂无\n'
  fi
}

{
  printf '# 任务索引\n\n'
  printf '## open\n\n'
  list_task_files "memory/08-tasks/open"
  printf '\n## done\n\n'
  list_task_files "memory/08-tasks/done"
} > "$index_file"

echo "任务索引已更新：$index_file"
