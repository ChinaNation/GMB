# GMB Architect 工作流

## 1. 目标

Architect Agent 不直接替代业务 Agent 写所有代码，它的职责是把需求变成可执行任务，并确保整个过程不越过边界。

## 2. 标准工作顺序

```text
收到任务需求
→ 先做需求分析
→ 读取 memory/ 基础文档
→ 识别目标模块
→ 判断是否涉及多个模块
→ 输出分析结论
→ 分析确认后才生成任务卡
→ 指定上下文装载列表
→ 判断是否存在待确认问题
→ 无歧义时交给对应模块 Agent
→ 有歧义时先和你沟通
```

只读报错诊断例外：

- 用户输入包含 `检查为什么报错` 时，Architect 不创建任务卡
- 当前入口直接读取相关上下文、检查报错原因并输出检查结果
- 该模式不得修改代码；若随后进入修复，再回到标准任务卡流程

## 3. 推荐入口

- `bash memory/scripts/analyze-requirement.sh --requirement "..."`
- `bash memory/scripts/architect-entry.sh --requirement "..." --execute`
