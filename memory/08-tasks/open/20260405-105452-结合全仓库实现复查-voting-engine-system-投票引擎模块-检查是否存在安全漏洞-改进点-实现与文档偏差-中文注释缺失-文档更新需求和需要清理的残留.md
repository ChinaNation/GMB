# 任务卡：结合全仓库实现复查 voting-engine-system 投票引擎模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

- 任务编号：20260405-105452
- 状态：open
- 所属模块：citizenchain/runtime/governance/voting-engine-system
- 当前负责人：Codex
- 创建时间：2026-04-05 10:54:52

## 任务需求

结合全仓库实现复查 voting-engine-system 投票引擎模块，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/voting-engine-system/VOTINGENGINE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/runtime-root-upgrade/RUNTIMEROOT_TECHNICAL.md

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
- 已复查 `citizenchain/runtime/governance/voting-engine-system`
- 已复查 `citizenchain/runtime/governance/runtime-root-upgrade`
- 已复查 `citizenchain/runtime/src/configs/mod.rs`
- 已复查 `citizenchain/node/src/ui/governance/proposal.rs`
- 已复查 `citizenchain/node/frontend/governance/ProposalDetailPage.tsx`
- 已复查 `memory/scripts/load-context.sh`
- 已修复 node 治理页对 runtime 升级提案的终态展示偏差：不再把 `Rejected/ExecutionFailed` 统一显示成 `已执行`
- 已修复 `runtime-root-upgrade` benchmark 对带 `MODULE_TAG` 的 `ProposalData` 误解码残留
- 已修复 `voting-engine-system` 的 `create_internal_proposal` benchmark 仍走旧手工写存储路径的残留
- 已修复 `runtime-root-upgrade` 单测、注释与技术文档的提案发起权限口径偏差
- 已修复 `runtime-root-upgrade` 技术文档对投票引擎状态机的过期描述
- 已修复 `DeveloperUpgradePage` 仅展示 NRC 管理员的前端入口偏差
- 已修复 `wuminapp/governance` 文档中 `runtime-root-upgrade` 摘要字段仍保留 `has_code` 的残留
- 已补 `memory/scripts/load-context.sh` 中 `runtime-root-upgrade` 模块入口
- 已更新 `RUNTIMEROOT_TECHNICAL.md`
- 已完成 `cargo test --offline --manifest-path citizenchain/runtime/governance/runtime-root-upgrade/Cargo.toml -- --nocapture`
- 已完成 `cargo test --offline --manifest-path citizenchain/runtime/governance/voting-engine-system/Cargo.toml -- --nocapture`
- 已完成 `cargo check --offline --manifest-path citizenchain/Cargo.toml -p runtime-root-upgrade --features runtime-benchmarks`
- 已完成 `cargo check --offline --manifest-path citizenchain/Cargo.toml -p voting-engine-system --features runtime-benchmarks`
- 已完成 `npm run build`（`citizenchain/node/frontend`）
- 已尝试 `cargo check --offline --manifest-path citizenchain/Cargo.toml -p node`
  - 结果：失败
  - 原因：`citizenchain/runtime/governance/admins-origin-gov/src/lib.rs` 中 `MaxAdminsPerInstitution` 与 `voting_engine_system::Config` 同名，触发关联类型歧义；该失败点不在本轮修改文件内，但会阻塞 node 包级验证

## 当前结论

### 已修复

1. runtime 升级提案在 node 治理列表和详情页里可能把 `Rejected` / `ExecutionFailed` 误显示成 `已执行`
   - 根因：UI 不能只依赖投票引擎通用 `Proposals.status`；`runtime-root-upgrade` 的真实业务终态仍以 `ProposalData` 内的 `ProposalStatus` 为准
   - 现状：已改为优先使用 `ProposalData` 里的业务 `ProposalStatus` 做用户展示

2. benchmark 残留导致运行时升级模块的基准断言不可靠
   - 根因：`runtime-root-upgrade` benchmark 直接按裸结构解码带 `MODULE_TAG` 的 `ProposalData`
   - 现状：已改为显式跳过 `MODULE_TAG` 后再解码

3. `voting-engine-system` 的 `create_internal_proposal` benchmark 仍沿用旧手工写存储路径
   - 根因：benchmark 没有走真实 `do_create_internal_proposal` 逻辑，无法覆盖当前提案 ID、活跃限额、管理员快照和 expiry 注册路径
   - 现状：已切回真实创建路径

4. `runtime-root-upgrade` 的提案发起权限口径曾存在实现/文档分歧
   - 已确认规则：国储会和 43 个省储会管理员都能发起联合提案
   - 现状：`runtime-root-upgrade` 的单测、代码注释和技术文档已统一到 `EnsureJointProposer` 口径

5. `runtime-root-upgrade` 的技术文档曾误写投票引擎状态机
   - 根因：文档仍沿用“所有终态统一写成 `STATUS_EXECUTED`”的旧描述
   - 现状：已按源码现状更新为“成功保持 `STATUS_PASSED`、否决保持 `STATUS_REJECTED`、执行失败覆写 `STATUS_EXECUTION_FAILED`”

6. 开发期 Runtime 升级页曾把省储会管理员排除在外
   - 根因：`DeveloperUpgradePage` 只拉取了国储会的已激活管理员
   - 现状：已改为汇总国储会 + 43 个省储会的已激活管理员，和 runtime `EnsureJointProposer` 口径一致
