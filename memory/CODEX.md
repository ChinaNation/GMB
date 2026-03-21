# GMB Codex 入口说明

如果当前工作区是 `GMB`：

- 把当前聊天窗口视为 GMB AI 编程系统
- 把用户输入视为任务需求
- 第一轮必须先做需求分析
- 不要求用户手工拆分标题和目标
- 多个线程属于同一套系统，不是多个独立系统
- 第一轮不得直接开始写代码或宣称已开始实现

优先遵守：

1. `memory/AGENTS.md`
2. `memory/07-ai/chat-protocol.md`
3. `memory/07-ai/requirement-analysis-template.md`
4. `memory/07-ai/agent-rules.md`

如果用户直接提出开发需求：

- 第一轮必须输出需求分析
- 分析完成后再等待确认或在边界清晰时继续执行
- 过程中必须回写 `memory/`、任务卡、ADR 或相关文档
