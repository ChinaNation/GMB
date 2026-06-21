# CPMS 公民档案详情左右导航改造任务卡

## 任务目标

将 CPMS 系统的公民档案详情页改为左侧导航 + 右侧内容区布局。左侧依次显示“返回列表、档案详情、资料库、操作记录”，档案详情和资料库沿用 CPMS 现有内容，操作记录使用 CPMS 自有 `audit_logs` 数据。

## 范围

- 前端：改造 `cpms/frontend/dangan/ArchiveDetail.tsx` 的详情页布局。
- 前端：新增档案操作记录读取和展示能力，数据来源限定为 CPMS 后端。
- 后端：补充按档案查询 CPMS `audit_logs` 的只读接口。
- 文档：更新 CPMS 技术文档与 dangan 模块文档。
- 残留：清理详情页中被左右导航替换的旧布局、旧文案和旧注释口径。

## 不做

- 不修改 SFID 目录和 SFID 文档。
- 不改变 CPMS 档案创建、编辑、删除、投票账户绑定、资料库上传下载删除、档案码生成打印的业务流程。
- 不新增联网能力，CPMS 仍保持离线系统边界。

## 执行记录

- 2026-06-13：创建任务卡，开始执行 CPMS 公民档案详情左右导航改造。
- 2026-06-13：前端 `ArchiveDetail.tsx` 改为左侧“返回列表 / 档案详情 / 资料库 / 操作记录”导航和右侧内容区，档案详情与资料库沿用 CPMS 现有业务内容。
- 2026-06-13：后端新增 `GET /api/v1/archives/:archive_id/audit-logs`，按档案 ID、档案号和审计 detail 聚合最近 100 条 CPMS 本机操作记录。
- 2026-06-13：更新 `cpms/CPMS_TECHNICAL.md` 与 `memory/05-modules/cpms/backend/dangan/DANGAN_TECHNICAL.md`，清理资料库旧上下堆叠展示口径。

## 验证记录

- 2026-06-13：`cd cpms/frontend && npm run build` 通过。
- 2026-06-13：`cd cpms/backend && cargo check` 通过。
- 2026-06-13：`cd cpms/backend && cargo test` 通过，32 个测试全部通过。
- 2026-06-13：临时启动 `CPMS_BIND=127.0.0.1:18080` 后端并服务 `cpms/frontend/dist`，`GET /api/v1/health` 返回 200。
- 2026-06-13：使用真实CPMS 机构管理员 cookie 请求 `GET /api/v1/archives/ar_d7fafa0eb7654fb9816c960cbadbff12/audit-logs`，返回 `code=0` 且包含该档案的 CPMS 审计记录。
- 2026-06-13：浏览器打开 `http://127.0.0.1:18080/admin/archives/ar_d7fafa0eb7654fb9816c960cbadbff12`，确认左侧导航显示“返回列表 / 档案详情 / 资料库 / 操作记录”，默认右侧显示档案详情；点击“操作记录”后右侧显示操作记录表格。
