# Resolution Issuance Gov 技术文档

## 0. 功能需求

`resolution-issuance-gov` 负责把“国储会决议发行”接入联合投票，并在投票通过后驱动 `resolution-issuance-iss` 执行铸币。

模块当前必须满足以下要求：

- 国储会或省储会管理员均可发起联合提案，具体权限由 runtime 中的 `EnsureJointProposer` 判定。
- 提案必须携带非空理由、总发行金额、完整分配明细以及联合投票所需的人口快照参数。
- 分配明细必须与链上 `AllowedRecipients` 完全一致，既不能缺项、也不能多项，金额和必须等于 `total_amount`。
- `AllowedRecipients` 只允许配置为 `CHINA_CB` 省储会固定多签地址，且机构集合只允许新增、不允许删除。
- 只要还有本模块的 `Voting` 中提案，就禁止修改 `AllowedRecipients`，避免投票前后口径漂移。
- 联合投票通过后，必须自动调用 `resolution-issuance-iss` 执行发行。
- 若投票通过但执行失败，投票引擎中的最终提案状态必须落为 `STATUS_EXECUTION_FAILED`。
- 提案创建、投票终结、发行执行和本模块计数更新必须保持原子一致。

## 1. 模块定位

本模块是“决议发行治理编排层”，只负责治理编排，不负责投票计票和发币执行本身。

职责边界：

- 提案创建时调用 `VotingEngineSystem::create_joint_proposal` 创建联合提案。
- 把业务数据编码后写入投票引擎的 `ProposalData`。
- 在联合投票回调中解析业务数据并调用 `ResolutionIssuanceIss` 执行发行。
- 维护链上合法收款账户集合 `AllowedRecipients`。
- 用 `VotingProposalCount` 锁住“治理中不得改名单”的制度约束。

代码位置：

- `/Users/rhett/GMB/citizenchain/runtime/governance/resolution-issuance-gov/src/lib.rs`

## 2. Runtime 接线与上下游

Runtime 接线位置：

- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

当前关键配置：

- `ProposeOrigin = EnsureJointProposer`
- `RecipientSetOrigin = EnsureRoot<AccountId>`
- `JointVoteFinalizeOrigin = EnsureJointVoteFinalizeOrigin`
- `IssuanceExecutor = ResolutionIssuanceIss`
- `JointVoteEngine = VotingEngineSystem`
- `WeightInfo = resolution_issuance_gov::weights::SubstrateWeight<Runtime>`

说明：

- 生产态 `JointVoteFinalizeOrigin` 拒绝所有外部 origin，`finalize_joint_vote` 仅用于 benchmark / 手工回放场景。
- 正常生产路径由投票引擎在 `set_status_and_emit` 中调用 `JointVoteResultCallback::on_joint_vote_finalized`。
- Runtime 通过 `ResolutionIssuanceGov::owns_proposal(proposal_id)` 识别该提案是否属于本模块。

上下游关系：

- 上游联合投票：`/Users/rhett/GMB/citizenchain/runtime/governance/voting-engine-system/src/lib.rs`
- 下游发行执行：`/Users/rhett/GMB/citizenchain/runtime/issuance/resolution-issuance-iss/src/lib.rs`

## 3. 数据结构与存储

核心数据结构：

- `RecipientAmount<AccountId> { recipient, amount }`
- `IssuanceProposalData<AccountId> { proposer, reason, total_amount, allocations }`
- `FinalizeOutcome`
  - `ApprovedExecutionSucceeded`
  - `ApprovedExecutionFailed`
  - `Rejected`

模块标识：

- `MODULE_TAG = b"res-iss"`

本模块本地存储仅有两项：

- `AllowedRecipients: BoundedVec<AccountId, MaxAllocations>`
- `VotingProposalCount: u32`

共享存储位置：

- 提案业务数据写入 `VotingEngineSystem::ProposalData`
- 提案辅助元数据写入 `VotingEngineSystem::ProposalMeta`
- 提案状态、阶段、过期时间写入 `VotingEngineSystem::Proposals`

已删除的旧存储：

- `NextProposalId`
- `Proposals`（本模块本地提案表）
- `GovToJointVote`
- `JointVoteToGov`
- `RetryCount`

当前 `STORAGE_VERSION = 3`。

## 4. 外部接口

### 4.1 `propose_resolution_issuance`（call index = 0）

流程：

1. 校验 `ProposeOrigin`。
2. 校验 `reason` 非空。
3. 校验 `allocations` 与 `AllowedRecipients` 完全一致。
4. 调用 `JointVoteEngine::create_joint_proposal(...)` 创建联合提案。
5. 将 `MODULE_TAG + IssuanceProposalData` 写入投票引擎 `ProposalData`。
6. 写入 `ProposalMeta` 创建时间。
7. `VotingProposalCount += 1`。
8. 发出 `ResolutionIssuanceProposed`。

原子性要求：

- 联合提案创建、业务数据写入、元数据写入和 `VotingProposalCount` 增量都在同一事务里提交。

### 4.2 `finalize_joint_vote`（call index = 1）

说明：

- 这是受限回放入口。
- 生产态正常路径由投票引擎回调触发，不接受普通外部调用。

流程：

1. 校验 `JointVoteFinalizeOrigin`。
2. 进入 `apply_joint_vote_result(proposal_id, approved)`。
3. `approved=true` 时：
   - 从 `ProposalData` 解码 `IssuanceProposalData`
   - 调用 `ResolutionIssuanceIss` 执行发行
   - 成功则返回 `ApprovedExecutionSucceeded`
   - 失败则返回 `ApprovedExecutionFailed`
4. `approved=false` 时返回 `Rejected`
5. 根据结果返回 post-dispatch `actual_weight`

### 4.3 `set_allowed_recipients`（call index = 2）

流程：

1. 校验 `RecipientSetOrigin`
2. 新名单不能为空
3. `VotingProposalCount` 必须为 0
4. 新名单不能有重复地址
5. 新名单必须是旧名单的超集
6. 新名单中的每个地址必须来自 `CHINA_CB` 省储会固定多签地址
7. 写入 `AllowedRecipients`
8. 发出 `AllowedRecipientsUpdated`

## 5. 回调路径与状态语义

### 5.1 联合投票回调

`JointVoteResultCallback::on_joint_vote_finalized(vote_proposal_id, approved)` 的逻辑：

1. 调用 `apply_joint_vote_result(...)`
2. 若结果为 `ApprovedExecutionFailed`
   - 在同一事务内调用 `VotingEngineSystem::override_proposal_status(...)`
   - 将投票引擎提案状态从临时的 `STATUS_PASSED` 覆盖为 `STATUS_EXECUTION_FAILED`

### 5.2 最终状态

投票引擎最终状态：

- `STATUS_VOTING(0) -> STATUS_PASSED(1)`：投票通过且发行执行成功
- `STATUS_VOTING(0) -> STATUS_EXECUTION_FAILED(4)`：投票通过但发行执行失败
- `STATUS_VOTING(0) -> STATUS_REJECTED(2)`：投票未通过

### 5.3 事件语义

本模块事件：

- `ResolutionIssuanceProposed`
- `JointVoteFinalized`
- `IssuanceExecutionTriggered`
- `IssuanceExecutionFailed`
- `AllowedRecipientsUpdated`

投票引擎事件：

- `ProposalFinalized`

当前口径：

- `VotingEngineSystem::ProposalFinalized` 会在联合回调完成后发出，因此事件中的 `status` 与最终链上存储状态保持一致。
- 若执行失败，外部消费方既可以读取 `ProposalFinalized { status = STATUS_EXECUTION_FAILED }`，也可以结合本模块的 `IssuanceExecutionFailed` 事件做细化展示。

## 6. 一致性与安全约束

参数与集合约束：

- `reason` 不能为空
- `allocations` 不能为空
- `total_amount > 0`
- 每条 `amount > 0`
- `allocations` 中不得有重复收款人
- `allocations` 的收款人集合必须与 `AllowedRecipients` 完全一致
- `allocations` 金额和必须等于 `total_amount`

治理约束：

- 发起权限由 `EnsureJointProposer` 统一控制
- 名单更新权限为 Root
- 生产态禁止外部直接调用 `finalize_joint_vote`

原子性约束：

- `propose_resolution_issuance` 使用事务包裹，避免留下孤儿 proposal
- `apply_joint_vote_result` 使用事务包裹，避免留下“投票已结束但执行/计数未落地”的半状态
- 联合回调若失败，投票引擎会回滚本次终结

## 7. Runtime Upgrade 与迁移

`on_runtime_upgrade` 当前处理：

- v0 -> v1：若 `AllowedRecipients` 为空，则按 `CHINA_CB` 回填默认名单
- v1 -> v2：若名单存在重复地址，则回填默认名单
- v2 -> v3：确认旧版本地提案存储族已废弃；预启动链上无历史数据，无需迁移

## 8. 权重与 benchmark

代码位置：

- `/Users/rhett/GMB/citizenchain/runtime/governance/resolution-issuance-gov/src/weights.rs`
- `/Users/rhett/GMB/citizenchain/runtime/governance/resolution-issuance-gov/src/benchmarks.rs`

当前现状：

- `weights.rs` 文件头已经明确标注该产物来自旧实现。
- 注释中仍包含 `NextProposalId`、`GovToJointVote`、`JointVoteToGov`、`RetryCount` 等已删除存储。
- 因此该文件目前只能作为“保守上界”参考，不能视为与现实现完全对齐的 benchmark 产物。

后续要求：

- 在本模块与投票引擎实现稳定后，重新运行 benchmark 生成新的 `weights.rs`。

## 9. 测试覆盖

当前单测覆盖重点：

- 提案创建成功路径
- 创建事务回滚
- 发行参数校验错误路径
- 名单更新限制
- 执行失败时状态覆盖为 `STATUS_EXECUTION_FAILED`
- 回调后本地计数回收
- `owns_proposal` 的模块归属识别

建议命令：

```bash
cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/resolution-issuance-gov/Cargo.toml
```

## 10. 运维建议

- 关注 `IssuanceExecutionFailed` 事件与 `STATUS_EXECUTION_FAILED` 提案。
- 如需排查参数来源，优先读取投票引擎 `ProposalData` 中的 `IssuanceProposalData`。
- 名单变更前先确认 `VotingProposalCount == 0`，避免治理期口径漂移。
- `weights.rs` 重跑前，不应把其中的旧存储 proof 注释当成现网真实读写路径。
