# 任务卡：修复 voting-engine M4 M5 与 admins-change MODULE_TAG 版本化问题

- 任务编号：20260430-213218
- 状态：done
- 所属模块：citizenchain-runtime-governance
- 当前负责人：Codex
- 创建时间：2026-04-30 21:32:18

## 任务需求

修复 voting-engine M4 M5 与 admins-change MODULE_TAG 版本化问题

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/voting-engine/VOTINGENGINE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/MODULE_TAG_REGISTRY.md

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
- 2026-04-30 21:32：确认无旧 `adm-rep` 提案需要兼容，按开发期直接升级 schema tag 处理。
- 2026-04-30 21:32：已将 `ProposalMutexBindings` 容量从硬编码 `ConstU32<128>` 改为 `MaxInternalProposalMutexBindings`，生产 runtime 与测试 runtime 当前配置为 256。
- 2026-04-30 21:32：已新增 `MissingThresholdSnapshot / MissingAdminSnapshot`，内部投票缺快照不再混用 `InvalidInternalOrg`。
- 2026-04-30 21:32：已修正内部投票超时注释，说明管理员名单与人数已快照，后续管理员更换不影响已有提案。
- 2026-04-30 21:32：已将 `admins-change::MODULE_TAG` 从 `adm-rep` 升级为 `adm-rep-v1`，并同步更新 tag 注册表与模块技术文档。
- 2026-04-30 21:32：验证通过 `cargo test -p voting-engine -p admins-change -p resolution-destro -p grandpakey-change -p runtime-upgrade -p resolution-issuance -p duoqian-manage -p duoqian-transfer --lib`。
- 2026-04-30 21:32：验证通过 `cargo test -p voting-engine -p admins-change --lib --features runtime-benchmarks`。
- 2026-04-30 21:32：尝试 `cargo test -p citizenchain --lib`，被 runtime/build.rs 门禁阻止：`WASM_FILE` 环境变量未设置，要求使用 CI 统一 WASM。

## 完成信息

- 完成时间：2026-04-30 21:49:35
- 完成摘要：修复 M4/M5/M6：ProposalMutexBindings 上限改为 MaxInternalProposalMutexBindings 并配置为 256；新增 MissingThresholdSnapshot/MissingAdminSnapshot，内部投票缺快照不再混用 InvalidInternalOrg；修正内部投票超时注释；admins-change MODULE_TAG 升级为 adm-rep-v1；同步 voting-engine、admins-change 技术文档与 MODULE_TAG 注册表；跨模块 lib 测试和 runtime-benchmarks 测试通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
