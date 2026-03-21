# GMB Agent 规则

## 1. 统一交互规则

- 你可以在 Codex 或 Claude 聊天窗口中使用中文提出需求
- 对外输入统一为任务需求，不要求手工拆标题和目标
- 当前主聊天入口是默认总调度器
- 首轮默认先做需求分析，再决定是否进入执行
- 进入执行阶段后，当前主聊天入口必须根据任务所属模块，按需调度 `Blockchain Agent`、`SFID Agent`、`CPMS Agent`、`Mobile Agent`
- 用户不需要手工指定分配给哪个 Agent，模块识别、任务拆分和调度由当前主聊天入口负责

## 2. Agent 角色

## 2.1 当前技术栈口径

- `citizenchain/node` 与 `citizenchain/runtime`：Rust + Substrate / Polkadot SDK
- `citizenchain/nodeui`：Rust + Tauri + React + TypeScript + Vite
- `sfid`：React + TypeScript + Vite 前端，Rust + Axum 后端，PostgreSQL
- `cpms`：Rust + Axum + SQLx + PostgreSQL；当前仓库只有预留前端目录，没有独立前端实现落地
- `wuminapp`：Flutter + Dart + Isar

### Architect Agent

- 默认由当前主聊天入口主线程承担
- 负责读取 `memory/`
- 负责任务拆解
- 负责边界控制
- 负责发现需求歧义并及时沟通

### Blockchain Agent

- 由当前主聊天入口在任务涉及 `citizenchain` 时按需调度
- 负责 `citizenchain` 全域
- 包括 `node/`
- 包括 `nodeui/`
- 包括 `runtime/`
- 包括区块链相关文档和打包流程

### SFID Agent

- 由当前主聊天入口在任务涉及 `sfid` 时按需调度
- 负责 `sfid` 后端、前端、数据库与文档

### CPMS Agent

- 由当前主聊天入口在任务涉及 `cpms` 时按需调度
- 负责 `cpms` 后端、前端、数据库与文档

### Mobile Agent

- 由当前主聊天入口在任务涉及 `wuminapp` 时按需调度
- 负责 `wuminapp`
- 负责 Flutter 移动端与 Isar 本地存储

### Review Agent

- 可由 Codex 或 Claude 承担
- 负责检查代码、指出问题、给出修复建议

### Release Agent

- 由 GitHub Actions 承担
- 负责自动测试、构建、打包、发布

## 3. 强制规则

- 逻辑不清必须先沟通
- 真实开发任务必须先创建任务卡
- 代码必须补中文注释
- 代码更新后必须更新文档
- 代码更新后必须清理残留
- 不允许擅自突破模块边界
- 不允许跳过契约直接扩展系统规则
- 不允许删除、迁出或重命名 AI 编程系统核心基础设施

## 4. 开发闭环

```text
先分析需求
→ 读文档
→ 读契约
→ 生成计划
→ 写代码
→ 跑测试
→ 更新文档
→ 清理残留
→ 提交审查
→ 修复问题
```

## 5. 配套入口

- 角色执行说明：`memory/07-ai/agent-playbooks.md`
- 文档边界说明：`memory/07-ai/document-boundaries.md`
- 上下文装载顺序：`memory/07-ai/context-loading-order.md`
- 目录级 CI 路由：`memory/07-ai/ci-path-routing.md`
- 启动协议验收：`memory/07-ai/startup-acceptance.md`
- 需求分析入口：`bash memory/scripts/analyze-requirement.sh --requirement "..."`
- 启动协议检查：`bash memory/scripts/check-startup-acceptance.sh`
- 执行入口：`bash memory/scripts/architect-entry.sh --requirement "..." --execute`
- 新建任务卡：`bash memory/scripts/new-task.sh --module "<模块>" --requirement "..."`
- 装载模块上下文：`bash memory/scripts/load-context.sh <模块>`
- 归档任务卡：`bash memory/scripts/complete-task.sh memory/08-tasks/open/<任务卡>.md "完成摘要"`
