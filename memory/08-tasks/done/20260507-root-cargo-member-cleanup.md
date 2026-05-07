# 任务卡:清理根 Cargo 空成员

## 任务需求

按用户要求执行第 1 项推荐修复：删除根 `Cargo.toml` 中不存在的 `tools/scripts` workspace member。

## 预计修改目录

| 目录 | 用途、边界和修改类型 |
|---|---|
| `./` | 修改根 `Cargo.toml`，只清理不存在的 workspace member，不调整各子工程依赖。 |
| `memory/08-tasks/` | 新增并归档本任务卡，更新任务索引；只改任务记录。 |

## 执行清单

- [x] 删除根 workspace 中不存在的 `tools/scripts` 成员。
- [x] 验证根 Cargo 配置不再引用缺失目录。
- [x] 归档任务卡并暂存。

## 验收标准

- `rg 'tools/scripts' Cargo.toml` 无命中。
- `git diff --check` 通过。

## 执行结果

- 已删除根 `Cargo.toml` 中不存在的 `tools/scripts` workspace member。
- 根 workspace 当前为空 workspace；`cargo metadata --no-deps --format-version 1` 可正常读取。

## 验证记录

- `rg -n "tools/scripts" Cargo.toml`：无命中。
- `cargo metadata --no-deps --format-version 1`：通过。
- `git diff --check`：通过。
