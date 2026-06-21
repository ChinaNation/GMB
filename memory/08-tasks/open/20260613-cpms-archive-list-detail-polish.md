# CPMS 公民列表与详情交互完善任务卡

## 任务目标

完善 CPMS 公民档案列表与详情页交互：修复操作记录 HTML 被当 JSON 解析的问题，操作记录表格对齐“操作、操作者账户、详情、时间”，替换详情左侧导航图标，列表整行点击进入详情并新增序号列，编辑态按钮移动到详情卡右上角，资料库上传改为弹窗。

## 范围

- 前端：调整 `cpms/frontend/dangan/ArchiveList.tsx` 列结构和整行点击。
- 前端：调整 `cpms/frontend/dangan/ArchiveDetail.tsx` 导航图标、操作记录列、编辑按钮位置和资料库上传弹窗。
- 前端：增强 `cpms/frontend/common/http.ts` 对非 JSON 响应的错误提示。
- 后端：完善 CPMS API 兜底和档案操作记录返回字段。
- 文档：更新 CPMS 技术文档与 dangan 模块文档。
- 前端：按当前确认，仅同步 SFID 机构详情共享导航的“返回列表”图标，其他 SFID 业务不在本卡范围内。
- 残留：清理旧操作列、旧详情按钮、资料库常驻上传表单和重复数字展示。

## 不做

- 不修改 SFID 业务流程、接口、数据结构和详情 tab 组织方式。
- 不改变 CPMS 档案创建、编辑、删除、投票账户绑定、资料上传下载删除、档案码生成打印的业务流程。
- 不新增外部联网能力。

## 执行记录

- 2026-06-13：创建任务卡，开始执行 CPMS 公民列表与详情交互完善。
- 2026-06-13：后端操作记录接口新增 `operator_account`，并在 `/api/` 请求误落前端 HTML 时统一返回 JSON 404。
- 2026-06-13：前端 HTTP 封装增加非 JSON 响应识别，避免 HTML 响应被误当成 JSON 解析。
- 2026-06-13：公民列表删除操作列和详情按钮，新增序号列，整行点击进入公民详情。
- 2026-06-13：公民详情编辑态“保存 / 取消”移动到详情卡右上角，左侧导航图标已替换。
- 2026-06-13：资料库上传改为弹窗，资料库卡片标题右侧只显示“上传”按钮，重复数量展示已清理。
- 2026-06-13：更新 CPMS 技术文档与 dangan 模块文档。
- 2026-06-13：确认 HTML 报错来自仍在 `127.0.0.1:5173` 运行的 citizenchain 前端，CPMS 后端 `8080` 与 CPMS Vite `5174` 均已命中 JSON API；未停止 citizenchain 进程。
- 2026-06-13：CPMS 公民详情左侧 tab 图标按 SFID 机构详情共享导航语义对齐为房子、文件夹、历史记录；SFID 共享导航“返回列表”图标统一为 CPMS 当前返回图标。
- 2026-06-13：按当前反馈，档案详情有效期改为护照号下一整行；公民状态和选举资格纳入详情两列网格，和姓名、城市等字段上下对齐。
- 2026-06-13：CPMS Vite 默认端口改为 5174，开发代理目标固定为 `127.0.0.1:8080`；前端非 JSON/HTML 错误提示补充具体 API 请求路径。

## 验证记录

- 2026-06-13：`cd cpms/frontend && npm run build` 通过。
- 2026-06-13：`cd cpms/backend && cargo fmt --check` 通过。
- 2026-06-13：`cd cpms/backend && cargo check` 通过。
- 2026-06-13：`cd cpms/backend && cargo test` 通过，32 个测试全部通过。
- 2026-06-13：临时启动 `CPMS_BIND=127.0.0.1:18080` 后端并服务 `cpms/frontend/dist`，`GET /api/v1/health` 返回 200。
- 2026-06-13：`GET /api/v1/not-exists` 返回 JSON 404，确认 `/api/` 未命中不再落到前端 HTML。
- 2026-06-13：使用真实会话请求 `GET /api/v1/archives/ar_d7fafa0eb7654fb9816c960cbadbff12/audit-logs`，返回 `code=0`、38 条记录，首条包含 `operator_account`。
- 2026-06-13：浏览器打开 `http://127.0.0.1:18080/admin`，确认列表表头为“序号、档案号、姓名、性别、年龄、市镇、公民状态、创建时间”，无详情按钮，点击整行进入详情。
- 2026-06-13：浏览器确认编辑后详情卡右上角显示“保存 / 取消”，资料库点击“上传”弹出资料类型/文件/备注弹窗，操作记录表头为“操作、操作者账户、详情、时间”。
- 2026-06-13：再次执行 `cd cpms/frontend && npm run build` 通过。
- 2026-06-13：执行 `cd sfid/frontend && npm run build` 通过；仅有既有 chunk 体积提示。
- 2026-06-13：当前运行态确认 `127.0.0.1:8080/api/v1/not-exists` 与 `localhost:5174/api/v1/not-exists` 均返回 JSON 404；`127.0.0.1:5173` 返回 `citizenchain` 前端 HTML，不属于 CPMS。
- 2026-06-13：复用现有未过期CPMS 机构管理员 session 请求 `localhost:5174/api/v1/archives/ar_d7fafa0eb7654fb9816c960cbadbff12/audit-logs`，返回 `200 application/json`、`code=0`、59 条记录，首条包含 `operator_account`。
- 2026-06-13：调整有效期与选举资格排版后，再次执行 `cd cpms/frontend && npm run build` 通过。
- 2026-06-13：残留扫描确认旧有效期拆行样式、旧 5173 Vite 默认端口、旧 `localhost:8080` 代理目标和旧 HTML 错误提示文案均无命中。
- 2026-06-13：当前 5174 运行态复验 `GET /api/v1/archives/ar_d7fafa0eb7654fb9816c960cbadbff12/audit-logs`，返回 `200 application/json`、`code=0`、59 条记录。
