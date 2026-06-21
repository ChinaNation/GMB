# 任务卡：审计日志 detail 结构化（事实与展示分离）

状态：代码完成（待用户端到端 QA）
完成：2026-06-12
创建：2026-06-12
模块：CID（backend + frontend）
来源：用户确认方案 A（detail 存结构化 JSON 事实，前端模板渲染人话，存量清空）。

## 口径

- 审计表 detail 列 TEXT → JSONB，值存系统原值（代码/布尔/原始字段），写入点只声明事实不写展示文案；`{:?}` Debug 格式泄漏（Some(...)）随结构化消失。
- 前端操作记录「详情」列按键名中文映射 + 值按类型翻译（机构代码→中文、布尔→是/否、状态→中文），未知键「键名: 值」兜底，非 JSON 旧值原样显示（异常兜底）。
- 存量清空：列类型收敛块里 TRUNCATE audit 后 ALTER 为 JSONB（开发库运行痕迹，系统未上线无合规包袱，不留旧方案）。

## 实施清单

- [x] core/db.rs：audit DDL detail JSONB + 收敛块（TEXT 时 TRUNCATE+ALTER，幂等）
- [x] core/runtime_ops.rs append_audit_log：detail String → serde_json::Value
- [x] 10 个写入点改 json!({...})（citizenpassport/handler 5 + citizens 5），字段名小写蛇形
- [x] audit.rs AuditLogEntry.detail → serde_json::Value，列表查询同步
- [x] 前端 GovDetailPage：详情列渲染器（键名中文映射 + 值翻译 + 兜底）
- [x] cargo check/test + npm build + 开发库实测写入/读取
- [x] FRONTEND_LAYOUT.md / 任务卡回写

## 验收

- 生成安装码后操作记录详情显示「市：锦程市；机构：政府」级别的纯中文。
- 审计行 detail 为 JSONB 可按字段查询；无 Some(...) 残留。

## 完成记录

- 后端:audit 表 detail TEXT→JSONB(收敛块幂等:TEXT 时 TRUNCATE+ALTER,开发库实测列已转 jsonb、字段级查询 detail->>'institution' 可用);append_audit_log/append_cpms_audit_log_best_effort/append_import_audit 签名改 serde_json::Value;10 个写入点全部 json!({...}) 结构化(导入审计=基础事实+调用方扩展字段合并,成功路径 5 个计数字段也结构化);audit.rs 关键词搜索 lower(detail::text) 适配 jsonb;Some(...) Debug 泄漏随结构化消失。cargo check 0 error + cargo test 58/58。
- 前端:GovDetailPage 详情列渲染器 formatAuditDetail——AUDIT_DETAIL_KEY_LABEL 22 个键名中文映射 + AUDIT_DETAIL_VALUE_LABEL 枚举值翻译(机构代码复用 INSTITUTION_CODE_LABEL、CPMS 状态、绑定方式 create/replace、结果 SUCCESS/FAILED)、布尔→是/否、空值跳过、未知键「键名: 值」兜底、旧文本行原样显示。npm build 0 error。
- 文档:FRONTEND_LAYOUT 增设「审计 detail 事实与展示分离」铁律。
- 验收示例:生成安装码记录详情显示「市：锦程市；机构：政府」。
