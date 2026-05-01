# Voting Engine 技术文档

## 0. 功能需求
### 0.1 统一投票引擎能力
`voting-engine` 必须作为治理基础设施，统一承载内部投票、联合机构投票、公民投票三类流程，并向上层事项模块暴露稳定 trait 能力：
- `InternalVoteEngine`：创建普通内部提案、创建 Pending 主体内部提案、创建管理员集合变更内部提案、清理内部提案（清理方法已废弃，由引擎自动注册）
- `JointVoteEngine`：创建联合提案、清理联合/公民投票状态（清理方法已废弃，由引擎自动注册）
- `JointVoteResultCallback`：联合提案终局后把结果回传给具体治理模块

### 0.2 内部投票功能需求
- 内部提案只能由业务治理模块通过 `InternalVoteEngine` trait 创建，不能直接通过外部 extrinsic 创建。
- 仅允许合法机构管理员为本机构创建内部提案。
- 仅允许同机构管理员参与内部投票，禁止跨机构投票。
- 普通内部提案只能读取 Active 管理员主体。
- Pending 主体只能通过 `create_pending_subject_internal_proposal` 创建自身激活投票。
- 管理员集合变更只能通过 `create_admin_set_mutation_internal_proposal` 创建，并与同一治理主体下的普通活跃提案互斥。
- 创建提案时必须锁定管理员快照和阈值快照，投票期间不再实时读取主体状态。
- 国储会、省储会、省储行使用永久固定治理阈值；注册个人多签/机构多签使用主体配置阈值，并在创建提案时写入快照。
- 达阈值时立即通过；到期未达阈值时自动否决。

### 0.3 联合投票功能需求
- 仅允许国储会管理员创建联合提案。
- 创建提案时必须一次性锁定公民投票总分母及人口快照凭证。
- 每个机构只能由提案创建时管理员快照中的本机构管理员直接参与联合投票，禁止跨机构代投。
- 链上必须按治理机构固定阈值自动形成机构结果，不再依赖机构多签地址、注册多签主体阈值或线下 approvals proof。
- 联合阶段全票通过时立即通过；未全票但已收齐全部机构票权时转入公民投票。
- 联合阶段超时后，若未全票通过，必须自动进入公民投票阶段。
- 联合投票在 `STAGE_JOINT` 管理员参与阶段占用所有参与治理主体的普通互斥锁，进入 `STAGE_CITIZEN` 后释放。

### 0.4 公民投票功能需求
- 仅允许具备资格的 SFID 哈希参与公民投票。
- 每个 `proposal_id + binding_id` 只能投一次，且投票凭证必须防重放。
- 公民投票只接收 SFID 哈希，不接收链上明文 SFID。
- 赞成票必须“严格大于 50%”才算通过；到期后按同一规则结算。

### 0.5 状态机与安全需求
- 提案 ID 必须单调递增且不可溢出覆盖旧提案。
- 提案状态必须走显式状态机，只允许 `VOTING -> PASSED / REJECTED` 与 `PASSED -> EXECUTED / EXECUTION_FAILED`。
- `REJECTED / EXECUTED / EXECUTION_FAILED` 是不可再变化的终态。
- `PASSED` 是可执行/可重试态；需要重试的业务自动执行失败不得写成 `EXECUTION_FAILED`。
- 自动超时结算必须受单块上限约束，避免 `on_initialize` 无界增长。
- 联合投票终结时，投票引擎状态变更与业务模块回调必须保持原子一致。
- 自动结算若遇到回调失败，必须保留重试索引，不能让提案卡在 `Voting` 且丢失后续处理入口。
- 同一 `org + institution` 下，管理员集合变更提案与普通活跃提案互斥；普通提案之间默认允许并行。
- 所有清理入口必须能释放对应提案的计票状态与对象层存储，避免存储长期累积。

## 1. 模块定位
`voting-engine` 是治理投票引擎基础模块，统一承载三类治理投票流程：
- 内部投票（`INTERNAL`）
- 联合机构投票（`JOINT`）
- 公民投票（`CITIZEN`）

它通过 trait 为上层治理模块提供标准化能力：
- `InternalVoteEngine`：创建普通内部提案、创建 Pending 主体内部提案、创建管理员集合变更内部提案、内部提案清理（清理方法已废弃 no-op）
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
  - `STATUS_EXECUTED = 3`：提案已执行完成（终态）。消费模块在回调内通过 `set_callback_execution_result(STATUS_EXECUTED)` 静默写入；手动补偿执行路径仍可通过 `set_status_and_emit(STATUS_EXECUTED)` 推进。
  - `STATUS_EXECUTION_FAILED = 4`：投票通过但业务执行失败（终态）。由消费模块回调通过 `set_callback_execution_result(STATUS_EXECUTION_FAILED)` 在同一事务内静默写入。

状态流转：
```
VOTING(0) → PASSED(1) → EXECUTED(3)（执行成功）
         → PASSED(1) → EXECUTION_FAILED(4)（投票通过但执行失败）
         → REJECTED(2)（投票超时/否决）
```

严格禁止：

```text
VOTING  → EXECUTED / EXECUTION_FAILED / VOTING
PASSED  → VOTING / REJECTED / PASSED
REJECTED / EXECUTED / EXECUTION_FAILED → any
unknown → any
any → unknown
```

- `internal_org`、`internal_institution`：内部提案专用字段
- `start`、`end`：当前阶段起止区块
- `citizen_eligible_total`：公民投票总分母

### 2.2 关键存储
- `CurrentProposalYear`：当前提案年份（`u16`），用于年度计数器重置；年份按真实 UTC 公历年边界计算，不使用平均年秒数
- `YearProposalCounter`：当前年份内的提案计数器（`u32`），每年从 0 开始
- `NextProposalId`：兼容别名（`u64`），值为 `年份 × 1,000,000 + 计数器 + 1`，仅供外部查询
- `Proposals`：提案主表
- `ProposalsByExpiry`：按阶段截止区块索引提案（用于自动超时结算）
- `PendingExpiryBucket`：自动结算游标（上块未处理完的过期桶）
- `InternalVotesByAccount` / `InternalTallies`
- `InternalThresholdSnapshot`：内部提案创建时锁定的通过阈值。治理三类机构写入固定制度阈值；注册个人多签/机构多签写入主体配置阈值。
- `JointVotesByAdmin` / `JointInstitutionTallies`
- `JointVotesByInstitution` / `JointTallies`
- `CitizenVotesByBindingId` / `CitizenTallies`
- `UsedPopulationSnapshotNonce`：人口快照 nonce 防重放
- `ProposalData`：提案摘要层存储（默认上限 100KB）
- `ProposalObjectMeta` / `ProposalObject`：提案对象层存储（默认上限 10MB）
- `CallbackExecutionScopes`：回调执行临时作用域，只在 `set_status_and_emit` 调用业务回调期间存在，用于限制 `set_callback_execution_result` 的使用位置。
- `InternalProposalMutexes`：同一治理主体 `(org, institution)` 的内部提案互斥状态。
- `ProposalMutexBindings`：`proposal_id` 持有的互斥锁反向绑定，用于终态、阶段切换和清理时释放。

### 2.3 内部提案互斥
互斥 key：

```text
(org, institution)
```

互斥类型：

- `Regular`：普通内部治理事项，允许同一主体多个普通事项并行。
- `AdminSetMutationExclusive`：管理员集合变更事项，同一主体下必须独占。

规则：

- 创建 `Regular` 时，如果同一主体已有 `AdminSetMutationExclusive`，拒绝创建。
- 创建 `AdminSetMutationExclusive` 时，如果同一主体已有管理员集合变更提案或 `regular_active_count > 0`，拒绝创建。
- `STATUS_PASSED` 不释放内部管理员集合变更锁；只有执行成功、否决或执行失败终态才释放。
- 联合提案在 `STAGE_JOINT` 锁定所有参与机构的 `Regular` 锁；进入 `STAGE_CITIZEN` 后释放这些锁。

## 3. 流程设计
### 3.1 内部提案
1. 普通业务通过 `do_create_internal_proposal` 创建提案，阶段为 `STAGE_INTERNAL`，只接受 Active 管理员主体，并登记 `Regular` 锁。
2. 创建多签主体的业务通过 `do_create_pending_subject_internal_proposal` 创建提案，只接受 Pending 管理员主体，并登记 `Regular` 锁。
3. 管理员集合变更通过 `do_create_admin_set_mutation_internal_proposal` 创建提案，并登记 `AdminSetMutationExclusive` 锁。
4. 创建时写入 `AdminSnapshot` 与 `InternalThresholdSnapshot`，后续投票只认快照。NRC/PRC/PRB 的快照值来自固定治理常量；ORG_DUOQIAN 的快照值来自 Active/Pending 注册多签主体。
5. `do_internal_vote` 由快照内管理员投票，按阈值快照判定是否通过。
6. 达阈值时立即 `Passed`（`set_status_and_emit`）。
7. 未达阈值且到期后，在 `on_initialize` 自动走 `do_finalize_internal_timeout`，直接 `Rejected`。

### 3.2 联合提案
1. 通过 `do_create_joint_proposal` 创建提案，阶段为 `STAGE_JOINT`，并为所有参与机构登记 `Regular` 锁。
2. `joint_vote` 由提案管理员快照中的机构管理员个人钱包直接上链投票：
   - `proposal_id + institution + who` 只能投一次
   - 仅允许当前机构管理员投票
   - 投票结果立即写入 `JointVotesByAdmin`
3. 链上同步维护 `JointInstitutionTallies`：
   - `yes >= fixed_governance_threshold` 时，自动把该机构结果记为 `approved`
   - `yes + remaining_admins < fixed_governance_threshold` 时，自动把该机构结果记为 `rejected`
   - 联合投票永远只覆盖国储会、省储会、省储行，不读取 ORG_DUOQIAN 注册多签主体阈值，也不新增联合阈值快照。
4. 机构结果形成后写入 `JointVotesByInstitution`，并按机构权重累计到 `JointTallies`。
5. 联合全票通过则立即 `Passed`。
6. 任一机构一旦自动形成 `rejected`，由于联合阶段要求全票通过，会立即进入 `STAGE_CITIZEN`，并释放管理员阶段互斥锁。
7. 联合阶段到期后，`on_initialize` 自动走 `do_finalize_joint_timeout`：
   - 全票：`Passed`
   - 非全票：自动进入 `STAGE_CITIZEN`，并释放管理员阶段互斥锁

### 3.3 公民投票
1. `citizen_vote` 入口参数为：`(proposal_id, binding_id, nonce, signature, approve)`。
2. `do_citizen_vote` 校验阶段、资格、凭证、去重后计票。
3. 公民投票链路仅接收 `binding_id`，Runtime 不再接收/处理 SFID 明文字段。
4. 赞成票超过 50%（严格大于）时立即 `Passed`。
5. 未达阈值且到期后，`on_initialize` 自动走 `do_finalize_citizen_timeout`，按阈值判定 `Passed/Rejected`（未达阈值即 `Rejected`）。

### 3.4 自动超时结算
1. 新建提案或联合转公民时，将提案写入 `ProposalsByExpiry(end + 1)`（`end` 为最后可投票区块）。联合转公民阶段时，`advance_joint_to_citizen` 会先移除旧联合阶段的 `ProposalsByExpiry` 条目，再注册新的公民阶段过期条目，避免 `on_initialize` 对过期旧条目的无效查询。
2. 每个区块 `on_initialize` 优先处理 `PendingExpiryBucket`，再处理当前区块到期桶。
3. 单块最多处理 `MaxAutoFinalizePerBlock` 个到期提案；超出部分回写原桶并记录游标，下块继续。
4. `advance_joint_to_citizen` 现在会主动移除旧联合阶段的 expiry 条目，因此正常路径下不再留下历史索引项。自动结算仍保留兜底逻辑：若过期桶中出现历史索引项，会按当前 `proposal.end/status` 判定并跳过。
5. 若自动结算时下游回调失败，提案会重新写回过期桶，等待后续区块继续重试。

## 4. 状态终结与回调
投票结果统一通过 `set_status_and_emit` 完成终结；回调内的业务执行结果通过 `set_callback_execution_result` 静默写入：
1. 原子更新 `Proposals.status`
2. 对联合提案触发 `JointVoteResultCallback`，对内部提案触发 `InternalVoteResultCallback`
3. 若回调返回错误，则整个终结动作回滚，不留下”投票引擎已终结、业务模块未消费”的不一致状态
4. 回调成功后，再由外层发送一次 `ProposalFinalized` 事件；若业务模块在回调中用 `set_callback_execution_result` 写入 `EXECUTED / EXECUTION_FAILED`，事件里的 `status` 使用该最终状态
5. 回调内不得调用 `set_status_and_emit(STATUS_EXECUTED)`，否则会形成嵌套终结事件；只能调用 `set_callback_execution_result`
6. `set_callback_execution_result` 必须处于 `CallbackExecutionScopes[proposal_id]` 作用域内，否则返回 `InvalidProposalStatus`
7. 手动补偿执行路径不在回调事务内，仍可调用 `set_status_and_emit(proposal_id, STATUS_EXECUTED)` 将提案标记为已执行终态，防止重复执行
8. 当 status 从 `STATUS_VOTING` 转为终态（`PASSED` / `REJECTED` / `EXECUTED` / `EXECUTION_FAILED`）时，自动调用 `schedule_cleanup` 注册 90 天延迟清理，消费模块无需手动调用清理方法
9. 内部提案互斥锁在 `STATUS_REJECTED / STATUS_EXECUTED / STATUS_EXECUTION_FAILED` 时释放；联合提案在 `STAGE_JOINT` 直接 `STATUS_PASSED` 时也释放。

注：`set_status_and_emit` 可见性已从 `pub(crate)` 提升为 `pub`，以便上层治理模块直接调用设置 `STATUS_EXECUTED`。

旧低级覆盖 API `override_proposal_status` 已删除，不再允许业务模块绕过状态机直接写状态。

`finalize_proposal` extrinsic 仍保留，作为手动补偿入口（例如诊断/运维场景），但正常超时路径由 `on_initialize` 自动结算。

## 5. 已修复的关键风险
### 5.1 Proposal ID 溢出
`allocate_proposal_id` 采用 `checked_add`，溢出返回 `ProposalIdOverflow`，避免 `u64::MAX` 饱和覆盖旧提案。

### 5.2 无 panic 的 NRC ID 解析
`nrc_pallet_id_bytes` 返回 `Option`，移除运行时执行路径中的 `expect`，避免潜在 panic 停链风险。

### 5.3 联合投票身份模型收敛
联合投票已收敛到“管理员直接上链投票”模型：
- 不再要求 `origin == main_address`
- 不再依赖线下门限签名 proof
- 权限校验完全基于当前链上管理员集合
- 机构结果完全由链上按阈值自动形成

### 5.4 冗余存储读取优化
- `internal_vote`：`InternalTallies::mutate` 直接返回 tally，移除额外 `get`
- `joint_vote`：`JointTallies::mutate` 直接返回 tally，移除额外 `get`
- `set_status_and_emit`：合并为单次 `try_mutate`
- `finalize_proposal`：主入口读取 proposal 后传入各 timeout 分支，避免重复读

### 5.5 清理机制
`cleanup_joint_proposal` / `cleanup_internal_proposal` 改为”统一注册、分阶段清理”：
- 摘要层：`ProposalData` / `ProposalMeta`
- 对象层：`ProposalObjectMeta` / `ProposalObject`
- 核心层：`Proposals` / `JointTallies` / `CitizenTallies` / `InternalTallies` / `InternalThresholdSnapshot` / `ProposalMutexBindings`
- 大体量前缀（`JointVotesByAdmin` / `JointVotesByInstitution` / `JointInstitutionTallies` / `CitizenVotesByBindingId` / `InternalVotesByAccount` / vote credential nonce）写入 `PendingProposalCleanups`
- `on_initialize` 按 `MaxCleanupStepsPerBlock` 与 `CleanupKeysPerStep` 分块续清，避免 finalize 路径单次无界 `clear_prefix`

**自动清理注册**：`set_status_and_emit` 在终态转换（`STATUS_VOTING` → `PASSED` / `REJECTED` / `EXECUTED` / `EXECUTION_FAILED`）时自动调用 `schedule_cleanup` 注册 90 天延迟清理。因此消费模块不再需要手动调用 `cleanup_joint_proposal` 或 `cleanup_internal_proposal`，这两个 trait 方法已废弃（保留空实现以兼容 trait 定义）。同一提案的多次终态转换（如 `PASSED` → `EXECUTED`）通过 `try_push` 保持幂等，不会导致重复清理。

**清理阶段顺序**：
- `InternalVotes → JointAdminVotes → JointInstitutionVotes → JointInstitutionTallies → CitizenVotes → VoteCredentials → ProposalObject → FinalCleanup`
- 其中 `ProposalObject` 专门负责删除对象层存储；`FinalCleanup` 释放残留互斥锁并删除摘要层、提案主表与 tally。

**`schedule_cleanup` 返回 `DispatchResult`**：该函数在目标区块队列已满时自动顺延到下一个区块（最多尝试 100 个连续区块）。如果连续 100 个区块队列均满（极端情况），使用 `defensive!` 宏在 debug/test 模式下发出警告，但仍返回 `Ok(())`，不阻塞主流程。

### 5.6 回调一致性与最终事件收口
`set_status_and_emit` 现已使用存储事务包裹：
- 若 `JointVoteResultCallback` 或 `InternalVoteResultCallback` 返回错误，则回滚 `Proposal.status` 与 `ProposalFinalized` 事件。
- `ProposalFinalized` 在回调完成后由外层统一发出一次，避免业务模块成功执行时出现重复最终事件。
- `set_callback_execution_result` 只允许回调在 `CallbackExecutionScopes` 内把 `STATUS_PASSED` 静默推进到 `STATUS_EXECUTED / STATUS_EXECUTION_FAILED`，不发事件、不登记清理、不释放互斥锁。
- 避免提案在业务模块拒绝/异常时被错误标记为已通过或已否决。

### 5.6.1 状态机强约束
`set_status_and_emit` 写状态前会校验旧状态和目标状态：

- 允许：`VOTING -> PASSED`
- 允许：`VOTING -> REJECTED`
- 允许：`PASSED -> EXECUTED`
- 允许：`PASSED -> EXECUTION_FAILED`
- 禁止：终态继续变化
- 禁止：`PASSED -> REJECTED`
- 禁止：同状态重复写入

当前无存量链、无存量提案，因此本次不包含存储迁移。

### 5.7 自动结算失败重试
`auto_finalize_expiry_bucket` 现会把终结失败的提案重新写回 `ProposalsByExpiry`：
- 避免 `on_initialize` 取出过期桶后因为回调失败直接“吞掉重试入口”。
- 下一块会通过 `PendingExpiryBucket` 继续重试，直到回调成功或人工介入。

### 5.8 到期桶有界化
`ProposalsByExpiry` 已改为 `BoundedVec`，由 `MaxProposalsPerExpiry` 限制单个 expiry 桶大小：
- 避免同一过期区块下的提案 ID 列表无界膨胀。
- 创建提案或阶段切换时若桶已满，会返回显式错误而不是悄悄留下未调度提案。

### 5.9 联合投票自动机构结算
`joint_vote` 不再接收线下 approvals proof，而是直接接收管理员个人钱包的链上投票：
- 仅允许提案管理员快照中的本机构管理员投票
- 同一管理员对同一 `proposal_id + institution` 只能投一次
- 赞成票达到固定治理阈值时，自动形成该机构 `approved`
- 剩余管理员已不足以让赞成达到固定治理阈值时，自动形成该机构 `rejected`
- 任一机构 `rejected` 后，联合阶段立即结束并进入公民投票

### 5.9.1 治理固定阈值与注册多签动态阈值边界
阈值来源按主体类型硬隔离：

- 联合投票只服务 NRC/PRC/PRB 三类治理机构，机构阈值来自 `NRC_INTERNAL_THRESHOLD`、`PRC_INTERNAL_THRESHOLD`、`PRB_INTERNAL_THRESHOLD` 固定常量。
- 联合投票不调用 `InternalThresholdProvider::pass_threshold`，避免把注册多签主体阈值误用于治理联合投票。
- NRC/PRC/PRB 的内部提案创建时也使用固定治理阈值写入 `InternalThresholdSnapshot`。
- ORG_DUOQIAN 注册个人多签/机构多签的阈值由注册主体配置提供；创建 Active/Pending 内部提案时写入 `InternalThresholdSnapshot`，投票期间只读快照。
- 因联合投票阈值是永久制度常量，本模块不新增 `JointThresholdSnapshot`，也不需要存储迁移。

### 5.10 Proposal ID 年份边界
`allocate_proposal_id` 的年份段按 UTC 公历年计算：

```text
proposal_id = UTC 公历年份 × 1,000,000 + 年内计数器
```

实现要求：

- 使用 Unix 秒数先换算 UTC 天数，再按公历闰年规则确定年份。
- 闰年规则为：能被 4 整除且不能被 100 整除，或能被 400 整除。
- 禁止使用 `365.2425 天` 这类平均年长直接整除计算年份，因为平均年边界会在真实元旦前后漂移，导致提案 ID 被分配到错误年份段。
- 单测必须覆盖真实元旦边界，尤其是曾经会错分的 `2028-01-01 00:00:00 UTC`。

### 5.11 管理员集合变更互斥
已新增内部提案互斥机制：

- 同一治理主体下，管理员集合变更与普通活跃提案互斥。
- 普通提案之间默认不互斥。
- 联合投票在管理员参与阶段占用所有参与机构的普通锁。
- 联合投票进入公民投票阶段后释放管理员互斥锁，避免长期公民投票阻塞管理员更换。
- `STATUS_PASSED` 的管理员集合变更提案不会释放独占锁，必须进入 `STATUS_EXECUTED / STATUS_REJECTED / STATUS_EXECUTION_FAILED` 后释放。

## 6. Weight 与计费
### 6.1 WeightInfo
模块定义 `WeightInfo`：
- `create_internal_proposal`
- `joint_vote`
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
3. `create_internal_proposal` / `create_pending_subject_internal_proposal` / `create_joint_proposal` / `internal_vote` 外部 extrinsic 已禁用，统一要求业务模块通过 trait 接入，避免生成无业务映射的悬空提案或绕过上层副作用。
4. 普通业务模块必须调用 `create_internal_proposal`；只有 `duoqian-manage` 创建机构多签/个人多签主体时可调用 `create_pending_subject_internal_proposal`。
5. 管理员集合变更只能由 `admins-change` 调用 `create_admin_set_mutation_internal_proposal`。
6. 当前生产链已确认无活跃未终态提案，新增互斥存储以空状态启用；若未来在有存量活跃提案的链上升级，需要先补一次性锁重建迁移。
7. 联合投票客户端必须保证“选中的管理员钱包 = 实际上链签名钱包”，否则会被链上管理员身份或重复投票校验拒绝。
8. 对生产链建议定期回归 benchmark，避免手工权重与实际执行漂移。

## 9. 文件索引
- 入口与存储定义：`src/lib.rs`
- 内部投票：`src/internal_vote.rs`
- 联合投票：`src/joint_vote.rs`
- 公民投票：`src/citizen_vote.rs`
- 提案清理调度：`src/proposal_cleanup.rs`
- 活跃提案限额：`src/active_proposal_limit.rs`
- Benchmark：`src/benchmarks.rs`
- Weight：`src/weights.rs`
