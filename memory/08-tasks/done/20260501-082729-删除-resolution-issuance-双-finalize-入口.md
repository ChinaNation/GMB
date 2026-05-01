# 任务卡：删除 resolution issuance 双 finalize 入口

- 任务编号：20260501-082729
- 状态：done
- 所属模块：citizenchain/issuance
- 当前负责人：Codex
- 创建时间：2026-05-01 08:27:29

## 任务需求

彻底删除 resolution-issuance 手工 finalize_joint_vote extrinsic，只保留 voting-engine callback finalize 入口；补充内部状态校验，更新测试、文档、中文注释并清理相关残留。

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
- 已删除 `resolution-issuance` 的手工 `finalize_joint_vote` extrinsic，保留 call index `1` 空缺。
- 已删除 `JointVoteFinalizeOrigin` Config、runtime `EnsureJointVoteFinalizeOrigin`、手续费分类中的 `ResolutionIssuance::finalize_joint_vote` 分支、对应 benchmark 和权重项。
- 已在 `apply_joint_vote_result` 增加内部兜底：必须处于 voting-engine `CallbackExecutionScopes`，提案必须是联合阶段，`approved=true` 只接受 `STATUS_PASSED`，`approved=false` 只接受 `STATUS_REJECTED`。
- 已补充测试：非终结状态拒绝、非 voting-engine 回调作用域拒绝、已执行后二次回调拒绝、暂停执行失败路径继续经 callback 收口。
- 已更新 `RESOLUTIONISSUANCE_TECHNICAL.md`，说明手工 finalize 已删除、call index `1` 空缺、唯一入口为 voting-engine callback。
- 已执行残留扫描，`resolution-issuance` 与 runtime 配置中不再存在旧 `finalize_joint_vote` 入口残留；仅 `runtime-upgrade` 保留自身同名入口，文档保留删除说明。
- 验证通过：`cargo test -p resolution-issuance`、`WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain`、`cargo check -p resolution-issuance --features runtime-benchmarks`、`WASM_BUILD_FROM_SOURCE=1 cargo test -p citizenchain`。
- 备注：`WASM_BUILD_FROM_SOURCE=1 cargo check -p citizenchain --features runtime-benchmarks` 仍受当前工程既有 `byte-slice-cast` 在 `wasm32v1-none` 下拉入 `std` 的问题阻塞，未作为本任务通过项；本模块 benchmark feature 已单独验证通过。

## 完成信息

- 完成时间：2026-05-01 08:34:06
- 完成摘要：删除 resolution-issuance 手工 finalize_joint_vote extrinsic，仅保留 voting-engine callback finalize；补充回调作用域和状态兜底，清理 runtime 配置、权重、benchmark 残留，更新测试与技术文档并完成验证。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
