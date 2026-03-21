# GMB AI 文档边界说明

## 1. 目标

本文件用于收口 AI 文档分工，避免同一规则在多份文档里重复漂移。

## 2. 入口层

- `memory/AGENTS.md`
  - 只负责 Codex 新线程强制启动协议
- `memory/CODEX.md`
  - 只负责 Codex 作为主开发入口时的执行约束
- `memory/CLAUDE.md`
  - 只负责 Claude 在 PR 审查场景中的规则

## 3. 总览层

- `memory/07-ai/ai-system-overview.md`
  - 只讲这套系统是什么、有哪些组成、入口在哪里
  - 不重复展开日常操作细节

## 4. 使用层

- `memory/07-ai/operator-manual.md`
  - 只讲你平时怎么用
- `memory/07-ai/chat-protocol.md`
  - 只讲聊天首轮应该怎么响应
- `memory/07-ai/startup-acceptance.md`
  - 只讲新线程是否真的接入了系统

## 5. 规则层

- `memory/07-ai/agent-rules.md`
  - 只讲角色分工和开发硬规则
- `memory/07-ai/workflow.md`
  - 只讲标准开发流程与门禁
- `memory/07-ai/ci-path-routing.md`
  - 只讲 GitHub workflow 的目录路由

## 6. 模板层

- `memory/07-ai/task-card-template.md`
  - 只讲任务卡模板
- `memory/07-ai/clarification-template.md`
  - 只讲需求澄清模板
- `memory/07-ai/pre-submit-checklist.md`
  - 只讲提交前收口清单

## 7. 已做的去重决定

- 删除 `memory/07-ai/chat-first-mode.md`
  - 原因：它和 `chat-protocol.md`、`operator-manual.md` 重叠
- 新线程规则只保留在：
  - `memory/AGENTS.md`
  - `memory/CODEX.md`
  - `memory/07-ai/chat-protocol.md`
  - `memory/07-ai/startup-acceptance.md`
