# 任务卡：清理 resolution issuance 无效 cleanup 和错误码

- 任务编号：20260501-085322
- 状态：done
- 所属模块：citizenchain/issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 08:53:22

## 任务需求

删除 resolution-issuance 中已 no-op 的 cleanup_joint_proposal 调用，修正 reason 反解失败错误码，补充回归测试，更新文档并清理残留。

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
- 已删除 `resolution-issuance/src/proposal.rs` 中三处已 no-op 的 `T::JointVoteEngine::cleanup_joint_proposal(proposal_id)` 调用。
- 已将 `data.reason.clone().try_into()` 失败路径从 `ProposalNotFound` 改为 `ReasonTooLong`，并补充中文注释说明这是 ProposalData 业务数据异常而非提案不存在。
- 已补充 `callback_rejects_corrupted_reason_with_reason_too_long` 回归测试，覆盖损坏 ProposalData 中 reason 超限时状态、执行记录和累计发行量不变。
- 已更新 `RESOLUTIONISSUANCE_TECHNICAL.md`，明确 proposal 核心数据、owner、业务 data 和投票凭证清理由 `voting-engine` 终态清理队列统一处理，本模块不再调用废弃 cleanup 接口。
- 已执行残留扫描：本模块代码中不再存在 `cleanup_joint_proposal`；`ProposalNotFound` 仅保留在 ProposalData 缺失路径。
- 验证通过：`cargo test -p resolution-issuance`、`WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain`、`cargo check -p resolution-issuance --features runtime-benchmarks`。

## 完成信息

- 完成时间：2026-05-01 08:55:18
- 完成摘要：删除 resolution-issuance 中已 no-op 的 cleanup_joint_proposal 调用，修正 reason 反解失败错误码为 ReasonTooLong，补充回归测试，更新技术文档并完成验证。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
