# RESOLUTION_ISSUANCE_GOV Technical Notes

## 1. 模块定位
`resolution-issuance-gov` 是“决议发行治理编排模块”，负责把国储会发起的发行决议接入联合投票，并在投票结果落地后驱动发行执行模块。

模块职责：
- 创建并维护“治理提案 <-> 联合投票提案”映射。
- 约束提案生命周期（`Voting -> Passed/Rejected/ExecutionFailed`）。
- 在投票通过时调用 `resolution-issuance-iss` 执行发行。
- 对执行失败提案提供受限重试入口。
- 维护链上合法收款账户集合，防止治理期间热切换带来口径漂移。

代码位置：
- `/Users/rhett/GMB/citizenchain/governance/resolution-issuance-gov/src/lib.rs`

## 2. 运行时接线与上下游
Runtime 接线位置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs:870`

关键配置：
- `NrcProposeOrigin = EnsureNrcAdmin`：仅 NRC 管理员可发起提案与执行重试。
- `RecipientSetOrigin = EnsureRoot<AccountId>`：仅 Root 可更新收款账户集合。
- `JointVoteFinalizeOrigin = EnsureJointVoteFinalizeOrigin`：生产态拒绝外部 finalize 调用；benchmark 下允许 Root。
- `IssuanceExecutor = ResolutionIssuanceIss`：发行执行委托给发行模块。
- `JointVoteEngine = VotingEngineSystem`：联合投票创建由投票引擎承担。
- `MaxExecutionRetries = ConstU32<5>`：生产态最多重试 5 次。

联合投票回调路由：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs:914`
- 回调先按 `joint_vote_to_gov` 判断归属，再转发给本模块 `JointVoteResultCallback` 实现。

## 3. 数据结构与存储
核心结构：
- `RecipientAmount { recipient, amount }`
- `Proposal { proposer, reason, total_amount, allocations, vote_kind, status }`
- `ProposalStatus = Voting | Passed | Rejected | ExecutionFailed`

核心存储：
- `NextProposalId: u64`
- `Proposals: Map<u64, Proposal>`
- `GovToJointVote: Map<u64, u64>`
- `JointVoteToGov: Map<u64, u64>`
- `AllowedRecipients: BoundedVec<AccountId, MaxAllocations>`
- `VotingProposalCount: u32`（治理中的提案计数，用于阻止改名单）
- `RetryCount: Map<u64, u32>`（仅统计 `retry_failed_execution`）

Genesis：
- 默认 `allowed_recipients` 来源于 `CHINA_CB` 的机构收款地址（跳过索引 0 的 NRC 节点）。
- 创世构建时强制唯一性检查，重复地址会直接失败。

## 4. 外部接口（Calls）
### 4.1 `propose_resolution_issuance`（call index = 0）
流程：
1. 校验 origin 为 NRC 管理员。
2. 校验 `reason` 非空。
3. 校验 allocations 与 `AllowedRecipients` 完全一致（数量、成员、无重复、金额和）。
4. 分配 `proposal_id`，调用联合投票引擎创建提案。
5. 写入 `Proposals`、双向映射、`VotingProposalCount += 1`。
6. 发 `ResolutionIssuanceProposed` 事件。

### 4.2 `finalize_joint_vote`（call index = 1）
流程：
1. 校验来源为 `JointVoteFinalizeOrigin`。
2. 进入 `apply_joint_vote_result`：
- `approved=true` 时尝试执行发行。
- 执行成功：状态置 `Passed`，清理映射，清理重试计数，`VotingProposalCount -= 1`。
- 执行失败：状态置 `ExecutionFailed`，清理映射，`VotingProposalCount -= 1`。
- `approved=false`：状态置 `Rejected`，清理映射，`VotingProposalCount -= 1`。
3. 返回 post-dispatch `actual_weight`（失败/拒绝路径退费）。

### 4.3 `set_allowed_recipients`（call index = 2）
流程：
1. Root 权限。
2. 新名单不能为空。
3. `VotingProposalCount` 必须为 0（治理中禁止切换名单）。
4. 新名单去重校验通过后写入存储并发事件。

### 4.4 `retry_failed_execution`（call index = 3）
流程：
1. 校验 origin 为 NRC 管理员。
2. 提案必须存在且状态为 `ExecutionFailed`。
3. `RetryCount < MaxExecutionRetries`。
4. 再次调用发行执行模块。
5. 成功则状态改 `Passed` 并清除 `RetryCount`；失败则 `RetryCount += 1` 并发失败事件。

## 5. 生命周期与状态机
状态流转：
- `Voting -> Passed`：投票通过且执行成功。
- `Voting -> ExecutionFailed`：投票通过但执行失败（可重试）。
- `Voting -> Rejected`：投票未通过。
- `ExecutionFailed -> Passed`：`retry_failed_execution` 成功后恢复。

关键一致性：
- 只有 `Voting` 状态允许 finalize，重复 finalize 会报 `ProposalNotVoting`。
- finalize 后会清理联合投票映射，避免重复回调和脏映射残留。
- `VotingProposalCount` 与 `Voting` 提案数量保持同向变化。

## 6. 需求与安全约束（实现口径）
业务约束：
- 发行分配名单必须与链上 `AllowedRecipients` 精确匹配，不允许少人、多人、换人、重复。
- `total_amount` 必须等于 allocations 金额和。
- `amount` 不可为 0。

治理约束：
- 仅 NRC 管理员可发起和重试。
- 仅受控来源可 finalize（生产环境禁外部直接调用）。
- 治理进行中（`VotingProposalCount > 0`）禁止修改 `AllowedRecipients`。

故障恢复约束：
- 投票通过但执行失败不会伪装成通过，而是进入 `ExecutionFailed`。
- 重试次数受 `MaxExecutionRetries` 硬上限约束。

## 7. 权重与计费口径
`WeightInfo` 当前为手工+线性项估算：
- `propose_resolution_issuance(allocation_count, reason_len)`
- `finalize_joint_vote_approved()`：叠加发行执行权重的最大参数。
- `finalize_joint_vote_rejected()`：`reads_writes(3, 4)`。
- `set_allowed_recipients(recipient_count)`。
- `retry_failed_execution()`：叠加发行执行权重的最大参数。

post-dispatch `actual_weight`：
- `ApprovedExecutionSucceeded`：不退费（`None`）。
- `ApprovedExecutionFailed`：`reads_writes(3, 5)`（保守高估可接受）。
- `Rejected`：`reads_writes(3, 4)`。

benchmark 口径（`runtime-benchmarks`）：
- `finalize_joint_vote_approved` 使用 `reason_max + full_allocations`。
- `retry_failed_execution` 同样使用 `reason_max + full_allocations`，覆盖最坏执行参数。

## 8. 运行时升级与迁移
当前 `STORAGE_VERSION = 2`，`on_runtime_upgrade` 处理：
- v0 -> v1：若 `AllowedRecipients` 为空，尝试按 `CHINA_CB` 回填默认名单。
- v1 -> v2：
- 若 `AllowedRecipients` 不唯一，回填默认名单。
- 扫描 `Proposals` 重建 `VotingProposalCount`。
- 最后写入最新 `StorageVersion`。

迁移权重：
- 按扫描提案数线性叠加迭代权重与 db 读写权重。

## 9. 事件与错误
核心事件：
- `ResolutionIssuanceProposed`
- `JointVoteFinalized`
- `IssuanceExecutionTriggered`
- `IssuanceExecutionFailed`
- `AllowedRecipientsUpdated`

核心错误：
- 参数与集合约束：`EmptyReason`、`EmptyAllocations`、`InvalidAllocationCount`、`InvalidRecipientSet`、`TotalMismatch`、`DuplicateRecipient`、`DuplicateAllowedRecipient`
- 生命周期：`ProposalNotFound`、`ProposalNotVoting`、`ProposalNotExecutionFailed`
- 治理控制：`RecipientsNotConfigured`、`ActiveVotingProposalsExist`
- 重试控制：`MaxRetriesExceeded`
- 内部状态：`ProposalIdOverflow`、`VotingProposalCountOverflow`、`VotingProposalCountUnderflow`、`JointVoteCreateFailed`、`JointVoteMappingNotFound`

## 10. 测试与基准
本地测试命令：
- `cargo test -p resolution-issuance-gov --quiet`

当前结果（本仓库）：
- `20 passed; 0 failed`

覆盖重点：
- 提案创建与映射写入。
- 回调通过/拒绝/执行失败状态流转。
- `ExecutionFailed` 重试成功恢复与重试上限约束。
- 非 `ExecutionFailed` 状态重试拒绝。
- 治理进行中禁止改收款集合、空集合/重复集合拒绝。
- 关键参数校验错误路径（recipient set、total mismatch 等）。

## 11. 运维建议
1. 若后续引入 benchmark CLI 自动产出权重，建议将产出值替换手工估算，并保持 worst-case benchmark 输入不回退。  
2. 监控 `IssuanceExecutionFailed` 事件，配合 `RetryCount` 做治理侧重试告警。  
3. 升级含迁移逻辑时，优先在预发布环境验证 `VotingProposalCount` 重建结果与现网提案状态一致。  
