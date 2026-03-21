# GMB AI 编程系统总览

## 1. 目标

GMB 的 AI 编程系统优先解决以下问题：

- 你可以在 Codex 或 Claude 聊天窗口里用中文沟通
- 你只需要输入任务需求，不需要手工拆标题和目标
- Codex 与 Claude 都可以承接主开发
- Codex 与 Claude 都可以承担代码检查与修复
- 项目知识长期保存在仓库中
- 代码提交后可以走 GitHub 自动化测试、审查与发布

## 2. 当前形态

当前阶段不先自研新的聊天窗口，而是采用：

- `Codex`：聊天入口之一
- `Claude`：聊天入口之一
- `GitHub Actions`：自动测试、自动构建、自动发布
- `memory/`：AI 永久记忆系统

交互方式固定为：

- 你在 Codex 或 Claude 聊天窗口直接输入中文任务需求
- AI 先做需求分析
- 当前主聊天入口根据模块边界按需分配给 `Blockchain Agent / SFID Agent / CPMS Agent / Mobile Agent`
- 分析完成后再进入任务创建和开发

## 3. 系统结构

```text
你（中文自然语言）
        ↓
Codex / Claude（聊天入口）
        ↓
当前主聊天入口（需求分析 + 总调度）
        ↓
Architect / Blockchain / SFID / CPMS / Mobile 工作线程
        ↓
memory/（项目目标、边界、ADR、规则、任务模板）
        ↓
各模块代码与文档
        ↓
GitHub PR / Actions
        ↓
任一聊天入口继续修复并回写文档
```

## 4. 核心规则

- AI 不记聊天，要记项目结构
- 所有关键决策必须回写 `memory/`
- 代码更新后必须更新文档
- 代码更新后必须清理残留
- 逻辑不清时必须先沟通
- 对外输入统一是任务需求，不强制手工拆标题和目标
- 用户不需要手工指定 Agent，当前主聊天入口必须按模块边界自动判断和调度专业工作线程

## 4.1 当前技术语言基线

- `citizenchain/node` 与 `citizenchain/runtime`：Rust + Substrate / Polkadot SDK
- `citizenchain/nodeui`：Rust + Tauri + React + TypeScript + Vite
- `sfid`：React + TypeScript + Vite 前端，Rust + Axum 后端，PostgreSQL
- `cpms`：Rust + Axum + SQLx + PostgreSQL；`frontend/` 当前仅为预留目录
- `wuminapp`：Flutter + Dart + Isar

## 5. 第一阶段建设内容

第一阶段的 AI 编程系统只做最关键的底座：

1. 建立 `memory/` 目录体系
2. 固化 Agent 规则
3. 固化任务模板
4. 固化模块执行清单
5. 固化模块级完成标准
6. 固化需求澄清与提交前收口清单
7. 固化代码更新后的文档回写规则
8. 固化 GitHub 自动测试与 Review 入口

当前已落地的基础文件：

- `memory/AGENTS.md`
- `memory/CODEX.md`
- `memory/CLAUDE.md`
- `.github/pull_request_template.md`
- `.github/workflows/ai-guardrails.yml`
- `.github/workflows/claude-pr-review.yml`
- `.github/workflows/claude-on-comment.yml`
- `.github/scripts/check-ai-guardrails.sh`
- `memory/07-ai/github-activation.md`
- `memory/07-ai/daily-operations.md`
- `memory/07-ai/context-loading-order.md`
- `memory/07-ai/agent-playbooks.md`
- `memory/07-ai/architect-workflow.md`
- `memory/07-ai/document-boundaries.md`
- `memory/07-ai/module-task-routing.md`
- `memory/07-ai/operator-manual.md`
- `memory/07-ai/module-checklists/`
- `memory/07-ai/module-definition-of-done/`
- `memory/07-ai/chat-protocol.md`
- `memory/07-ai/startup-acceptance.md`
- `memory/07-ai/requirement-analysis-template.md`
- `memory/07-ai/thread-model.md`
- `memory/07-ai/clarification-template.md`
- `memory/07-ai/pre-submit-checklist.md`
- `memory/08-tasks/`
- `memory/08-tasks/templates/`
- `memory/04-decisions/ADR-TEMPLATE.md`
- `memory/06-quality/bug-template.md`
- `memory/06-quality/change-log-template.md`
- `memory/scripts/module-router.sh`
- `memory/scripts/analyze-requirement.sh`
- `memory/scripts/check-startup-acceptance.sh`
- `memory/scripts/architect-entry.sh`
- `memory/scripts/new-task.sh`
- `memory/scripts/start-task.sh`
- `memory/scripts/load-context.sh`
- `memory/scripts/complete-task.sh`
- `memory/scripts/index-tasks.sh`

仓库根目录保留的 `AGENTS.md`、`CODEX.md`、`CLAUDE.md` 只是指向 `memory/` 的入口别名，不是第二份内容。

AI 文档边界统一收口在：

- `memory/07-ai/document-boundaries.md`

## 6. 第二阶段建设内容

第二阶段再逐步增强：

1. 增加 AI 任务记录、变更记录和回归记录的自动索引
2. 继续把模块级完成标准接入更多自动化入口
3. 继续细化各系统独立 workflow 的构建深度与发布门禁
4. 如果确实需要，再建设自有 Flutter Desktop AI 控制台
