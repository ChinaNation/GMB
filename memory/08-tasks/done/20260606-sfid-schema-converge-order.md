# SFID schema 收敛顺序修复

## 任务需求

修复 SFID 后端启动时因旧 `subjects` 表缺少 `legal_rep_sfid_number` 字段而在创建索引阶段崩溃的问题。不能依赖清空数据库解决字段变更,必须让 schema 在启动期按当前目标状态自动收敛。

## 范围

- `sfid/backend/core`:调整 PostgreSQL schema 初始化顺序,先同步字段目标状态,再创建索引。
- `memory/01-architecture/sfid`:更新 SFID 数据库启动流程文档。
- 残留扫描:确认机构链上状态旧字段不再作为机构字段保留,确认索引不再先于字段创建。

## 非目标

- 不重构 SFID 业务模块。
- 不恢复旧兼容流程。
- 不清空数据库。
- 不修改 CPMS、wuminapp 或 citizenchain。

## 验收标准

- 旧库已存在 `subjects` 但缺少 `legal_rep_*` 字段时,后端启动 schema 初始化可以先补字段再建索引。
- 目标状态校验能明确检查 `subjects.legal_rep_*` 存在、`subjects.chain_status` 和 `gov.chain_status` 不存在。
- `cargo fmt` 和 `cargo check` 通过。
- 文档已更新,残留扫描完成。

## 进度

- 2026-06-06: 创建任务卡,准备执行。
- 2026-06-06: 已将 `subjects/gov` 字段收敛提前到索引创建之前,新增目标状态校验。
- 2026-06-06: 已验证当前旧库启动后自动补齐 `subjects.legal_rep_*`,删除 `subjects/gov.chain_status`,并成功创建 `idx_subjects_legal_rep`。
- 2026-06-06: 已更新 SFID 架构文档,完成格式化、编译检查和残留扫描。

## 完成记录

- 后端验证: `cargo fmt && cargo check` 通过。
- 启动验证: 使用当前 `.env.dev.local` 短启动 `sfid-backend`,健康检查通过。
- 数据库验证: `subjects_legal_rep_columns=6`, `subjects_chain_status_columns=0`, `gov_chain_status_columns=0`, `idx_subjects_legal_rep=1`。
- 残留扫描:未发现 `subjects.chain_status`、`gov.chain_status` 或机构旧链上状态字段代码残留。
