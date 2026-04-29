# AI 系统完成标准

- 新线程启动协议仍然成立
- 真实开发任务仍然要求任务卡
- `检查为什么报错` 只读报错诊断例外仍然不创建任务卡、不修改代码
- 文档边界没有重新漂移
- `memory/scripts/check-startup-acceptance.sh --ci` 通过
- `.github/scripts/check-ai-guardrails.sh` 语法通过
- 改动已同步更新 `memory/07-ai/`
