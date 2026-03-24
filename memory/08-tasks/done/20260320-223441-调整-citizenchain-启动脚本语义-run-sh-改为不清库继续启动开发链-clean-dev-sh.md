# 任务卡：调整 citizenchain 启动脚本语义：run.sh 改为不清库继续启动开发链，clean-dev.sh 保持清库后启动开发链，并同步更新文档说明

- 任务编号：20260320-223441
- 状态：done
- 所属模块：citizenchain/nodeui
- 当前负责人：Codex
- 创建时间：2026-03-20 22:34:41

## 任务需求

调整 citizenchain 启动脚本语义：run.sh 改为不清库继续启动开发链，clean-dev.sh 保持清库后启动开发链，并同步更新文档说明

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

- 模板来源：memory/08-tasks/templates/citizenchain-nodeui.md

### 默认改动范围

- `citizenchain/nodeui`
- `memory/05-modules/citizenchain/nodeui`

### 先沟通条件

- 修改节点 UI 与 node 的交互边界
- 修改桌面打包、sidecar 或安装包行为

## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`、`nodeui` 或 `primitives`
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
- 已确认当前问题根因：`run.sh` 未传 `dev-chain` feature，导致它启动默认链而不是继续使用开发链数据
- 已将 `run.sh` 调整为“不清库，继续启动开发链”
- 已补充 nodeui 模块文档，记录 `run.sh` 与 `clean-dev.sh` 的开发脚本语义差异
- 已执行 `bash -n citizenchain/scripts/run.sh citizenchain/scripts/clean-dev.sh`，脚本语法通过

## 完成信息

- 完成时间：2026-03-20 22:35:49
- 完成摘要：已将 run.sh 调整为不清库继续启动开发链，clean-dev.sh 保持清库后启动开发链，并同步更新 nodeui 模块文档说明。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
