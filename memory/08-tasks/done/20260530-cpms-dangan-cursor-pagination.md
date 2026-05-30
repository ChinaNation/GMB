# 任务卡：CPMS 档案列表游标分页与模块边界整改

## 任务需求

将 CPMS 档案列表分页从操作管理员模块中抽离，归属到档案 `dangan` 领域；删除 `page/OFFSET/实时 COUNT` 小表分页思路，改为百万级可用的游标分页、统计表总数和索引化检索；同步调整前端档案列表目录、文档、注释和残留。

## 建议模块

- CPMS 后端 `dangan`
- CPMS 前端档案目录
- CPMS 数据库 schema/migration
- CPMS 技术文档与任务索引

## 影响范围

- `cpms/backend/src/dangan`：新增档案列表分页、cursor 和统计读取能力，作为档案业务归属。
- `cpms/backend/src/operator_admin`：清理档案列表分页实现残留，操作管理员模块不再承载列表分页业务。
- `cpms/backend/db`：新增档案统计表和百万级列表/检索索引。
- `cpms/frontend`：档案列表页面与 API 从 `operator_admin` 迁入档案领域目录。
- `memory/05-modules/cpms` 与 `cpms/CPMS_TECHNICAL.md`：更新分页模型和模块边界。

## 主要风险点

- 当前工作树已有其他 CPMS/SFID 改动，必须避免误回滚。
- Axum 路由不能重复覆盖同一路径，需要保证 `/api/v1/archives` 的 GET/POST 合并清晰。
- 游标分页必须保持排序稳定，不能使用 `OFFSET`。
- 统计表必须与创建/删除档案事务一致更新，避免总数漂移。

## 是否需要先沟通

- 否。用户已确认模块边界并要求执行。

## 执行清单

- [x] 梳理当前档案路由、列表分页、前端列表和数据库索引。
- [x] 在 `dangan` 中实现 cursor 分页与统计读取。
- [x] 调整创建/删除档案时的统计表事务更新。
- [x] 迁移前端档案列表/API 到档案领域目录并清理 operator_admin 残留。
- [x] 更新 schema/migration、文档和任务索引。
- [x] 运行后端测试、clippy、前端构建和残留扫描。

## 完成记录

- 2026-05-30：创建任务卡，开始执行 CPMS 档案列表游标分页与模块边界整改。
- 2026-05-30：完成整改。档案列表归属 `dangan`，后端改为游标分页、统计表总数和索引化精确检索；前端迁入 `frontend/dangan`；同步更新 schema、migration、技术文档、中文注释并清理 `operator_admin` 残留。
- 2026-05-30：验证通过 `cargo test`、`cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、`npm run build`、`git diff --check` 和残留扫描。
