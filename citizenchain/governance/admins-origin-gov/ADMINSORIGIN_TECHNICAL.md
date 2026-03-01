# ADMINS_ORIGIN_GOV Technical Notes

## 1. 模块定位
`admins-origin-gov` 是“机构管理员替换治理”pallet，只负责管理员替换动作本身，不负责投票引擎实现。

核心职责：
- 发起“同机构内管理员替换”提案。
- 将投票委托给 `voting-engine-system` 的内部投票。
- 在投票通过后执行管理员替换（可自动尝试，也可手动触发）。
- 维护提案生命周期（活跃限制、过期清理、拒绝后自动解阻塞）。

代码位置：
- `/Users/rhett/GMB/citizenchain/governance/admins-origin-gov/src/lib.rs`

---

## 2. 需求口径（业务与安全）
业务约束：
- 仅允许替换（`old_admin -> new_admin`），不允许增删管理员数量。
- 仅允许本机构管理员发起与投票。
- `org` 必须与 `institution` 实际归属一致。

治理约束：
- 同一机构同一时间只允许 1 个活跃提案。
- 提案通过后应可执行；执行失败不应回滚已记账投票。
- 被否决/终结提案不应长期阻塞后续提案。

存储治理约束：
- 提案要有过期清理能力，避免长期累积。
- 已执行提案动作应可释放存储空间。

---

## 3. 上下游关系与运行时接线
上游常量来源：
- `/Users/rhett/GMB/primitives/src/count_const.rs`
  - 管理员数量：`NRC=19`, `PRC=9`, `PRB=9`
  - 内部投票阈值：`NRC=13`, `PRC=6`, `PRB=6`
  - 投票时长：`VOTING_DURATION_BLOCKS`

投票引擎：
- `/Users/rhett/GMB/citizenchain/governance/voting-engine-system/src/lib.rs`
  - 状态：`STATUS_VOTING=0`, `STATUS_PASSED=1`, `STATUS_REJECTED=2`
- 本模块使用：
  - `InternalVoteEngine::create_internal_proposal`
  - `Pallet::internal_vote`
  - `Pallet::proposals`

Runtime 配置：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
  - `MaxAdminsPerInstitution = 32`
  - `AdminReplacementStaleProposalLifetime = VOTING_DURATION_BLOCKS * 2`
  - `impl admins_origin_gov::Config for Runtime`：
    - `type InternalVoteEngine = VotingEngineSystem`
    - `type StaleProposalLifetime = AdminReplacementStaleProposalLifetime`

---

## 4. 存储模型（当前现实）
1. `ProposalActions: Map<u64, AdminReplacementAction>`
- hasher: `Twox64Concat`
- key: `proposal_id`
- value: `{ institution, old_admin, new_admin, executed }`

2. `CurrentAdmins: Map<InstitutionPalletId, BoundedVec<AccountId>>`
- hasher: `Blake2_128Concat`
- 机构当前管理员名单（创世后仅信任链上状态）

3. `ProposalCreatedAt: Map<u64, BlockNumber>`
- hasher: `Twox64Concat`
- 提案创建高度（用于 stale 清理）

4. `ActiveProposalByInstitution: Map<InstitutionPalletId, u64>`
- hasher: `Blake2_128Concat`
- 机构到“当前活跃提案”的索引

版本信息：
- `#[pallet::storage_version(StorageVersion::new(1))]`

---

## 5. 外部接口（Calls）
### 5.1 `propose_admin_replacement`（call index = 0）
入参：
- `org`
- `institution`
- `old_admin`
- `new_admin`

主要流程：
1. 校验 `institution -> org` 映射一致。
2. 校验该机构无活跃提案（含自动清理已终结/脏索引）。
3. 校验 `who` 为该机构管理员。
4. 校验 `old_admin` 在名单、`new_admin` 不在名单。
5. 调投票引擎创建内部提案，获得真实 `proposal_id`。
6. 写入 `ProposalActions`、`ProposalCreatedAt`、`ActiveProposalByInstitution`。

### 5.2 `vote_admin_replacement`（call index = 1）
主要流程：
1. 校验提案动作存在且未执行。
2. 校验投票人是目标机构管理员。
3. 代理调用投票引擎 `internal_vote`。
4. 仅当 `approve=true` 且投票引擎状态为 `STATUS_PASSED` 时尝试自动执行。

关键语义：
- 自动执行失败不会回滚投票；会发出 `AdminReplacementExecutionFailed` 事件。

### 5.3 `execute_admin_replacement`（call index = 2）
语义：
- 独立执行入口；任意签名账户可触发。
- 仅当投票状态 `STATUS_PASSED` 且动作合法时执行替换。

用途：
- 解决“已通过但自动执行失败”的人工重试场景。

### 5.4 `cancel_stale_proposal`（call index = 3）
语义：
- 任意签名账户可触发。
- 当 `now >= created_at + StaleProposalLifetime` 时允许清理未执行提案。
- 删除 `ProposalActions` / `ProposalCreatedAt` / 机构活跃索引。

---

## 6. 提案生命周期与状态现实
生命周期（业务侧）：
1. `propose` 创建动作并建立机构活跃索引。
2. `vote` 持续计票。
3. 若通过：
   - 自动执行成功：立即替换并清理动作存储。
   - 自动执行失败：保留动作，等待 `execute_admin_replacement` 重试。
4. 若否决/终结：
   - 下次该机构 `propose` 时会检测到旧活跃索引并自动清理，不再阻塞。
5. 若长期无人处理：
   - 可通过 `cancel_stale_proposal` 清理。

当前解阻塞规则：
- `ensure_no_active_proposal` 会读取投票引擎状态：
  - `STATUS_REJECTED` 或提案不存在：视为非活跃并清理旧动作/索引。
  - 非 `STATUS_REJECTED`：仍视为活跃并阻塞新提案。

注意：
- 若投票提案已超时但尚未在投票引擎 finalize，状态仍可能是 `STATUS_VOTING`，此时仍会阻塞；可先调用投票引擎 `finalize_proposal` 使其终结。

---

## 7. 数据一致性与安全约束
1. 人数恒定约束
- 替换前后都会校验管理员数量必须等于该组织固定人数（19/9/9）。

2. 替换原子语义
- 执行路径先完成名单替换写入，再清理提案动作与索引。

3. 权限约束
- 发起与投票必须是目标机构管理员。
- 执行与 stale 清理是“公开触发型”接口（`ensure_signed`），不要求管理员身份。

4. 回滚隔离
- 投票成功后即记账；自动执行失败不会回滚投票行为。

5. Panic 风险控制
- `nrc_pallet_id_bytes` 返回 `Option`，避免 runtime `expect` panic。

---

## 8. 事件与错误（观测口径）
关键事件：
- `AdminReplacementProposed`
- `AdminReplacementVoteSubmitted`
- `AdminReplacementExecutionFailed`
- `AdminReplaced`
- `StaleProposalCancelled`

关键错误：
- `InstitutionOrgMismatch`
- `UnauthorizedAdmin`
- `OldAdminNotFound`
- `NewAdminAlreadyExists`
- `ProposalNotPassed`
- `ActiveProposalExists`
- `ProposalNotStale`
- `ProposalActionNotFound`

---

## 9. Weight 策略（当前为保守估算）
- `propose_admin_replacement`  
  `Weight::from_parts(80_000_000, 4_096) + reads_writes(8, 8)`

- `vote_admin_replacement`  
  `Weight::from_parts(200_000_000, 8_192) + reads_writes(12, 10)`

- `execute_admin_replacement`  
  `Weight::from_parts(120_000_000, 4_096) + reads_writes(8, 7)`

- `cancel_stale_proposal`  
  `Weight::from_parts(60_000_000, 4_096) + reads_writes(4, 4)`

说明：
- 目前是手工保守估算，尚未接入 benchmark 生成的 `WeightInfo`。

---

## 10. 完整性与创世约束
`integrity_test` 会断言：
- `MaxAdminsPerInstitution >= max(NRC_ADMIN_COUNT, PRC_ADMIN_COUNT, PRB_ADMIN_COUNT)`。

创世构建：
- 从 `CHINA_CB` / `CHINA_CH` 读取机构管理员写入 `CurrentAdmins`。

---

## 11. 当前测试覆盖（基于现状）
运行命令：
- `cargo test -p admins-origin-gov`

覆盖重点：
- 不同组织提案/投票权限校验。
- 达阈值自动执行成功。
- 自动执行失败不回滚投票。
- 独立执行入口失败/成功路径。
- 同机构并发提案限制。
- 否决后不再阻塞同机构新提案。
- stale 提案清理与解阻塞。
- org 与 institution 不匹配校验。
- `old_admin` 缺失、`new_admin` 已存在校验。
- 无效机构拒绝。

---

## 12. 与“需求和现实”的对齐结论
已对齐的关键点：
- 投票与执行已解耦，执行失败不会导致投票丢失回滚。
- 同机构提案并发已受控，且“已否决提案”不再形成长期阻塞。
- 已执行提案动作会清理，不再无限增长。
- 提案具备过期清理通道。

当前现实中的运维要求：
- 对超时未 finalize 的内部投票，需及时调用投票引擎 finalize 以尽快解锁机构提案通道。
