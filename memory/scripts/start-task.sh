#!/usr/bin/env bash
set -euo pipefail

# 中文注释：快速建任务入口，支持任务需求优先。

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=memory/scripts/module-router.sh
source "$SCRIPT_DIR/module-router.sh"

TITLE=""
MODULE=""
GOAL=""
OWNER="Codex"
SKIP_SUMMARY="false"
REQUIREMENT=""

usage() {
  cat <<'EOF'
用法：
  bash memory/scripts/start-task.sh (--title "任务标题" --goal "任务目标" | --requirement "任务需求") [--module "模块路径"] [--owner "负责人"] [--skip-summary]
EOF
}

is_error_diagnosis_request() {
  # 中文注释：只读报错诊断不进入任务卡流程。
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
    --skip-summary)
      SKIP_SUMMARY="true"
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

if is_error_diagnosis_request "$TITLE $GOAL"; then
  echo "只读报错诊断请求：不创建任务卡，请直接检查为什么报错并输出检查结果。"
  exit 0
fi

if [[ -z "$MODULE" ]]; then
  MODULE="$(infer_module "${TITLE} ${GOAL}")"
fi

if [[ "$SKIP_SUMMARY" != "true" ]]; then
  echo "=== Architect 路由结果 ==="
  echo "任务需求：$GOAL"
  echo "建议模块：$MODULE"
  echo "负责人：$OWNER"
fi

if [[ "$MODULE" == "unknown" ]]; then
  echo ""
  echo "未能可靠判断所属模块。"
  echo "请补充 --module，或先人工确认边界后再创建任务。"
  exit 1
fi

if [[ "$SKIP_SUMMARY" != "true" ]]; then
  echo ""
  echo "=== 创建任务卡 ==="
fi

bash "$SCRIPT_DIR/new-task.sh" --title "$TITLE" --module "$MODULE" --goal "$GOAL" --owner "$OWNER"

if [[ "$SKIP_SUMMARY" != "true" ]]; then
  echo ""
  echo "=== 建议上下文装载 ==="
fi

bash "$SCRIPT_DIR/load-context.sh" "$MODULE"
