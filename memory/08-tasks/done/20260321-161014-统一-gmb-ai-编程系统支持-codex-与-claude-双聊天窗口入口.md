任务需求：
统一 GMB AI 编程系统的聊天入口定义，在保留现有 Codex 使用方式的前提下，新增 Claude 聊天窗口作为等价入口，并把规则统一收口到 `memory/`。

所属模块：
- memory/07-ai
- 仓库根协议

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/ai-system-overview.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/thread-model.md
- memory/07-ai/context-loading-order.md
- AGENTS.md

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

输出物：
- 双入口系统规则文档
- 聊天协议更新
- 多线程模型更新
- 根协议更新
- 任务卡回写

验收标准：
- Codex 与 Claude 都被定义为 GMB AI 编程系统入口
- `memory/07-ai/` 文档口径一致
- 仓库根 `AGENTS.md` 口径一致
- 不再把 Claude 仅定义为后台 Review Agent
- 相关任务卡已创建并回写
