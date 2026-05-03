# 任务卡：修复 admins-change 生命周期 API 与 voting-engine retry/cleanup 风险

- 任务编号：20260502-fix-admins-lifecycle-voting-retry-cleanup
- 状态：done
- 所属模块：citizenchain/runtime/governance/admins-change, citizenchain/runtime/governance/voting-engine, citizenchain/runtime/transaction/duoqian-manage
- 当前负责人：Codex
- 创建时间：2026-05-02

## 任务需求

按确认方案修复 3 个问题：

1. `admins-change` 生命周期 API 裸 `pub fn` 暴露，依赖调用契约保护。
2. `voting-engine` retry deadline 重排失败被静默忽略，可能丢失 deadline 引用。
3. `CleanupQueue` 单桶 `ConstU32<50>` 和 100 桶搜索窗口硬编码。

## 计划

- 收口 `admins-change` 4 个生命周期 mutator，改为带生命周期作用域校验的专用 trait 入口。
- `duoqian-manage` 改用新 trait 入口，不再直接调用裸 mutator。
- 修复 retry deadline 重排失败路径，确保失败可观测且不会丢失后续处理入口。
- 参数化 cleanup bucket 容量与搜索窗口，更新 runtime 配置。
- 补充单元测试，更新技术文档，清理残留。

## 实施记录

- 任务卡已创建。
- `admins-change` 增加 `SubjectLifecycle` trait，4 个生命周期写状态入口收口为 crate 内 `do_*`，跨 pallet 调用必须携带 voting-engine 提案上下文。
- `duoqian-manage` 创建个人/机构多签时改用 `create_pending_subject_internal_proposal_with_snapshot_data` 先固化投票快照，再在同一事务内写 Pending 主体、proposal data、reserve/索引。
- `voting-engine` 新增 `PendingExecutionRetryExpirations`，retry deadline 初次登记或重排失败时进入待处理队列，后续 `on_initialize` 兜底转 `STATUS_EXECUTION_FAILED`。
- `CleanupQueue` 单桶容量和 cleanup 搜索窗口改为 runtime 配置项，生产 runtime 配置为 512 / 1024。
- 相关 mock/runtime 配置补齐新参数，grandpakey/resolution-destro 测试 mock 对齐新投票凭证签名参数。
- 技术文档已更新：`ADMINSCHANGE_TECHNICAL.md`、`VOTINGENGINE_TECHNICAL.md`、`DUOQIAN_TECHNICAL.md`。

## 验证记录

- `cargo test -p voting-engine --lib`：通过，74 passed。
- `cargo test -p admins-change --lib`：通过，28 passed。
- `cargo test -p duoqian-manage --lib`：通过，26 passed。
- `cargo test -p resolution-issuance --lib`：通过，16 passed。
- `cargo check -p grandpakey-change -p runtime-upgrade -p resolution-destro -p duoqian-transfer --tests`：通过。
- `cargo check --manifest-path citizenchain/runtime/Cargo.toml`：被 runtime `build.rs` 拦截，原因是本地未设置 `WASM_FILE`，当前仓库规则禁止本地编译统一 WASM。
