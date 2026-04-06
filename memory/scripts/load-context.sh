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
  citizenchain/runtime/issuance/citizen-lightnode-issuance|citizenchain/issuance/citizen-lightnode-issuance)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/issuance/citizen-lightnode-issuance/CITIZENISS_TECHNICAL.md"
    ;;
  citizenchain/runtime/otherpallet/sfid-code-auth|citizenchain/otherpallet/sfid-code-auth|sfid-code-auth)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/otherpallet/sfid-code-auth/SFIDCODEAUTH_TECHNICAL.md"
    ;;
  citizenchain/runtime/otherpallet/pow-difficulty-module|citizenchain/otherpallet/pow-difficulty-module|pow-difficulty-module)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/otherpallet/pow-difficulty-module/POW_DIFFICULTY_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/grandpa-key-gov|citizenchain/governance/grandpa-key-gov|grandpa-key-gov)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/grandpa-key-gov/GRANDPAKEYGOV_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/admins-origin-gov|citizenchain/governance/admins-origin-gov|admins-origin-gov)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/admins-origin-gov/ADMINSORIGIN_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/resolution-destro-gov|citizenchain/governance/resolution-destro-gov|resolution-destro-gov)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/resolution-destro-gov/RESOLUTIONDESTRO_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/resolution-issuance-gov|citizenchain/governance/resolution-issuance-gov|resolution-issuance-gov)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/resolution-issuance-gov/RESOLUTIONISSUANCEGOV_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/voting-engine-system|citizenchain/governance/voting-engine-system|voting-engine-system)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/voting-engine-system/VOTINGENGINE_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/runtime-root-upgrade|citizenchain/governance/runtime-root-upgrade|runtime-root-upgrade)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/runtime-root-upgrade/RUNTIMEROOT_TECHNICAL.md"
    ;;
  citizenchain/runtime/transaction/duoqian-transfer-pow|citizenchain/transaction/duoqian-transfer-pow|duoqian-transfer-pow)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer-pow/DUOQIAN_TRANSFER_TECHNICAL.md"
    ;;
  citizenchain/runtime/transaction/institution-asset-guard|citizenchain/transaction/institution-asset-guard|institution-asset-guard)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/institution-asset-guard/INSTITUTION_ASSET_GUARD_TECHNICAL.md"
    ;;
  citizenchain/runtime/genesis|citizenchain/genesis|genesis-pallet)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/genesis/GENESIS_TECHNICAL.md"
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
    print_if_exists "memory/05-modules/citizenchain/node/NODE_TECHNICAL.md"
    ;;
  citizenchain/nodeui)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md"
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
