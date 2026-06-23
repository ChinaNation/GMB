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
- `memory/07-ai/unified-required-reading.md`
  - 只讲每次设计、编程、改协议、改命名、改文档、改流程前必须读取和遵守哪些文件
  - 不替代具体规则文件；只做必读入口和读取分流
- `memory/07-ai/unified-protocols.md`
  - 只讲协议、载荷、接口契约、字段顺序、签名验签、nonce、era、pallet/call index、storage key、subject id 的统一登记和变更规则
  - 不展开模块内部实现细节；实现细节继续放在 `memory/05-modules/` 或对应架构文档
- `memory/07-ai/unified-naming.md`
  - 只讲目录、文件、字段、变量、类、模块、API 字段、storage 字段、扫码端解码展示字段、任务卡文件名、文档文件名的统一命名规则和登记
  - 不替代模块技术文档；模块内特殊命名必须回链到本文件

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
