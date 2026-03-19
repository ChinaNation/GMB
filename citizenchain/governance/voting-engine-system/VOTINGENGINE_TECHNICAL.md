# Voting Engine System 技术文档

## 0. 功能需求
### 0.1 统一投票引擎能力
`voting-engine-system` 必须作为治理基础设施，统一承载内部投票、联合机构投票、公民投票三类流程，并向上层事项模块暴露稳定 trait 能力：
- `InternalVoteEngine`：创建内部提案、代理内部投票、清理内部提案（清理方法已废弃，由引擎自动注册）
- `JointVoteEngine`：创建联合提案、清理联合/公民投票状态（清理方法已废弃，由引擎自动注册）
- `JointVoteResultCallback`：联合提案终局后把结果回传给具体治理模块

### 0.2 内部投票功能需求
- 内部提案只能由业务治理模块通过 `InternalVoteEngine` trait 创建，不能直接通过外部 extrinsic 创建。
- 仅允许合法机构管理员为本机构创建内部提案。
- 仅允许同机构管理员参与内部投票，禁止跨机构投票。
- 按机构类型（NRC/PRC/PRB）使用不同通过阈值。
- 达阈值时立即通过；到期未达阈值时自动否决。

### 0.3 联合投票功能需求
- 仅允许国储会管理员创建联合提案。
- 创建提案时必须一次性锁定公民投票总分母及人口快照凭证。
- 每个机构只能由自己的多签地址提交本机构内部投票结果。
- 联合投票提交必须附带当前机构管理员的门限签名证明，链上必须校验签名人与阈值。
- 联合阶段全票通过时立即通过；未全票但已收齐全部机构票权时转入公民投票。
- 联合阶段超时后，若未全票通过，必须自动进入公民投票阶段。

### 0.4 公民投票功能需求
- 仅允许具备资格的 SFID 哈希参与公民投票。
- 每个 `proposal_id + sfid_hash` 只能投一次，且投票凭证必须防重放。
- 公民投票只接收 SFID 哈希，不接收链上明文 SFID。
- 赞成票必须“严格大于 50%”才算通过；到期后按同一规则结算。

### 0.5 状态机与安全需求
- 提案 ID 必须单调递增且不可溢出覆盖旧提案。
- 自动超时结算必须受单块上限约束，避免 `on_initialize` 无界增长。
- 联合投票终结时，投票引擎状态变更与业务模块回调必须保持原子一致。
- 自动结算若遇到回调失败，必须保留重试索引，不能让提案卡在 `Voting` 且丢失后续处理入口。
- 所有清理入口必须能释放对应提案的计票状态，避免存储长期累积。

## 1. 模块定位
`voting-engine-system` 是治理投票引擎基础模块，统一承载三类治理投票流程：
- 内部投票（`INTERNAL`）
- 联合机构投票（`JOINT`）
- 公民投票（`CITIZEN`）

它通过 trait 为上层治理模块提供标准化能力：
- `InternalVoteEngine`：创建内部提案、内部投票、内部提案清理（清理方法已废弃 no-op）
- `JointVoteEngine`：创建联合提案、联合提案清理（清理方法已废弃 no-op）
- `JointVoteResultCallback`：联合提案终结后回调目标治理模块

## 2. 核心数据结构
### 2.1 Proposal
`Proposal<BlockNumber>` 字段：
- `kind`：提案类型（内部/联合）
- `stage`：当前阶段（内部/联合/公民）
- `status`：投票中/通过/否决/已执行
  - `STATUS_VOTING = 0`：投票进行中
  - `STATUS_PASSED = 1`：投票已通过
  - `STATUS_REJECTED = 2`：投票被否决
  - `STATUS_EXECUTED = 3`：提案已执行完成（终态）。消费模块在业务逻辑成功后调用 `set_status_and_emit(STATUS_EXECUTED)` 设置。

状态流转：
```
VOTING(0) → PASSED(1) → EXECUTED(3)（执行成功）
         → REJECTED(2)（投票超时/否决）
```
- `internal_org`、`internal_institution`：内部提案专用字段
- `start`、`end`：当前阶段起止区块
- `citizen_eligible_total`：公民投票总分母

### 2.2 关键存储
- `NextProposalId`：全局提案 ID 自增计数器（`u64`）
- `Proposals`：提案主表
- `ProposalsByExpiry`：按阶段截止区块索引提案（用于自动超时结算）
- `PendingExpiryBucket`：自动结算游标（上块未处理完的过期桶）
- `InternalVotesByAccount` / `InternalTallies`
- `JointVotesByInstitution` / `JointTallies`
- `CitizenVotesBySfid` / `CitizenTallies`
- `UsedPopulationSnapshotNonce`：人口快照 nonce 防重放

## 3. 流程设计
### 3.1 内部提案
1. 通过 `do_create_internal_proposal` 创建提案，阶段为 `STAGE_INTERNAL`。
2. `do_internal_vote` 由机构管理员投票，按组织阈值判定是否通过。
3. 达阈值时立即 `Passed`（`set_status_and_emit`）。
4. 未达阈值且到期后，在 `on_initialize` 自动走 `do_finalize_internal_timeout`，直接 `Rejected`。

### 3.2 联合提案
1. 通过 `do_create_joint_proposal` 创建提案，阶段为 `STAGE_JOINT`。
2. `do_submit_joint_institution_vote` 先校验机构多签地址，再校验当前机构管理员对本次 `internal_passed` 结果的门限签名证明：
   - proof 绑定 `proposal_id + institution + internal_passed + expires_at`
   - proof 只接受当前链上管理员集合
   - proof 达到对应机构阈值后才允许记票
3. 联合全票通过则立即 `Passed`。
4. 联合未全票但已收齐总票权时，立即进入 `STAGE_CITIZEN`。
5. 联合阶段到期后，`on_initialize` 自动走 `do_finalize_joint_timeout`：
   - 全票：`Passed`
   - 非全票：自动进入 `STAGE_CITIZEN`

### 3.3 公民投票
1. `citizen_vote` 入口参数为：`(proposal_id, sfid_hash, nonce, signature, approve)`。
2. `do_citizen_vote` 校验阶段、资格、凭证、去重后计票。
3. 公民投票链路仅接收 `sfid_hash`，Runtime 不再接收/处理 SFID 明文字段。
4. 赞成票超过 50%（严格大于）时立即 `Passed`。
5. 未达阈值且到期后，`on_initialize` 自动走 `do_finalize_citizen_timeout`，按阈值判定 `Passed/Rejected`（未达阈值即 `Rejected`）。

### 3.4 自动超时结算
1. 新建提案或联合转公民时，将提案写入 `ProposalsByExpiry(end + 1)`（`end` 为最后可投票区块）。联合转公民阶段时，`advance_joint_to_citizen` 会先移除旧联合阶段的 `ProposalsByExpiry` 条目，再注册新的公民阶段过期条目，避免 `on_initialize` 对过期旧条目的无效查询。
2. 每个区块 `on_initialize` 优先处理 `PendingExpiryBucket`，再处理当前区块到期桶。
3. 单块最多处理 `MaxAutoFinalizePerBlock` 个到期提案；超出部分回写原桶并记录游标，下块继续。
4. `advance_joint_to_citizen` 现在会主动移除旧联合阶段的 expiry 条目，因此正常路径下不再留下历史索引项。自动结算仍保留兜底逻辑：若过期桶中出现历史索引项，会按当前 `proposal.end/status` 判定并跳过。
5. 若自动结算时下游回调失败，提案会重新写回过期桶，等待后续区块继续重试。

## 4. 状态终结与回调
统一通过 `set_status_and_emit`（`pub`，允许消费模块直接调用）完成终结：
1. 原子更新 `Proposals.status`
2. 发送 `ProposalFinalized` 事件
3. 对联合提案触发 `JointVoteResultCallback`
4. 若联合回调失败，则整个终结动作回滚，不留下”投票引擎已终结、业务模块未消费”的不一致状态
5. 消费模块在业务执行成功后，可调用 `set_status_and_emit(proposal_id, STATUS_EXECUTED)` 将提案标记为已执行终态，防止重复执行
6. 当 status 从 `STATUS_VOTING` 转为终态（`PASSED` / `REJECTED` / `EXECUTED`）时，自动调用 `schedule_cleanup` 注册 90 天延迟清理，消费模块无需手动调用清理方法

注：`set_status_and_emit` 可见性已从 `pub(crate)` 提升为 `pub`，以便上层治理模块直接调用设置 `STATUS_EXECUTED`。

`finalize_proposal` extrinsic 仍保留，作为手动补偿入口（例如诊断/运维场景），但正常超时路径由 `on_initialize` 自动结算。

## 5. 已修复的关键风险
### 5.1 Proposal ID 溢出
`allocate_proposal_id` 采用 `checked_add`，溢出返回 `ProposalIdOverflow`，避免 `u64::MAX` 饱和覆盖旧提案。

### 5.2 无 panic 的 NRC ID 解析
`nrc_pallet_id_bytes` 返回 `Option`，移除运行时执行路径中的 `expect`，避免潜在 panic 停链风险。

### 5.3 编码错误可观测
联合投票提交人账号编码失败时返回 `AccountIdEncodingMismatch`，不再混淆为权限错误。

### 5.4 冗余存储读取优化
- `internal_vote`：`InternalTallies::mutate` 直接返回 tally，移除额外 `get`
- `joint_vote`：`JointTallies::mutate` 直接返回 tally，移除额外 `get`
- `set_status_and_emit`：合并为单次 `try_mutate`
- `finalize_proposal`：主入口读取 proposal 后传入各 timeout 分支，避免重复读

### 5.5 清理机制
`cleanup_joint_proposal` / `cleanup_internal_proposal` 改为”小对象立即删，大前缀分块删”：
- `Proposals`
- `JointTallies` / `CitizenTallies` / `InternalTallies`
- 大体量前缀（`JointVotesByInstitution` / `CitizenVotesBySfid` / `InternalVotesByAccount` / vote credential nonce）改为写入 `PendingProposalCleanups`
- `on_initialize` 按 `MaxCleanupStepsPerBlock` 与 `CleanupKeysPerStep` 分块续清，避免 finalize 路径单次无界 `clear_prefix`

**自动清理注册**：`set_status_and_emit` 在终态转换（`STATUS_VOTING` → `PASSED` / `REJECTED` / `EXECUTED`）时自动调用 `schedule_cleanup` 注册 90 天延迟清理。因此消费模块不再需要手动调用 `cleanup_joint_proposal` 或 `cleanup_internal_proposal`，这两个 trait 方法已废弃（保留空实现以兼容 trait 定义）。同一提案的多次终态转换（如 `PASSED` → `EXECUTED`）通过 `try_push` 保持幂等，不会导致重复清理。

**`schedule_cleanup` 返回 `DispatchResult`**：该函数在目标区块队列已满时自动顺延到下一个区块（最多尝试 100 个连续区块）。如果连续 100 个区块队列均满（极端情况），使用 `defensive!` 宏在 debug/test 模式下发出警告，但仍返回 `Ok(())`，不阻塞主流程。

### 5.6 联合回调一致性
`set_status_and_emit` 现已使用存储事务包裹：
- 若 `JointVoteResultCallback` 返回错误，则回滚 `Proposal.status` 与 `ProposalFinalized` 事件。
- 避免联合提案在业务模块拒绝/异常时被错误标记为已通过或已否决。

### 5.7 自动结算失败重试
`auto_finalize_expiry_bucket` 现会把终结失败的提案重新写回 `ProposalsByExpiry`：
- 避免 `on_initialize` 取出过期桶后因为回调失败直接“吞掉重试入口”。
- 下一块会通过 `PendingExpiryBucket` 继续重试，直到回调成功或人工介入。

### 5.8 到期桶有界化
`ProposalsByExpiry` 已改为 `BoundedVec`，由 `MaxProposalsPerExpiry` 限制单个 expiry 桶大小：
- 避免同一过期区块下的提案 ID 列表无界膨胀。
- 创建提案或阶段切换时若桶已满，会返回显式错误而不是悄悄留下未调度提案。

### 5.9 联合投票门限证明上链校验
`submit_joint_institution_vote` 不再接受裸的机构自报结果，而是要求同时提交：
- `expires_at`
- `approvals[] = { public_key, signature }`

Runtime 必须对以下 payload 验签并验证阈值：

```text
payload = (
  "GMB_JOINT_DECISION_V1",
  genesis_hash,
  proposal_id,
  institution,
  internal_passed,
  expires_at
)
message = blake2_256(SCALE.encode(payload))
```

验签规则：
- 签名人必须属于该机构当前链上管理员集合
- 同一管理员不得重复计数
- 赞成签名数量必须达到该机构类型对应的内部通过阈值
- 过期 proof 必须拒绝，避免旧决定被重放

## 6. Weight 与计费
### 6.1 WeightInfo
模块定义 `WeightInfo`：
- `create_internal_proposal`
- `submit_joint_institution_vote`
- `citizen_vote`
- `finalize_proposal_internal`
- `finalize_proposal_joint`
- `finalize_proposal_citizen`

### 6.2 finalize 动态退费
`finalize_proposal` 返回 `DispatchResultWithPostInfo`，按实际阶段路径返回实际 weight，避免按最坏路径统一收费。
自动超时结算由 `on_initialize` 承担，单块处理量受 `MaxAutoFinalizePerBlock` 限制。
历史提案清理由同一个 hook 分块续跑，额度受 `MaxCleanupStepsPerBlock` / `CleanupKeysPerStep` 限制。

## 7. Benchmark 设计
启用 `runtime-benchmarks` 后提供 6 个基准入口，对应上面的 6 个 weight 函数。
其中 `citizen_vote` benchmark 走完整 `do_citizen_vote` 逻辑，而非仅存储写入。

## 8. 运行与集成注意事项
1. `JointVoteResultCallback` 应保证可恢复、可重放，不依赖脆弱临时映射。
2. ~~上层治理模块在消费联合终结结果后应调用 `cleanup_joint_proposal`。~~ **已废弃**：`set_status_and_emit` 在终态转换时自动注册 90 天延迟清理，消费模块无需手动调用 `cleanup_joint_proposal` 或 `cleanup_internal_proposal`（这两个 trait 方法现为空实现 no-op）。
3. `create_internal_proposal` / `create_joint_proposal` / `internal_vote` 外部 extrinsic 已禁用，统一要求业务模块通过 trait 接入，避免生成无业务映射的悬空提案或绕过上层副作用。
4. 联合机构线下投票系统必须产出与上面 payload 完全一致的 `sr25519` 签名，否则链上会拒绝联合投票提交。
5. 对生产链建议定期回归 benchmark，避免手工权重与实际执行漂移。

## 9. 文件索引
- 入口与存储定义：`src/lib.rs`
- 内部投票：`src/internal_vote.rs`
- 联合投票：`src/joint_vote.rs`
- 公民投票：`src/citizen_vote.rs`
- Benchmark：`src/benchmarks.rs`
- Weight：`src/weights.rs`
