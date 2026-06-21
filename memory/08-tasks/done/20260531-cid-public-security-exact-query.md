# CID 公安局确定性展示与公民精确查询修复

## 任务需求

- 删除开发期旧公民、机构、机构账户业务数据，不做迁移、不做兼容。
- 公民列表只允许使用档案号、身份 ID、投票账户地址或投票账户公钥精确检索。
- 公安局作为确定性机构独立展示，进入公安局 tab 后直接显示，不依赖搜索。
- 公安局前端首次获取后写入本地缓存，再次进入直接显示缓存。
- 修复后更新文档、补充必要中文注释、清理残留。

## 修改范围

- `citizencode/backend/citizens`
- `citizencode/backend/institutions`
- `citizencode/backend/app_core`
- `citizencode/frontend/citizens`
- `citizencode/frontend/institutions`
- `memory/01-architecture/citizencode`
- `memory/05-modules/citizencode`

## 约束

- 不保留旧数据兼容逻辑。
- CID 公民查询不得包含姓名。
- 公安局不属于普通公权机构搜索列表。
- 前端缓存只能作为展示缓存，不能作为业务真源。

## 验收

- 精确档案号、身份 ID、投票账户能命中新流程数据。
- 空关键词不会返回公民或普通机构全量列表。
- 公安局 tab 无搜索框，首次加载后缓存，再次进入直接显示。
- 普通公权机构列表不显示公安局。
- 后端格式检查、测试、前端构建通过。

## 完成记录

- 后端新增公安局确定性列表接口，普通机构查询拒绝 `PUBLIC_SECURITY`。
- 前端公安局 tab 接入本地展示缓存和手动刷新。
- 公民搜索文案收敛为档案号、身份 ID、投票账户。
- CPMS 年度导入和机构账户删除同步查询行表。
- 本地开发库旧公民、机构、账户、CPMS 业务数据已清空。
- 验证通过：`cargo fmt --check`、`cargo check`、`cargo test`、`npm run build`。
