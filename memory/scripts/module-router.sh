#!/usr/bin/env bash

# 中文注释：统一维护 AI 系统的模块路由规则。

infer_module() {
  local text="$1"
  local lower
  lower="$(printf '%s' "$text" | tr '[:upper:]' '[:lower:]')"

  case "$lower" in
    *ai编程系统*|*ai*system*|*task*card*|*任务卡*|*guardrail*|*workflow*|*claude*review*|*启动协议*|*需求分析协议*|*memory/*)
      printf '%s' "ai/system"
      ;;
    *permit*|*sfid*|*绑定*|*验签*|*身份*|*授权*|*登录*)
      printf '%s' "sfid/backend"
      ;;
    *cpms*|*实名*|*档案*|*柜员*|*审批*|*审计*|*二维码签发*)
      printf '%s' "cpms/backend"
      ;;
    *wallet*|*钱包*|*手机*app*|*手机端*|*isar*|*转账*|*扫码登录*)
      printf '%s' "wuminapp"
      ;;
    *nodeui*|*节点ui*|*桌面端*|*挖矿界面*|*节点状态界面*|*flutter*desktop*)
      printf '%s' "citizenchain/nodeui"
      ;;
    *node*|*节点*|*出块*|*同步*|*cli*|*矿工*|*挖矿*)
      printf '%s' "citizenchain/node"
      ;;
    *runtime*|*pallet*|*治理*|*投票*|*提案*|*发行*|*交易规则*|*手续费*|*链上资格*)
      printf '%s' "citizenchain/runtime"
      ;;
    *primitive*|*常量*|*基础类型*)
      printf '%s' "primitives"
      ;;
    *)
      printf '%s' "unknown"
      ;;
  esac
}

impact_scope_for_module() {
  local module="$1"

  case "$module" in
    ai/system)
      printf '%s\n' ".github" "memory" "AGENTS.md" "CODEX.md" "CLAUDE.md"
      ;;
    citizenchain/runtime|citizenchain/governance|citizenchain/issuance|citizenchain/otherpallet|citizenchain/transaction)
      printf '%s\n' "citizenchain/runtime" "citizenchain/node" "memory"
      ;;
    citizenchain/node)
      printf '%s\n' "citizenchain/node" "memory"
      ;;
    citizenchain/nodeui|citizenchain/nodeuitauri)
      printf '%s\n' "citizenchain/nodeui" "citizenchain/nodeuitauri" "memory"
      ;;
    sfid|sfid/backend)
      printf '%s\n' "sfid/backend" "memory"
      ;;
    sfid/frontend)
      printf '%s\n' "sfid/frontend" "memory"
      ;;
    cpms|cpms/backend)
      printf '%s\n' "cpms/backend" "memory"
      ;;
    cpms/frontend)
      printf '%s\n' "cpms/frontend" "memory"
      ;;
    wuminapp|wuminapp/lib)
      printf '%s\n' "wuminapp" "memory"
      ;;
    primitives)
      printf '%s\n' "primitives" "citizenchain/runtime" "citizenchain/node" "memory"
      ;;
    *)
      printf '%s\n' "memory"
      ;;
  esac
}

clarification_reason_for_module() {
  local module="$1"

  case "$module" in
    unknown)
      printf '%s' "未能可靠判断目标模块，需要先澄清边界。"
      ;;
    primitives)
      printf '%s' "这是共享目录改动，开工前应先确认影响范围。"
      ;;
    *)
      printf '%s' ""
      ;;
  esac
}

risk_points_for_module() {
  local module="$1"

  case "$module" in
    ai/system)
      printf '%s\n' \
        "启动协议、任务卡和门禁规则一旦漂移，新线程就可能不再接入 AI 编程系统。" \
        "这类改动必须同步更新 memory 文档、入口协议和 GitHub 门禁。"
      ;;
    citizenchain/runtime|citizenchain/governance|citizenchain/issuance|citizenchain/otherpallet|citizenchain/transaction)
      printf '%s\n' \
        "链上规则变更可能影响治理、发行、交易和资格模型。" \
        "如果涉及存储或 feature 组合，必须先确认兼容性和边界。"
      ;;
    citizenchain/node)
      printf '%s\n' \
        "节点行为可能与 runtime、安装包和启动流程联动。" \
        "不能在 node 层越权决定链上规则。"
      ;;
    citizenchain/nodeui|citizenchain/nodeuitauri)
      printf '%s\n' \
        "节点 UI 只能承载交互，不能把链规则固化在界面层。" \
        "新版 nodeui 与旧版 nodeuitauri 的迁移边界要先分清。"
      ;;
    sfid|sfid/backend|sfid/frontend|sfid/deploy)
      printf '%s\n' \
        "SFID 不得保存原始实名，也不能替代 CPMS 成为实名信任根。" \
        "接口、permit、绑定规则变化容易波及 App 和链边界。"
      ;;
    cpms|cpms/backend|cpms/frontend|cpms/deploy)
      printf '%s\n' \
        "CPMS 永不联网，二维码也不能包含实名原文。" \
        "本地审批、签发和存储结构变化必须非常谨慎。"
      ;;
    wuminapp|wuminapp/lib)
      printf '%s\n' \
        "App 只是交互入口，不能承担信任裁决。" \
        "Isar 结构或认证流程变化会直接影响用户路径。"
      ;;
    primitives)
      printf '%s\n' \
        "共享常量和基础类型会同时波及 runtime 与 node。" \
        "这类改动属于共享变更，开工前必须先看影响范围。"
      ;;
    unknown)
      printf '%s\n' \
        "当前无法可靠判断模块，直接实现容易越界或误改目录。"
      ;;
    *)
      printf '%s\n' \
        "请先确认该模块的边界、文档和影响范围。"
      ;;
  esac
}
