# 任务卡：CPMS 年度状态导出与注销状态统一

## 任务需求

按最新确认统一 CPMS 公民状态模型：公民状态只分 `NORMAL`（正常）和 `REVOKED`（注销）；投票资格只分有和没有。正常公民可以有或没有投票资格，注销公民必须没有投票资格。软删除就是注销，年度导出文件用于让 SFID 更新该档案号对应的公民状态和投票资格；硬删除只用于释放档案号、护照号复用，不再表示公民状态变化。

## 建议模块

- CPMS 后端
- CPMS 前端
- CPMS 数据库
- CPMS/SFID 协议与模块文档

## 影响范围

- `cpms/backend/src/dangan`：新增状态导出文件构造逻辑，统一 `REVOKED` 状态校验。
- `cpms/backend/src/operator_admin`：软删除时强制写入注销状态和无投票资格，新增导出接口。
- `cpms/backend/src/super_admin`：状态更新逻辑改为正常/注销，并保持注销无投票资格。
- `cpms/backend/db`：更新 `citizen_status` 约束为 `NORMAL / REVOKED`。
- `cpms/frontend/web/src`：清理“异常/ABNORMAL”残留，改为“注销/REVOKED”。
- `memory/05-modules` 与 `memory/07-ai`：同步协议、命名和技术文档。

## 主要风险点

- 不能保留 `ABNORMAL` 兼容分支。
- 注销公民必须强制 `voting_eligible=false`。
- 导出文件不能包含实名原文。
- 硬删除释放记录只表达号码可复用，不表达公民状态变化。
- CPMS 仍必须保持永不联网，导出只生成离线文件内容。

## 执行清单

- [x] 创建 CPMS 年度状态导出模块。
- [x] 统一公民状态枚举为 `NORMAL / REVOKED`。
- [x] 软删除强制注销和无投票资格。
- [x] 导出文件包含状态记录和号码释放记录。
- [x] 清理前端、数据库、文档中的 `ABNORMAL` 残留。
- [x] 增加年度报告导出窗口、超级管理员权限和操作管理员逾期锁定。
- [x] 运行 CPMS 后端格式化、测试、clippy 和前端构建。

## 完成记录

- 2026-05-30：新增 `cpms/backend/src/dangan/export.rs`，生成 `SFID_CPMS_V1 / CPMS_STATUS_EXPORT` 离线 JSON 文件。
- 2026-05-30：统一公民状态为 `NORMAL / REVOKED`，软删除时同步写入注销状态和无投票资格。
- 2026-05-30：系统设置页新增状态更新文件导出按钮，前端文案清理为“正常/注销”。
- 2026-05-30：年度报告仅 SUPER_ADMIN 可导出；UTC 1 月 6 日到 1 月 10 日未导出上一年度报告时锁定 OPERATOR_ADMIN 登录和已有会话。
- 2026-05-30：同步 CPMS/SFID 协议、命名和模块文档，登记 `CPMS_STATUS_EXPORT`。
- 2026-05-30：已运行 `cargo fmt`、`cargo test`、`cargo clippy --all-targets -- -D warnings` 和 `npm run build`，均通过。
