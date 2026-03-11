# Runtime Root Upgrade 技术文档

## 0. 功能需求
### 0.1 模块职责
`runtime-root-upgrade` 负责把“Runtime wasm 升级”包装成一个受治理约束的链上流程，核心要求是：
- 仅允许国储会（NRC）管理员发起升级提案。
- 升级提案必须先经过 `voting-engine-system` 的联合投票。
- 联合投票通过后才允许执行 `set_code`。
- 投票结果、执行结果、重试状态必须在链上可追踪。

### 0.2 提案创建需求
- 提案必须携带非空升级理由 `reason`。
- 提案必须携带非空 wasm `code`。
- 创建提案时必须同步创建联合投票提案，并建立双向映射：
  - `gov proposal_id -> joint_vote_id`
  - `joint_vote_id -> gov proposal_id`
- 提案 ID 必须单调递增，且不能因溢出覆盖旧提案。

### 0.3 联合投票回调需求
- 联合投票拒绝时，提案必须进入 `Rejected`，并清空保存的 wasm code。
- 联合投票通过时，模块必须尝试执行 runtime code。
- 联合投票结束后，必须释放与投票引擎的双向映射并清理投票引擎侧提案状态。
- 模块必须支持通过回调 `joint_vote_id` 反查本模块 `proposal_id`。

### 0.4 执行失败与重试需求
- 若联合投票已通过但 `set_code` 执行失败，提案不能误记为 `Passed`。
- 执行失败时，提案必须进入 `ExecutionFailed`，并保留原始 wasm code 供后续重试。
- 必须记录手动重试次数，限制最大重试次数，避免无限重试。
- 生产链当前限制为每个提案最多重试 3 次。
- 仅允许受限治理角色触发重试。

### 0.5 可审计与运维需求
- 需要区分以下事件：提案创建、联合投票终结、升级执行成功、升级执行失败。
- 提案状态机必须清晰可恢复：
  - `Voting -> Passed`
  - `Voting -> Rejected`
  - `Voting -> ExecutionFailed`
  - `ExecutionFailed -> Passed`
  - `ExecutionFailed -> ExecutionFailed`

## 1. 模块定位
`runtime-root-upgrade` 是“Runtime 升级治理编排模块”，负责：
- 接收 NRC 管理员提交的 wasm 升级提案；
- 调用 `voting-engine-system` 创建联合投票；
- 在联合投票回调后执行 `set_code`；
- 在执行失败时保留 code 并支持有限次数重试。

代码位置：
- `/Users/rhett/GMB/citizenchain/governance/runtime-root-upgrade/src/lib.rs`

## 2. 运行时接线
Runtime 配置位置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

当前接线：
- `NrcProposeOrigin = EnsureNrcAdmin`
- `JointVoteEngine = VotingEngineSystem`
- `RuntimeCodeExecutor = RuntimeSetCodeExecutor`
- `MaxExecutionRetries = ConstU32<3>`

说明：
- `finalize_joint_vote` 当前仅允许 `Root` 手工回放；
- 正常生产路径由投票引擎通过 `JointVoteResultCallback` 自动回调本模块。

## 3. 核心数据结构
### 3.1 ProposalStatus
- `Voting`：联合投票中
- `Passed`：联合投票通过且 runtime code 执行成功
- `Rejected`：联合投票拒绝
- `ExecutionFailed`：联合投票通过，但 runtime code 执行失败，可重试

### 3.2 Proposal
- `proposer`：提案发起人
- `reason`：升级理由
- `code_hash`：升级 code 哈希
- `code`：待执行 wasm code
- `status`：当前提案状态

## 4. 存储模型
- `NextProposalId`：本模块提案 ID 计数器
- `Proposals`：提案主表
- `GovToJointVote`：本模块提案 ID -> 联合投票提案 ID
- `JointVoteToGov`：联合投票提案 ID -> 本模块提案 ID
- `RetryCount`：执行失败后的手动重试次数

## 5. 外部接口
### 5.1 `propose_runtime_upgrade`（call index = 0）
流程：
1. 校验 `NrcProposeOrigin`。
2. 校验 `reason` 与 `code` 非空。
3. 分配 `proposal_id`，使用 `checked_add` 防溢出。
4. 调用 `JointVoteEngine::create_joint_proposal` 创建联合投票。
5. 写入 `Proposals`、`GovToJointVote`、`JointVoteToGov`。
6. 发出 `RuntimeUpgradeProposed` 事件。

### 5.2 `finalize_joint_vote`（call index = 1）
说明：
- 该入口仅作为 `Root` 手工补偿/回放入口。
- 正常情况下由投票引擎回调 `on_joint_vote_finalized` 进入同一套逻辑。

流程：
1. 校验 `Root`。
2. 读取提案并要求当前状态为 `Voting`。
3. 若 `approved=false`：
   - 标记 `Rejected`
   - 清空 `code`
   - 清理联合投票映射
   - 发出 `JointVoteFinalized`
4. 若 `approved=true`：
   - 清理联合投票映射
   - 尝试执行 `RuntimeCodeExecutor::execute_runtime_code`
   - 成功：标记 `Passed`，清空 `code`，删除 `RetryCount`
   - 失败：标记 `ExecutionFailed`，保留 `code`，初始化 `RetryCount=0`
   - 发出 `JointVoteFinalized`，并额外发出执行成功或失败事件

### 5.3 `retry_failed_execution`（call index = 2）
流程：
1. 校验 `NrcProposeOrigin`。
2. 仅允许 `ExecutionFailed` 状态提案进入重试。
3. 校验 `RetryCount < MaxExecutionRetries`。
4. 再次执行同一份保留的 wasm code。
5. 成功：标记 `Passed`，清空 `code`，清理 `RetryCount`。
6. 失败：保持 `ExecutionFailed`，`RetryCount += 1`。
7. 当失败重试次数达到 3 次后，后续重试请求会被拒绝。

## 6. 回调路径
`JointVoteResultCallback::on_joint_vote_finalized`：
1. 用 `joint_vote_id` 查询 `JointVoteToGov`
2. 找到本模块 `proposal_id`
3. 调用 `apply_joint_vote_result(proposal_id, approved)`

这保证了投票引擎只需知道联合投票提案 ID，而业务模块仍可独立维护自己的提案编号。

## 7. 安全审查结论
### 7.1 已修复风险：Proposal ID 溢出覆盖
旧实现使用 `saturating_add` 推进 `NextProposalId`。若计数达到 `u64::MAX`，后续提案会重复使用同一 ID，存在覆盖旧提案风险。

现已修复：
- 改为 `checked_add`
- 溢出时返回 `ProposalIdOverflow`

### 7.2 已修复风险：执行失败误记为 Passed
旧实现中，联合投票通过后会先把提案写成 `Passed` 并清空 `code`，再尝试执行 `set_code`。如果执行失败：
- 链上状态仍显示 `Passed`
- 原始 code 已丢失
- 无法重试

现已修复：
- 执行成功才进入 `Passed`
- 执行失败进入 `ExecutionFailed`
- 保留原始 code
- 新增 `retry_failed_execution`

### 7.3 推荐改进
1. 当前 `finalize_joint_vote` 手工入口使用 `Root`，权限已经足够严格；若后续想与其他模块统一，可评估抽象出专用 `JointVoteFinalizeOrigin`。
2. 模块现已补上 `runtime-benchmarks` 入口与专用 `WeightInfo`；当前 `weights.rs` 仍是保守手写值，后续可用 benchmark CLI 实测产物替换。

## 8. 中文注释覆盖重点
本模块当前已在以下关键位置补充中文注释：
- `RuntimeCodeExecutor` 职责边界
- `ProposalStatus` 各状态语义
- `allocate_proposal_id` 溢出保护
- `cleanup_joint_vote_mapping` 清理语义
- 联合投票通过后的执行/失败分叉
- `retry_failed_execution` 的重试边界
- `on_joint_vote_finalized` 的映射反查逻辑

## 9. 测试覆盖
已覆盖：
- 仅 NRC 管理员可发起提案
- `joint_vote_id` 映射缺失时报错
- Proposal ID 溢出保护
- 联合投票拒绝进入 `Rejected`
- 联合投票通过并成功执行进入 `Passed`
- 联合投票通过但执行失败进入 `ExecutionFailed`
- 重试成功转为 `Passed`
- 重试失败增加 `RetryCount`
- 重试次数达到上限后拒绝
- 非 `ExecutionFailed` 状态禁止重试

本地验证：
- `cargo test -p runtime-root-upgrade`

## 10. 文件索引
- 模块代码：`src/lib.rs`
- 技术文档：`RUNTIMEROOT_TECHNICAL.md`
