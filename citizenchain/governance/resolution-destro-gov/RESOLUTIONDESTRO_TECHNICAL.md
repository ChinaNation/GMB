# RESOLUTION_DESTRO_GOV Technical Notes

## 1. 模块定位
`resolution-destro-gov` 是“机构资金销毁治理执行”模块，负责：
- 发起机构销毁提案（内部投票提案）。
- 记录并约束提案生命周期（活跃提案、过期处理、清理）。
- 在提案通过后执行销毁（自动尝试 + 手动重试）。

该模块不实现投票计票逻辑，投票由 `voting-engine-system` 承担。

代码位置：
- `/Users/rhett/GMB/citizenchain/governance/resolution-destro-gov/src/lib.rs`

---

## 2. 需求口径（业务与安全）
业务需求：
- 仅允许有效机构发起销毁；`org` 必须与 `institution` 实际归属一致。
- 仅机构内部管理员可发起和投票。
- 销毁金额必须大于 0。

治理需求：
- 同一机构同一时刻只允许 1 个活跃提案。
- 投票达到 `PASSED` 后应可执行销毁。
- 已被否决或已失效提案不应长期阻塞新提案。

安全需求：
- 投票与执行解耦：自动执行失败不能回滚投票。
- 销毁后机构账户余额不能低于 ED（Existential Deposit）。
- 过期提案可清理，避免存储长期膨胀。

---

## 3. 上下游关系与 Runtime 接线
上游常量（机构、管理员、投票时长）：
- `/Users/rhett/GMB/primitives/src/count_const.rs`
  - `NRC_ADMIN_COUNT = 19`
  - `PRC_ADMIN_COUNT = 9`
  - `PRB_ADMIN_COUNT = 9`
  - `VOTING_DURATION_BLOCKS`

投票引擎依赖：
- `/Users/rhett/GMB/citizenchain/governance/voting-engine-system/src/lib.rs`
- 使用 trait：
  - `InternalVoteEngine::create_internal_proposal`
  - `InternalVoteEngine::cast_internal_vote`
  - `InternalVoteEngine::cleanup_internal_proposal`
- 状态常量：
  - `STATUS_PASSED`
  - `STATUS_REJECTED`

Runtime 接线：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
  - `impl resolution_destro_gov::Config for Runtime`
  - `type Currency = Balances`
  - `type StaleProposalLifetime = AdminReplacementStaleProposalLifetime`
  - `type InternalVoteEngine = VotingEngineSystem`
  - `type WeightInfo = resolution_destro_gov::SubstrateWeight<Runtime>`

说明：
- 当前系统设计下，业务模块均通过 trait 转发内部投票。
- `voting-engine-system` 的 `internal_vote` 外部 extrinsic 已被拒绝（`NoPermission`），避免绕过业务模块侧校验和副作用。

---

## 4. 数据结构与存储模型
### 4.1 动作结构
`DestroyAction<Balance>`：
- `institution: InstitutionPalletId`
- `amount: Balance`

### 4.2 存储
1. `ProposalActions: Map<u64, DestroyAction<BalanceOf<T>>>`
- key: `proposal_id`
- hasher: `Blake2_128Concat`

2. `ProposalCreatedAt: Map<u64, BlockNumber>`
- 提案创建高度

3. `ProposalPassedAt: Map<u64, BlockNumber>`
- 首次进入 `PASSED` 的高度（用于“已通过但未执行”的 stale 判断锚点）

4. `ActiveProposalByInstitution: Map<InstitutionPalletId, u64>`
- 机构当前活跃提案索引

完整性约束：
- `integrity_test` 强制 `StaleProposalLifetime > 0`。
- 运行时函数 `effective_stale_lifetime()` 额外防御：若配置为 0，会退化为 1 block。

---

## 5. 外部接口（Calls）
### 5.1 `propose_destroy`（call index = 0）
主要流程：
1. `ensure_signed`。
2. 校验 `amount > 0`。
3. 校验机构有效且 `org` 匹配。
4. `check_no_active_proposal` 检查同机构活跃提案。
5. 校验发起者是机构内部管理员。
6. 通过投票引擎创建提案，拿到真实 `proposal_id`。
7. 若存在可清理旧提案，执行清理并按条件发 `StaleDestroyCancelled`。
8. 写入 `ProposalActions`、`ProposalCreatedAt`、`ActiveProposalByInstitution`。
9. 发 `DestroyProposed`。

防御细节：
- 清理旧提案前判断 `stale_id != proposal_id`，避免极端 ID 回绕误删新提案。

### 5.2 `vote_destroy`（call index = 1）
主要流程：
1. 读取提案动作。
2. 校验投票者是目标机构管理员。
3. 调 `cast_internal_vote` 计票。
4. 发 `DestroyVoteSubmitted`。
5. 读取投票引擎状态：
   - `PASSED`：首次写入 `ProposalPassedAt`；若本票是 `approve=true` 则尝试自动执行。
   - 自动执行失败只发 `DestroyExecutionFailed`，不回滚投票。
   - `REJECTED`：清理本模块与投票引擎的该提案数据。

### 5.3 `execute_destroy`（call index = 2）
语义：
- 任意签名账户可调用（公开重试入口）。
- 仅当提案已 `PASSED` 且余额校验通过时执行销毁。

用途：
- 解决“提案已通过但自动执行失败（如余额不足）”的后续重试。

### 5.4 `cancel_stale_destroy`（call index = 3）
语义：
- 任意签名账户可调用。
- 仅可取消“未通过且已 stale”提案。
- 对 `PASSED` 提案直接拒绝：`PassedProposalCannotBeCancelled`。

stale 判定：
- `now >= created_at + effective_stale_lifetime()`。

---

## 6. 提案生命周期（现实实现）
1. 创建：
- `propose_destroy` 创建内部投票提案并建立机构活跃索引。

2. 投票：
- `vote_destroy` 通过投票引擎计票。

3. 通过后：
- 自动执行成功：销毁并清理所有关联存储。
- 自动执行失败：投票保留为通过状态，等待 `execute_destroy` 手动重试。

4. 否决后：
- 在 `vote_destroy` 看到 `STATUS_REJECTED` 时立即清理。

5. 卡住的已通过提案：
- `check_no_active_proposal` 会基于 `ProposalPassedAt`（若无则回退 `ProposalCreatedAt`）判断 stale。
- 通过且 stale 的旧提案允许被新提案覆盖并清理（避免长期阻塞同机构治理）。

6. 清理行为：
- `cleanup_inactive_proposal` 同时清理：
  - `ProposalActions`
  - `ProposalCreatedAt`
  - `ProposalPassedAt`
  - `ActiveProposalByInstitution`（条件匹配）
  - 投票引擎侧提案与投票记录（经 `cleanup_internal_proposal`）

---

## 7. 关键安全设计
1. 权限边界：
- 发起/投票需内部管理员身份。
- 执行与 stale 取消是公开触发型接口（`ensure_signed`）。

2. 投票执行解耦：
- 自动执行错误不会回滚已提交投票，避免“最后一票循环回滚卡死”。

3. ED 保护：
- 执行销毁前强制 `free_balance >= amount + minimum_balance`，避免账户被 reap。

4. panic 避免：
- `nrc_pallet_id_bytes()` 返回 `Option`，避免 runtime `expect` 崩溃路径。

5. 恢复策略：
- 通过提案的 stale 判定锚点优先使用 `ProposalPassedAt`，并允许时间戳缺失时恢复覆盖，避免极端脏数据永久锁死。

---

## 8. 权重策略（当前状态）
当前为保守手工权重：
- `propose_destroy`: `80_000_000 + DbWeight(reads_writes(8, 8))`
- `vote_destroy`: `220_000_000 + DbWeight(reads_writes(14, 12))`
- `execute_destroy`: `140_000_000 + DbWeight(reads_writes(9, 8))`
- `cancel_stale_destroy`: `70_000_000 + DbWeight(reads_writes(6, 6))`

模块内已提供 `runtime-benchmarks` 结构，可用于后续 CLI 产出精确值并替换保守估算。

---

## 9. 事件与错误口径
关键事件：
- `DestroyProposed`
- `DestroyVoteSubmitted`
- `DestroyExecutionFailed`
- `DestroyExecuted`
- `StaleDestroyCancelled`

关键错误：
- `InvalidInstitution`
- `InstitutionOrgMismatch`
- `UnauthorizedAdmin`
- `ZeroAmount`
- `ProposalActionNotFound`
- `ProposalNotPassed`
- `InstitutionAccountDecodeFailed`
- `InsufficientBalance`
- `ActiveProposalExists`
- `ProposalNotStale`
- `PassedProposalCannotBeCancelled`

---

## 10. 测试覆盖（现实结果）
执行命令：
- `cargo test -p resolution-destro-gov`

当前结果：
- `18 passed; 0 failed`

覆盖重点：
- NRC/PRC/PRB 提案通过并执行销毁。
- 非管理员发起/投票拦截。
- 零金额拒绝、余额不足拒绝。
- 自动执行失败后手动执行成功。
- ED 保留校验。
- 被否决提案不阻塞新提案。
- stale 提案取消。
- `PASSED` 提案不可取消，但 stale 可被新提案覆盖。
- 非管理员可执行 `execute_destroy` 与 `cancel_stale_destroy`（按当前设计）。
- 重复投票由投票引擎拒绝。
- 非法机构返回 `None`。

---

## 11. 运维建议
1. 上线前建议跑 benchmark CLI，用实测权重替换当前手工保守值。  
2. 监控 `DestroyExecutionFailed`，出现后优先补齐机构余额再调用 `execute_destroy`。  
3. 对长期未处理的旧提案，按策略使用 `cancel_stale_destroy` 或由新提案触发覆盖清理。  
