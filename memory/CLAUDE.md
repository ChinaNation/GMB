# GMB Claude 入口说明

如果当前工作区是 `GMB`：

- 把当前聊天窗口视为 GMB AI 编程系统入口之一
- 把当前聊天窗口在承接任务时视为主入口和任务调度器
- 把用户输入视为任务需求
- 第一轮必须先做需求分析
- 不要求用户手工拆分标题和目标
- 多个线程属于同一套系统，不是多个独立系统
- 第一轮不得直接开始写代码或宣称已开始实现
- 进入真实开发前必须创建任务卡
- 进入执行阶段后按需自动分工给 `Blockchain Agent / SFID Agent / CPMS Agent / Mobile Agent`

优先遵守：

1. `memory/AGENTS.md`
2. `memory/07-ai/chat-protocol.md`
3. `memory/07-ai/requirement-analysis-template.md`
4. `memory/07-ai/agent-rules.md`
5. `memory/07-ai/dual-chat-entry.md`

如果用户直接提出开发需求：

- 第一轮必须输出需求分析
- 分析完成后先等待确认或在边界清晰时创建任务卡，再继续执行
- 用户不需要手工指定要分配给哪个 Agent，由当前主聊天入口根据模块边界自动判断
- 过程中必须回写 `memory/`、任务卡、ADR 或相关文档

如果当前任务是 Review：

- 先给出问题，再给出总结
- 优先识别 bug、安全风险、行为回归和缺失测试
- 说明风险等级和影响范围
- 修复建议尽量具体到模块或文件
- 如缺少上下文，要明确写出“基于当前 diff 的判断”
