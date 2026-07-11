# 2026-06-23 根 tools 清理与新增文件确认硬规则

## 任务需求

- 检查仓库根目录 `tools/` 下文件是否有用。
- 有用文件移动到根目录 `scripts/`，无用文件删除。
- 在 GMB AI 编程系统死规则中新增仓库硬性规则：仓库中增加文件夹或文件必须经过用户确认。

## 改动范围

- `tools/`：清理根目录残留工具文件。
- `scripts/`：承接仍有用途的本机脚本文件。
- `memory/07-ai/`、`memory/AGENTS.md`：同步 AI 编程系统硬规则。
- `memory/04-decisions/`、`memory/08-tasks/`：更新当前仍引用根 `tools/` 的文档路径。

## 验收

- 根目录 `tools/` 不再保留文件。
- 有用途脚本位于根目录 `scripts/`。
- AI 编程系统规则明确要求新增文件或目录必须先经用户确认。

## 完成记录

- 2026-06-23：确认 `tools/sync-derive-vectors.sh` 是账户派生金标本机同步守卫，仍有用途。
- 2026-06-23：已移动为 `scripts/sync-derive-vectors.sh`，根 `tools/` 目录已删除。
- 2026-06-23：已在 `memory/AGENTS.md`、`memory/07-ai/agent-rules.md`、`memory/07-ai/unified-required-reading.md` 增加新增文件/目录必须经用户确认的硬规则。
- 2026-06-23：已将 ADR-024 和当前任务卡中的脚本路径从 `tools/` 同步为 `scripts/`。
- 2026-06-23：已执行 `bash -n scripts/sync-derive-vectors.sh`、根 `tools/` 删除检查、根 `scripts/` 忽略检查、`git diff --check`。
