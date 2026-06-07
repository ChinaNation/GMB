任务需求：
根据 SFID 绑定接口误用 401 导致前端退出和 sfid_number 空读取的问题，完整检查 CPMS 与 SFID 系统，设计并落地统一错误码方案，完成代码、文档、中文注释和残留清理。

所属模块：
- SFID
- CPMS
- Architect

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/unified-required-reading.md
- memory/07-ai/unified-protocols.md
- memory/07-ai/unified-naming.md
- memory/07-ai/workflow.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/module-definition-of-done/sfid.md
- memory/07-ai/module-definition-of-done/cpms.md
- memory/05-modules/sfid/SFID-CPMS-QR-v1.md
- memory/05-modules/sfid/backend/BACKEND_LAYOUT.md
- memory/05-modules/sfid/frontend/FRONTEND_LAYOUT.md
- cpms/CPMS_TECHNICAL.md

必须遵守：
- 401 只能表示登录态无效，不得表示业务校验失败。
- CPMS 永不联网，错误码设计不得引入在线认证语义。
- SFID 不保存原始实名数据，不得因错误响应泄露实名原文或敏感细节。
- 不兼容旧错误语义，不保留旧 401 业务失败分支。
- 代码修改后必须更新文档、完善中文注释、清理残留。

预计修改目录：
- sfid/backend/：统一 SFID 后端错误类型、状态码映射和绑定接口业务错误返回，涉及代码与中文注释。
- sfid/frontend/utils/：调整通用 HTTP 请求封装，401 抛认证错误，业务错误抛普通 API 错误，涉及代码。
- sfid/frontend/citizens/：调整绑定弹窗业务错误展示与成功结果读取防护，涉及代码。
- cpms/backend/：检查并统一 CPMS 后端错误响应结构和状态码语义，涉及代码与中文注释。
- memory/05-modules/sfid/：更新 SFID 错误码规范，涉及文档。
- memory/05-modules/cpms/：补齐 CPMS 错误码规范，涉及文档。

输出物：
- 代码
- 中文注释
- 测试或验证结果
- 文档更新
- 残留清理

验收标准：
- SFID 绑定业务错误不再返回 401，不再触发前端自动退出。
- SFID 前端 request 不再通过 undefined 表示 401。
- CPMS 与 SFID 错误响应具备稳定 code 字段。
- 文档记录 HTTP 状态码和业务错误码边界。
- 测试或构建验证通过，无法执行时说明原因。
- 残留已清理。

执行记录：
- 2026-05-26：创建任务卡，开始检查 CPMS/SFID 错误处理。
- 2026-05-26：SFID 后端 `ApiError` 增加稳定 `error_code`；公民绑定、登录 challenge、QR 状态、CPMS ARCHIVE 验真中的业务失败不再返回 401。
- 2026-05-26：SFID 前端 `request()` 改为 401 抛 `AuthExpiredError`，业务失败抛 `ApiError`；绑定弹窗按 `errorCode` 展示业务文案。
- 2026-05-26：CPMS 后端 `ApiError` 增加稳定 `error_code`；challenge 过期改 410、签名失败改 422、管理员停用改 403。
- 2026-05-26：补齐 SFID/CPMS 错误码规范文档，更新目录布局、CPMS 技术说明与统一命名登记。
- 2026-05-26：验证通过：`cargo check`/`cargo test` for SFID backend and CPMS backend；`npm run build` for SFID frontend and CPMS frontend。

- 状态：done

## 完成信息

- 完成时间：2026-05-26 11:21:06
- 完成摘要：统一 CPMS/SFID 错误码方案并落地 401 边界、稳定 error_code、前端错误处理与文档规范
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
