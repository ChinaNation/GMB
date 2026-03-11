# ADMINS_ORIGIN_GOV Technical Notes

## 0. 功能需求
### 0.1 模块职责
`admins-origin-gov` 负责把“同机构管理员替换”包装成一个受治理约束的链上流程，要求：
- 只允许做替换，不允许增删管理员人数。
- 只允许本机构管理员发起与投票。
- 内部投票由 `voting-engine-system` 统一承载。

### 0.2 提案创建需求
- `org` 必须与 `institution` 的真实归属严格一致。
- `old_admin` 必须存在于当前管理员列表中。
- `new_admin` 不能已在当前管理员列表中。
- 同一机构同一时间只允许一个活跃提案。

### 0.3 执行与失败恢复需求
- 提案投票通过后应自动尝试执行管理员替换。
- 自动执行失败不能回滚已通过的投票结果。
- 已通过但执行失败的提案应保留一段补救窗口，允许后续手动执行。
- 已通过但执行失败的提案不能被第三方直接用普通 stale 清理取消。

### 0.4 生命周期与清理需求
- 被拒绝或不存在的历史提案不能长期阻塞同机构新提案。
- 未通过且长期无人处理的 stale 提案应可被清理。
- 已通过但执行失败的 stale 提案，在补救窗口结束后，应只在机构再次发起新提案时自动解阻塞。

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

4. `ProposalPassedAt: Map<u64, BlockNumber>`
- hasher: `Twox64Concat`
- 首次达到 `STATUS_PASSED` 的区块高度（用于给失败后的手动补救保留窗口）

5. `ActiveProposalByInstitution: Map<InstitutionPalletId, u64>`
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
- 仅允许清理“未通过且未执行”的 stale 提案。
- 当 `now >= created_at + StaleProposalLifetime` 时允许清理。
- 删除 `ProposalActions` / `ProposalCreatedAt` / `ProposalPassedAt` / 机构活跃索引。

---

## 6. 提案生命周期与状态现实
生命周期（业务侧）：
1. `propose` 创建动作并建立机构活跃索引。
2. `vote` 持续计票。
3. 若通过：
   - 自动执行成功：立即替换并清理动作存储。
   - 自动执行失败：记录 `ProposalPassedAt`，保留动作，等待 `execute_admin_replacement` 重试。
4. 若否决/终结：
   - 下次该机构 `propose` 时会检测到旧活跃索引并自动清理，不再阻塞。
5. 若长期无人处理：
   - 未通过提案：可通过 `cancel_stale_proposal` 清理。
   - 已通过但执行失败提案：在通过后窗口结束后，仅在机构下次 `propose` 时自动清理。

当前解阻塞规则：
- `ensure_no_active_proposal` 会读取投票引擎状态：
  - `STATUS_REJECTED` 或提案不存在：视为非活跃并清理旧动作/索引。
  - `STATUS_PASSED`：在 `ProposalPassedAt + StaleProposalLifetime` 之前继续阻塞；超时后在下次 `propose` 时自动清理。
  - 其他状态：仍视为活跃并阻塞新提案。

注意：
- 投票引擎已在 `on_initialize` 自动处理超时提案，通常无需人工 finalize。
- `finalize_proposal` 仍可作为手动补偿入口（诊断/运维场景）。

---

## 7. 数据一致性与安全约束
1. 人数恒定约束
- 替换前后都会校验管理员数量必须等于该组织固定人数（19/9/9）。

2. 替换原子语义
- 执行路径先完成名单替换写入，再清理提案动作与索引。

3. 权限约束
- 发起与投票必须是目标机构管理员。
- 执行与 stale 清理是“公开触发型”接口（`ensure_signed`），不要求管理员身份。
 - 但 stale 清理入口不能取消已通过提案，避免第三方删除治理已批准但待补救的动作。

4. 回滚隔离
- 投票成功后即记账；自动执行失败不会回滚投票行为。

5. Panic 风险控制
- `nrc_pallet_id_bytes` 返回 `Option`，避免 runtime `expect` panic。

6. 已修复风险：已通过提案可被第三方 stale 清理
- 旧实现中，`cancel_stale_proposal` 只看 `created_at`，不区分提案是否已经通过。
- 这意味着“已通过但自动执行失败”的提案，理论上可能被任意签名账户在超时后直接取消。
- 现已修复：
  - 新增 `ProposalPassedAt`
  - `cancel_stale_proposal` 禁止取消 `STATUS_PASSED` 提案
  - 已通过失败提案仅在补救窗口结束后，由本机构再次发起新提案时自动解阻塞

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
- `PassedProposalCannotBeCancelled`

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
- 已通过失败提案不能被普通 stale 取消。
- 已通过失败提案在补救窗口前阻塞、窗口后自动解阻塞。
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
- 常规场景依赖投票引擎自动超时结算即可；仅在异常排障时使用 `finalize_proposal` 手动补偿。

---

## 13. 推送层（Rust 内置，不改 pallet 本体）

定位：
- 推送层已内置到 `nodeui-desktop-shell`（`desktop/src-tauri/src/main.rs`）。
- 只做“链上管理员状态 -> 登录端授权快照 JSON”的同步，不参与投票、不参与替换执行。
- 不修改 `admins-origin-gov/src/lib.rs` 的任何业务逻辑。

同步来源：
- 读取 `adminsOriginGov.currentAdmins` 链上存储（finalized 状态）。

同步目标：
- 运行时快照文件：`app_data/org-registry.snapshot.json`（由桌面壳维护）。
- `nodeui` 登录判权实时读取该快照，不再依赖静态 `ORG_REGISTRY`。

运行方式：
- `nodeui` 启动时自动拉起内置同步线程，以 finalized 区块订阅驱动链上变更并写入快照（断线自动重连）。
- 无需部署独立 Node 推送服务，无需额外安装 `push-layer` 依赖。
