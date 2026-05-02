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
  citizenchain/runtime/issuance/citizen-issuance|citizenchain/issuance/citizen-issuance)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/issuance/citizen-issuance/CITIZENISS_TECHNICAL.md"
    ;;
  citizenchain/runtime/issuance/fullnode-issuance|citizenchain/issuance/fullnode-issuance)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/issuance/fullnode-issuance/FULLNODE_TECHNICAL.md"
    ;;
  citizenchain/runtime/issuance/shengbank-interest|citizenchain/issuance/shengbank-interest)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/issuance/shengbank-interest/SHENGBANK_TECHNICAL.md"
    ;;
  citizenchain/runtime/otherpallet/sfid-system|citizenchain/otherpallet/sfid-system|sfid-system)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/otherpallet/sfid-system/SFID_SYSTEM_TECHNICAL.md"
    ;;
  citizenchain/runtime/otherpallet/pow-difficulty|citizenchain/otherpallet/pow-difficulty|pow-difficulty)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/otherpallet/pow-difficulty/POW_DIFFICULTY_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/grandpakey-change|citizenchain/governance/grandpakey-change|grandpakey-change)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/grandpakey-change/GRANDPAKEYCHANGE_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/admins-change|citizenchain/governance/admins-change|admins-change)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/resolution-destro|citizenchain/governance/resolution-destro|resolution-destro)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/resolution-destro/RESOLUTIONDESTRO_TECHNICAL.md"
    ;;
  citizenchain/runtime/issuance/resolution-issuance|citizenchain/issuance/resolution-issuance|resolution-issuance)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/issuance/resolution-issuance/RESOLUTIONISSUANCE_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/voting-engine|citizenchain/governance/voting-engine|voting-engine)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md"
    ;;
  citizenchain/runtime/governance/runtime-upgrade|citizenchain/governance/runtime-upgrade|runtime-upgrade)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/governance/runtime-upgrade/RUNTIMEUPGRADE_TECHNICAL.md"
    ;;
  citizenchain/runtime/transaction/duoqian-transfer|citizenchain/transaction/duoqian-transfer|duoqian-transfer)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md"
    ;;
  citizenchain/runtime/transaction/duoqian-manage|citizenchain/transaction/duoqian-manage|duoqian-manage)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/duoqian-manage/DUOQIAN_TECHNICAL.md"
    ;;
  citizenchain/runtime/transaction/institution-asset|citizenchain/transaction/institution-asset|institution-asset)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/institution-asset/INSTITUTION_ASSET_TECHNICAL.md"
    ;;
  citizenchain/runtime/transaction/offchain-transaction|citizenchain/transaction/offchain-transaction|offchain-transaction)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/offchain-transaction/STEP1_TECHNICAL.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/offchain-transaction/STEP2A_RUNTIME.md"
    ;;
  citizenchain/runtime/transaction/onchain-transaction|citizenchain/transaction/onchain-transaction|onchain-transaction)
    print_base_context
    print_if_exists "memory/01-architecture/citizenchain-target-structure.md"
    print_if_exists "memory/07-ai/ci-path-routing.md"
    print_if_exists "citizenchain/CITIZENCHAIN_TECHNICAL.md"
    print_if_exists "citizenchain/runtime/README.md"
    print_if_exists "memory/05-modules/citizenchain/runtime/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md"
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
  sfid|sfid/backend|sfid/frontend|sfid/deploy)
    print_base_context
    print_if_exists "sfid/README.md"
    print_if_exists "sfid/SFID_TECHNICAL.md"
    print_if_exists "memory/05-modules/sfid/backend/BACKEND_LAYOUT.md"
    print_if_exists "memory/05-modules/sfid/backend/business/BUSINESS_TECHNICAL.md"
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
