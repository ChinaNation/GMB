# GMB AI 开发流程

## 1. 标准流程

```text
你提出中文需求
→ Codex 读取 memory/
→ Codex 读取对应模块文档
→ Codex 判断边界与影响范围
→ 如逻辑不清先沟通
→ Codex 改代码并补中文注释
→ Codex 更新 memory/ 或技术文档
→ Codex 清理残留
→ GitHub PR 触发门禁
→ Claude 自动审查
→ Codex 修复问题
→ GitHub Actions 测试、构建、发布
```

## 2. 强制门禁

- 改代码后必须更新文档
- 改代码后必须清理残留
- 逻辑不清时必须先沟通
- 关键逻辑必须补中文注释

## 3. GitHub 自动化入口

- `.github/workflows/ai-guardrails.yml`
- `.github/workflows/claude-pr-review.yml`
- `.github/workflows/claude-on-comment.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/benchmark-weights.yml`

## 4. 路径分流执行原则

- `citizenchain/runtime` 与 `citizenchain/node` 分开执行
- 共享 Rust 目录变更时，允许多侧联动执行
- 纯文档、Pages、SFID 等目录按各自二级目录规则触发
- 目录路由细则统一记录在 `memory/07-ai/ci-path-routing.md`

## 5. 当前执行方式

- 你只使用 Codex 主窗口
- Claude 在 GitHub PR 与评论中承担审查与辅助角色
- 项目长期记忆统一保存在 `memory/`
