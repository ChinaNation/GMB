# GMB AI 编程系统总览

## 1. 目标

GMB 的 AI 编程系统优先解决以下问题：

- 你只使用一个中文聊天窗口沟通
- Codex 负责主开发
- Claude 负责代码检查与修复建议
- 项目知识长期保存在仓库中
- 代码提交后可以走 GitHub 自动化测试、审查与发布

## 2. 当前形态

当前阶段不先自研新的聊天窗口，而是采用：

- `Codex`：唯一主聊天窗口
- `Claude`：后台 Review Agent
- `GitHub Actions`：自动测试、自动构建、自动发布
- `memory/`：AI 永久记忆系统

## 3. 系统结构

```text
你（中文自然语言）
        ↓
Codex（唯一主开发入口）
        ↓
memory/（项目目标、边界、ADR、规则、任务模板）
        ↓
各模块代码与文档
        ↓
GitHub PR / Actions
        ↓
Claude Review
        ↓
Codex 修复并回写文档
```

## 4. 核心规则

- AI 不记聊天，要记项目结构
- 所有关键决策必须回写 `memory/`
- 代码更新后必须更新文档
- 代码更新后必须清理残留
- 逻辑不清时必须先沟通

## 5. 第一阶段建设内容

第一阶段的 AI 编程系统只做最关键的底座：

1. 建立 `memory/` 目录体系
2. 固化 Agent 规则
3. 固化任务模板
4. 固化代码更新后的文档回写规则
5. 固化 GitHub 自动测试与 Review 入口

当前已落地的基础文件：

- `CLAUDE.md`
- `.github/pull_request_template.md`
- `.github/workflows/ai-guardrails.yml`
- `.github/workflows/claude-pr-review.yml`
- `.github/workflows/claude-on-comment.yml`
- `.github/scripts/check-ai-guardrails.sh`
- `memory/07-ai/github-activation.md`
- `memory/07-ai/daily-operations.md`

## 6. 第二阶段建设内容

第二阶段再逐步增强：

1. 把 Claude 的 PR 审查流程接入固定工作流
2. 增加 AI 任务记录、变更记录和回归记录
3. 增加模块级 Definition of Done
4. 如果确实需要，再建设自有 Flutter Desktop AI 控制台
