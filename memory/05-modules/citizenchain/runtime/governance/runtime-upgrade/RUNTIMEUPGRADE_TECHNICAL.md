# runtime-upgrade 技术文档

## 0. 功能需求
### 0.1 模块职责
`runtime-upgrade` 负责把"Runtime wasm 升级"包装成一个受治理约束的链上流程，核心要求是：
- 仅允许国储会（NRC）和 43 个省储会（PRC）管理员发起升级提案。
- 升级提案必须先经过 `voting-engine` 的联合投票。
- 联合阶段由各机构管理员个人钱包直接上链投票，链上按机构阈值自动形成机构结果。
- 联合投票通过后才允许执行 `set_code`。
- 投票结果、执行结果必须在链上可追踪。

### 0.2 提案创建需求
- 提案必须携带非空升级理由 `reason`。
- 提案必须携带非空 wasm `code`。
- 创建提案时同步在 `voting-engine` 创建联合投票，使用投票引擎统一分配的 `proposal_id`（本模块不维护独立 ID）。

### 0.3 联合投票回调需求
- 联合投票拒绝时，提案必须进入 `Rejected`。
- 联合投票通过时，模块必须尝试执行 runtime code。
- 联合投票结束后，投票引擎侧状态保持真实治理结果：通过保持 `STATUS_PASSED`，否决保持 `STATUS_REJECTED`，仅执行失败时覆写为 `STATUS_EXECUTION_FAILED`。
- 回调直接使用投票引擎的 `proposal_id`，无需映射反查。

### 0.4 执行失败处理
- 若联合投票已通过但 `set_code` 执行失败，提案进入 `ExecutionFailed`。
- runtime wasm 不再内嵌在摘要结构里，而是统一存入 `voting-engine::ProposalObject`。
- 执行成功、拒绝、执行失败后，wasm 对象继续保留到投票引擎 90 天延迟清理统一删除，不由业务模块手工删除。

### 0.5 可审计与运维需求
- 需要区分以下事件：提案创建、联合投票终结、升级执行成功、升级执行失败。
- 提案状态机：
  - `Voting → Passed`（投票通过且执行成功）
  - `Voting → Rejected`（投票拒绝）
  - `Voting → ExecutionFailed`（投票通过但执行失败）
- 所有终态均为不可逆（无重试、无取消）。

## 1. 模块定位
`runtime-upgrade` 是"Runtime 升级治理编排模块"，负责：
- 接收国储会或省储会管理员提交的 wasm 升级提案；
- 调用 `voting-engine` 创建联合投票；
- 在联合投票回调后执行 `set_code`；
- 摘要数据存储在 `voting-engine` 的 `ProposalData`；
- 原始 wasm 对象存储在 `voting-engine` 的 `ProposalObject`；
- 本模块零本地存储。

代码位置：
- `runtime/governance/runtime-upgrade/src/lib.rs`

命名说明：
- 2026-04-29 起，本模块统一使用 `runtime-upgrade` / `runtime_upgrade` / `RuntimeUpgrade`。
- 模块位于 `citizenchain/runtime/governance/runtime-upgrade/`。
- `pallet_index = 13`、call index 与 `MODULE_TAG = b"rt-upg"` 保持不变。

## 2. 运行时接线
Runtime 配置位置：
- `runtime/src/configs/mod.rs`

当前接线：
- `ProposeOrigin = EnsureJointProposer`
- `JointVoteEngine = VotingEngine`
- `RuntimeCodeExecutor = RuntimeSetCodeExecutor`
- `MaxReasonLen = RuntimeUpgradeMaxReasonLen`（1024）
- `MaxRuntimeCodeSize = RuntimeUpgradeMaxCodeSize`（5 * 1024 * 1024）
- `VotingEngine::MaxProposalDataLen = 100 * 1024`
- `VotingEngine::MaxProposalObjectLen = 10 * 1024 * 1024`
- `MaxSnapshotNonceLength = 64`
- `MaxSnapshotSignatureLength = 64`
- `WeightInfo = runtime_upgrade::weights::SubstrateWeight<Runtime>`

说明：
- `finalize_joint_vote` 当前仅允许 `Root` 手工回放；
- 正常生产路径由投票引擎通过 `JointVoteResultCallback` 自动回调本模块。

## 3. 核心数据结构
### 3.1 ProposalStatus
- `Voting`：联合投票中
- `Passed`：联合投票通过且 runtime code 执行成功
- `Rejected`：联合投票拒绝
- `ExecutionFailed`：联合投票通过，但 runtime code 执行失败

### 3.2 Proposal（摘要，序列化存入 voting-engine ProposalData）
- `proposer: AccountId`：提案发起人（国储会或省储会管理员）
- `reason: BoundedVec<u8, MaxReasonLen>`：升级理由
- `code_hash: Hash`：升级 code 哈希，便于事件与链下审计对齐
- `status: ProposalStatus`：当前提案状态

### 3.3 对象层数据（统一存入 voting-engine ProposalObject）
- `kind = 1`：表示 runtime wasm 对象
- `object_len`：wasm 字节长度
- `object_hash`：对象哈希
- `object bytes`：原始 wasm 字节（对象层上限 10MB，业务自身继续限制 5MB）

### 3.4 模块标识
- `MODULE_TAG = b"rt-upg"`：存入 ProposalData 的前缀，用于区分不同业务模块，防止跨模块误解码。

## 4. 存储模型
本模块无本地存储。所有提案数据、投票数据、元数据均存储在 `voting-engine`：
- `ProposalData`：存放 `MODULE_TAG + Proposal<T>` 摘要的 SCALE 编码
- `ProposalObjectMeta`：存放 runtime wasm 的对象元数据（kind / len / hash）
- `ProposalObject`：存放 runtime wasm 原始字节
- `ProposalMeta`：存放提案创建时间
- `Proposals`：投票引擎核心提案表（状态、阶段、截止区块等）

## 5. 外部接口
### 5.1 `propose_runtime_upgrade`（call index = 0）
流程：
1. 校验 `ProposeOrigin`（`EnsureJointProposer`）。
2. 校验 `reason` 与 `code` 非空。
3. 调用 `JointVoteEngine::create_joint_proposal` 创建联合投票，获取统一 `proposal_id`。
4. 计算 `code_hash`，构造摘要 `Proposal` 并序列化存入 `ProposalData`。
5. 将原始 wasm 写入 `store_proposal_object(proposal_id, kind=1, code)`。
6. 调用 `store_proposal_meta` 记录创建时间。
7. 发出 `RuntimeUpgradeProposed` 事件。

### 5.2 `finalize_joint_vote`（call index = 1）
说明：
- 该入口仅作为 `Root` 手工补偿/回放入口。
- 正常情况下由投票引擎回调 `on_joint_vote_finalized` 进入同一套逻辑。

流程：
1. 校验 `Root`。
2. 从 `ProposalData` 加载提案摘要并要求当前状态为 `Voting`。
3. 若 `approved=false`：
   - 标记 `Rejected`
   - 保持投票引擎状态为 `STATUS_REJECTED`
   - 发出 `JointVoteFinalized`
4. 若 `approved=true`：
   - 从 `ProposalObject` 加载 runtime wasm
   - 尝试执行 `RuntimeCodeExecutor::execute_runtime_code`
   - 成功：标记 `Passed`，保持投票引擎状态为 `STATUS_PASSED`
   - 失败：标记 `ExecutionFailed`，并覆写投票引擎状态为 `STATUS_EXECUTION_FAILED`
   - 发出 `JointVoteFinalized` + 执行成功或失败事件
5. wasm 对象不由本模块手工删除，统一交由投票引擎 90 天延迟清理。

### 5.3 `developer_direct_upgrade`（call index = 2）
说明：
- 开发期快捷通道：联合提案发起人（国储会或省储会管理员）直接 `set_code`，不走联合投票。
- 仅在 `genesis-pallet` 的 `DeveloperUpgradeEnabled` 为 `true` 时可用。
- 链进入运行期后此调用永久失效，升级必须走 `propose_runtime_upgrade` 联合投票。

流程：
1. 校验 `ProposeOrigin`。
2. 校验 `DeveloperUpgradeCheck::is_enabled()`，关闭则拒绝（`DeveloperUpgradeDisabled`）。
3. 校验 `code` 非空。
4. 计算 `code_hash`，调用 `RuntimeCodeExecutor::execute_runtime_code`。
5. 发出 `DeveloperDirectUpgradeExecuted` 事件。

权重：使用 `frame_system::set_code()` 的系统权重。

### 5.4 投票引擎状态协同

本模块不再把所有终态统一收口到 `STATUS_EXECUTED`。当前实现与 `voting-engine` 的协作关系如下：

- 联合投票通过时，投票引擎先按通用路径把提案写成 `STATUS_PASSED`
- 联合投票拒绝时，投票引擎保持 `STATUS_REJECTED`
- 只有“联合投票已通过，但 runtime code 执行失败”这一条路径，本模块才会调用 `override_proposal_status` 原子覆写成 `STATUS_EXECUTION_FAILED`

原因：本模块的执行逻辑运行在投票引擎 `set_status_and_emit` 的回调事务内。执行失败时只需最小化覆写状态字段，不应在回调内部再次触发一轮状态机和事件链，以避免重入和双重事件问题。

提案状态流转（投票引擎侧）：
- `VOTING → PASSED`（联合投票通过且 runtime code 执行成功）
- `VOTING → REJECTED`（联合投票拒绝）
- `VOTING → PASSED → EXECUTION_FAILED`（联合投票通过，但 runtime code 执行失败并在回调内被覆写）

说明：
- 本模块自身的 `ProposalStatus`（`Passed`/`ExecutionFailed`/`Rejected`）与投票引擎侧通用状态并非一一等价，前者表达业务结果，后者表达投票引擎主状态机结果。
- 节点 UI / RPC 查询层如果需要面向用户展示真实升级结果，不能只读 `VotingEngine::Proposals.status`；
  必须继续解码本模块写入 `ProposalData` 的 `ProposalStatus`：
  - `Voting` → `投票中`
  - `Passed` → `已执行`
  - `Rejected` → `已否决`
  - `ExecutionFailed` → `执行失败`

## 6. 回调路径
`JointVoteResultCallback::on_joint_vote_finalized`：
1. 接收投票引擎统一的 `proposal_id`（无需映射反查）
2. 调用 `apply_joint_vote_result(proposal_id, approved)`

Runtime 层的 `RuntimeJointVoteResultCallback` 负责路由：先尝试 `resolution-issuance`，再尝试 `runtime-upgrade`。

## 7. 安全审查结论
### 7.1 已修复风险：执行失败误记为 Passed
旧实现中，联合投票通过后会先把提案写成 `Passed` 并清空 `code`，再尝试执行 `set_code`。如果执行失败：
- 链上状态仍显示 `Passed`
- 原始 code 已丢失

现已修复：
- 先执行，根据结果决定状态
- 执行成功才进入 `Passed`
- 执行失败进入 `ExecutionFailed`

### 7.2 已修复风险：大 wasm 直接塞入 ProposalData 导致提案创建失败
旧实现中整份 runtime wasm 会直接编码进 `ProposalData`，而投票引擎通用摘要存储无法承载 MB 级对象，runtime 升级提案会在创建阶段触发 `ProposalDataTooLarge`。

现已修复：
- `ProposalData` 只存摘要
- wasm 改为统一写入投票引擎对象层 `ProposalObject`
- 投票引擎摘要上限提升到 `100KB`
- 投票引擎对象层上限提升到 `10MB`
- runtime 升级业务自身 `MaxRuntimeCodeSize` 继续保持 `5MB`

### 7.3 已修复风险：投票引擎状态与业务执行结果脱节
旧实现/旧文档把投票引擎终态过度抽象成统一的 `STATUS_EXECUTED`，无法准确表达“联合投票已通过，但 runtime code 执行失败”的差异，也容易让查询层误判真实业务结果。

现已修复：
- 联合投票通过且执行成功时保持 `STATUS_PASSED`
- 联合投票拒绝时保持 `STATUS_REJECTED`
- 联合投票通过但执行失败时覆写为 `STATUS_EXECUTION_FAILED`
- 查询层文档已明确：展示真实升级结果时优先解码业务 `ProposalStatus`

### 7.4 已修复风险：benchmark 与实际逻辑不一致
旧版 benchmark 存在偏差。现已修复：
- `propose_runtime_upgrade` benchmark 改为真实 extrinsic
- `finalize_joint_vote` 拆分为 `approved/rejected` 两条 benchmark
- `finalize_joint_vote` benchmark 的断言已改为先跳过 `MODULE_TAG` 再解码 `ProposalData`，避免把带标签摘要误当成裸 `Proposal` 解析
- 由于 benchmark 环境不会真的改写链上 `:code`，`finalize_joint_vote(approved)` 在权重声明中会额外叠加 `frame_system::set_code()` 的系统权重

### 7.5 推荐改进
1. 当前 `finalize_joint_vote` 手工入口使用 `Root`，权限已经足够严格；若后续想与其他模块统一，可评估抽象出专用 `JointVoteFinalizeOrigin`。

## 8. 中文注释覆盖重点
本模块当前已在以下关键位置补充中文注释：
- `RuntimeCodeExecutor` 职责边界
- `ProposalStatus` 各状态语义
- 联合投票通过后的执行/失败分叉
- `STATUS_EXECUTION_FAILED` 覆写投票引擎状态的原因
- `on_joint_vote_finalized` 回调入口

## 9. 测试覆盖
已覆盖（当前单测与框架完整性检查共 16 个测试）：
- 国储会和省储会管理员均可发起提案，非联合提案发起人拒绝
- 提案摘要与对象数据正确分别存入 voting-engine
- 联合投票拒绝进入 `Rejected`（含 wasm 对象保留到统一清理）
- 联合投票通过并成功执行进入 `Passed`
- 联合投票通过但执行失败进入 `ExecutionFailed`
- 联合投票通过成功时不额外覆写投票引擎状态
- `owns_proposal` 能正确识别本模块提案
- 已终结的提案不可重复终结（`ProposalNotVoting`）
- 不存在的提案终结失败（`ProposalNotFound`）
- 开发者直升：联合提案发起人可直接升级
- 开发者直升：开关关闭时拒绝（`DeveloperUpgradeDisabled`）
- 开发者直升：非联合提案发起人拒绝（`BadOrigin`）
- 开发者直升：空 code 拒绝（`EmptyRuntimeCode`）
- GenesisConfig 构建成功
- Runtime 完整性检查

Runtime 集成测试：
- 不存在的 proposal_id 回调返回错误
- 回调正确路由到本模块并执行拒绝流程

本地验证：
- `cargo test -p runtime-upgrade`
- `cargo check -p citizenchain --features runtime-benchmarks`

## 10. 文件索引
- 模块代码：`src/lib.rs`
- Benchmark：`src/benchmarks.rs`
- 权重：`src/weights.rs`
- 技术文档：`RUNTIMEUPGRADE_TECHNICAL.md`
