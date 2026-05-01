# resolution-destro 技术说明

## 0. 功能需求
`resolution-destro` 的功能需求是：为国储会、各省储会、各省储行提供"机构自有资金销毁"治理流程，由机构内部管理员发起和投票，在提案通过后自动或手动执行链上销毁。

模块必须满足以下要求：
- 仅允许有效机构发起销毁提案，且 `org` 必须与 `institution` 的真实归属一致。
- 仅允许目标机构自己的内部管理员发起提案和参与投票。
- 销毁金额必须大于 0，且执行时必须保证机构账户保留最小余额 `ED`。
- 提案投票通过后，系统应自动尝试执行销毁；若自动执行失败，提案保持 `STATUS_PASSED`，允许后续手动重试执行。
- 自动执行失败不能回滚已通过的投票结果。
- 销毁执行通过 `Currency::slash` 减少机构账户余额与总发行量，实现链上销毁。

## 1. 模块定位
`resolution-destro` 是"机构资金销毁治理执行"模块，负责：
- 发起机构销毁提案（内部投票提案）。
- 在提案通过后执行销毁（自动尝试 + 手动重试）。

该模块不实现投票计票逻辑，投票由 `voting-engine` 承担。
提案数据、元数据、活跃提案限额均由 `voting-engine` 统一管控。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/governance/resolution-destro/src/lib.rs`

命名说明：
- 2026-04-29 起，本模块统一使用 `resolution-destro` / `resolution_destro` / `ResolutionDestro`。
- 模块位于 `citizenchain/runtime/governance/resolution-destro/`。
- `pallet_index = 14`、call index 与 `MODULE_TAG = b"res-dst"` 保持不变。

---

## 2. 上下游关系与 Runtime 接线
上游常量（机构、管理员、投票时长）：
- `/Users/rhett/GMB/citizenchain/runtime/primitives/src/count_const.rs`
  - `NRC_ADMIN_COUNT = 19`
  - `PRC_ADMIN_COUNT = 9`
  - `PRB_ADMIN_COUNT = 9`

投票引擎依赖：
- `/Users/rhett/GMB/citizenchain/runtime/governance/voting-engine/src/lib.rs`
- 使用 trait：
  - `InternalVoteEngine::create_internal_proposal_with_data`
  - `InternalVoteResultCallback`
- 使用方法：
  - `Pallet::get_proposal_data`
  - `Pallet::proposals`
  - `Pallet::retry_passed_proposal_for`
- 状态常量：`STATUS_PASSED`
- 管理员校验：`InternalAdminProvider::is_internal_admin`

Runtime 接线：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
  - `type Currency = Balances`
  - `type InternalVoteEngine = VotingEngine`

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
无。提案数据、元数据、活跃提案列表均已移至 `voting-engine` 统一管控（lib.rs:103 注释说明）。

### 机构账户地址
- 通过 `institution_pallet_address` 从 `CHINA_CB` / `CHINA_CH` 常量中查找机构的 `main_address`。
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
5. 将 `DestroyAction` 加 `MODULE_TAG` 编码。
6. 通过 `create_internal_proposal_with_data` 创建内部提案，并在同一事务中写入 owner/data/meta（活跃提案限额由投票引擎统一检查）。
7. 发 `DestroyProposed` 事件。

### 4.2 投票入口
Phase 2 整改后，本模块不再暴露独立 `vote_destroy` call。管理员投票统一走：

- `VotingEngine::internal_vote(proposal_id, approve)`

投票通过后由 `InternalVoteExecutor` 回调本模块自动执行销毁；自动执行失败时保持投票引擎状态为 `STATUS_PASSED`，并发出 `DestroyExecutionFailed` 事件，不回滚已通过投票。

### 4.3 `execute_destroy`（call index = 1）
语义：
- 兼容入口：签名账户必须是提案快照管理员，权限和重试次数由 `voting-engine` 统一校验。
- 仅当提案已 `STATUS_PASSED` 且存在 retry state 时可重试执行。

用途：
- 解决"提案已通过但自动执行失败（如余额不足）"后的后续重试。

---

## 5. 执行逻辑（`try_execute_destroy_from_action`）
1. 校验投票引擎提案状态为 `STATUS_PASSED`。
2. 从常量表查找机构账户地址并 decode 为 `AccountId`。
3. 校验 `free_balance >= amount + minimum_balance`（ED 保护）。
4. 调用 `Currency::slash` 执行销毁。
5. 校验 `remaining.is_zero()` 确认全额销毁成功。
6. 发 `DestroyExecuted` 事件。
7. 返回 `ProposalExecutionOutcome::Executed`，由投票引擎统一标记 `STATUS_EXECUTED`。

重复执行防护：
- `STATUS_EXECUTED` 后提案不再是 `STATUS_PASSED`，后续重试被投票引擎 `ProposalNotRetryable` 拒绝。

---

## 6. 关键安全设计
1. 权限边界：
   - 发起/投票需内部管理员身份。
   - 手动执行必须由提案快照管理员触发，统一走投票引擎 retry 权限校验。

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
- `execute_destroy()`

注意：
- 当前 `weights.rs` 仍是在旧代码上生成，包含已删除存储项（ProposalActions、ActiveProposalByInstitution、ProposalCreatedAt、ProposalPassedAt）的 proof 注释。权重数值为过估（安全），但不精确。
- 2026-04-05 复查时已确认 `resolution-destro` 自身 benchmark 夹具可编译，且不再把 `proposal_id` 写死为 `0`。
- 若本地直接拿标准 CI WASM 构建的节点跑 benchmark，会因为 runtime blob 不带 benchmarking runtime api 而失败；要重生成正式 `weights.rs`，需要使用带 benchmark api 的 runtime blob（例如专门的 benchmark 构建，而不是默认 CI 运行时产物）。

---

## 9. 测试覆盖
运行命令：
```
cargo test --offline --manifest-path citizenchain/runtime/governance/resolution-destro/Cargo.toml -- --nocapture
```

当前结果：14 passed

覆盖重点：
- NRC/PRC/PRB 三种组织达阈值自动执行销毁
- 非管理员不能发起/投票
- 零金额拒绝 + 余额不足拒绝
- ED 保留校验（销毁全部余额被拒）
- 自动执行失败后手动执行成功
- 被拒绝提案不阻塞新提案
- 已执行提案不阻塞新提案
- 重复投票由投票引擎拒绝
- 非管理员不能触发 execute_destroy
- 无效机构返回 None
- mock runtime 已跟进投票引擎新契约：`MaxAdminsPerInstitution` 与管理员快照 `get_admin_list`

---

## 10. 运维建议
1. 监控 `DestroyExecutionFailed` 事件，出现后优先补齐机构余额，再由快照管理员调用 `execute_destroy` 或投票引擎 `retry_passed_proposal`。
2. 若 3 次手动执行仍失败，或超过 `ExecutionRetryGraceBlocks` 无人处理，提案会由投票引擎统一转 `STATUS_EXECUTION_FAILED`。
3. `weights.rs` 须在代码稳定后重新运行 benchmark 以获取精确权重。
