# CPMS 年度报告字段收口

## 任务需求

- CPMS 年度报告不再导出护照号。
- CPMS 与 SFID 年度报告字段统一。
- 导出主要表达：
  - 档案号对应公民状态。
  - 档案号对应投票资格。
  - 满 100 年硬删除后需要 SFID 释放绑定关系的档案号。
- 本轮只修复 CPMS 端；SFID 端随后输出后端和前端技术方案。

## 预计修改目录

- `cpms/backend/src/dangan`：调整 `CPMS_STATUS_EXPORT` DTO、哈希内容和导出查询，去掉 `passport_no`。
- `cpms/backend/db`：同步当前基准 schema 和迁移中的年度报告计数字段命名。
- `cpms/frontend/admins`：同步年度报告前端类型。
- `cpms/CPMS_TECHNICAL.md` 与 `memory/05-modules/cpms`：更新 CPMS 年度报告文档并清理旧字段残留。
- `memory/05-modules/sfid` 与 `memory/07-ai`：只同步协议字段说明，SFID 实现方案另行输出。

## 执行清单

- [x] 创建任务卡。
- [x] 修改 CPMS 后端导出结构。
- [x] 修改 CPMS 前端类型。
- [x] 更新 CPMS/SFID 协议文档。
- [x] 清理旧字段残留。
- [x] 运行验证。
