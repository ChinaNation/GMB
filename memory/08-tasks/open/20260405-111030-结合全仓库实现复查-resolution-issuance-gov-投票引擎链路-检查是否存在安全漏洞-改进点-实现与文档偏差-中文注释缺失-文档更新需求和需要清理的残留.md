# 任务卡：结合全仓库实现复查 resolution-issuance-gov 投票引擎链路，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

- 任务编号：20260405-111030
- 状态：open
- 所属模块：citizenchain/runtime/governance/resolution-issuance-gov
- 当前负责人：Codex
- 创建时间：2026-04-05 11:10:30

## 任务需求

结合全仓库实现复查 resolution-issuance-gov 投票引擎链路，检查是否存在安全漏洞、改进点、实现与文档偏差、中文注释缺失、文档更新需求和需要清理的残留

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
- citizenchain/runtime/governance/resolution-issuance-gov/src/lib.rs
- citizenchain/runtime/governance/voting-engine-system/src/lib.rs
- citizenchain/runtime/issuance/resolution-issuance-iss/src/lib.rs
- memory/05-modules/citizenchain/runtime/governance/resolution-issuance-gov/RESOLUTIONISSUANCEGOV_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/voting-engine-system/VOTINGENGINE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/MODULE_TAG_REGISTRY.md

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
- 已复查 `resolution-issuance-gov`、`voting-engine-system`、`resolution-issuance-iss`、runtime 回调路由与 `wuminapp` 治理文档对齐情况
- 已确认当前未发现可由普通用户直接利用的高危权限绕过或重复发行漏洞
- 已确认一处真实一致性问题：联合投票回调可在事务内把提案状态覆盖为 `STATUS_EXECUTION_FAILED`，但投票引擎原先会先发出 `ProposalFinalized(STATUS_PASSED)`，导致事件语义与最终存储状态不一致
- 已修复投票引擎事件语义：`ProposalFinalized` 改为在联合回调完成后发出，事件状态与最终链上状态一致
- 已新增投票引擎单测，覆盖“回调覆盖状态后，`ProposalFinalized` 事件必须反映最终状态”
- 已更新 `resolution-issuance-gov` 模块注释与技术文档，修正发起权限口径、状态语义、乱码和旧路径残留
- 已更新 `voting-engine-system` 技术文档，补充联合回调后再发 `ProposalFinalized` 的真实语义
- 已更新 `resolution-issuance-iss` 技术文档，移除“执行失败时状态仍可能保持 Passed”的过期描述
- 已更新 `wuminapp` 治理技术文档中的 citizenchain 源码路径残留
- 交叉验证过程中发现 `admins-origin-gov` 的测试桩未适配投票引擎管理员快照模型，且 `MaxAdminsPerInstitution` 关联类型路径存在歧义；已一并修复测试桩与类型限定，避免全仓库检查时出现假失败

## 当前结论

- 已修复问题：
  - `ProposalFinalized` 事件状态与最终存储状态不一致
  - `resolution-issuance-gov` 技术文档权限口径错误、内容乱码、旧路径残留
  - `resolution-issuance-iss` 技术文档中过期的失败状态描述
  - `wuminapp` 治理技术文档中的旧目录路径残留
  - `admins-origin-gov` 测试桩未实现 `get_admin_list`，导致与投票引擎管理员快照模型不一致
- 仍待后续处理：
  - `resolution-issuance-gov/src/weights.rs` 仍是旧实现 benchmark 产物，注释里保留已删除存储；本轮未重跑 benchmark
  - runtime crate 级集成测试受 `WASM_FILE` 强制约束，当前环境未提供 CI 统一 WASM，无法在本轮直接完成该层验证

## 已执行验证

- `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/voting-engine-system/Cargo.toml`
- `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/resolution-issuance-gov/Cargo.toml`
- `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/governance/admins-origin-gov/Cargo.toml`

## 验证阻塞

- `cargo test --manifest-path /Users/rhett/GMB/citizenchain/runtime/Cargo.toml joint_vote_callback_`
  - 运行到 runtime build script 时被项目规则拦截：`WASM_FILE` 未设置，当前环境未提供 CI 统一 WASM，因此未继续绕过该限制
