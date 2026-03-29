# RESOLUTION_DESTRO_GOV Technical Notes

## 0. 功能需求
`resolution-destro-gov` 的功能需求是：为国储会、各省储会、各省储行提供"机构自有资金销毁"治理流程，由机构内部管理员发起和投票，在提案通过后自动或手动执行链上销毁。

模块必须满足以下要求：
- 仅允许有效机构发起销毁提案，且 `org` 必须与 `institution` 的真实归属一致。
- 仅允许目标机构自己的内部管理员发起提案和参与投票。
- 销毁金额必须大于 0，且执行时必须保证机构账户保留最小余额 `ED`。
- 提案投票通过后，系统应自动尝试执行销毁；若自动执行失败，提案保留为已通过状态，允许后续手动重试执行。
- 自动执行失败不能回滚已通过的投票结果。
- 销毁执行通过 `Currency::slash` 减少机构账户余额与总发行量，实现链上销毁。

## 1. 模块定位
`resolution-destro-gov` 是"机构资金销毁治理执行"模块，负责：
- 发起机构销毁提案（内部投票提案）。
- 在提案通过后执行销毁（自动尝试 + 手动重试）。

该模块不实现投票计票逻辑，投票由 `voting-engine-system` 承担。
提案数据、元数据、活跃提案限额均由 `voting-engine-system` 统一管控。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/governance/resolution-destro-gov/src/lib.rs`

---

## 2. 上下游关系与 Runtime 接线
上游常量（机构、管理员、投票时长）：
- `/Users/rhett/GMB/citizenchain/runtime/primitives/src/count_const.rs`
  - `NRC_ADMIN_COUNT = 19`
  - `PRC_ADMIN_COUNT = 9`
  - `PRB_ADMIN_COUNT = 9`

投票引擎依赖：
- `/Users/rhett/GMB/citizenchain/runtime/governance/voting-engine-system/src/lib.rs`
- 使用 trait：
  - `InternalVoteEngine::create_internal_proposal`
  - `InternalVoteEngine::cast_internal_vote`
- 使用方法：
  - `Pallet::store_proposal_data` / `get_proposal_data`
  - `Pallet::store_proposal_meta` / `get_proposal_meta` / `set_proposal_passed`
  - `Pallet::proposals`
  - `Pallet::set_status_and_emit`
- 状态常量：`STATUS_PASSED`、`STATUS_EXECUTED`
- 管理员校验：`InternalAdminProvider::is_internal_admin`

Runtime 接线：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
  - `type Currency = Balances`
  - `type InternalVoteEngine = VotingEngineSystem`

---

## 3. 数据结构与存储模型
### 动作结构
```rust
pub struct DestroyAction<Balance> {
    pub institution: InstitutionPalletId,
    pub amount: Balance,
}
```
- 编码后存入投票引擎 `ProposalData`，通过 `get_proposal_data` 读取并解码。

### 模块标识
- `MODULE_TAG = b"res-dst"`：存入 ProposalData 的前缀，用于区分不同业务模块，防止跨模块误解码。

### 本模块存储
无。提案数据、元数据、活跃提案列表均已移至 `voting-engine-system` 统一管控（lib.rs:103 注释说明）。

### 机构账户地址
- 通过 `institution_pallet_address` 从 `CHINA_CB` / `CHINA_CH` 常量中查找机构的 `duoqian_address`。
- 执行销毁时从该地址 decode 出 `AccountId`。

---

## 4. 外部接口（Calls）
### 4.1 `propose_destroy`（call index = 0）
入参：`org`, `institution`, `amount`

流程：
1. `ensure_signed`。
2. 校验 `amount > 0`。
3. 校验机构有效且 `org` 匹配。
4. 校验发起者是机构内部管理员。
5. 通过投票引擎创建内部提案（活跃提案限额由投票引擎统一检查）。
6. 将 `DestroyAction` 编码存入 ProposalData。
7. 记录提案元数据（创建时间）。
8. 发 `DestroyProposed` 事件。

### 4.2 `vote_destroy`（call index = 1）
流程：
1. 从 ProposalData 解码 `DestroyAction`。
2. 校验投票者是目标机构管理员。
3. 调 `cast_internal_vote` 计票。
4. 发 `DestroyVoteSubmitted` 事件。
5. 若投票引擎状态达到 `STATUS_PASSED`：
   - 首次进入 PASSED 时记录 `passed_at`。
   - 若本票是 `approve=true`，尝试自动执行。
   - 自动执行失败只发 `DestroyExecutionFailed` 事件，不回滚投票。

### 4.3 `execute_destroy`（call index = 2）
语义：
- 任意签名账户可调用（公开重试入口）。
- 仅当提案已 `STATUS_PASSED` 且余额校验通过时执行销毁。

用途：
- 解决"提案已通过但自动执行失败（如余额不足）"的后续重试。

---

## 5. 执行逻辑（`try_execute_destroy_from_action`）
1. 校验投票引擎提案状态为 `STATUS_PASSED`。
2. 从常量表查找机构账户地址并 decode 为 `AccountId`。
3. 校验 `free_balance >= amount + minimum_balance`（ED 保护）。
4. 调用 `Currency::slash` 执行销毁。
5. 校验 `remaining.is_zero()` 确认全额销毁成功。
6. 调用 `set_status_and_emit(STATUS_EXECUTED)` 标记终态。
7. 发 `DestroyExecuted` 事件。

重复执行防护：
- `STATUS_EXECUTED` 后提案不再是 `STATUS_PASSED`，后续执行被 `ProposalNotPassed` 拒绝。

---

## 6. 关键安全设计
1. 权限边界：
   - 发起/投票需内部管理员身份。
   - 执行是公开触发型接口（`ensure_signed`）。

2. 投票执行解耦：
   - 自动执行错误不回滚已提交投票。

3. ED 保护：
   - 执行前强制 `free_balance >= amount + minimum_balance`，避免账户被 reap。
   - `checked_add` 防止 amount + ed 溢出。

4. slash 完整性校验：
   - `ensure!(remaining.is_zero())` 确保 slash 全额完成，防止静默部分销毁。

5. panic 避免：
   - `nrc_pallet_id_bytes()` / `institution_pallet_address()` 返回 `Option`。

---

## 7. 事件与错误
事件：
- `DestroyProposed { proposal_id, org, institution, proposer, amount }`
- `DestroyVoteSubmitted { proposal_id, who, approve }`
- `DestroyExecutionFailed { proposal_id }`
- `DestroyExecuted { proposal_id, institution, amount }`

错误：
- `InvalidInstitution`：无效机构
- `InstitutionOrgMismatch`：机构类型与 org 参数不匹配
- `UnauthorizedAdmin`：非该机构管理员
- `ZeroAmount`：销毁金额为 0
- `ProposalActionNotFound`：找不到提案动作数据
- `ProposalNotPassed`：投票尚未通过
- `InstitutionAccountDecodeFailed`：机构账户地址解码失败
- `InsufficientBalance`：余额不足（含 ED 保护）

---

## 8. Weight 策略
`WeightInfo` 由 benchmark 自动产出（`weights.rs` 由 `frame-benchmarking-cli` 生成）：
- `propose_destroy()`
- `vote_destroy()`
- `execute_destroy()`

注意：当前 `weights.rs` 在旧代码上生成，包含已删除存储项（ProposalActions、ActiveProposalByInstitution、ProposalCreatedAt、ProposalPassedAt）的 proof 注释。权重数值为过估（安全），须在代码稳定后重跑 benchmark。

---

## 9. 测试覆盖
运行命令：
```
cargo test -p resolution-destro-gov
```

当前结果：12 passed

覆盖重点：
- NRC/PRC/PRB 三种组织达阈值自动执行销毁
- 非管理员不能发起/投票
- 零金额拒绝 + 余额不足拒绝
- ED 保留校验（销毁全部余额被拒）
- 自动执行失败后手动执行成功
- 被拒绝提案不阻塞新提案
- 已执行提案不阻塞新提案
- 重复投票由投票引擎拒绝
- 非管理员可触发 execute_destroy
- 无效机构返回 None

---

## 10. 运维建议
1. 监控 `DestroyExecutionFailed` 事件，出现后优先补齐机构余额再调用 `execute_destroy`。
2. `execute_destroy` 是公开触发入口；如果后续制度要求"只能本机构管理员推动执行"，需要收紧 origin。
3. `weights.rs` 须在代码稳定后重新运行 benchmark 以获取精确权重。
