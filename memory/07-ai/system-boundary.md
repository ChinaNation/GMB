# GMB AI 编程系统边界

`memory/` 就是 GMB 的 AI 编程系统。

以下内容统一视为 AI 编程系统核心基础设施：

- `memory/AGENTS.md`
- `memory/CODEX.md`
- `memory/CLAUDE.md`
- `memory/05-modules/`
- `memory/07-ai/`
- `memory/08-tasks/`
- `memory/scripts/`

仓库根目录仅保留入口别名：

- `AGENTS.md`
- `CODEX.md`
- `CLAUDE.md`
这些入口别名不承载主规则，只负责把新线程和 Claude 审查引导到 `memory/`。它们本身不是第二份内容。

## 受保护规则

以下对象禁止在 PR 中删除、迁出或重命名：

- `memory/00-vision/`
- `memory/01-architecture/`
- `memory/03-security/`
- `memory/04-decisions/`
- `memory/05-modules/`
- `memory/06-quality/`
- `memory/07-ai/`
- `memory/scripts/`
- `memory/AGENTS.md`
- `memory/CODEX.md`
- `memory/CLAUDE.md`
- `memory/README.md`
- `AGENTS.md`
- `CODEX.md`
- `CLAUDE.md`

相关门禁由 `.github/scripts/check-ai-guardrails.sh` 负责执行。
