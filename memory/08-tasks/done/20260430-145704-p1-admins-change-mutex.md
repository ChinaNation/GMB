# 任务卡：修复管理员更换与同主体提案互斥

- 任务编号：20260430-145704
- 状态：done
- 所属模块：voting-engine / admins-change
- 当前负责人：Codex
- 创建时间：2026-04-30 14:57:04

## 任务需求

修复 Review Finding 1：管理员更换提案必须与同一治理主体下的其他活跃提案互斥，避免管理员集合变更和依赖旧管理员快照的提案并行，并消除自动执行失败后旧 PASSED 提案跨时期复活执行的风险。

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

- voting-engine 负责通用互斥锁、状态机与锁释放。
- admins-change 只声明管理员集合变更提案，并在执行前校验锁 owner。
- 同一 `org + institution` 下管理员更换与普通活跃提案互斥。
- 普通内部提案之间默认不互斥。
- 联合投票在管理员参与阶段占用普通锁，进入公民投票阶段释放锁。
- 自动执行失败必须进入 `STATUS_EXECUTION_FAILED` 终态，不保留可重试的 `PASSED`。
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
- 已在 `voting-engine` 新增内部提案互斥模型：
  - `InternalProposalMutexKind`
  - `InternalProposalMutexState`
  - `InternalProposalMutexes`
  - `ProposalMutexBindings`
- 已新增管理员集合变更专用创建入口 `create_admin_set_mutation_internal_proposal`。
- 普通内部提案与 Pending 主体创建提案登记 `Regular` 锁。
- 管理员更换提案登记 `AdminSetMutationExclusive` 锁。
- 联合投票 `STAGE_JOINT` 创建时为所有参与机构登记 `Regular` 锁；进入 `STAGE_CITIZEN` 时释放。
- `admins-change` 已改为使用管理员集合变更专用入口，并在执行前校验独占锁 owner。
- 管理员更换自动执行失败已改为 `STATUS_EXECUTION_FAILED` 终态，释放独占锁且不允许手动重试。
- 当前生产链已确认无活跃未终态提案，新增互斥存储以空状态启用；如未来在有活跃存量提案的链上升级，需要另补一次性锁重建迁移。
- 已更新 `VOTINGENGINE_TECHNICAL.md` 与 `ADMINSCHANGE_TECHNICAL.md`。
- 已完成残留扫描与验证。

## 验证记录

- `cargo fmt --manifest-path citizenchain/Cargo.toml --package voting-engine --package admins-change`
- `cargo test -p voting-engine --lib`
- `cargo test -p admins-change --lib`
- `cargo test -p voting-engine --lib --features runtime-benchmarks`
- `cargo test -p admins-change --lib --features runtime-benchmarks`
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/ci-wasm/citizenchain.compact.compressed.wasm cargo check -p citizenchain`
- `cargo test -p duoqian-manage --lib`
- `git diff --check`
