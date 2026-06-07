# 任务卡：SFID 删除迁移兼容并收口初始省级管理员真源

## 任务需求
- 删除 SFID 系统内数据库迁移、旧结构兼容、旧回填和本地 schema finalizer 残留。
- 初始省级管理员只允许以 `sfid/backend/admins/province_admins.rs` 为唯一真源。
- 查明 `0xe0fb43daac7243a64e90b95250e4ffac3d47549c72b53b086785c902365ed148` 与 `LN001-GCB05-944805165-2026` 的绑定位置。

## 预计修改目录
- `sfid/backend/`：后端启动直接创建当前目标结构；删除迁移兼容逻辑；省级管理员启动校验只读取 `province_admins.rs`。
- `sfid/deploy/`：删除生产部署中的迁移脚本和迁移目录依赖。
- `sfid/`：清理本地启动脚本中的本地 schema 修复残留。
- `memory/01-architecture/sfid/`：更新 SFID 架构文档，删除迁移执行说明。
- `memory/05-modules/sfid/`：更新模块文档和部署文档，删除迁移策略残留。

## 验收
- [x] `sfid/backend/db/migrations/` 下不再保留 SQL 迁移文件或 seed 真源。
- [x] `006_sheng_admin_catalog.sql` 删除，初始省级管理员只来自 `province_admins.rs`。
- [x] 启动脚本和部署脚本不再调用迁移脚本。
- [x] 文档不再要求执行 SFID SQL migration。
- [x] 完成 Rust 格式检查、编译检查和测试。

## 完成记录
- 2026-05-31：删除 SFID 后端 SQL migration 文件和 `apply_sfid_migrations.sh`。
- 2026-05-31：后端启动改为创建当前目标结构；初始省级管理员启动对齐只读取 `admins/province_admins.rs`。
- 2026-05-31：清理本地启动脚本、生产安装/更新脚本中的迁移调用。
- 2026-05-31：同步更新 SFID 架构、后端布局、登录、部署和 SFID 工具模块文档。
- 2026-05-31：`cargo fmt --check`、`cargo check`、`cargo test` 全部通过。
