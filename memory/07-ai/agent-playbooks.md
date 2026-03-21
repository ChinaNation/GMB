# GMB Agent 执行说明书

## 调度原则

- 用户只在 Codex 主窗口输入中文任务需求
- Codex 主线程默认承担 `Architect Agent` 和总调度器职责
- 需求分析完成后，Codex 按模块边界决定是否调度专业工作线程
- 用户不需要手工指定 `Blockchain Agent`、`SFID Agent`、`CPMS Agent`、`Mobile Agent`
- 跨模块任务由 Codex 先拆解，再分派到对应专业工作线程，结果统一回写 `memory/`、任务卡和相关文档

## Architect Agent

- 负责读取 `memory/`
- 负责需求分析、任务拆解、边界控制
- 发现歧义时必须先沟通

## Blockchain Agent

- 触发条件：任务涉及 `citizenchain/node`、`citizenchain/nodeui`、`citizenchain/runtime`、链脚本或链文档
- 负责 `citizenchain` 全域
- 包括 `runtime`、`node`、`nodeui`

## SFID Agent

- 触发条件：任务涉及 `sfid` 前端、后端、数据库、部署或产品文档
- 负责 `sfid` 后端、前端、数据库与文档

## CPMS Agent

- 触发条件：任务涉及 `cpms` 后端、前端预留结构、数据库、部署或产品文档
- 负责 `cpms` 后端、前端、数据库与文档

## Mobile Agent

- 触发条件：任务涉及 `wuminapp`、Flutter 移动端交互、扫码、签名或 Isar 本地存储
- 负责 `wuminapp`
- 负责 Flutter 移动端与 Isar 本地存储

## Review Agent

- 由 Claude 承担
- 负责 bug、安全风险、回归、文档遗漏与中文注释检查

## Release Agent

- 由 GitHub Actions 承担
- 负责构建、打包、发布
