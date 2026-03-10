# Voting Engine System 技术文档

## 1. 模块定位
`voting-engine-system` 是治理投票引擎基础模块，统一承载三类治理投票流程：
- 内部投票（`INTERNAL`）
- 联合机构投票（`JOINT`）
- 公民投票（`CITIZEN`）

它通过 trait 为上层治理模块提供标准化能力：
- `InternalVoteEngine`：创建内部提案、内部投票、内部提案清理
- `JointVoteEngine`：创建联合提案、联合提案清理
- `JointVoteResultCallback`：联合提案终结后回调目标治理模块

## 2. 核心数据结构
### 2.1 Proposal
`Proposal<BlockNumber>` 字段：
- `kind`：提案类型（内部/联合）
- `stage`：当前阶段（内部/联合/公民）
- `status`：投票中/通过/否决
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
2. `do_submit_joint_institution_vote` 校验机构多签地址后计票。
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
1. 新建提案或联合转公民时，将提案写入 `ProposalsByExpiry(end + 1)`（`end` 为最后可投票区块）。
2. 每个区块 `on_initialize` 优先处理 `PendingExpiryBucket`，再处理当前区块到期桶。
3. 单块最多处理 `MaxAutoFinalizePerBlock` 个到期提案；超出部分回写原桶并记录游标，下块继续。
4. 过期桶里的“历史索引项”（例如联合提前转公民后留下的旧 end）会在自动结算时按当前 `proposal.end/status` 判定并跳过。

## 4. 状态终结与回调
统一通过 `set_status_and_emit` 完成终结：
1. 原子更新 `Proposals.status`
2. 发送 `ProposalFinalized` 事件
3. 对联合提案触发 `JointVoteResultCallback`

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
新增 `cleanup_joint_proposal`，可清理联合/公民相关索引：
- `Proposals`
- `JointTallies` + `JointVotesByInstitution`
- `CitizenTallies` + `CitizenVotesBySfid`

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

## 7. Benchmark 设计
启用 `runtime-benchmarks` 后提供 6 个基准入口，对应上面的 6 个 weight 函数。
其中 `citizen_vote` benchmark 走完整 `do_citizen_vote` 逻辑，而非仅存储写入。

## 8. 运行与集成注意事项
1. `JointVoteResultCallback` 应保证可恢复、可重放，不依赖脆弱临时映射。
2. 上层治理模块在消费联合终结结果后应调用 `cleanup_joint_proposal`，避免状态无限增长。
3. 对生产链建议定期回归 benchmark，避免手工权重与实际执行漂移。

## 9. 文件索引
- 入口与存储定义：`src/lib.rs`
- 内部投票：`src/internal_vote.rs`
- 联合投票：`src/joint_vote.rs`
- 公民投票：`src/citizen_vote.rs`
