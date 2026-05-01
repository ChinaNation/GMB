# 任务卡：检查 governance proposal_cleanup schedule_cleanup 连续队列满后是否导致提案存储永久不清理

- 任务编号：20260430-185906
- 状态：open
- 所属模块：citizenchain-runtime-governance
- 当前负责人：Codex
- 创建时间：2026-04-30 18:59:06

## 任务需求

检查 governance proposal_cleanup schedule_cleanup 连续队列满后是否导致提案存储永久不清理

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 只读检查结论
- 风险判断
- 后续修复建议

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已读取 `citizenchain/runtime/governance/voting-engine/src/proposal_cleanup.rs` 与 `citizenchain/runtime/governance/voting-engine/src/lib.rs`。
- 检查结论：用户指出的 `schedule_cleanup` 队列连续 100 区块满后静默丢失清理登记问题存在。`schedule_cleanup` 在 100 次偏移尝试失败后只调用 `defensive!`，随后仍返回 `Ok(())`，调用方无法感知失败。
- 证据 1：`proposal_cleanup.rs` 中 `CleanupQueue` 单区块容量为 50，`schedule_cleanup` 最多尝试 100 个区块；全部失败后 `defensive!` 并返回 `Ok(())`。
- 证据 2：`lib.rs` 中 `finish_terminal_status` 使用 `?` 传播 `schedule_cleanup` 结果，但由于失败分支返回 `Ok(())`，实际不会传播错误；`set_status_and_emit` 的事务收口路径还使用 `let _ = schedule_cleanup(...)`，进一步吞掉结果。
- 证据 3：真正删除 `ProposalData`、`ProposalOwner`、`ProposalMeta`、`Proposals`、`InternalTallies`、`JointTallies`、`CitizenTallies`、`AdminSnapshot` 等数据的路径依赖 `CleanupQueue` 到期后注册 `PendingProposalCleanups`；如果清理登记丢失，该提案没有其他补偿清理入口。
- 额外发现：`process_cleanup_queue` 每区块最多触发 5 个提案，但当同一 `CleanupQueue[now]` 中超过 5 个提案时，剩余项被写回同一个旧 `now` 键。后续区块只读取当前区块键，因此剩余项也会永久滞留。这个问题比 100 个队列全满更容易触发。
- 未执行测试：本次按用户要求先做存在性检查，未修改代码，未跑测试。

## 建议修复方向

- 将 `schedule_cleanup` 的 100 次尝试失败改为真实错误返回，避免调用方误判清理已登记。
- 统一调用点，不再使用 `let _ =` 吞掉清理登记失败；事务路径需要决定是回滚终态变更，还是进入可重试的待调度清理补偿队列。
- 修复 `process_cleanup_queue` 的剩余项处理：超过 `MAX_TRIGGERS_PER_BLOCK` 的剩余清理项应顺延到下一个可处理区块，或引入独立游标/待处理队列，不能写回旧区块键后失联。
- 补充单元测试覆盖：单区块队列超过 5 个、连续 100 个目标区块满、清理登记失败不会静默成功。

## 推荐修复方案

- 采用最小安全修复，不先重构清理队列存储结构。
- 新增 `CleanupQueueFull` 错误，`schedule_cleanup` 连续 100 个目标区块都满时返回该错误，不再 `defensive!` 后伪装为 `Ok(())`。
- 将 `schedule_cleanup` 改为基于 `try_mutate` 的显式插入逻辑，保证“写入成功才返回 Ok”。
- 将 `process_cleanup_queue` 改为一次处理当前桶内全部 `CleanupQueue[now]` 项。因为单桶本身被 `BoundedVec<_, ConstU32<50>>` 限死，处理全部 50 个是有界的，可直接删除旧的“剩余项写回旧区块”分支。
- 同步调整注释和 weight 估算口径：每个到期区块最多触发 50 个提案进入 `PendingProposalCleanups`，真正大规模删除仍由后续分块状态机限速。
- 调用点统一禁止吞错：`set_status_and_emit` 中的 `let _ = schedule_cleanup(...)` 改为错误回滚；`finish_terminal_status` 的调用方也需要避免“先改状态、后发现清理登记失败”的半成功状态，必要时用事务包住终态转换。
- 对 `process_execution_retry_deadlines` 这类 `on_initialize` 路径，不应继续 `let _ = finish_terminal_status(...)`；如果清理登记失败，保留或重排该提案的待处理状态，让下一块继续尝试，而不是把状态改成终态后丢掉清理。
- 补测试：
  - 单个 `CleanupQueue[now]` 塞满 50 个，`on_initialize(now)` 后 50 个全部进入 `PendingProposalCleanups`，旧桶删除。
  - 连续 100 个目标清理桶塞满后，再调用 `schedule_cleanup` 返回 `CleanupQueueFull`，不返回 `Ok(())`。
  - `set_status_and_emit` 遇到 `CleanupQueueFull` 时事务回滚，不产生“终态已写入但清理未登记”的状态。
  - 执行重试超时路径遇到清理登记失败时不会吞错并永久丢失清理。
