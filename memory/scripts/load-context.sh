#!/usr/bin/env bash
set -euo pipefail

# 中文注释：按模块输出建议阅读的上下文文件顺序。

MODULE="${1:-global}"

print_if_exists() {
  local path="$1"
  if [[ -f "$path" ]]; then
    printf '%s\n' "$path"
  fi
}

print_base_context() {
  print_if_exists "memory/00-vision/project-goal.md"
  print_if_exists "memory/00-vision/trust-boundary.md"
  print_if_exists "memory/01-architecture/repo-map.md"
  print_if_exists "memory/03-security/security-rules.md"
  print_if_exists "memory/07-ai/agent-rules.md"
  print_if_exists "memory/07-ai/context-loading-order.md"
  print_if_exists "memory/07-ai/workflow.md"
}

case "$MODULE" in
  global)
    print_base_context
    ;;
  ai/system)
    print_base_context
    print_if_exists "memory/AGENTS.md"
    print_if_exists "memory/CODEX.md"
    print_if_exists "memory/CLAUDE.md"
    print_if_exists "memory/07-ai/ai-system-overview.md"
    print_if_exists "memory/07-ai/document-boundaries.md"
    print_if_exists "memory/07-ai/startup-acceptance.md"
    ;;
  citizenchain/runtime|citizenchain/governance|citizenchain/issuance|citizenchain/otherpallet|citizenchain/transaction)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    ;;
  citizenchain/node)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/node/Cargo.toml"
    ;;
  citizenchain/nodeui|citizenchain/nodeuitauri)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "citizenchain/nodeui/README.md"
    ;;
  sfid|sfid/backend|sfid/frontend|sfid/deploy)
    print_base_context
    print_if_exists "sfid/README.md"
    print_if_exists "sfid/SFID_TECHNICAL.md"
    print_if_exists "sfid/backend/src/BUSINESS_TECHNICAL.md"
    print_if_exists "sfid/backend/src/business/BUSINESS_TECHNICAL.md"
    ;;
  cpms|cpms/backend|cpms/frontend|cpms/deploy)
    print_base_context
    print_if_exists "cpms/README.md"
    print_if_exists "cpms/CPMS_TECHNICAL.md"
    ;;
  wuminapp|wuminapp/lib)
    print_base_context
    print_if_exists "wuminapp/WUMINAPP_TECHNICAL.md"
    ;;
  primitives)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    ;;
  *)
    print_base_context
    echo "# 未识别的模块：$MODULE"
    echo "# 请手动补充该模块技术文档"
    ;;
esac
