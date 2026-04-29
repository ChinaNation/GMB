# 任务卡：删除已无业务代码的 citizenchain/node 目录，并将桌面节点功能统一收口到 citizenchain/node，同步更新脚本与 memory 文档引用。

- 任务编号：20260320-194945
- 状态：done
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-03-20 19:49:45

## 任务需求

删除已无业务代码的 citizenchain/node 目录，并将桌面节点功能统一收口到 citizenchain/node，同步更新脚本与 memory 文档引用。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-node.md

### 默认改动范围

- `citizenchain/node`
- `memory/05-modules/citizenchain/node`

### 先沟通条件

- 修改节点 UI 与 node 的交互边界
- 修改迁移策略或安装包行为


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`、`node` 或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
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

## 完成信息

- 完成时间：2026-03-20 19:53:37
- 完成摘要：已删除空的 citizenchain/node 目录，并完成 node 当前实现口径、任务模板、上下文脚本与模块路由的同步收口。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
