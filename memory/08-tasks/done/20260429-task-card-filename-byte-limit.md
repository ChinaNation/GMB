# 任务卡：AI 编程系统增加任务卡文件名 160 字节上限

- 任务编号：20260429-task-card-filename-byte-limit
- 状态：done
- 所属模块：ai/system
- 当前负责人：Codex
- 创建时间：2026-04-29

## 任务需求

在 AI 编程系统中增加规则：任务卡文件名不能超过 160 个字节。

## 必读上下文

- AGENTS.md
- memory/07-ai/agent-rules.md
- memory/07-ai/workflow.md
- memory/07-ai/task-card-template.md
- memory/scripts/new-task.sh
- memory/scripts/complete-task.sh

## 实施记录

- 已在新线程启动协议、Agent 强制规则、开发流程和任务卡模板中增加任务卡文件名 160 字节上限。
- 已在 `new-task.sh` 中增加自动截断短 slug 与最终文件名字节检查。
- 已在 `complete-task.sh` 中增加归档前文件名字节检查，避免超长任务卡继续流入 done。
- 已将历史超限任务卡文件名改为短文件名，文件内容标题不变。

## 验证记录

- `bash memory/scripts/index-tasks.sh`：通过，任务索引已刷新。
- `bash -n memory/scripts/new-task.sh`：通过。
- `bash -n memory/scripts/complete-task.sh`：通过。
- 超限扫描：`memory/08-tasks` 下未发现文件名超过 160 字节的任务卡。
- 临时集成验证：使用超长中文需求创建任务卡时，`new-task.sh` 自动生成 160 字节以内的文件名。
- `git diff --check`：通过。

## 完成信息

- 完成时间：2026-04-29
- 完成摘要：AI 编程系统已增加任务卡文件名 160 字节上限，并清理存量超限任务卡文件名。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
