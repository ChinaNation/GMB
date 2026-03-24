# 任务卡：把 Codex 主窗口自动调度 Blockchain Agent / SFID Agent / CPMS Agent / Mobile Agent 的机制写入 AI 编程系统，明确用户只需在 Codex 用中文提需求，系统按需自动分工并回写 memory

- 任务编号：20260320-225846
- 状态：done
- 所属模块：ai/system
- 当前负责人：Codex
- 创建时间：2026-03-20 22:58:46

## 任务需求

把 Codex 主窗口自动调度 Blockchain Agent / SFID Agent / CPMS Agent / Mobile Agent 的机制写入 AI 编程系统，明确用户只需在 Codex 用中文提需求，系统按需自动分工并回写 memory

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/AGENTS.md
- memory/CODEX.md
- memory/CLAUDE.md
- memory/07-ai/ai-system-overview.md
- memory/07-ai/document-boundaries.md
- memory/07-ai/startup-acceptance.md

## 模块模板

- 模板来源：memory/08-tasks/templates/ai-system.md

# AI 系统任务模板

## 目标

- 保持 Codex 新线程稳定接入 GMB AI 编程系统
- 保持 Claude 审查和 GitHub 门禁稳定可用
- 保持 `memory/` 作为唯一 AI 实体目录

## 额外注意

- 不要把 AI 规则散回多个目录
- 不要删除根目录入口软链接
- 不要让任务卡、文档回写和启动协议脱节

## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/ai-system.md

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

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/ai-system.md

# AI 系统完成标准

- 新线程启动协议仍然成立
- 真实开发任务仍然要求任务卡
- 文档边界没有重新漂移
- `memory/scripts/check-startup-acceptance.sh --ci` 通过
- `.github/scripts/check-ai-guardrails.sh` 语法通过
- 改动已同步更新 `memory/07-ai/`

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已识别当前缺口：Agent 名称和职责已经存在，但“Codex 必须自动调度专业工作线程”尚未固化为硬规则
- 已开始同步更新启动协议、Agent 规则、执行说明、聊天协议、线程模型和验收标准
- 已更新 `memory/AGENTS.md`、`memory/CODEX.md`、`memory/07-ai/agent-rules.md`、`memory/07-ai/agent-playbooks.md`、`memory/07-ai/chat-protocol.md`、`memory/07-ai/ai-system-overview.md`、`memory/07-ai/thread-model.md`、`memory/07-ai/startup-acceptance.md`
- 已执行 `bash memory/scripts/check-startup-acceptance.sh --ci`，结果通过
- 已执行 `.github/scripts/check-ai-guardrails.sh`，本次因脚本判定“未检测到变更文件”而跳过

## 完成信息

- 完成时间：2026-03-20 23:00:51
- 完成摘要：已将 Codex 主窗口自动调度专业 Agent 的机制写入 AI 系统主规则、聊天协议、执行说明和验收标准，并完成启动协议检查。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
