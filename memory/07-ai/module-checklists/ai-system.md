# AI 系统执行清单

## 开工前

- 确认变更确实属于 AI 编程系统，而不是业务功能
- 确认 `memory/` 仍然是唯一实体目录
- 确认根目录入口只作为 `memory/` 的别名

## 实施中

- 启动协议改动时同步检查 `memory/AGENTS.md`、`memory/CODEX.md`
- Claude 审查改动时同步检查 `memory/CLAUDE.md`
- GitHub 门禁改动时同步检查 `.github/workflows/ai-guardrails.yml`
- 真实开发任务必须创建任务卡

## 收口时

- 更新 `memory/07-ai/` 对应规则
- 更新必要的任务卡
- 运行 `bash memory/scripts/check-startup-acceptance.sh --ci`
