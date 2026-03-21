#!/usr/bin/env bash
set -euo pipefail

# 中文注释：刷新任务索引文件，方便在 memory 中查看 open/done 概览。

index_file="memory/08-tasks/index.md"
mkdir -p "memory/08-tasks"

{
  printf '# 任务索引\n\n'
  printf '## open\n\n'
  if find "memory/08-tasks/open" -maxdepth 1 -type f ! -name 'README.md' | grep -q .; then
    find "memory/08-tasks/open" -maxdepth 1 -type f ! -name 'README.md' | sort | while read -r file; do
      printf -- '- `%s`\n' "$file"
    done
  else
    printf -- '- 暂无\n'
  fi
  printf '\n## done\n\n'
  if find "memory/08-tasks/done" -maxdepth 1 -type f ! -name 'README.md' | grep -q .; then
    find "memory/08-tasks/done" -maxdepth 1 -type f ! -name 'README.md' | sort | while read -r file; do
      printf -- '- `%s`\n' "$file"
    done
  else
    printf -- '- 暂无\n'
  fi
} > "$index_file"

echo "任务索引已更新：$index_file"

