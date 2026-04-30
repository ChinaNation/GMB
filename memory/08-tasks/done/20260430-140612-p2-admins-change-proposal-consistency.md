# 任务卡：修复 admins-change 执行缺少提案一致性校验

- 任务编号：20260430-140612
- 状态：done
- 所属模块：admins-change / voting-engine
- 当前负责人：Codex
- 创建时间：2026-04-30 14:06:12

## 任务需求

修复 Review Finding 2：管理员更换执行路径不能只校验提案状态和 ProposalData，还必须校验投票引擎提案元数据与管理员更换动作一致。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不可绕过投票引擎提案状态机
- 执行管理员替换前必须校验 proposal kind / stage / org / institution 与业务 action 一致
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 测试
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已在 `admins-change` 执行路径补充投票引擎提案元数据校验：
  - `proposal.kind == PROPOSAL_KIND_INTERNAL`
  - `proposal.stage == STAGE_INTERNAL`
  - `proposal.internal_institution == Some(action.institution)`
  - `proposal.internal_org == Some(subject.org)`
- 已补充执行路径防御测试：
  - 拒绝错误提案 kind / stage
  - 拒绝提案 institution / org 与管理员更换动作不一致
- 已更新 `ADMINSCHANGE_TECHNICAL.md`，记录执行一致性校验边界与测试覆盖。
- 已完成残留检查与验证。

## 验证记录

- `cargo fmt --manifest-path citizenchain/Cargo.toml --package admins-change`
- `cargo test -p admins-change --lib`
- `cargo test -p admins-change --lib --features runtime-benchmarks`
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo check -p citizenchain`
- `git diff --check`
