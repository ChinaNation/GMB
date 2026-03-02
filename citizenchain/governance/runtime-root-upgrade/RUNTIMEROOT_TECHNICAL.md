# RUNTIME_ROOT_UPGRADE Technical Notes

## 1. 模块定位
`runtime-root-upgrade` 是“Runtime 升级治理编排模块”，负责：
- 由 NRC 管理员提交 Runtime 升级提案（携带 wasm code）。
- 创建并绑定联合投票提案。
- 在联合投票结果回调后执行 runtime code。
- 执行失败时保留 code 并支持受限重试。

代码位置：
- `/Users/rhett/GMB/citizenchain/governance/runtime-root-upgrade/src/lib.rs`

## 2. Runtime 接线
Runtime 配置位置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

关键接线（当前实现）：
- `NrcProposeOrigin = EnsureNrcAdmin`：仅 NRC 管理员可发起提案/重试。
- `JointVoteFinalizeOrigin = EnsureJointVoteFinalizeOrigin`：finalize 使用专用 origin。
- `JointVoteEngine = VotingEngineSystem`：联合投票由投票引擎创建。
- `RuntimeCodeExecutor = RuntimeSetCodeExecutor`：升级代码实际执行器。
- `MaxExecutionRetries = RuntimeUpgradeMaxExecutionRetries`（当前为 5）。
- `WeightInfo = runtime_root_upgrade::SubstrateWeight<Runtime>`。

## 3. 数据模型与存储
### 3.1 核心类型
- `ProposalStatus`：
  - `Voting`
  - `Passed`
  - `Rejected`
  - `ExecutionFailed`
- `Proposal`：
  - `proposer`
  - `reason`
  - `code_hash`
  - `code`
  - `status`

### 3.2 存储
- `NextProposalId: StorageValue<u64>`
- `Proposals: StorageMap<u64, Proposal>`
- `GovToJointVote: StorageMap<u64, u64>`
- `JointVoteToGov: StorageMap<u64, u64>`
- `RetryCount: StorageMap<u64, u32>`（仅 `retry_failed_execution` 使用）

## 4. 外部接口（Calls）
### 4.1 `propose_runtime_upgrade`（call index = 0）
流程：
1. 校验 `NrcProposeOrigin`。
2. 校验 `reason` 非空、`code` 非空。
3. 读取 `NextProposalId`，`checked_add` 防溢出。
4. 调用 `JointVoteEngine::create_joint_proposal` 创建联合投票。
5. 计算 `code_hash`，写入 `Proposals` 与双向映射。
6. 触发 `RuntimeUpgradeProposed` 事件。

### 4.2 `finalize_joint_vote`（call index = 1）
流程：
1. 校验 `JointVoteFinalizeOrigin`。
2. 调 `apply_joint_vote_result`：
   - `approved=true`：先清理映射并发 `JointVoteFinalized`，再执行 code。
   - 执行成功：`status=Passed`，清空 `code`，清理 `RetryCount`，发 `RuntimeUpgradeExecuted`。
   - 执行失败：`status=ExecutionFailed`，保留 `code`，初始化 `RetryCount=0`，发 `RuntimeUpgradeExecutionFailed`。
   - `approved=false`：`status=Rejected`，清空 `code`，清理映射与 `RetryCount`，发 `JointVoteFinalized`。
3. 返回 `DispatchResultWithPostInfo`，rejected 路径按 `actual_weight` 退费。

### 4.3 `retry_failed_execution`（call index = 2）
流程：
1. 校验 `NrcProposeOrigin`。
2. 仅允许 `ExecutionFailed` 状态提案。
3. 校验 `RetryCount < MaxExecutionRetries`。
4. 重试执行 `RuntimeCodeExecutor::execute_runtime_code`。
5. 成功：`status=Passed`，清空 `code`，删除 `RetryCount`，发 `RuntimeUpgradeExecuted`。
6. 失败：`RetryCount += 1`，发 `RuntimeUpgradeExecutionFailed`。

## 5. 回调入口与状态机
`JointVoteResultCallback::on_joint_vote_finalized`：
- 通过 `JointVoteToGov` 找到治理提案 ID。
- 转发到 `apply_joint_vote_result`。

状态机：
- `Voting -> Passed`（批准且执行成功）
- `Voting -> ExecutionFailed`（批准但执行失败）
- `Voting -> Rejected`（投票拒绝）
- `ExecutionFailed -> Passed`（重试成功）
- `ExecutionFailed -> ExecutionFailed`（重试失败并计数）

## 6. 事件与错误
### 6.1 事件
- `RuntimeUpgradeProposed`
- `JointVoteFinalized { proposal_id, joint_vote_id, approved }`
- `RuntimeUpgradeExecuted`
- `RuntimeUpgradeExecutionFailed`

### 6.2 错误
- 参数与存在性：`EmptyReason`、`EmptyRuntimeCode`、`ProposalNotFound`
- 生命周期：`ProposalNotVoting`、`ProposalNotExecutionFailed`
- 投票映射：`JointVoteCreateFailed`、`JointVoteMappingNotFound`
- 计数与重试：`ProposalIdOverflow`、`MaxRetriesExceeded`

## 7. 权重策略
模块已引入 `WeightInfo`：
- `propose_runtime_upgrade(code_len, reason_len)`  
  - `DbWeight(reads_writes(2,5)) + code_len 线性项 + reason_len 线性项`
- `finalize_joint_vote_approved()`  
  - `DbWeight(reads_writes(2,3)) + MaxRuntimeCodeSize 线性项`
- `finalize_joint_vote_rejected()`  
  - `DbWeight(reads_writes(2,3))`
- `retry_failed_execution()`  
  - `DbWeight(reads_writes(2,2)) + MaxRuntimeCodeSize 线性项`

post-dispatch（finalize）：
- `ApprovedExecutionSucceeded/ApprovedExecutionFailed`：不退费（`None`）。
- `Rejected`：`Some(DbWeight::reads_writes(2,3))`。

## 8. Benchmark
`runtime-benchmarks` 已实现，覆盖：
- `propose_runtime_upgrade`
- `finalize_joint_vote_approved`
- `finalize_joint_vote_rejected`
- `retry_failed_execution`

基准输入使用 `reason_max` 与 `code_max`，覆盖最坏参数规模。

## 9. 测试覆盖（当前）
本地执行：
- `cargo test -p runtime-root-upgrade --quiet`

结果：
- `14 passed; 0 failed`

覆盖重点：
- 发起权限（仅 NRC 管理员）。
- 空 reason / 空 code 拒绝。
- mapping 缺失回调报错。
- approved/rejected finalize 状态更新与映射清理。
- 执行失败进入 `ExecutionFailed` 且保留 code。
- retry 成功恢复、retry 上限、非失败状态重试拒绝。
- finalize 权限检查。
- finalize 双次调用拒绝（`ProposalNotVoting`）。

## 10. 安全与运维建议
1. 生产环境继续保持 `JointVoteFinalizeOrigin` 收敛，避免开放 finalize 外部入口。  
2. 监控 `RuntimeUpgradeExecutionFailed` 事件与 `RetryCount`，及时触发重试治理流程。  
3. 发布前用 benchmark CLI 产出权重并替换手工参数，避免长期偏保守/偏乐观。  
