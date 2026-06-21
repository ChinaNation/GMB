# CPMS 年度导出长期补导与前端角标

## 任务需求

- CPMS 机构管理员从每年 1 月 1 日起可导出上一年度数据；未导出时不再在 1 月 10 日后关闭导出窗口。
- 超过 1 月 10 日仍未导出时，operators一直不能登录，直到CPMS 机构管理员完成导出。
- 系统设置页导出按钮按状态变灰或可点击；待导出期间显示角标提示，导出完成后角标消失。
- CID 是否收到数据、是否禁用 CPMS 安装码由 CID 系统实现，CPMS 不处理。

## 预计修改目录

- `citizenpassport/backend/dangan`：年度待导出年度、补导、逾期锁定和状态查询规则。
- `citizenpassport/backend/dangan`：新增年度导出状态接口，导出接口调用新规则。
- `citizenpassport/frontend/admins`：系统设置页按钮状态、系统设置页签角标、年度导出 API/type。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md`：同步 CPMS 技术文档中的年度导出规则。
- `memory/05-modules/citizenpassport`：同步登录、档案导出和错误码模块文档。
- `memory/08-tasks`：记录本次执行、验证和残留清理。

## 执行清单

- [x] 创建任务卡。
- [x] 实现后端年度导出状态计算。
- [x] 修改operators逾期锁定规则。
- [x] 新增前端状态接口和角标。
- [x] 更新文档和中文注释。
- [x] 运行测试、构建和残留扫描。

## 验收标准

- CPMS 机构管理员在每年 1 月 1 日后只要存在未导出年度即可导出最早未导出年度。
- 导出完成后该年度按钮置灰，下一年度 1 月 1 日再提示。
- 1 月 11 日起仍有未导出年度时，operators登录和已有会话被锁定。
- 前端不会在非可导出状态点出后端英文错误。
- 文档、注释、残留和测试同步完成。

## 完成记录

- 2026-05-30：后端改为按 `system_install.initialized_at` 所在年份开始查找最早未导出年度，支持 1 月 10 日后继续补导。
- 2026-05-30：新增 `GET /api/v1/archives/status-export/state`，返回按钮可用状态、角标状态和operators锁定状态。
- 2026-05-30：operators锁定改为 UTC 1 月 11 日起持续生效，直到所有逾期年度补导完成；CPMS 机构管理员不受锁定影响。
- 2026-05-30：前端系统设置页按状态置灰或启用导出按钮，顶部系统设置页签显示“待导出/逾期”角标，导出后刷新状态。
- 2026-05-30：同步 CPMS 技术文档、登录文档、档案导出文档和错误码文档，清理旧“导出窗口关闭”残留。
- 2026-05-30：`cargo test`、`cargo clippy --all-targets -- -D warnings`、`npm run build`、`git diff --check` 均通过。
