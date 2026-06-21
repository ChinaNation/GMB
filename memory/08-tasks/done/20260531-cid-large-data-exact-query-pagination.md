# 任务卡：CID 大数据精确查询与服务端分页

## 任务需求
- CID 首页公民数据不再默认加载列表，管理员输入精确条件后只返回命中的记录。
- 私权机构、公权机构列表不再一次性返回全量数据，改为精确查询优先，服务端游标分页兜底。
- 公民、机构写入落到数据库行表，通过唯一约束保证精确写入和重复拦截。

## 预计修改目录
- `citizencode/backend/`：新增当前目标行表、索引、游标分页 DTO 和精确查询逻辑。
- `citizencode/backend/citizens/`：公民后台查询改为精确查询/游标分页返回对象。
- `citizencode/backend/institutions/`：机构列表改为精确查询优先和服务端分页返回对象。
- `citizencode/frontend/citizens/`：首页不自动加载全量公民列表，输入精确条件后查询。
- `citizencode/frontend/institutions/`：机构表格改为服务端分页和精确搜索触发。
- `memory/01-architecture/citizencode/`：更新 CID 大数据查询和分页架构说明。
- `memory/05-modules/citizencode/`：更新公民与机构模块技术说明，清理全量列表表述。

## 验收
- [x] 公民首页登录后不自动请求全量列表。
- [x] 公民查询接口返回分页对象，不再返回裸数组。
- [x] 机构列表接口返回分页对象，不再返回裸数组。
- [x] 后端数据库包含公民和机构行表及必要唯一索引/复合索引。
- [x] 公民、机构新增/更新写入行表。
- [x] 文档已更新，旧全量列表说明已清理。
- [x] 完成格式检查、编译检查和相关测试。

## 完成记录
- 2026-05-31：创建任务卡。
- 2026-05-31：新增 `cid_citizens / cid_institutions / cid_institution_accounts` 当前目标行表和索引。
- 2026-05-31：公民首页、私权机构列表、公权机构列表改为精确查询和游标分页返回对象。
- 2026-05-31：公民绑定、机构创建/更新、机构账户创建同步写入行表。
- 2026-05-31：更新 CID 架构、后端 citizens/institutions 和前端布局文档。
- 2026-05-31：`cargo fmt --check`、`cargo check`、`cargo test`、`npm --prefix citizencode/frontend run build` 通过。
