# 任务卡：修复管理员更换创建事务与最终事件重复

- 任务编号：20260430-154307
- 状态：done
- 所属模块：voting-engine / admins-change / 治理与交易提案消费模块
- 当前负责人：Codex
- 创建时间：2026-04-30 15:43:07

## 任务需求

修复 Review Finding 2 和 Finding 3：

- 管理员更换提案创建必须把投票引擎提案、互斥锁、业务数据、业务元数据和事件放进同一个链上事务，避免中途失败留下无业务数据的提案或锁。
- 回调执行产生的最终状态必须静默写入，由投票引擎外层 `set_status_and_emit` 统一发出一次 `ProposalFinalized` 事件，避免链下索引看到重复终结事件。

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

- 本任务不修复状态机强约束问题，`STATUS_EXECUTION_FAILED` 是否允许重试另行确认。
- `admins-change` 创建提案必须全部成功或全部回滚。
- 回调内只能静默记录业务执行最终状态，不直接发最终事件。
- 外层投票引擎统一负责最终事件、清理登记和互斥锁释放。
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
- `admins-change::propose_admin_replacement` 已用 `with_transaction` 包裹投票引擎提案创建、互斥锁、业务 `ProposalData`、`ProposalMeta` 和 `AdminReplacementProposed` 事件。
- `voting-engine` 已新增回调专用 `set_callback_execution_result`，回调只静默写入 `STATUS_EXECUTED / STATUS_EXECUTION_FAILED`。
- `set_status_and_emit` 继续作为外层统一终结入口，负责读取回调后的最终状态并只发一次 `ProposalFinalized`、登记清理、释放互斥锁。
- 已将 `admins-change`、`resolution-destro`、`grandpakey-change`、`duoqian-transfer`、`duoqian-manage`、`runtime-upgrade`、`resolution-issuance` 的回调执行结果改接静默 API。
- 已清理 `duoqian-manage` 中重构后不再使用的内部包装 helper。
- 已更新 `VOTINGENGINE_TECHNICAL.md` 与 `ADMINSCHANGE_TECHNICAL.md`。

## 验证记录

- `cargo fmt --manifest-path citizenchain/Cargo.toml --package voting-engine --package admins-change --package resolution-destro --package grandpakey-change --package duoqian-transfer --package duoqian-manage --package runtime-upgrade --package resolution-issuance`
- `cargo test -p voting-engine --lib`
- `cargo test -p admins-change --lib`
- `cargo test -p voting-engine --lib --features runtime-benchmarks`
- `cargo test -p admins-change --lib --features runtime-benchmarks`
- `cargo test -p resolution-destro --lib`
- `cargo test -p grandpakey-change --lib`
- `cargo test -p duoqian-transfer --lib`
- `cargo test -p duoqian-manage --lib`
- `cargo test -p runtime-upgrade --lib`
- `cargo test -p resolution-issuance --lib`
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo check -p citizenchain`
- `git diff --check`
