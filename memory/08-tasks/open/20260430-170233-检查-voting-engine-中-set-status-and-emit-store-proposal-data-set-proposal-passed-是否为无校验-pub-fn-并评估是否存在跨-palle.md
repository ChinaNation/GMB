# 任务卡：检查 voting-engine 中 set_status_and_emit、store_proposal_data、set_proposal_passed 是否为无校验 pub fn，并评估是否存在跨 pallet 强推提案状态或覆写 proposal data 的安全风险

- 任务编号：20260430-170233
- 状态：open
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
