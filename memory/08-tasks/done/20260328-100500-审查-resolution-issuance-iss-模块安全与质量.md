# 任务卡：全面仔细的检查一遍 resolution-issuance-iss 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-100500
- 状态：done
- 所属模块：citizenchain/issuance
- 当前负责人：Codex
- 创建时间：2026-03-28 10:05:00

## 任务需求

全面仔细的检查一遍 resolution-issuance-iss 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
- citizenchain/runtime/README.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- 必要时联动 `primitives`

### 先沟通条件

- 修改 runtime 存储结构
- 修改资格模型
- 修改提案、投票、发行核心规则


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`、`nodeui` 或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 冷钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 冷钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
- 文档与残留必须一起收口

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenchain.md

# CitizenChain 完成标准

- 改动范围和所属模块清晰
- 关键逻辑已补中文注释
- 文档已同步更新
- 影响链规则、存储或发布行为的点都已先沟通
- 残留已清理


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
- 已读取模块代码、benchmark、weights、runtime 配置、治理联动模块、投票引擎回调路径与模块技术文档。
- 已执行验证：
  - `cargo test -p resolution-issuance-iss`
  - `cargo test -p resolution-issuance-gov`
  - `cargo check -p citizenchain`

## 审查结论

### 总体判断

- `resolution-issuance-iss` 执行层本体未发现新的高风险越权执行、重复发行或原子性失效漏洞。
- 执行层的防重放、暂停、ED 校验、求和校验、单次上限、累计量写前校验和 `with_storage_layer` 回滚机制基本成立。
- 主要问题集中在“治理联动后的状态语义”和“联动文档 / weights 残留是否仍可信”两类。

### 主要发现

1. 联合投票通过但发行执行失败时，投票引擎里的提案状态仍会停留在 `PASSED`，而不是链上显式进入独立失败状态。`voting-engine-system` 在回调前先把状态写成 `STATUS_PASSED`，`resolution-issuance-gov` 在执行失败时又返回 `Ok(ApprovedExecutionFailed)`，只额外发 `IssuanceExecutionFailed` 事件，不会再把提案改写成新的失败状态。结果是：只看提案状态或 `ProposalFinalized` 事件会把“投票通过但发行失败”误判为成功，审计和运维必须强依赖额外事件补语义。
2. `resolution-issuance-gov` 的技术文档已经明显过期，仍描述旧版 `ExecutionFailed + retry_failed_execution + RetryCount + 双向映射` 设计，但当前代码已删除这些状态和存储，并改为统一 ID + 无重试逻辑。这会直接误导后续维护者对 `resolution-issuance-iss` 联动语义的理解，也会让“功能需求是否严格实现”的判断出现假阳性。
3. `resolution-issuance-gov/src/weights.rs` 的存储注释同样停留在旧版实现，仍列出 `NextProposalId`、`GovToJointVote`、`JointVoteToGov`、`RetryCount`、本地 `Proposals` 等已删除存储。即使当前数值未必立刻错误，这份 benchmark 产物至少已经不能作为“现实现状已重新测量”的可信证据，属于需要清理的残留。

### 功能实现判断

- 执行层核心需求基本已实现：
  - 仅执行，不创建提案、不参与投票。
  - 防重放依赖 `EverExecuted`，`clear_executed` 不会打开重放窗口。
  - 执行原子化，失败时不会留下部分到账。
  - `Paused`、`Executed`、`TotalIssued`、审计事件都按设计工作。
- 需要注意的现实口径：
  - 文档所说“总发行量双重约束”在代码上存在，但 runtime 当前 `MaxTotalIssuance = u128::MAX`，因此累计总量上限更像类型级兜底，而不是实际业务硬上限。
  - 治理联动里“通过但执行失败”的链上状态语义弱于旧文档描述，当前主要靠事件区分。

### 注释、文档与残留判断

- `resolution-issuance-iss` 本体中文注释整体完整，关键路径可读性足够。
- `memory/05-modules/citizenchain/runtime/issuance/resolution-issuance-iss/RESOLUTIONISS_TECHNICAL.md` 与当前执行层代码基本一致，还主动写明了“执行失败时提案状态仍可能保持 Passed”。
- 但上游治理文档 `memory/05-modules/citizenchain/runtime/governance/resolution-issuance-gov/RESOLUTIONISSUANCEGOV_TECHNICAL.md` 已经严重过期。
- `resolution-issuance-gov/src/weights.rs` 也存在明显旧实现残留。

## 完成信息

- 完成时间：2026-03-28 10:08:36
- 完成摘要：完成 resolution-issuance-iss 审查；确认执行层核心防重放、原子性和权限边界基本成立，但发现 3 个需要后续处理的问题：治理回调失败时提案状态仍表现为 PASSED、resolution-issuance-gov 技术文档仍保留旧版 ExecutionFailed/重试设计、resolution-issuance-gov weights 注释仍引用已删除存储。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
