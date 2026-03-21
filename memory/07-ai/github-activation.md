# GMB AI 系统 GitHub 激活指南

## 1. 目标

本指南用于把仓库中的 AI 自动化配置真正启用起来。

启用后，仓库将具备以下能力：

- PR 自动执行 AI 门禁检查
- Claude 自动审查 PR
- 在 PR 评论中通过 `@claude` 触发辅助分析

## 2. 需要手动完成的 GitHub 配置

以下步骤需要仓库管理员在 GitHub 中手动完成：

### 2.1 安装 Claude GitHub App

- 安装官方 Claude GitHub App 到当前仓库
- 确保它拥有以下仓库权限：
  - Contents: Read & Write
  - Issues: Read & Write
  - Pull requests: Read & Write

## 2.2 配置 GitHub Secret

在仓库的 `Settings -> Secrets and variables -> Actions` 中添加：

- `ANTHROPIC_API_KEY`

说明：

- 这是 `claude-pr-review.yml` 和 `claude-on-comment.yml` 的必需项
- 未配置时，Claude 相关 workflow 会自动跳过

## 3. 仓库中已经准备好的文件

- `CLAUDE.md`
- `.github/pull_request_template.md`
- `.github/scripts/check-ai-guardrails.sh`
- `.github/workflows/ai-guardrails.yml`
- `.github/workflows/claude-pr-review.yml`
- `.github/workflows/claude-on-comment.yml`

## 4. 激活后的验证方式

### 4.1 验证 AI 门禁

新开一个 PR，修改任意代码文件但不更新文档。

预期结果：

- `AI Guardrails` workflow 失败
- 报告“改代码后未更新文档”或“检测到残留”

### 4.2 验证 Claude 自动审查

在已经配置 `ANTHROPIC_API_KEY` 的前提下，新开一个 PR。

预期结果：

- `Claude PR Review` workflow 运行
- Claude 在 PR 中给出中文审查意见

### 4.3 验证 Claude 评论响应

在 PR 评论中输入：

```text
@claude 请检查这个 PR 有没有边界越权、安全风险和文档遗漏。
```

预期结果：

- `Claude On Comment` workflow 运行
- Claude 回复中文分析结果

## 5. 当前限制

- Codex 仍然是唯一主开发入口
- Claude 主要运行在 GitHub PR 与评论场景
- 本阶段还没有建设自有 Flutter AI 控制台

## 6. 常见故障排查

### 6.1 Claude 仍提示额度不足

- Claude GitHub Actions 使用的是 `ANTHROPIC_API_KEY` 对应的 API workspace 额度
- 如果已经充值但仍报 `Credit balance is too low`，优先检查：
  - 当前 secret 里的 key 是否来自刚刚充值的同一个 workspace
  - Claude Console 左上角切换到正确 workspace 后，Billing 是否确实显示有余额
  - 旧 key 是否仍然指向未充值的默认 workspace

建议做法：

- 在正确 workspace 中重新生成一个新的 API key
- 用新 key 覆盖 GitHub 仓库里的 `ANTHROPIC_API_KEY`
- 然后重新运行 `Claude PR Review`

如果不再报额度不足，但日志里出现 `error_max_turns`：

- 说明 Claude 已经真正跑起来了
- 失败原因变成审查轮数限制过小
- 当前仓库默认把 `Claude PR Review` 的 `--max-turns` 提高到 10

## 7. 启用完成判定

只有同时满足以下条件，才算 GitHub AI 编程系统真正启用：

- Claude GitHub App 已安装
- `ANTHROPIC_API_KEY` 已配置
- 新 PR 能触发 `AI Guardrails`
- 新 PR 能触发 `Claude PR Review`
- PR 评论中的 `@claude` 可以正常响应
