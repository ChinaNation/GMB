# 任务卡：分析 citizenchain/runtime/governance 下每个模块的功能边界，并重点明确 admins-change 与 voting-engine 的功能边界

- 任务编号：20260430-102248
- 状态：open
- 所属模块：citizenchain-runtime-governance
- 当前负责人：Codex
- 创建时间：2026-04-30 10:22:48

## 任务需求

分析 citizenchain/runtime/governance 下每个模块的功能边界，并重点明确 admins-change 与 voting-engine 的功能边界

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
- 2026-04-30 10:22：已读取 governance 模块文档、源码与 runtime 接线。
- 2026-04-30 10:22：本轮为只读边界分析，未修改业务代码，未执行测试。

## 分析结论摘要

- `voting-engine` 是治理投票基础设施，负责提案 ID、提案状态机、内部/联合/公民三类投票、管理员快照、计票、超时结算、统一提案数据/对象存储、活跃提案数量限制和 90 天延迟清理。
- `admins-change` 是管理员主体真源和管理员等长替换业务模块，负责 `Institutions` 管理员主体表、创世机构管理员初始化、动态多签主体生命周期、管理员读取 API，以及管理员替换提案通过后的落地执行。
- `admins-change` 与 `voting-engine` 的边界是：前者决定“谁是管理员、替换谁、替换后写回什么业务状态”，后者决定“提案如何创建编号、投票如何计票、何时通过/否决、何时回调、何时清理”。
- `grandpakey-change` 是 GRANDPA 公钥替换业务模块，走内部投票；只负责 key 校验、key 所属索引和通过后调用 `pallet-grandpa::schedule_change`。
- `resolution-destro` 是机构自有资金销毁业务模块，走内部投票；只负责销毁动作数据、余额/ED 校验和通过后 `Currency::slash`。
- `runtime-upgrade` 是 runtime wasm 升级业务模块，走联合投票；只负责升级摘要、wasm 对象、联合投票回调后的 `set_code` 执行和开发期直升入口。
- 当前实现中，`admins-change` 自动执行失败后保留 `STATUS_PASSED` 并发失败事件，等待公开重试；`resolution-destro` 自动执行失败会覆写为 `STATUS_EXECUTION_FAILED`。这是两个业务模块的恢复语义差异。
- 早期任务卡 `20260404-admin-replacement-proposal-mutex.md` 设计过“管理员更换提案与其他提案互斥”，当前源码实际落地的是 `voting-engine` 每机构最多 10 个活跃提案限制，并未实现管理员替换专项互斥。
