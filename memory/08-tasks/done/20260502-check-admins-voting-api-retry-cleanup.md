# 任务卡：检查 admins-change 生命周期 API 与 voting-engine retry/cleanup 风险

- 任务编号：20260502-check-admins-voting-api-retry-cleanup
- 状态：done
- 所属模块：citizenchain/runtime/governance/admins-change, citizenchain/runtime/governance/voting-engine
- 当前负责人：Codex
- 创建时间：2026-05-02

## 任务需求

只读检查用户列出的 3 项问题是否存在，评估影响，并给出推荐修复方案。

## 检查项

- 高：`admins-change` 生命周期 API 是否为 `pub fn` 且可直接 mutate `Institutions`。
- 中：`voting-engine` retry deadline 重排失败是否被静默忽略，导致 PASSED 提案永久卡死。
- 中：`CleanupQueue` 单桶 50 硬编码是否可能让 finalize 在同一区块满载时回滚。

## 输出物

- 存在性判断
- 影响评估
- 推荐修复方案

## 实施记录

- 任务卡已创建。
- 2026-05-02：已核查 `admins-change` 生命周期函数、`duoqian-manage` 调用点、`voting-engine` retry deadline 重排路径、cleanup 队列定义与相关测试/技术文档。

## 检查结果

- 高风险项成立：`create_pending_subject` / `activate_subject` / `remove_pending_subject` / `close_subject` 是 `pub fn`，直接写 `Institutions`，当前调用点主要在 `duoqian-manage`，但 API 边界依赖调用契约。
- 中风险项成立：`process_execution_retry_deadlines` 在终态清理登记失败后会重排 retry deadline，但重排连续 100 个 deadline 桶失败时返回 `false`，调用方 `let _ = ...` 静默忽略，可能留下无 deadline 队列引用的 `STATUS_PASSED + ProposalExecutionRetryStates`。
- 中风险项部分成立：`CleanupQueue` 单桶确实硬编码 `ConstU32<50>`，`schedule_cleanup` 连续 100 桶全满时回滚终态写入。生产配置 `MaxProposalsPerExpiry = 2048`，而 cleanup 窗口总容量为 5000，因此单个 expiry 桶不会立即打满，但硬编码容量和治理配置不一致，仍值得参数化。

## 影响评估

- `admins-change` 生命周期 API 当前不是 extrinsic，直接攻击面不在链外用户，而在未来 runtime pallet 误用或绕过投票回调。
- retry deadline 重排失败后不会自动进入 `EXECUTION_FAILED`，管理员仍可手动 retry/cancel，但若无人介入，提案会长期停在 `PASSED`，并保留 retry state 与内部互斥锁。
- cleanup 桶满导致 finalize 回滚是当前文档化设计，用于避免“终态但无清理入口”。风险主要是高并发终态结算时出现临时不可终结，需要参数化降低容量毛刺。

## 推荐处理

- `admins-change`：收窄 4 个 mutator 的直接可见性；暴露专用 `SubjectLifecycle` trait 时增加投票引擎状态/回调作用域校验，不建议只靠调用方传入枚举伪装 caller 断言。
- retry deadline：把 `reschedule_execution_retry_deadline` 从 `bool` 改为 `DispatchResult`，失败时写入可观测 backlog/stuck 队列并由 `on_initialize` 继续重试，或显式进入受控的 terminal-cleanup 待处理状态，不能静默丢失 deadline 引用。
- cleanup：新增 `MaxCleanupQueueBucketLimit` 与 `MaxCleanupScheduleOffset` 配置，生产值建议与 `MaxProposalsPerExpiry = 2048` 同量级；同步测试、文档和权重/benchmark 注释。
