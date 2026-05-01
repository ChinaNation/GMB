# 任务卡：审查 resolution issuance 双 finalize

- 任务编号：20260501-081728
- 状态：done
- 所属模块：citizenchain/issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 08:17:28

## 任务需求

只读审查 resolution-issuance finalize_joint_vote extrinsic 与 JointVoteResultCallback 回调是否存在生产侧误配触发的双 finalize 风险，并输出证据、影响和修复建议。

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

- 开工前先确认任务属于 `runtime`、`node`（含桌面端）或 `primitives`
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
- 只读检查结论：问题“部分存在”。
- 生产 runtime 当前通过 `EnsureJointVoteFinalizeOrigin` 在非 `runtime-benchmarks` 下强制 `Err(o)`，外部 `finalize_joint_vote` extrinsic 当前不可用。
- `resolution-issuance` pallet 自身仍保留 `finalize_joint_vote` extrinsic，并与 `JointVoteResultCallback::on_joint_vote_finalized` 共用 `apply_joint_vote_result`，代码层缺少 voting-engine 状态校验；未来如果 runtime 误配为 `EnsureRoot`，Root 可绕过 voting-engine 直接执行发行。
- 二次 finalize 的具体后果需精确表述：直接 extrinsic 不会写 voting-engine 状态，因此不会由该入口直接把 `STATUS_EXECUTED` 改成 `STATUS_EXECUTION_FAILED`；但会在提案业务数据清理前重复进入业务 finalize 逻辑，可能产生 `AlreadyExecuted`、`VotingProposalCountUnderflow`、错误的 `IssuanceExecutionFailed` 事件或 `VotingProposalCount` 计数污染。
- 对照项：`runtime-upgrade` 的 `apply_joint_vote_result` 会校验 voting-engine proposal 状态，`resolution-issuance` 当前缺少同等代码兜底。

## 完成信息

- 完成时间：2026-05-01 08:19:00
- 完成摘要：完成 resolution-issuance 双 finalize 入口只读审查，确认生产配置当前阻断外部入口，但 pallet 代码层缺少 voting-engine 状态兜底；误配后可绕过投票引擎，具体后果不是直接由 extrinsic 降级 STATUS_EXECUTED，而是可能出现重复业务 finalize、计数污染和错误失败事件。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
