# GMB 上下文装载顺序

AI 开工前按以下顺序装载上下文：

1. `memory/00-vision/project-goal.md`
2. `memory/00-vision/trust-boundary.md`
3. `memory/01-architecture/repo-map.md`
4. `memory/03-security/security-rules.md`
5. `memory/07-ai/agent-rules.md`
6. `memory/07-ai/workflow.md`
7. 对应模块技术文档
8. 对应模块执行清单
9. 对应模块完成标准

补充规则：

- 无论聊天入口是 Codex 还是 Claude，都必须遵守同一装载顺序
- 不允许因为聊天入口不同而跳过 `memory/` 主文档
