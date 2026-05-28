# CPMS 姓名字段统一为 last_name / first_name

## 任务需求

- CPMS 公民姓名不能只在前端拆分展示。
- 前端、后端、数据库统一使用 `last_name` 和 `first_name`。
- 其他字段继续使用统一英文 snake_case 字段名。

## 完成情况

- 新增数据库迁移 `0008_archive_last_first_name.sql`，从 `full_name` 迁移到 `last_name / first_name` 并删除 `full_name`。
- 后端请求、响应、查询、更新统一改为 `last_name / first_name`。
- 前端类型、创建表单、编辑表单、列表和详情统一改为 `last_name / first_name`。
