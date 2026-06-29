# 迁移 AI 系统 citizencode 文档到 OnChina

## 任务需求

把 AI 编程系统中 `citizencode` 的有价值内容迁移到 `onchina`，删除旧 `citizencode` 残留。仓库今后只保留公民、公民链、公民钱包和官方网站四个产品，OnChina 作为公民链内置能力维护。

## 执行范围

- 新建 OnChina 架构文档：`memory/01-architecture/onchina/`
- 新建 OnChina 模块文档：`memory/05-modules/citizenchain/onchina/`
- 更新 AI 检查清单和完成定义：`memory/07-ai/module-checklists/onchina.md`、`memory/07-ai/module-definition-of-done/onchina.md`
- 删除旧 `memory/01-architecture/citizencode/`、`memory/05-modules/citizencode/` 和旧 AI 清单。
- 清理当前规则、架构、代码注释和包名中的 `citizencode` 残留。

## 验收要求

- 当前规则文档不得再把 `citizencode` 或 `citizenpassport` 作为产品。
- 当前实现路径统一为 `citizenchain/onchina/`。
- 历史任务卡和 ADR 可保留历史上下文，但当前规则、架构、模块文档和代码注释不得继续使用旧产品口径。
- 本次提交不得包含其它线程未完成的代码改动。
