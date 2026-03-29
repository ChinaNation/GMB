# RESOLUTION_ISSUANCE_GOV Technical Notes

## 0. 功能需求
`resolution-issuance-gov` 的功能需求是：把"国储会决议发行"治理流程接入联合投票，并在投票结果确定后自动驱动发行执行模块完成铸币分配。

模块必须满足以下业务要求：
- 仅允许国储会管理员发起决议发行提案。
- 每个提案必须携带发行理由、总发行金额、完整的收款账户分配明细和联合投票所需的人口快照参数。
- 分配明细必须与链上配置的合法收款账户集合完全一致，且金额总和必须等于 `total_amount`。
- 合法收款账户对应省储会固定多签账户地址；账户管理员可变更，但账户地址本身不变。
- 省储会机构集合只允许新增，不允许删除；且收款账户必须是 CHINA_CB 省储会地址。
- 联合投票通过后，系统必须自动调用 `resolution-issuance-iss` 执行发行；未通过则直接结束提案。
- 若投票通过但发行执行失败，提案状态必须覆盖为 `STATUS_EXECUTION_FAILED`（值 4），与执行成功的 `STATUS_PASSED`（值 1）明确区分。
- 治理进行中不得切换合法收款账户集合，避免同一业务口径在投票前后发生漂移。
- 提案创建、联合投票终结、发行执行结果落账和计数变更必须原子提交，不能出现半完成状态。

## 1. 模块定位
`resolution-issuance-gov` 是"决议发行治理编排模块"，负责把国储会发起的发行决议接入联合投票，并在投票结果落地后驱动发行执行模块。

模块职责：
- 将业务数据（理由、金额、分配明细）编码后存入投票引擎的 ProposalData 统一存储。
- 在投票通过时调用 `resolution-issuance-iss` 执行发行。
- 执行失败时覆盖投票引擎的提案状态为 `STATUS_EXECUTION_FAILED`。
- 维护链上合法收款账户集合，防止治理期间热切换带来口径漂移。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/governance/resolution-issuance-gov/src/lib.rs`

---

## 2. 运行��接线与上下游
Runtime 接线位置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

关键配置：
- `NrcProposeOrigin = EnsureNrcAdmin`：仅 NRC 管理员可发起提案。
- `RecipientSetOrigin = EnsureRoot<AccountId>`：仅 Root 可更新收款账户集合。
- `JointVoteFinalizeOrigin = EnsureJointVoteFinalizeOrigin`：生产态拒绝外部 finalize 调用；benchmark 下允许 Root。
- `IssuanceExecutor = ResolutionIssuanceIss`：发行执行委托给发行模块，trait 载荷受 `MaxReasonLen` / `MaxAllocations` 限制。
- `JointVoteEngine = VotingEngineSystem`：联合投票创建由投票引擎承担。

联合投票回调路由：
- 投票引擎 `set_status_and_emit` 内调用 `on_joint_vote_finalized` 回调。
- 回调通过 `owns_proposal`（检查 `MODULE_TAG` 前缀）判断归属，由本模块消费。

---

## 3. 数据结构与存储
### 核心结构
- `RecipientAmount { recipient: AccountId, amount: u128 }`：单条收款分配项。
- `IssuanceProposalData { proposer, reason, total_amount, allocations }`：编码后存入投票引�� `ProposalData`。
- `FinalizeOutcome { ApprovedExecutionSucceeded, ApprovedExecutionFailed, Rejected }`：内部��行结果。

### 模块��识
- `MODULE_TAG = b"res-iss"`：存入 ProposalData 的前缀，用于区分不同业务模块。

### 本模块存储（仅 2 项）
- `AllowedRecipients: BoundedVec<AccountId, MaxAllocations>`：合法收款账户集合。
- `VotingProposalCount: u32`：当前 Voting 状态的提案数量，用于阻止治理中途切换名单。

### 业务数据存储位置
- 提案业务数据（理由、金额、分配明细）存储在投票引擎的 `ProposalData<T>`，以 `MODULE_TAG` 前缀编码。
- 提案状态（VOTING/PASSED/REJECTED/EXECUTION_FAILED）存储在投票引擎的 `Proposals<T>`。
- 本模块不维护自己的提案表或 ID 分配器。

### Genesis
- 默认 `allowed_recipients` 来源于 `CHINA_CB` 的机构收款地址（跳过索引 0 的 NRC 节点）。
- 创世构建时强制唯一性检查��重复地址会直接��败。

### 版本
- `STORAGE_VERSION = 3`

---

## 4. 外部接口（Calls + Trait）
### 4.1 `propose_resolution_issuance`（call index = 0）
流程：
1. 校验 origin 为 NRC 管理员。
2. 校验 `reason` 非空。
3. 校验 allocations 与 `AllowedRecipients` 完全一致（数量、成员、无重复、金额和）。
4. 调用联合投票引擎创建提案，获取 `proposal_id`。
5. 将 `IssuanceProposalData` 以 MODULE_TAG 前缀编��后存入投票引擎 ProposalData。
6. `VotingProposalCount += 1`。
7. 发 `ResolutionIssuanceProposed` 事件。

实现约束：
- 联合投票创建、ProposalData 写入与 VotingProposalCount 增量在同一事务里提交。

### 4.2 `finalize_joint_vote`（call index = 1）
流程：
1. 校验来源为 `JointVoteFinalizeOrigin`。
2. 进入 `apply_joint_vote_result`：
   - `approved=true` 时从 ProposalData 解码业务数据并尝试执行发行。
   - 执行成功：cleanup，`VotingProposalCount -= 1`，发 `JointVoteFinalized + IssuanceExecutionTriggered`，返回 `ApprovedExecutionSucceeded`。
   - 执行失败：cleanup，`VotingProposalCount -= 1`，发 `JointVoteFinalized + IssuanceExecutionFailed`，返回 `ApprovedExecutionFailed`。
   - `approved=false`：cleanup，`VotingProposalCount -= 1`，发 `JointVoteFinalized { approved: false }`，返回 `Rejected`。
3. 返回 post-dispatch `actual_weight`（失败/拒绝路径退费）。

说明：
- 联合投票终结、发行执行和计数变更在同一事务里提交。
- 在生产环境中，此 extrinsic 被 `JointVoteFinalizeOrigin` 封死；实际触发路径是投票引擎 `set_status_and_emit` → `on_joint_vote_finalized` 回调。

### 4.3 `set_allowed_recipients`（call index = 2）
流程：
1. Root 权限。
2. 新名单不能为空。
3. `VotingProposalCount` 必须为 0（治理中禁止切换名单）。
4. 新名单去重校验通过后写入存储并发 `AllowedRecipientsUpdated` 事件。

### 4.4 Trait 回调：`JointVoteResultCallback::on_joint_vote_finalized`
- 由投票引擎 `set_status_and_emit(STATUS_PASSED)` 内部调用。
- 本模块实��中��调用 `apply_joint_vote_result` 获取 `FinalizeOutcome`。
- 若结果为 `ApprovedExecutionFailed`，回调在同一事务内调用 `override_proposal_status` 将投票引擎的提案状态从 STATUS_PASSED 覆盖为 STATUS_EXECUTION_FAILED。
- 若 `override_proposal_status` 失败则返回 Err，投票引擎回滚整个 `set_status_and_emit` 事务。

---

## 5. 生命周期与状态机
提案状态存储在投票引擎 `Proposals` 中，状态流转：
- `STATUS_VOTING(0) → STATUS_PASSED(1)`：投票通过且执行成功。
- `STATUS_VOTING(0) → STATUS_EXECUTION_FAILED(4)`：投票通过但执行失败。
- `STATUS_VOTING(0) → STATUS_REJECTED(2)`：投票未通过。

关键��致性：
- 只有 `STATUS_VOTING` 状态允许 finalize，重复 finalize 由投票引擎层面拒绝。
- `VotingProposalCount` 与 Voting ��案数量保持同向变化。
- 提案创建和 finalize 采用事务提交，后半段任何错误都会回滚前面的本地治理写入。

---

## 6. 需求与安全约束
业务��束：
- 发行分配名单必须与链上 `AllowedRecipients` 精确匹配，不允许少人、多人、换人、重复。
- `total_amount` 必须等于 allocations 金额和。
- `amount` 不可为 0。
- `AllowedRecipients` 表示固定机构收款账户集合；管理员变更不改变账户地址语义。

治理约束：
- 仅 NRC 管理员可发起。
- 仅受控来源可 finalize（生产环境禁外部直接调用）。
- 治理进行中（`VotingProposalCount > 0`）禁止修改 `AllowedRecipients`。

执行失败语义：
- 投票通过但执行失败不会��装成通过，而是在同一事务内将状态覆盖为 `STATUS_EXECUTION_FAILED`。
- 当前无重试机制；执行失败即为终态。

---

## 7. 权重与计费口径
`WeightInfo` 由 benchmark 产出（`weights.rs` 由 `frame-benchmarking-cli` 生成）：
- `propose_resolution_issuance()`
- `finalize_joint_vote_approved()`：叠加发行执行权重的最大参数。
- `finalize_joint_vote_rejected()`
- `set_allowed_recipients()`

post-dispatch `actual_weight`：
- `ApprovedExecutionSucceeded`：不退费（`None`）。
- `ApprovedExecutionFailed`：退费到估算的 `reads_writes(3, 5)`。
- `Rejected`：退费到估算的 `reads_writes(3, 4)`。

注意：当前 `weights.rs` 在旧代码上生成，包含已删除存储项的 proof 注释。权重数值为过估（安全），但须在代码稳定后重跑 benchmark。

---

## 8. 运行时升级与迁移
当前 `STORAGE_VERSION = 3`，`on_runtime_upgrade` 处理：
- v0 → v1：若 `AllowedRecipients` 为空，按 `CHINA_CB` 回填默认名单。
- v1 → v2：若 `AllowedRecipients` 不唯一，回填默认名单。
- v3：旧存储（NextProposalId / Proposals / GovToJointVote / JointVoteToGov / RetryCount）已在链上无数据（预启动链），无需迁移。

---

## 9. 事件与错误
核心事件：
- `ResolutionIssuanceProposed { proposal_id, proposer, total_amount, allocation_count }`
- `JointVoteFinalized { proposal_id, approved }`
- `IssuanceExecutionTriggered { proposal_id, total_amount }`
- `IssuanceExecutionFailed { proposal_id }`
- `AllowedRecipientsUpdated { count }`

核心错误：
- 参数与集合约束：`EmptyReason`、`EmptyAllocations`、`InvalidAllocationCount`、`InvalidRecipientSet`、`TotalMismatch`、`DuplicateRecipient`、`DuplicateAllowedRecipient`、`ZeroAmount`、`AllocationOverflow`、`RecipientRemoved`、`RecipientNotInChinaCb`
- 生命周期：`ProposalNotFound`
- 治理控制：`RecipientsNotConfigured`、`ActiveVotingProposalsExist`
- 内部状态：`VotingProposalCountOverflow`、`VotingProposalCountUnderflow`、`JointVoteCreateFailed`、`ProposalDataStoreFailed`

---

## 10. 测试��基准
本地测试命令：
```
cargo test -p resolution-issuance-gov --quiet
```

当前��果：
- `18 passed; 0 failed`

覆盖重点：
- 提案创建与 ProposalData 写入
- 提案创建路��的事务回滚
- 回调通过/拒绝状态流转
- 执行失败时提案状态被覆盖为 STATUS_EXECUTION_FAILED
- 投票通过后若本模块后续记账失败，finalize 事务回滚
- 治理进行中禁止改收款集合、空集合/重复集合拒绝
- 关��参数校验错误路径（recipient set、total mismatch 等）

---

## 11. 运维建议
1. 监控 `IssuanceExecutionFailed` 事件，及时发现执行失败的提案。
2. 查询 `STATUS_EXECUTION_FAILED` 状态的提案进行人工排查。
3. 升级含迁移逻辑时，优先在预发布环境验证 `AllowedRecipients` 回填结果与现网一致。
4. `AllowedRecipients` 的设计前提是机构收款账户地址长期固定��机构集合只增不减。
5. `weights.rs` 须��代码稳定后重新运行 benchmark 以获取精确权重。
