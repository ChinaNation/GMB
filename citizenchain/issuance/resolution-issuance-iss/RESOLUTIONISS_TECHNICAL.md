# RESOLUTION Issuance Execution Technical Notes

## 1. 模块定位
`resolution-issuance-iss` 是“决议发行执行层”pallet，负责把已通过治理流程的发行决议落地到链上账本。

核心职责：
- 执行发行，不负责提案创建、投票或表决流程。
- 对发行参数做一致性与安全校验（总额、条目数、理由长度、ED、防重放等）。
- 记录可审计状态（累计发行量、执行标记、事件哈希）。

代码位置：
- `/Users/rhett/GMB/citizenchain/issuance/resolution-issuance-iss/src/lib.rs`

---

## 2. 上下游关系
上游（治理）：
- `/Users/rhett/GMB/citizenchain/governance/resolution-issuance-gov/src/lib.rs`
- 通过 `ResolutionIssuanceExecutor<AccountId, Amount>` trait 调用本模块执行发行。

Runtime 接线：
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
- `resolution_issuance_iss::Config` 当前配置：
  - `ExecuteOrigin = EnsureNoPrivilegeOrigin`（拒绝所有外部 origin）
  - `MaintenanceOrigin = Root`
  - `MaxReasonLen = RESOLUTION_ISSUANCE_MAX_REASON_LEN`
  - `MaxAllocations = RESOLUTION_ISSUANCE_MAX_ALLOCATIONS`
  - `MaxTotalIssuance = u128::MAX`

常量来源（默认值）：
- `/Users/rhett/GMB/primitives/src/count_const.rs`
- `RESOLUTION_ISSUANCE_MAX_REASON_LEN = 1024`
- `RESOLUTION_ISSUANCE_MAX_ALLOCATIONS = PRB_COUNT`（随省储行数量变化）

关键语义：
- 在当前 runtime 中，`execute_resolution_issuance` extrinsic 被协议层封死，实际执行入口是治理模块经 trait 调用。

---

## 3. 配置与数据类型
`Config` 关键项：
- `Currency`：发币与余额记账实现。
- `ExecuteOrigin`：执行入口权限。
- `MaintenanceOrigin`：维护入口权限（清理执行标记、暂停开关）。
- `MaxReasonLen`：理由最大字节长度。
- `MaxAllocations`：单次最大分配条数。
- `MaxTotalIssuance`：本模块累计发行上限。
- `WeightInfo`：weight 估算实现。

核心类型：
- `BalanceOf<T>`：从 `Currency` 推导出的链上余额类型。
- `ReasonOf<T> = BoundedVec<u8, MaxReasonLen>`。
- `AllocationOf<T> = BoundedVec<RecipientAmount<AccountId, BalanceOf<T>>, MaxAllocations>`。
- `RecipientAmount { recipient, amount }`：单条分配记录。

---

## 4. 存储模型
- `Executed: Map<proposal_id -> block_number>`  
  用于近期执行状态与审计展示，可被维护入口清理。

- `EverExecuted: Map<proposal_id -> ()>`  
  永久防重放标记，不可通过 `clear_executed` 清除。

- `TotalIssued: Balance`  
  本模块累计已执行发行总量。

- `Paused: bool`  
  紧急暂停开关；为 `true` 时拒绝一切发行执行。

版本：
- `#[pallet::storage_version(StorageVersion::new(1))]`

---

## 5. 外部接口（Calls + Trait）
### 5.1 `execute_resolution_issuance`（call index = 0）
- 权限：`ExecuteOrigin`
- 入参：`proposal_id`, `reason`, `total_amount`, `allocations`
- 行为：进入统一执行函数 `do_execute`
- weight：`T::WeightInfo::execute_resolution_issuance(reason_len, allocation_count)`

### 5.2 `clear_executed`（call index = 1）
- 权限：`MaintenanceOrigin`
- 行为：
1. 检查 `Executed` 中存在该 `proposal_id`
2. 删除 `Executed` 记录
3. 发送 `ExecutedCleared` 事件
- 注意：不会删除 `EverExecuted`，因此不会引入重放风险。

### 5.3 `set_paused`（call index = 2）
- 权限：`MaintenanceOrigin`
- 行为：
1. 读取当前 `Paused`
2. 与目标值一致则报错 `AlreadyInState`
3. 写入新值并发出 `PausedSet`

### 5.4 Trait 执行入口
- 接口：`ResolutionIssuanceExecutor::execute_resolution_issuance(...)`
- 作用：供治理模块或其他 runtime 内组件直接调用，不依赖 extrinsic origin。
- 在 `resolution-issuance-gov` 中，联合投票通过后调用该接口触发执行。

---

## 6. 执行流程与一致性约束
核心执行在 `do_execute_inner`，并由 `with_storage_layer` 包裹，确保失败回滚：

1. 全局开关与重放检查
- `Paused` 必须为 `false`
- `EverExecuted` 不得已存在 `proposal_id`

2. 输入合法性检查
- `reason` 非空且长度 `<= MaxReasonLen`
- `allocations` 非空且长度 `<= MaxAllocations`

3. 分配逐条校验与求和
- 每条 `amount > 0`
- 每条 `amount >= ExistentialDeposit`
- 累加求和不得溢出（`AllocationOverflow`）
- 求和必须等于 `total_amount`（`TotalMismatch`）

4. 累计发行量检查
- `TotalIssued + total_amount` 不得溢出（`TotalIssuedOverflow`）
- 不得超过 `MaxTotalIssuance`（`ExceedsTotalIssuanceCap`）

5. 发币执行
- 按 allocation 调用 `deposit_creating`
- 校验 `imbalance.peek() == item.amount`，防止静默失败（`DepositFailed`）
- 使用合并 imbalance，最终一次性 drop，避免多次发行总量写入

6. 状态提交与事件
- 写入 `EverExecuted`
- 写入 `Executed(proposal_id -> current_block)`
- 更新 `TotalIssued`
- 发出 `ResolutionIssuanceExecuted`（含 `reason_hash` 与 `allocations_hash`）

---

## 7. 事件与错误
事件：
- `ResolutionIssuanceExecuted { proposal_id, total_amount, recipient_count, reason_hash, allocations_hash }`
- `ExecutedCleared { proposal_id }`
- `PausedSet { paused }`

错误：
- 执行状态类：`AlreadyExecuted`, `PalletPaused`, `AlreadyInState`, `NotExecuted`
- 参数类：`EmptyReason`, `ReasonTooLong`, `EmptyAllocations`, `TooManyAllocations`, `ZeroAmount`
- 数值类：`AllocationOverflow`, `TotalMismatch`, `TotalIssuedOverflow`, `ExceedsTotalIssuanceCap`, `BelowExistentialDeposit`, `DepositFailed`

---

## 8. Weight 策略
当前 `WeightInfo for ()` 为手工估算：
- `execute_resolution_issuance`：基础权重 + 按 `allocation_count`、`reason_len` 线性项 + DB 读写项。
- `clear_executed`：`from_parts(10_000_000, 128) + reads_writes(1,2)`。
- `set_paused`：`from_parts(5_000_000, 64) + reads_writes(1,2)`。

说明：
- 现阶段可用，但生产链建议补齐 benchmark 生成权重，以获得更准确的执行时间与 PoV 估算。

---

## 9. 安全设计要点
- 原子性：`with_storage_layer` 防止部分发币后失败造成账本不一致。
- 永久防重放：`EverExecuted` 与可清理 `Executed` 分离。
- 发行上限：`MaxTotalIssuance` 可配置。
- Dust 防护：要求每笔分配 `>= ED`。
- 暂停机制：`Paused` 可快速停止所有发行执行。
- 审计增强：事件含 `reason_hash`、`allocations_hash` 与执行块高记录。

---

## 10. 测试覆盖（当前）
`cargo test --manifest-path /Users/rhett/GMB/citizenchain/issuance/resolution-issuance-iss/Cargo.toml`

已覆盖重点：
- 正常执行与余额更新
- 防重放（含 `clear_executed` 后仍不可重放）
- 总额不一致、求和溢出、累计上限溢出
- 原因长度边界与空理由
- 空分配、超分配、零金额、低于 ED
- 暂停机制（trait 路径与 extrinsic 路径）
- `set_paused` 事件与幂等保护
- `clear_executed` 权限与事件
- `deposit_creating` 失败检测
- 事件字段完整性（含 hash）与 `Executed` 块高记录

---

## 11. 运维建议
- 常规审计优先看：
  - `ResolutionIssuanceExecuted`
  - `ExecutedCleared`
  - `PausedSet`
- 紧急处置流程建议：
1. 先 `set_paused(true)` 停止执行
2. 排查异常提案与参数来源
3. 修复后 `set_paused(false)` 恢复
- `clear_executed` 仅用于清理可见状态，不应用于“允许重放”。
- 治理侧语义：若治理模块调用执行失败，提案状态仍可能保持 `Passed`，需结合治理模块事件 `IssuanceExecutionFailed` 进行联动排查。
