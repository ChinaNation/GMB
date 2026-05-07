# 任务卡:清空 open 任务卡

## 任务需求

用户要求“全部删除，清空任务卡”。

结合上一轮上下文，本次执行边界确定为清空 `memory/08-tasks/open/` 下的 open 任务卡，保留：

- `memory/08-tasks/open/README.md`：空目录说明，不作为任务卡。
- `memory/08-tasks/done/`：历史归档，不清除。
- `memory/08-tasks/templates/`：任务卡模板，不清除。

## 预计修改目录

| 目录 | 用途、边界和修改类型 |
|---|---|
| `memory/08-tasks/open/` | 删除全部 open 任务卡，仅保留 README；涉及任务卡清理。 |
| `memory/08-tasks/done/` | 新增本执行记录，保留清空操作的可追溯记录；涉及任务归档文档。 |
| `memory/08-tasks/index.md` | 更新任务索引，将 open 区域置空；涉及任务索引文档。 |

## 执行结果

2026-05-07：

- 已删除 `memory/08-tasks/open/` 下 82 张 open 任务卡。
- `memory/08-tasks/open/README.md` 已更新为空目录说明。
- `memory/08-tasks/index.md` 的 open 区域已更新为“当前无 open 任务卡”。

## 验证记录

- `find memory/08-tasks/open -maxdepth 1 -type f ! -name 'README.md'`：无任务卡文件。
- `git diff --check` / `git diff --cached --check`：通过。
