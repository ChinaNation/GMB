# 任务卡：检查 voting-engine 中 set_status_and_emit、store_proposal_data、set_proposal_passed 是否为无校验 pub fn，并评估是否存在跨 pallet 强推提案状态或覆写 proposal data 的安全风险

- 任务编号：20260430-170233
- 状态：done
- 所属模块：citizenchain/runtime/governance
- 当前负责人：Codex
- 创建时间：2026-04-30 17:02:33

## 任务需求

检查 voting-engine 中 set_status_and_emit、store_proposal_data、set_proposal_passed 是否为无校验 pub fn，并评估是否存在跨 pallet 强推提案状态或覆写 proposal data 的安全风险

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- <补充该模块对应技术文档路径>

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已装载通用上下文；`load-context.sh "citizenchain/runtime/governance"` 提示当前模块无专门技术文档，需要以源码补充上下文。
- 已核查 `citizenchain/runtime/governance/voting-engine/src/lib.rs`：
  - `set_status_and_emit` 是 `pub fn`，只校验 `ensure_valid_status_transition`，不校验调用 pallet、提案 owner 或投票计票结果。
  - `store_proposal_data` 是 `pub fn`，只校验数据长度，直接按 `proposal_id` 覆写 `ProposalData`。
  - `set_proposal_passed` 是 `pub fn`，只在 `ProposalMeta` 存在时写入 `passed_at`，不校验回调作用域、状态或提案 owner。
- 已追踪调用面：`governance`、`transaction`、`issuance` 下多个业务 pallet 直接调用上述公开接口，说明它们确实是跨 crate 公开能力。
- 核查结论：问题存在，安全性依赖“runtime 内所有 caller pallet 均可信且不会误用”的假设；如新增或误用业务 pallet，可绕过正常投票路径或污染业务数据。

## 修复技术方案

- 将状态推进能力拆分为投票引擎内部函数与受限业务结果函数：
  - `set_status_and_emit` 收口为 `pub(crate)`，仅允许 `voting-engine` crate 内部投票、超时、阶段流转逻辑调用。
  - 业务模块不得再调用 `set_status_and_emit(STATUS_PASSED/STATUS_REJECTED)`。
  - 保留或新增专用业务执行结果接口，只允许 `STATUS_EXECUTED/STATUS_EXECUTION_FAILED`，并校验提案已是 `STATUS_PASSED` 且 caller 拥有该提案。
- 给提案业务数据增加 owner/module 绑定：
  - 新增 `ProposalOwner` / `ProposalModule` 存储，按 `proposal_id` 记录业务模块标识。
  - 创建提案时同步写入 owner、`ProposalData`、`ProposalMeta`，避免创建提案与写业务数据分成多个公开步骤。
  - `store_proposal_data` 改为 `pub(crate)` 或删除外部公开用法；业务模块改走带 owner 参数的 trait 方法。
  - 后续更新数据必须校验传入 owner 与存储 owner 一致，禁止跨模块覆写。
- 将 `set_proposal_passed` 收口为回调作用域内的 owner 校验接口：
  - 外部公开函数删除或改为 `pub(crate)`。
  - 新接口要求 `CallbackExecutionScopes` 存在、提案状态为 `STATUS_PASSED` 或回调内待执行状态、owner 匹配。
  - 只允许首次写入 `passed_at`，避免后续重写审计时间。
- 重构业务 pallet 调用点：
  - `admins-change`、`resolution-destro`、`grandpakey-change`、`runtime-upgrade`、`resolution-issuance`、`duoqian-transfer`、`duoqian-manage` 创建提案时改用新 trait 方法一次性注册 owner/data/meta。
  - 业务执行完成时改用受限业务结果接口，不再直接调用 `set_status_and_emit`。
  - 回调认领由“只看 `MODULE_TAG` 前缀”升级为“owner 存储 + `MODULE_TAG` 双校验”。
- 测试与文档：
  - 增加负向测试：非 owner 覆写数据失败、非回调作用域写 `passed_at` 失败、业务模块无法把 `VOTING` 提案直接置为 `PASSED`。
  - 更新治理模块文档，说明 voting-engine 是提案生命周期唯一状态机，业务 pallet 只持有受限 capability。
  - 清理旧公开接口、旧测试中允许覆盖 `ProposalData` 的断言和相关残留注释。

## PASSED 等待手动重试模块清单

- `governance/resolution-destro`：自动销毁失败只发 `DestroyExecutionFailed`，提案保留 `STATUS_PASSED`，公开 `execute_destroy` 供后续补余额后重试。
- `governance/grandpakey-change`：自动替换失败只发 `GrandpaKeyExecutionFailed`，提案保留 `STATUS_PASSED`，公开 `execute_replace_grandpa_key` 供管理员重试；确定不可执行时用 `cancel_failed_replace_grandpa_key` 转 `STATUS_EXECUTION_FAILED`。
- `transaction/duoqian-transfer`：transfer / safety fund / sweep 三类执行失败均发对应失败事件并保留 `STATUS_PASSED`，公开 `execute_transfer`、`execute_safety_fund_transfer`、`execute_sweep_to_main` 重试。
- `transaction/duoqian-manage`：callback 注释与代码表现为执行失败后发事件并保留 `STATUS_PASSED`，但当前未发现公开 `execute_create` / `execute_close` / `execute_create_institution` 重试 call；属于修复时必须明确取舍的残留边界。
- 明确不属于等待重试：`governance/admins-change` 自动执行失败会写 `STATUS_EXECUTION_FAILED`，`runtime-upgrade` 和 `resolution-issuance` 联合投票回调执行失败也写 `STATUS_EXECUTION_FAILED`。

## 统一状态语义决策

- `STATUS_REJECTED` 是投票否决终态，不允许执行或重试。
- `STATUS_PASSED` 表示提案已通过并获得业务执行授权，同时也是自动执行暂时失败后的统一可重试态。
- `STATUS_EXECUTED` 是业务执行成功终态。
- `STATUS_EXECUTION_FAILED` 只表示确定不可执行、人工取消或永久失败终态；进入该状态后不允许继续执行或取消。
- 因此“手动重试”不应发生在 `STATUS_EXECUTION_FAILED` 下，而应统一发生在 `STATUS_PASSED` 下。
- 统一取消流程应从 `STATUS_PASSED -> STATUS_EXECUTION_FAILED`，由 voting-engine 统一入口做 owner、状态、权限和业务模块可取消性校验。
- 统一手动执行流程应由 voting-engine 暴露单一入口，例如 `retry_passed_proposal(proposal_id)`：
  - 要求提案状态必须是 `STATUS_PASSED`。
  - voting-engine 根据 `ProposalOwner` 分发到对应业务 executor。
  - 执行成功则进入 `STATUS_EXECUTED` 终态。
  - 暂时失败则保持 `STATUS_PASSED`，继续允许后续重试。
  - 确定不可执行不应在 retry 中直接静默吞掉，应返回错误或提示走 `cancel_passed_proposal`。
- 如果无人手动执行，提案会一直保持 `STATUS_PASSED`，直到有人重试成功、有人取消为 `STATUS_EXECUTION_FAILED`，或后续另行设计“通过后执行宽限期/过期取消”规则。当前建议先不自动过期取消，避免把余额不足、GRANDPA pending 等可恢复问题误杀。

## 用户确认的新执行失败策略草案

- 自动执行失败后，提案保持 `STATUS_PASSED`，进入统一可重试状态。
- 管理员可通过 voting-engine 的统一入口手动执行，最多允许 3 次手动失败。
- 第 3 次手动执行仍失败后，voting-engine 自动将提案转为 `STATUS_EXECUTION_FAILED` 终态。
- 自动执行失败后若超过配置的执行宽限区块数仍无人手动执行，也由 voting-engine 自动转为 `STATUS_EXECUTION_FAILED` 终态。
- 需要新增重试元数据存储，例如 `ProposalExecutionRetryState`：
  - `manual_attempts`
  - `first_auto_failed_at`
  - `retry_deadline`
  - `last_attempt_at`
- 当前代码已有 `RETENTION_DAYS = 90` 的延迟清理逻辑，但它是“90 天后清理数据”，不是“90 个区块后执行/失败”。
- 当前 `set_status_and_emit` 对所有非 `STATUS_VOTING` 状态都会注册清理，包含 `STATUS_PASSED`；若 `STATUS_PASSED` 改为可重试态，则必须改为只对真正终态 `STATUS_REJECTED` / `STATUS_EXECUTED` / `STATUS_EXECUTION_FAILED` 注册 90 天清理。

## 完整修复技术方案：统一执行重试模型

### 目标状态机

```text
STATUS_VOTING
  -> STATUS_REJECTED                  // 投票否决，终态
  -> STATUS_PASSED                    // 投票通过，业务执行授权态

STATUS_PASSED
  -> STATUS_EXECUTED                  // 自动或手动执行成功，终态
  -> STATUS_PASSED                    // 自动执行失败或手动失败未满 3 次，继续可重试
  -> STATUS_EXECUTION_FAILED          // 手动失败满 3 次，或超过宽限期无人手动执行，终态
```

### voting-engine 新增存储

- `ProposalOwner<T>`：`proposal_id -> BoundedVec<u8, MaxModuleTagLen>`，记录业务模块 owner，禁止跨模块覆写和误路由。
- `ProposalExecutionRetryState<T>`：`proposal_id -> ExecutionRetryState<BlockNumberFor<T>>`，记录自动执行失败后的可重试状态。
- `ExecutionRetryDeadlines<T>`：`block_number -> BoundedVec<proposal_id>`，用于 `on_initialize` 到期后把无人处理的 `PASSED` 提案转 `STATUS_EXECUTION_FAILED`。

`ExecutionRetryState` 字段：

- `manual_attempts: u8`
- `first_auto_failed_at: BlockNumber`
- `retry_deadline: BlockNumber`
- `last_attempt_at: Option<BlockNumber>`

配置项：

- `MaxManualExecutionAttempts = 3`
- `ExecutionRetryGraceBlocks`
- `MaxExecutionRetryDeadlinesPerBlock`
- `MaxModuleTagLen`

### voting-engine 新增/调整接口

- `set_status_and_emit` 改为 `pub(crate)`，仅 voting-engine 内部投票状态机调用。
- 新增 `register_internal_proposal_with_data(...)` / `register_joint_proposal_with_data(...)`，创建提案时原子写入 `Proposals`、`ProposalOwner`、`ProposalData`、`ProposalMeta`，业务模块不再直接调用 `store_proposal_data`。
- `store_proposal_data`、`store_proposal_meta`、`set_proposal_passed` 收口为 `pub(crate)` 或删除公开外部用法。
- 新增公开 extrinsic：`retry_passed_proposal(origin, proposal_id)`。
- 新增公开 extrinsic：`cancel_passed_proposal(origin, proposal_id, reason)`，只允许把 `STATUS_PASSED` 转 `STATUS_EXECUTION_FAILED`。
- `on_initialize` 新增处理 `ExecutionRetryDeadlines[now]`，对仍处于 `STATUS_PASSED` 且未执行成功的提案自动转 `STATUS_EXECUTION_FAILED`。

### 业务 executor 统一返回值

替换当前 `DispatchResult` 语义，统一为：

```text
Ignored
Executed
RetryableFailed
FatalFailed
```

- `Ignored`：不是本模块提案。
- `Executed`：业务执行成功，voting-engine 转 `STATUS_EXECUTED`。
- `RetryableFailed`：暂时失败，voting-engine 保持 `STATUS_PASSED`，注册/更新 retry state。
- `FatalFailed`：确定不可执行，voting-engine 转 `STATUS_EXECUTION_FAILED`。

### 自动执行失败流程

- 投票通过后进入 `STATUS_PASSED`。
- voting-engine 立即调用 owner 对应 executor 自动执行一次。
- 自动执行成功：转 `STATUS_EXECUTED`，注册 90 天终态清理。
- 自动执行返回 `RetryableFailed`：保持 `STATUS_PASSED`，写入 `ProposalExecutionRetryState`，注册 `ExecutionRetryDeadlines`。
- 自动执行返回 `FatalFailed`：转 `STATUS_EXECUTION_FAILED`，注册 90 天终态清理。

### 手动执行流程

- `retry_passed_proposal` 校验：
  - 提案存在且 `status == STATUS_PASSED`
  - `ProposalOwner` 存在
  - caller 是该提案所属机构/业务允许的管理员
  - 未超过 `MaxManualExecutionAttempts`
  - 未超过 `retry_deadline`
- 分发 owner 对应 executor 执行。
- 成功：转 `STATUS_EXECUTED`，删除 retry state，注册 90 天终态清理。
- 失败且未满 3 次：`manual_attempts += 1`，保持 `STATUS_PASSED`。
- 失败达到第 3 次：转 `STATUS_EXECUTION_FAILED`，删除 retry state，注册 90 天终态清理。

### 手动取消流程

- `cancel_passed_proposal` 校验：
  - 提案存在且 `status == STATUS_PASSED`
  - caller 有权限
  - owner executor 返回“允许取消/确定不可执行”
- 成功后转 `STATUS_EXECUTION_FAILED`，删除 retry state，注册 90 天终态清理。

### 90 天清理修正

- 当前代码会在所有非 `STATUS_VOTING` 状态注册 90 天清理，包含 `STATUS_PASSED`。
- 新模型下必须改为只在真正终态注册清理：
  - `STATUS_REJECTED`
  - `STATUS_EXECUTED`
  - `STATUS_EXECUTION_FAILED`
- `STATUS_PASSED` 是执行授权/可重试态，不允许被 90 天清理删除业务数据。

### 业务模块迁移

- 硬约束：所有业务模块必须统一使用 voting-engine 的提案状态机，不允许业务模块各自维护一套可冲突的投票/执行状态语义。
- 业务模块只允许表达业务执行结果，不允许直接推进 `Proposal.status`，不允许直接写 `ProposalData`，不允许绕过 owner 分发。
- 所有业务模块的投票通过、自动执行、手动重试、取消失败、终态清理必须走同一套 voting-engine 状态机和统一入口。
- `resolution-destro`：删除独立 `execute_destroy` 公开重试入口，改由 voting-engine 统一 retry 分发；失败余额不足返回 `RetryableFailed`。
- `grandpakey-change`：删除或保留为兼容层的 `execute_replace_grandpa_key`，主入口改为 voting-engine retry；`GrandpaChangePending` 返回 `RetryableFailed`，格式错误/旧 key 不存在等返回 `FatalFailed` 或允许 cancel。
- `duoqian-transfer`：删除独立 `execute_transfer` / `execute_safety_fund_transfer` / `execute_sweep_to_main` 公开重试入口，统一 owner 分发；余额不足/权限暂时不可用返回 `RetryableFailed`。
- `duoqian-manage`：补齐当前缺失的统一重试路径；创建/关闭/机构创建失败统一进入 retry state 或按业务判定 `FatalFailed`。
- `admins-change`：按统一 executor 返回值改造；当前自动失败直接终态的语义可映射为 `FatalFailed`。
- `runtime-upgrade`、`resolution-issuance`：按统一 executor 返回值改造；执行失败可继续保持 `FatalFailed`，不强制进入可重试。

### 测试要求

- `VOTING -> PASSED` 只能由投票计票/超时逻辑触发，业务 pallet 无法直接调用。
- `ProposalData` 非 owner 覆写失败。
- 自动执行 `RetryableFailed` 后状态仍为 `STATUS_PASSED`，写入 retry state。
- 第 1、2 次手动失败仍为 `STATUS_PASSED`，第 3 次失败转 `STATUS_EXECUTION_FAILED`。
- 手动成功转 `STATUS_EXECUTED`。
- 超过 `ExecutionRetryGraceBlocks` 未手动执行，`on_initialize` 转 `STATUS_EXECUTION_FAILED`。
- `STATUS_PASSED` 不注册 90 天清理；`REJECTED` / `EXECUTED` / `EXECUTION_FAILED` 注册清理。
- 旧的 `PASSED` 90 天清理测试需要改为终态清理测试。

## 实施结果

- `voting-engine` 已收口 `set_status_and_emit` / `store_proposal_data` / `set_proposal_passed` 为 crate 内部或测试专用能力，生产业务 pallet 不能再直接强推状态或覆写 ProposalData。
- 新增 `ProposalOwner`、`ProposalExecutionRetryStates`、`ExecutionRetryDeadlines`，创建提案时通过 `*_with_data` trait 原子绑定 owner/data/meta。
- 新增统一执行结果 `ProposalExecutionOutcome::{Ignored, Executed, RetryableFailed, FatalFailed}`；所有业务回调改为返回该结果，由投票引擎统一推进状态。
- 新增统一重试/取消入口：`retry_passed_proposal`、`cancel_passed_proposal`，业务模块保留的 `execute_xxx` / `cancel_xxx` call 仅作为兼容入口委托投票引擎。
- 自动执行失败保持 `STATUS_PASSED` 并进入 retry state；第 3 次手动失败或超过 `ExecutionRetryGraceBlocks` 后自动转 `STATUS_EXECUTION_FAILED`。
- 90 天清理只登记 `STATUS_REJECTED / STATUS_EXECUTED / STATUS_EXECUTION_FAILED`，不再清理可重试的 `STATUS_PASSED`。
- 已统一迁移 `resolution-destro`、`grandpakey-change`、`admins-change`、`duoqian-manage`、`duoqian-transfer`、`runtime-upgrade`、`resolution-issuance`。
- 已清理 `duoqian-transfer` 旧离线 finalize 残留事件/错误和相关注释。
- 已更新 `voting-engine`、`MODULE_TAG_REGISTRY`、相关治理/交易/发行模块技术文档。

## 验证结果

- `cargo fmt`：通过。
- `cargo test -p voting-engine`：66 passed。
- `cargo test -p duoqian-transfer`：20 passed。
- `cargo test -p resolution-destro -p grandpakey-change -p admins-change -p duoqian-manage -p runtime-upgrade -p resolution-issuance`：全部通过。
- `cargo check -p citizenchain`：被 runtime/build.rs 的统一 WASM 保护阻断，错误为 `WASM_FILE 环境变量未设置`；这是仓库当前禁止本地 runtime WASM 编译的保护，不是本次代码编译错误。
