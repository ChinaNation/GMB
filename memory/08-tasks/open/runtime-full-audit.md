# 任务卡：全面检查 citizenchain/runtime，每一行代码、文档注释都检查，检查是否有漏洞、是否有应该改进的地方、是否有残留代码、残留注释和残留文档、是否有需要更新的文档和注释、是否有重复实现、是否有跨模块边界不清晰和越界，并输出完整检查报告。

- 任务编号：20260516-103758
- 状态：open
- 所属模块：citizenchain-runtime
- 当前负责人：Codex
- 创建时间：2026-05-16 10:37:58

## 任务需求

全面检查 citizenchain/runtime，每一行代码、文档注释都检查，检查是否有漏洞、是否有应该改进的地方、是否有残留代码、残留注释和残留文档、是否有需要更新的文档和注释、是否有重复实现、是否有跨模块边界不清晰和越界，并输出完整检查报告。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 本任务只输出检查报告，不修改 runtime 代码

## 输出物

- runtime 代码、文档、注释、残留、重复实现、模块边界检查报告

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已修正任务卡文件名与任务索引，确保任务卡登记路径为 `memory/08-tasks/open/runtime-full-audit.md`
