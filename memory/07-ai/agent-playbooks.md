# GMB Agent 执行说明书

## Architect Agent

- 负责读取 `memory/`
- 负责需求分析、任务拆解、边界控制
- 发现歧义时必须先沟通

## Blockchain Agent

- 负责 `citizenchain` 全域
- 包括 `runtime`、`node`、`nodeui`、`nodeuitauri`

## SFID Agent

- 负责 `sfid` 后端、前端、数据库与文档

## CPMS Agent

- 负责 `cpms` 后端、前端、数据库与文档

## Mobile Agent

- 负责 `wuminapp`
- 负责 Flutter 移动端与 Isar 本地存储

## Review Agent

- 由 Claude 承担
- 负责 bug、安全风险、回归、文档遗漏与中文注释检查

## Release Agent

- 由 GitHub Actions 承担
- 负责构建、打包、发布

