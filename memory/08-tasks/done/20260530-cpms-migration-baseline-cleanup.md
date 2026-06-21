# 任务卡：CPMS 开发期 migration 基线整理

## 任务需求

CPMS 仍处于开发期、没有正式发行版；允许清空开发库。整理 migration 规则，保留数据库一致性检查，清理开发期空占位和重复 migration，明确正式版发布后的升级规则。

## 建议模块

- CPMS 数据库 migration
- CPMS 启动脚本
- CPMS 技术文档

## 影响范围

- `citizenpassport/backend/db/migrations/`：当前完整结构收敛到开发期基线 `0001`，删除后续占位/重复 migration。
- `citizenpassport/citizenpassport.sh`：保留 migration 检查，启动失败时提示开发期执行 `--reset`。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md`：记录开发期与正式版 migration 规则。
- `memory/01-architecture/citizenpassport`：同步记录正式版升级边界。

## 主要风险点

- 已存在的开发库执行过旧 migration，整理后必须 `./citizenpassport.sh --reset` 重建。
- 正式版发布后不能再用这种整理方式，只能新增 migration。

## 是否需要先沟通

- 否。用户已确认开发期可清库，要求执行。

## 执行清单

- [x] 让 `0001_init_citizenpassport_pg.sql` 和当前 `schema.sql` 对齐。
- [x] 删除开发期后续空占位和重复 migration。
- [x] 保留启动 migration 检查并补充清晰提示。
- [x] 更新正式版升级规则文档。
- [x] 运行后端测试、前端构建和残留扫描。

## 完成记录

- 2026-05-30：创建任务卡，开始执行。
- 2026-05-30：完成 CPMS 开发期 migration 基线整理；`db/migrations/` 只保留当前完整 `0001`，删除开发期后续占位/重复 migration。
- 2026-05-30：启动脚本保留 migration 校验，后端提前退出时提示开发期使用 `./citizenpassport.sh --reset` 重建开发库。
- 2026-05-30：验证通过 `cargo test`、`cargo clippy --all-targets -- -D warnings`、`npm run build`、`bash -n citizenpassport/citizenpassport.sh`、临时空库执行 `0001`、`git diff --check` 和残留扫描。
- 2026-05-30：按开发期可清库规则重建本地 `cpms` 开发库，并用后端临时端口启动验证 `MIGRATOR.run()` 可正常写入 `_sqlx_migrations`。
