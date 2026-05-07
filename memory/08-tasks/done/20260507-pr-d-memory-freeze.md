# 任务卡:PR-D memory 创世冻结

## 任务需求

执行重新创世前总审计 PR-D：冻结 `memory/` 目录结构、任务卡状态和文档入口，清理非标准目录、旧任务目录、已完成任务卡和路径引用残留。

## 背景

- `memory/` 是 GMB AI 编程系统唯一主目录。
- `memory/01-architecture/repo-map.md` 已定义当前主目录结构。
- `memory/05-architecture/` 与当前编号体系冲突，QR 文档已迁出，仍剩一个非 QR 文档。
- `memory/tasks/` 与 `memory/08-tasks/` 任务卡制度冲突。
- `memory/08-tasks/open/` 当前 open 任务数量过多，其中部分已完成、被替代或应迁入 done。

## 预计修改目录

| 目录 | 用途、边界和修改类型 |
|---|---|
| `memory/01-architecture/` | 接收架构级文档并更新仓库目录图；涉及文档移动和架构索引。 |
| `memory/05-architecture/` | 清理非标准残留目录；涉及残留清理。 |
| `memory/05-modules/` | 如文档属于模块实现细节，则迁入对应模块文档目录；涉及文档归位。 |
| `memory/tasks/` | 清理旧任务目录；涉及任务记录归并或删除。 |
| `memory/08-tasks/open/` | 保留真实未完成任务，移出已完成/废弃任务；涉及任务状态整理。 |
| `memory/08-tasks/done/` | 接收已完成或已被替代任务卡；涉及任务归档。 |
| `memory/07-ai/` | 必要时同步统一命名、必读、规则入口；涉及 AI 系统规则文档。 |

## 执行清单

- [x] 归位 `memory/05-architecture/` 剩余文档。
- [x] 清理空的 `memory/05-architecture/`。
- [x] 归并或清理 `memory/tasks/` 旧任务目录。
- [x] 将本轮已完成任务卡从 `open/` 移到 `done/`。
- [x] 对明显已完成/被替代的历史任务卡做第一批归档。
- [x] 更新 `repo-map.md`、统一命名和审计记录。
- [x] 扫描旧路径、旧任务目录和非标准目录残留。

## 验收标准

- `memory/05-architecture/` 不再存在。
- `memory/tasks/` 不再存在 tracked 文件。
- `repo-map.md` 与 `memory/` 实际主目录一致。
- `memory/08-tasks/open/` 只保留仍需要后续处理的任务。
- 全仓库不再引用 `memory/05-architecture/` 作为当前路径。

## 执行结果

2026-05-07：

- 已将 `memory/05-architecture/20260409-sfid-50k-concurrent-framework.md` 迁入 `memory/01-architecture/sfid/SFID_50K_CONCURRENT_FRAMEWORK.md`。
- 已将 `memory/tasks/` 下 3 个 smoldot 文档迁入 `memory/05-modules/wuminapp/rpc/`。
- 已删除空的 `memory/05-architecture/` 和 `memory/tasks/`。
- 已把 17 张明确完成或已被替代的任务卡从 `open/` 归档到 `done/`；本任务卡完成后 `open/` 为 83 张、`done/` 为 235 张。
- 已更新 `memory/01-architecture/repo-map.md`、`memory/07-ai/unified-naming.md`、`memory/07-ai/unified-protocols.md` 和重新创世审计记录。
- 已补齐 `CLAUDE.md` / `memory/CLAUDE.md` 的只读报错诊断例外，保持 Claude/Codex 入口规则读感一致。

## 验证记录

- `find memory/05-architecture memory/tasks -maxdepth 0`：确认两个旧目录不存在。
- `rg 'memory/05-architecture|memory/tasks' memory/01-architecture memory/05-modules memory/07-ai`：仅命中 repo-map 的“不得新建或恢复”清单，无当前路径引用。
- `bash memory/scripts/check-startup-acceptance.sh`：通过。
- `git diff --check` / `git diff --cached --check`：通过。
