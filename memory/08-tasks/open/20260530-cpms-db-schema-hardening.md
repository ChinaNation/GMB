# CPMS 数据库字段约束加固

## 任务需求

- 修复 CPMS 当前基准数据库字段设计问题。
- 加固号码回收池，支持档案号和护照号多轮复用。
- 加固档案状态、公民状态和投票资格数据库约束。
- 将日期字段从文本收口为数据库日期类型。
- 在安装状态中显式保存省市代码。
- 删除管理员停用状态残留字段。
- 持久化年度导出文件，便于重复下载同一份已签名报告。

## 预计修改目录

- `citizenpassport/backend/db`：更新当前基准 schema/migration；涉及数据库结构和约束调整。
- `citizenpassport/backend`：同步 SQL 查询、写入、类型读取、年度导出和初始化逻辑。
- `cpms`：更新 CPMS 技术文档，说明当前数据库结构。
- `memory/01-architecture/citizenpassport`：同步架构技术文档。
- `memory/08-tasks`：登记任务卡和完成记录。

## 执行清单

- [x] 创建任务卡并登记索引。
- [x] 调整 `archive_number_recycle_pool` 唯一约束。
- [x] 为 `archives` 增加状态一致性约束。
- [x] 将档案日期字段改为 `DATE` 并同步后端读写。
- [x] 为 `system_install` 增加省市代码字段。
- [x] 删除 `admin_users.status` 及后端依赖。
- [x] 为 `cpms_status_exports` 增加导出 JSON 持久化。
- [x] 更新文档并运行验证。

## 验收标准

- 新库初始化 schema 能直接支撑当前 CPMS 业务。
- 回收号码复用后，未来再次硬删除不会被回收池唯一约束挡住。
- 数据库层拒绝非法档案状态、公民状态和投票资格组合。
- 后端测试与格式化通过。

## 完成记录

- 2026-05-30：`system_install` 新增 `province_code / city_code`，后端安装初始化和运行时读取改为使用显式字段。
- 2026-05-30：`admin_users.status` 已从基准 schema、鉴权查询、管理员创建和CPMS 机构管理员绑定响应中移除。
- 2026-05-30：`archives.birth_date / valid_from / valid_until` 改为 `DATE`，并增加档案状态、公民状态和投票资格组合约束。
- 2026-05-30：`archive_number_recycle_pool` 改为只约束未使用号码唯一，补充多轮复用后再次入池的数据库测试。
- 2026-05-30：`cpms_status_exports` 增加 `export_file JSONB`，同一年重复导出返回首次生成的已签名 JSON。
- 2026-05-30：已运行 `cargo fmt`、`cargo test`、`cargo clippy --all-targets -- -D warnings`，均通过。
