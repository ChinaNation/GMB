#!/usr/bin/env bash
set -euo pipefail

# 中文注释：Architect 总入口。

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=memory/scripts/module-router.sh
source "$SCRIPT_DIR/module-router.sh"

TITLE=""
MODULE=""
GOAL=""
OWNER="Codex"
FORCE_CLARIFY="false"
REQUIREMENT=""
EXECUTE="false"

usage() {
  cat <<'EOF'
用法：
  bash memory/scripts/architect-entry.sh (--requirement "任务需求" | --title "任务标题" --goal "任务目标") [--module "模块路径"] [--owner "负责人"] [--clarify] [--execute]
EOF
}

print_impact_scope() {
  local module="$1"
  while IFS= read -r scope; do
    [[ -n "$scope" ]] && printf -- '- %s\n' "$scope"
  done < <(impact_scope_for_module "$module")
}

print_risk_points() {
  local module="$1"
  while IFS= read -r item; do
    [[ -n "$item" ]] && printf -- '- %s\n' "$item"
  done < <(risk_points_for_module "$module")
}

is_error_diagnosis_request() {
  # 中文注释：包含固定短语时按只读报错诊断处理，不自动创建任务卡。
  local text="$1"
  [[ "$text" == *"检查为什么报错"* ]]
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
    --clarify)
      FORCE_CLARIFY="true"
      shift
      ;;
    --execute)
      EXECUTE="true"
      shift
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

if [[ -z "$TITLE" || -z "$GOAL" ]]; then
  echo "缺少必要参数：(--title 与 --goal) 或 --requirement" >&2
  usage
  exit 2
fi

if [[ -z "$MODULE" ]]; then
  MODULE="$(infer_module "${TITLE} ${GOAL}")"
fi

reason="$(clarification_reason_for_module "$MODULE")"
needs_clarify="false"
error_diagnosis="false"

if is_error_diagnosis_request "$TITLE $GOAL"; then
  error_diagnosis="true"
fi

if [[ "$error_diagnosis" != "true" && ( "$MODULE" == "unknown" || "$FORCE_CLARIFY" == "true" || -n "$reason" ) ]]; then
  needs_clarify="true"
fi

echo "=== 需求分析 ==="
echo "任务需求：$GOAL"
echo "建议模块：$MODULE"
echo "当前负责人：$OWNER"
if [[ "$error_diagnosis" == "true" ]]; then
  echo "任务类型：只读报错诊断（不创建任务卡、不修改代码）"
fi
echo "影响范围："
print_impact_scope "$MODULE"
echo "主要风险点："
print_risk_points "$MODULE"
echo "是否需要先沟通：$needs_clarify"

if [[ "$needs_clarify" == "true" ]]; then
  echo ""
  echo "=== 澄清优先 ==="
  if [[ -n "$reason" ]]; then
    echo "原因：$reason"
  else
    echo "原因：你要求先做澄清。"
  fi
  echo "请先参考：memory/07-ai/clarification-template.md"
  exit 0
fi

if [[ "$EXECUTE" != "true" ]]; then
  echo ""
  echo "=== 需求分析完成 ==="
  echo "建议先在 Codex 聊天窗口确认以上分析。"
  echo "建议下一步："
  if [[ "$error_diagnosis" == "true" ]]; then
    echo "- 直接读取相关上下文，检查为什么报错，并输出检查结果；不要创建任务卡"
  else
    echo "- 如果分析正确，继续执行：bash memory/scripts/architect-entry.sh --requirement \"$GOAL\" --module \"$MODULE\" --execute"
  fi
  echo "- 如果边界不清，先参考：memory/07-ai/clarification-template.md"
  exit 0
fi

if [[ "$error_diagnosis" == "true" ]]; then
  echo ""
  echo "=== 只读报错诊断 ==="
  echo "该请求包含“检查为什么报错”，不创建任务卡。请直接读取相关上下文、检查错误原因并输出检查结果。"
  exit 0
fi

echo ""
echo "=== 创建任务卡 ==="
bash "$SCRIPT_DIR/start-task.sh" --requirement "$GOAL" --module "$MODULE" --owner "$OWNER" --skip-summary

echo ""
echo "=== 收口提醒 ==="
echo "- 提交前先看：memory/07-ai/pre-submit-checklist.md"
echo "- 完成标准先看：memory/07-ai/definition-of-done.md"
echo "- 任务完成后执行：bash memory/scripts/complete-task.sh memory/08-tasks/open/<任务卡>.md \"完成摘要\""
