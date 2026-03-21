# GMB 多线程工作模型

## 1. 结论

可以在 Codex 左侧开多个线程。

但要明确：

- 它们是同一套 AI 编程系统的多个工作线程
- 不是多个彼此独立、各记各的系统

## 2. 推荐实践

- 主线程：由 Codex 承担，负责需求分析、架构决策、边界确认、模块识别和任务调度
- 工作线程：按需承担 `Blockchain Agent / SFID Agent / CPMS Agent / Mobile Agent` 的专业执行任务
- 结果必须回写到 `memory/`、任务卡、ADR 或变更记录
