# GMB 上下文装载顺序

AI 开工前按以下顺序装载上下文：

1. `memory/07-ai/unified-required-reading.md`
2. `memory/00-vision/project-goal.md`
3. `memory/00-vision/trust-boundary.md`
4. `memory/01-architecture/repo-map.md`
5. `memory/03-security/security-rules.md`
6. `memory/07-ai/agent-rules.md`
7. `memory/07-ai/workflow.md`
8. 涉及协议、载荷、接口契约、字段顺序、签名验签、nonce、era、pallet/call index、storage key、subject id 时，必须读取 `memory/07-ai/unified-protocols.md`
9. 涉及新建或重命名目录、文件、字段、变量、类、模块、API 字段、storage 字段、扫码端解码展示字段、任务卡文件名、文档文件名时，必须读取 `memory/07-ai/unified-naming.md`
10. 对应模块技术文档
11. 对应模块执行清单
12. 对应模块完成标准

补充规则：

- 无论聊天入口是 Codex 还是 Claude，都必须遵守同一装载顺序
- 不允许因为聊天入口不同而跳过 `memory/` 主文档
- `memory/07-ai/unified-required-reading.md` 是必读入口文件；新增或调整必读文档时必须先更新该文件
- `memory/07-ai/unified-protocols.md` 是协议入口文件；详细协议文档可以分散在模块文档中，但必须从该文件登记和跳转
- `memory/07-ai/unified-naming.md` 是命名入口文件；新命名必须先按该文件登记或确认
