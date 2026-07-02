# 任务卡：建立 AI 系统统一命名文件

- 任务编号：20260507-ai-unified-naming
- 状态：completed
- 所属模块：memory / AI 编程系统

## 任务需求

在 AI 系统中新增“统一命名文件”，以后所有新建目录、文件、字段、变量、类、模块、接口字段等命名，都必须先遵守该文件。

用户要求：

- 命名文件必须包含目录结构、中英名称、简介
- 所有命名尽量精简
- 不确定的命名必须先报告确认

## 预计修改目录

- `memory/07-ai/`
  - 新增统一命名文件，并把命名规则接入 AI 开发硬规则和上下文装载顺序；只涉及 AI 系统文档。
- `memory/08-tasks/open/`
  - 新增本任务卡并更新重新创世审计记录；只涉及任务记录。

## 执行清单

- [x] 新增 `memory/07-ai/unified-naming.md`
- [x] 写明目录结构、中英名称、简介
- [x] 写明命名尽量精简
- [x] 写明不确定命名必须先报告确认
- [x] 接入 `agent-rules.md`
- [x] 接入 `context-loading-order.md`
- [x] 接入 `document-boundaries.md`
- [x] 回写重新创世审计记录

## 完成记录

2026-05-07：

- 已创建 `memory/07-ai/unified-naming.md`
- 已把它纳入 `agent-rules.md`、`context-loading-order.md`、`document-boundaries.md`
- 已登记仓库核心目录结构、中英名称和简介
- 已写入“命名尽量精简”和“不确定命名先报告确认”硬规则
