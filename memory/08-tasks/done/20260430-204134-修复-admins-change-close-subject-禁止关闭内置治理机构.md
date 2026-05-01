# 任务卡：修复 admins-change close_subject 禁止关闭内置治理机构

- 任务编号：20260430-204134
- 状态：done
- 所属模块：citizenchain-runtime-governance
- 当前负责人：Codex
- 创建时间：2026-04-30 20:41:34

## 任务需求

修复 admins-change close_subject 禁止关闭内置治理机构

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md

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
- 2026-04-30 20:41：确认 `close_subject` 是管理员主体关闭的底层状态转换函数，必须自行保护 NRC/PRC/PRB 等 `BuiltinInstitution`。
- 2026-04-30 20:41：已新增 `BuiltinSubjectCannotClose` 错误，并在 `close_subject` 写入 `Closed` 前拒绝关闭 `BuiltinInstitution`。
- 2026-04-30 20:41：已补充单测覆盖：内置治理主体不可关闭、动态主体激活后仍可关闭。
- 2026-04-30 20:41：已更新 `ADMINSCHANGE_TECHNICAL.md`，记录内置治理机构永不可关闭规则。
- 2026-04-30 20:41：验证通过 `cargo test -p admins-change --lib`，结果 26 passed。
- 2026-04-30 20:41：验证通过 `cargo test -p admins-change --lib --features runtime-benchmarks`，结果 26 passed。

## 完成信息

- 完成时间：2026-04-30 20:54:59
- 完成摘要：修复 admins-change::close_subject 缺少内置治理主体保护：新增 BuiltinSubjectCannotClose，禁止 BuiltinInstitution 进入 Closed；补充 NRC/PRC/PRB 不可关闭和动态主体可关闭测试；更新 admins-change 技术文档；admins-change 单测与 runtime-benchmarks 单测均 26 passed。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
