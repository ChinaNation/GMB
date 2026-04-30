# 任务卡：收口投票引擎状态机并删除旧状态覆盖入口

- 任务编号：20260430-161713
- 状态：done
- 所属模块：voting-engine / admins-change / resolution-destro / grandpakey-change / runtime-upgrade / resolution-issuance / duoqian-transfer
- 当前负责人：Codex
- 创建时间：2026-04-30 16:17:13

## 任务需求

修复 Review Finding 1：投票引擎状态转换缺少强约束。当前旧覆盖 API 可能允许终态重新改回非终态，或让 `EXECUTION_FAILED` 被重新推进到 `EXECUTED`。在确认无存量链、无存量提案的前提下，直接删除旧 `override_proposal_status`，并建立严格状态机。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md

## 必须遵守

- 只允许 `VOTING -> PASSED / REJECTED` 与 `PASSED -> EXECUTED / EXECUTION_FAILED`。
- `REJECTED / EXECUTED / EXECUTION_FAILED` 都是终态，进入后不可再变化。
- `PASSED` 是可执行/可重试态；需要重试的自动执行失败应保持 `PASSED`。
- 删除旧 `override_proposal_status`，不保留兼容。
- `set_callback_execution_result` 必须只能在投票引擎回调作用域内使用。
- 无存量链、无存量提案，不做 storage migration。
- 改代码后必须更新文档、补测试、清理残留。

## 输出物

- 代码
- 中文注释
- 测试
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建。
- 已在 `voting-engine` 新增严格状态机校验，只允许 `VOTING -> PASSED / REJECTED` 与 `PASSED -> EXECUTED / EXECUTION_FAILED`。
- 已删除旧 `override_proposal_status`，业务模块不再存在绕过状态机直接写状态的入口。
- 已新增 `CallbackExecutionScopes`，`set_callback_execution_result` 只能在投票引擎回调作用域内使用。
- 已补 `voting-engine` 状态机单测和非回调作用域拒绝测试。
- 已调整 `resolution-destro` 自动执行失败语义：保持 `STATUS_PASSED`，补余额后手动重试，不再走 `EXECUTION_FAILED -> EXECUTED`。
- 已调整 `grandpakey-change` 的取消失败提案语义：`PASSED -> EXECUTION_FAILED`，不再 `PASSED -> REJECTED`。
- 已调整 `runtime-upgrade` 与 `resolution-issuance` 成功回调写 `STATUS_EXECUTED`，失败回调写 `STATUS_EXECUTION_FAILED`。
- 已更新 `voting-engine`、`resolution-destro`、`grandpakey-change`、`runtime-upgrade`、`resolution-issuance`、`duoqian-transfer` 技术文档。
- 已完成旧覆盖 API 残留扫描。

## 验证记录

- `cargo fmt --manifest-path citizenchain/Cargo.toml --package voting-engine --package admins-change --package resolution-destro --package grandpakey-change --package duoqian-transfer --package duoqian-manage --package runtime-upgrade --package resolution-issuance`
- `cargo test -p voting-engine --lib`
- `cargo test -p admins-change --lib`
- `cargo test -p resolution-destro --lib`
- `cargo test -p grandpakey-change --lib`
- `cargo test -p duoqian-transfer --lib`
- `cargo test -p duoqian-manage --lib`
- `cargo test -p runtime-upgrade --lib`
- `cargo test -p resolution-issuance --lib`
- `cargo test -p voting-engine --lib --features runtime-benchmarks`
- `cargo test -p admins-change --lib --features runtime-benchmarks`
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo check -p citizenchain`
- `rg -n "override_proposal_status" citizenchain || true`
- `git diff --check`
