#!/usr/bin/env bash
set -euo pipefail

# 中文注释：创建新的 AI 任务卡。

TITLE=""
MODULE=""
GOAL=""
OWNER="Codex"
REQUIREMENT=""

usage() {
  cat <<'EOF'
用法：
  bash memory/scripts/new-task.sh --module "模块路径" (--title "任务标题" --goal "任务目标" | --requirement "任务需求") [--owner "负责人"]
EOF
}

slugify() {
  local raw="$1"
  local slug
  slug="$(printf '%s' "$raw" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^[:alnum:][:space:]\x80-\xFF]+/-/g; s/[[:space:]]+/-/g; s/^-+//; s/-+$//; s/-+/-/g')"
  if [[ -z "$slug" ]]; then
    slug="task"
  fi
  printf '%s' "$slug"
}

append_line() {
  local file="$1"
  local text="$2"
  printf '%s\n' "$text" >> "$file"
}

template_file_for_module() {
  local module="$1"
  case "$module" in
    ai/system)
      printf '%s' "memory/08-tasks/templates/ai-system.md"
      ;;
    citizenchain/runtime|citizenchain/governance|citizenchain/issuance|citizenchain/otherpallet|citizenchain/transaction)
      printf '%s' "memory/08-tasks/templates/citizenchain-runtime.md"
      ;;
    citizenchain/node)
      printf '%s' "memory/08-tasks/templates/citizenchain-node.md"
      ;;
    citizenchain/nodeui)
      printf '%s' "memory/08-tasks/templates/citizenchain-nodeui.md"
      ;;
    sfid|sfid/backend|sfid/deploy)
      printf '%s' "memory/08-tasks/templates/sfid-backend.md"
      ;;
    sfid/frontend)
      printf '%s' "memory/08-tasks/templates/sfid-frontend.md"
      ;;
    cpms|cpms/backend|cpms/deploy)
      printf '%s' "memory/08-tasks/templates/cpms-backend.md"
      ;;
    cpms/frontend)
      printf '%s' "memory/08-tasks/templates/cpms-frontend.md"
      ;;
    wuminapp|wuminapp/lib)
      printf '%s' "memory/08-tasks/templates/wuminapp.md"
      ;;
    primitives)
      printf '%s' "memory/08-tasks/templates/primitives.md"
      ;;
    *)
      printf '%s' ""
      ;;
  esac
}

checklist_file_for_module() {
  local module="$1"
  case "$module" in
    ai/system)
      printf '%s' "memory/07-ai/module-checklists/ai-system.md"
      ;;
    citizenchain/runtime|citizenchain/governance|citizenchain/issuance|citizenchain/otherpallet|citizenchain/transaction|citizenchain/node|citizenchain/nodeui|primitives)
      printf '%s' "memory/07-ai/module-checklists/citizenchain.md"
      ;;
    sfid|sfid/backend|sfid/frontend|sfid/deploy)
      printf '%s' "memory/07-ai/module-checklists/sfid.md"
      ;;
    cpms|cpms/backend|cpms/frontend|cpms/deploy)
      printf '%s' "memory/07-ai/module-checklists/cpms.md"
      ;;
    wuminapp|wuminapp/lib)
      printf '%s' "memory/07-ai/module-checklists/wuminapp.md"
      ;;
    *)
      printf '%s' ""
      ;;
  esac
}

dod_file_for_module() {
  local module="$1"
  case "$module" in
    ai/system)
      printf '%s' "memory/07-ai/module-definition-of-done/ai-system.md"
      ;;
    citizenchain/runtime|citizenchain/governance|citizenchain/issuance|citizenchain/otherpallet|citizenchain/transaction|citizenchain/node|citizenchain/nodeui|primitives)
      printf '%s' "memory/07-ai/module-definition-of-done/citizenchain.md"
      ;;
    sfid|sfid/backend|sfid/frontend|sfid/deploy)
      printf '%s' "memory/07-ai/module-definition-of-done/sfid.md"
      ;;
    cpms|cpms/backend|cpms/frontend|cpms/deploy)
      printf '%s' "memory/07-ai/module-definition-of-done/cpms.md"
      ;;
    wuminapp|wuminapp/lib)
      printf '%s' "memory/07-ai/module-definition-of-done/wuminapp.md"
      ;;
    *)
      printf '%s' ""
      ;;
  esac
}

refresh_index() {
  bash "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/index-tasks.sh" >/dev/null
}

write_context_block() {
  local file="$1"
  local module="$2"

  append_line "$file" "- memory/00-vision/project-goal.md"
  append_line "$file" "- memory/00-vision/trust-boundary.md"
  append_line "$file" "- memory/01-architecture/repo-map.md"
  append_line "$file" "- memory/03-security/security-rules.md"
  append_line "$file" "- memory/07-ai/agent-rules.md"
  append_line "$file" "- memory/07-ai/context-loading-order.md"

  case "$module" in
    ai/system)
      append_line "$file" "- memory/AGENTS.md"
      append_line "$file" "- memory/CODEX.md"
      append_line "$file" "- memory/CLAUDE.md"
      append_line "$file" "- memory/07-ai/ai-system-overview.md"
      append_line "$file" "- memory/07-ai/document-boundaries.md"
      append_line "$file" "- memory/07-ai/startup-acceptance.md"
      ;;
    citizenchain/runtime|citizenchain/governance|citizenchain/issuance|citizenchain/otherpallet|citizenchain/transaction)
      append_line "$file" "- memory/01-architecture/citizenchain-target-structure.md"
      append_line "$file" "- citizenchain/CITIZENCHAIN_TECHNICAL.md"
      append_line "$file" "- citizenchain/runtime/README.md"
      ;;
    citizenchain/node)
      append_line "$file" "- memory/01-architecture/citizenchain-target-structure.md"
      append_line "$file" "- citizenchain/CITIZENCHAIN_TECHNICAL.md"
      ;;
    citizenchain/nodeui)
      append_line "$file" "- memory/01-architecture/citizenchain-target-structure.md"
      append_line "$file" "- memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md"
      ;;
    sfid|sfid/backend|sfid/frontend|sfid/deploy)
      append_line "$file" "- sfid/README.md"
      append_line "$file" "- sfid/SFID_TECHNICAL.md"
      ;;
    cpms|cpms/backend|cpms/frontend|cpms/deploy)
      append_line "$file" "- cpms/README.md"
      append_line "$file" "- cpms/CPMS_TECHNICAL.md"
      ;;
    wuminapp|wuminapp/lib)
      append_line "$file" "- wuminapp/WUMINAPP_TECHNICAL.md"
      ;;
    primitives)
      append_line "$file" "- memory/01-architecture/citizenchain-target-structure.md"
      append_line "$file" "- citizenchain/CITIZENCHAIN_TECHNICAL.md"
      ;;
    *)
      append_line "$file" "- <补充该模块对应技术文档路径>"
      ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --title)
      TITLE="${2:-}"
      shift 2
      ;;
    --module)
      MODULE="${2:-}"
      shift 2
      ;;
    --goal)
      GOAL="${2:-}"
      shift 2
      ;;
    --requirement)
      REQUIREMENT="${2:-}"
      shift 2
      ;;
    --owner)
      OWNER="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "未知参数: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -n "$REQUIREMENT" ]]; then
  [[ -z "$TITLE" ]] && TITLE="$REQUIREMENT"
  [[ -z "$GOAL" ]] && GOAL="$REQUIREMENT"
fi

if [[ -z "$TITLE" || -z "$MODULE" || -z "$GOAL" ]]; then
  echo "缺少必要参数：--module，以及 (--title 与 --goal) 或 --requirement" >&2
  usage
  exit 2
fi

timestamp="$(date '+%Y%m%d-%H%M%S')"
slug="$(slugify "$TITLE")"
task_dir="memory/08-tasks/open"
task_file="${task_dir}/${timestamp}-${slug}.md"
template_file="$(template_file_for_module "$MODULE")"
checklist_file="$(checklist_file_for_module "$MODULE")"
dod_file="$(dod_file_for_module "$MODULE")"

mkdir -p "$task_dir"

cat > "$task_file" <<EOF
# 任务卡：${TITLE}

- 任务编号：${timestamp}
- 状态：open
- 所属模块：${MODULE}
- 当前负责人：${OWNER}
- 创建时间：$(date '+%F %T')

## 任务需求

${GOAL}

## 必读上下文

EOF

write_context_block "$task_file" "$MODULE"

if [[ -n "$template_file" && -f "$template_file" ]]; then
  cat >> "$task_file" <<EOF

## 模块模板

- 模板来源：${template_file}

EOF
  cat "$template_file" >> "$task_file"
fi

if [[ -n "$checklist_file" && -f "$checklist_file" ]]; then
  cat >> "$task_file" <<EOF

## 模块执行清单

- 清单来源：${checklist_file}

EOF
  cat "$checklist_file" >> "$task_file"
fi

if [[ -n "$dod_file" && -f "$dod_file" ]]; then
  cat >> "$task_file" <<EOF

## 模块级完成标准

- 标准来源：${dod_file}

EOF
  cat "$dod_file" >> "$task_file"
fi

cat >> "$task_file" <<'EOF'

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
EOF

refresh_index

echo "已创建任务卡：$task_file"
echo "建议下一步：bash memory/scripts/load-context.sh \"$MODULE\""
