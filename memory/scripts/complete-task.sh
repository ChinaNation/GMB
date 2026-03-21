#!/usr/bin/env bash
set -euo pipefail

# 中文注释：把进行中的任务卡归档到 done 目录，并补充完成信息。

TASK_FILE="${1:-}"
SUMMARY="${2:-}"

usage() {
  cat <<'EOF'
用法：
  bash memory/scripts/complete-task.sh memory/08-tasks/open/<task>.md "完成摘要"
EOF
}

if [[ -z "$TASK_FILE" ]]; then
  usage
  exit 2
fi

if [[ ! -f "$TASK_FILE" ]]; then
  echo "任务卡不存在：$TASK_FILE" >&2
  exit 1
fi

done_dir="memory/08-tasks/done"
mkdir -p "$done_dir"

tmp_file="$(mktemp)"
cp "$TASK_FILE" "$tmp_file"

if grep -q '^- 状态：' "$tmp_file"; then
  sed 's/^- 状态：.*/- 状态：done/' "$tmp_file" > "${tmp_file}.new"
  mv "${tmp_file}.new" "$tmp_file"
else
  printf '\n- 状态：done\n' >> "$tmp_file"
fi

if ! grep -q '^## 完成信息' "$tmp_file"; then
  cat >> "$tmp_file" <<EOF

## 完成信息

- 完成时间：$(date '+%F %T')
- 完成摘要：${SUMMARY:-<补充完成摘要>}
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
EOF
fi

target_file="${done_dir}/$(basename "$TASK_FILE")"
mv "$tmp_file" "$target_file"
rm -f "$TASK_FILE"
bash "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/index-tasks.sh" >/dev/null

echo "任务卡已归档：$target_file"
