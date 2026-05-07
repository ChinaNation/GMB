# 任务卡：建立 AI 系统统一必读文件

- 任务编号：20260507-ai-required-reading
- 状态：completed
- 所属模块：memory / AI 编程系统

## 任务需求

在 AI 系统中新增“统一必读文件”，以后每次设计、编程、改协议、改命名、改文档、改流程前，都从该文件确认必须读取和遵守的规则。

本任务是用户提出的三个统一文件中的第三个：

- `unified-protocols.md`：统一协议文件
- `unified-naming.md`：统一命名文件
- `unified-required-reading.md`：统一必读文件

## 预计修改目录

- `memory/07-ai/`
  - 中文注释：新增统一必读文件，并把必读入口接入 AI 规则、上下文装载顺序、文档边界和统一命名登记；只涉及 AI 系统文档。
- `memory/`
  - 中文注释：同步启动协议入口，让新线程首轮必读清单指向统一必读文件；只涉及 AI 启动协议文档。
- `memory/08-tasks/open/`
  - 中文注释：新增本任务卡并更新重新创世审计记录；只涉及任务记录。
- `/Users/rhett/GMB/`
  - 中文注释：同步根 `AGENTS.md` 启动协议别名，保持根入口和 `memory/AGENTS.md` 一致；只涉及文档。

## 执行清单

- [x] 新增 `memory/07-ai/unified-required-reading.md`
- [x] 写明首轮必读、执行前必读、按任务类型必读
- [x] 接入 `AGENTS.md` / `memory/AGENTS.md`
- [x] 接入 `agent-rules.md`
- [x] 接入 `context-loading-order.md`
- [x] 接入 `document-boundaries.md`
- [x] 登记到 `unified-naming.md`
- [x] 回写重新创世审计记录

## 完成记录

2026-05-07：

- 已创建 `memory/07-ai/unified-required-reading.md`
- 已把它纳入 `AGENTS.md`、`memory/AGENTS.md`、`agent-rules.md`、`context-loading-order.md`、`document-boundaries.md`
- 已登记到 `unified-naming.md`
