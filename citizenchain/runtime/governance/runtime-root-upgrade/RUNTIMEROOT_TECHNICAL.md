# Runtime Root Upgrade 技术文档

## 0. 功能需求
### 0.1 模块职责
`runtime-root-upgrade` 负责把"Runtime wasm 升级"包装成一个受治理约束的链上流程，核心要求是：
- 仅允许国储会（NRC）管理员发起升级提案。
- 升级提案必须先经过 `voting-engine-system` 的联合投票。
- 联合投票通过后才允许执行 `set_code`。
- 投票结果、执行结果必须在链上可追踪。

### 0.2 提案创建需求
- 提案必须携带非空升级理由 `reason`。
- 提案必须携带非空 wasm `code`。
- 创建提案时同步在 `voting-engine-system` 创建联合投票，使用投票引擎统一分配的 `proposal_id`（本模块不维护独立 ID）。

### 0.3 联合投票回调需求
- 联合投票拒绝时，提案必须进入 `Rejected`，并清空保存的 wasm code。
- 联合投票通过时，模块必须尝试执行 runtime code。
- 联合投票结束后，将投票引擎侧提案状态设为 `STATUS_EXECUTED`，标记投票引擎职责完成。
- 回调直接使用投票引擎的 `proposal_id`，无需映射反查。

### 0.4 执行失败处理
- 若联合投票已通过但 `set_code` 执行失败，提案进入 `ExecutionFailed`。
- 执行失败时清空 code 释放存储（不支持重试，需重新发起提案）。
- 设计理由：runtime 升级失败通常是 WASM 本身有问题，重试同一份代码意义不大，修复后重新提案更合理。

### 0.5 可审计与运维需求
- 需要区分以下事件：提案创建、联合投票终结、升级执行成功、升级执行失败。
- 提案状态机：
  - `Voting → Passed`（投票通过且执行成功）
  - `Voting → Rejected`（投票拒绝）
  - `Voting → ExecutionFailed`（投票通过但执行失败）
- 所有终态均为不可逆（无重试、无取消）。

## 1. 模块定位
`runtime-root-upgrade` 是"Runtime 升级治理编排模块"，负责：
- 接收 NRC 管理员提交的 wasm 升级提案；
- 调用 `voting-engine-system` 创建联合投票；
- 在联合投票回调后执行 `set_code`；
- 业务数据存储在 `voting-engine-system` 的 `ProposalData`，本模块零本地存储。

代码位置：
- `runtime/governance/runtime-root-upgrade/src/lib.rs`

## 2. 运行时接线
Runtime 配置位置：
- `runtime/src/configs/mod.rs`

当前接线：
- `NrcProposeOrigin = EnsureNrcAdmin`
- `JointVoteEngine = VotingEngineSystem`
- `RuntimeCodeExecutor = RuntimeSetCodeExecutor`
- `MaxReasonLen = RuntimeUpgradeMaxReasonLen`（1024）
- `MaxRuntimeCodeSize = RuntimeUpgradeMaxCodeSize`（5 * 1024 * 1024）
- `MaxSnapshotNonceLength = 64`
- `MaxSnapshotSignatureLength = 64`
- `WeightInfo = runtime_root_upgrade::weights::SubstrateWeight<Runtime>`

说明：
- `finalize_joint_vote` 当前仅允许 `Root` 手工回放；
- 正常生产路径由投票引擎通过 `JointVoteResultCallback` 自动回调本模块。

## 3. 核心数据结构
### 3.1 ProposalStatus
- `Voting`：联合投票中
- `Passed`：联合投票通过且 runtime code 执行成功
- `Rejected`：联合投票拒绝
- `ExecutionFailed`：联合投票通过，但 runtime code 执行失败

### 3.2 Proposal（序列化存入 voting-engine-system ProposalData）
- `proposer: AccountId`：提案发起人（仅允许 NRC 管理员）
- `reason: BoundedVec<u8, MaxReasonLen>`：升级理由
- `code_hash: Hash`：升级 code 哈希，便于事件与链下审计对齐
- `code: BoundedVec<u8, MaxRuntimeCodeSize>`：待执行 wasm code；终态后清空
- `status: ProposalStatus`：当前提案状态

## 4. 存储模型
本模块无本地存储。所有提案数据、投票数据、元数据均存储在 `voting-engine-system`：
- `ProposalData`：存放 `Proposal<T>` 的 SCALE 编码
- `ProposalMeta`：存放提案创建时间
- `Proposals`：投票引擎核心提案表（状态、阶段、截止区块等）

## 5. 外部接口
### 5.1 `propose_runtime_upgrade`（call index = 0）
流程：
1. 校验 `NrcProposeOrigin`。
2. 校验 `reason` 与 `code` 非空。
3. 调用 `JointVoteEngine::create_joint_proposal` 创建联合投票，获取统一 `proposal_id`。
4. 计算 `code_hash`，构造 `Proposal` 结构并序列化存入 `ProposalData`。
5. 调用 `store_proposal_meta` 记录创建时间。
6. 发出 `RuntimeUpgradeProposed` 事件。

### 5.2 `finalize_joint_vote`（call index = 1）
说明：
- 该入口仅作为 `Root` 手工补偿/回放入口。
- 正常情况下由投票引擎回调 `on_joint_vote_finalized` 进入同一套逻辑。

流程：
1. 校验 `Root`。
2. 从 `ProposalData` 加载提案并要求当前状态为 `Voting`。
3. 若 `approved=false`：
   - 标记 `Rejected`，清空 `code`
   - 设投票引擎状态为 `STATUS_EXECUTED`
   - 发出 `JointVoteFinalized`
4. 若 `approved=true`：
   - 尝试执行 `RuntimeCodeExecutor::execute_runtime_code`
   - 成功：标记 `Passed`，清空 `code`
   - 失败：标记 `ExecutionFailed`，清空 `code`
   - 设投票引擎状态为 `STATUS_EXECUTED`
   - 发出 `JointVoteFinalized` + 执行成功或失败事件

### 5.2.1 投票引擎 STATUS_EXECUTED 标记

无论执行成功、失败还是被拒绝，本模块都会将投票引擎侧 `Proposals` 的状态直接修改为 `STATUS_EXECUTED`。

实现方式：直接通过 `Proposals::<T>::mutate` 修改投票引擎存储中的 `status` 字段，而非调用 `set_status_and_emit`。

原因：本模块的执行逻辑运行在投票引擎的回调路径中（`on_joint_vote_finalized`）。若在回调内部再调用 `set_status_and_emit`，会触发投票引擎的状态变更事件和潜在的回调链，产生回调重入风险。直接修改存储字段可以安全地标记提案为已执行，同时避免重入问题。

提案状态流转（投票引擎侧）：`VOTING → PASSED/REJECTED → EXECUTED`

说明：
- 本模块自身的 `ProposalStatus`（`Passed`/`ExecutionFailed`/`Rejected`）与投票引擎侧的 `STATUS_EXECUTED` 是独立的状态维度。
- 投票引擎侧的 `EXECUTED` 标记在所有终态都会设置，因为无论结果如何，投票引擎的职责（投票与回调触发）已经完成。

## 6. 回调路径
`JointVoteResultCallback::on_joint_vote_finalized`：
1. 接收投票引擎统一的 `proposal_id`（无需映射反查）
2. 调用 `apply_joint_vote_result(proposal_id, approved)`

Runtime 层的 `RuntimeJointVoteResultCallback` 负责路由：先尝试 `resolution-issuance-gov`，再尝试 `runtime-root-upgrade`。

## 7. 安全审查结论
### 7.1 已修复风险：执行失败误记为 Passed
旧实现中，联合投票通过后会先把提案写成 `Passed` 并清空 `code`，再尝试执行 `set_code`。如果执行失败：
- 链上状态仍显示 `Passed`
- 原始 code 已丢失

现已修复：
- 先执行，根据结果决定状态
- 执行成功才进入 `Passed`
- 执行失败进入 `ExecutionFailed`

### 7.2 已修复风险：ExecutionFailed 不清空 code
旧实现中执行失败时保留 code 供重试，但重试功能未实现，导致 5MB WASM 在存储中滞留 90 天。

现已修复：
- 执行失败时也清空 `code`，与 `Rejected` 路径一致
- 失败后需重新提案，而非重试同一份可能有问题的 WASM

### 7.3 已修复风险：Rejected 路径未设 STATUS_EXECUTED
旧实现中 `approved=true` 路径会设投票引擎 `STATUS_EXECUTED`，但 `approved=false` 路径不设，导致两种终态行为不一致。

现已修复：
- 三种终态（Passed/ExecutionFailed/Rejected）均统一设置 `STATUS_EXECUTED`

### 7.4 已修复风险：benchmark 与实际逻辑不一致
旧版 benchmark 存在偏差。现已修复：
- `propose_runtime_upgrade` benchmark 改为真实 extrinsic
- `finalize_joint_vote` 拆分为 `approved/rejected` 两条 benchmark
- 由于 benchmark 环境不会真的改写链上 `:code`，`finalize_joint_vote(approved)` 在权重声明中会额外叠加 `frame_system::set_code()` 的系统权重

### 7.5 推荐改进
1. 当前 `finalize_joint_vote` 手工入口使用 `Root`，权限已经足够严格；若后续想与其他模块统一，可评估抽象出专用 `JointVoteFinalizeOrigin`。

## 8. 中文注释覆盖重点
本模块当前已在以下关键位置补充中文注释：
- `RuntimeCodeExecutor` 职责边界
- `ProposalStatus` 各状态语义
- 联合投票通过后的执行/失败分叉
- `STATUS_EXECUTED` 直接修改存储的原因
- `on_joint_vote_finalized` 回调入口

## 9. 测试覆盖
已覆盖（10 个测试）：
- 仅 NRC 管理员可发起提案
- 提案数据正确存入 voting-engine-system
- 联合投票拒绝进入 `Rejected` 并清空 code
- 联合投票通过并成功执行进入 `Passed` 并清空 code
- 联合投票通过但执行失败进入 `ExecutionFailed` 并清空 code
- 已终结的提案不可重复终结（`ProposalNotVoting`）
- 不存在的提案终结失败（`ProposalNotFound`）
- GenesisConfig 构建成功
- Runtime 完整性检查

Runtime 集成测试：
- 不存在的 proposal_id 回调返回错误
- 回调正确路由到本模块并执行拒绝流程

本地验证：
- `cargo test -p runtime-root-upgrade`
- `cargo check -p citizenchain --features runtime-benchmarks`

## 10. 文件索引
- 模块代码：`src/lib.rs`
- Benchmark：`src/benchmarks.rs`
- 权重：`src/weights.rs`
- 技术文档：`RUNTIMEROOT_TECHNICAL.md`
