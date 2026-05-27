任务需求：
执行第 1 步改造 CPMS 系统和 wuminapp：CPMS 公民档案保存用户钱包账户，签出 ARCHIVE 档案码前必须完成 wuminapp 钱包签名证明，ARCHIVE 携带钱包证明字段；wuminapp 支持识别并签署 CPMS 档案钱包证明签名请求。完成后完善中文注释、更新文档、清理残留。

所属模块：
- CPMS
- wuminapp
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
- memory/07-ai/module-definition-of-done/cpms.md
- memory/07-ai/module-definition-of-done/wuminapp.md
- cpms/CPMS_TECHNICAL.md
- memory/05-modules/wuminapp/qr/QR_TECHNICAL.md
- memory/05-modules/wuminapp/signer/SIGNER_TECHNICAL.md

必须遵守：
- CPMS 仍然是离线实名系统，不引入联网依赖。
- CPMS 公民真实档案允许保存钱包账户地址和公钥，换钱包必须回公安局更新。
- ARCHIVE 必须携带钱包证明字段，且 CPMS 签名必须覆盖钱包证明字段。
- 无钱包证明的档案不得签出 ARCHIVE。
- wuminapp 只负责识别签名请求和输出签名回执，不保存 CPMS 业务状态。
- 代码修改后必须更新文档、完善中文注释、清理残留。

预计修改目录：
- memory/07-ai/unified-protocols.md：登记 ARCHIVE 钱包证明字段和签名原文，涉及协议文档。
- cpms/backend/db/migrations/：新增 archives 钱包字段迁移，涉及数据库。
- cpms/backend/src/dangan/：调整 ARCHIVE 载荷、签名原文、钱包证明要求，涉及代码。
- cpms/backend/src/operator_admin/：新增钱包证明 challenge/verify 接口并返回钱包字段，涉及代码。
- cpms/frontend/web/src/：增加档案钱包绑定/更新和签名回执扫码流程，涉及代码。
- wuminapp/lib/qr/：识别 CPMS 档案钱包证明签名请求，涉及代码。
- wuminapp/lib/signer/：签署 CPMS 档案钱包证明并输出回执，涉及代码。
- memory/05-modules/cpms/：更新 CPMS 钱包证明和错误码文档，涉及文档。
- memory/05-modules/wuminapp/：更新 wuminapp QR/签名文档，涉及文档。

输出物：
- 代码
- 数据库迁移
- 中文注释
- 测试或验证结果
- 文档更新
- 残留清理

验收标准：
- CPMS 档案可保存 wallet_address / wallet_pubkey / wallet_signature。
- CPMS 可生成钱包证明签名请求并验证 wuminapp 签名回执。
- CPMS ARCHIVE 包含 wallet_* 字段，且 CPMS 签名覆盖 wallet_* 字段。
- 没有钱包证明的档案不能生成 ARCHIVE。
- wuminapp 能识别 CPMS 钱包证明签名请求并生成签名回执。
- CPMS 后端、前端和 wuminapp 必要验证通过。
- 文档已更新，残留已清理。

执行记录：
- 2026-05-26：创建任务卡，开始执行 CPMS + wuminapp 钱包证明改造。
- 2026-05-26：CPMS 后端新增档案钱包字段、钱包证明 challenge/verify 接口、ARCHIVE 钱包证明载荷和签名覆盖规则。
- 2026-05-26：CPMS 前端新增档案详情页钱包地址录入、签名请求二维码、签名回执扫码/粘贴验证、ARCHIVE 生成前置限制。
- 2026-05-26：wuminapp 身份签名页文案调整为通用身份签名，复用现有 sign_request/sign_response 机制签署 CPMS 档案钱包证明。
- 2026-05-26：更新 CPMS、wuminapp 与统一协议文档，记录 ARCHIVE 钱包证明字段、签名原文、错误码和流程边界。
- 2026-05-26：完成验证：cpms/backend `cargo fmt && cargo check && cargo test` 通过；cpms/frontend/web `npm run build` 通过；wuminapp `flutter analyze` 通过。
- 2026-05-26：按交互调整继续收敛钱包绑定流程：CPMS 公民信息区“钱包账户：未绑定”旁显示绑定按钮；点击后弹窗展示开放式签名请求二维码；wuminapp 回执带回钱包地址和公钥；CPMS 扫描回执后直接验签保存钱包账户；删除单独的钱包账户绑定板块。
- 2026-05-26：完成追加验证：cpms/backend `cargo fmt && cargo check && cargo test` 通过；cpms/frontend/web `npm run build` 通过；wuminapp `dart format ... && flutter analyze` 通过。

- 状态：done

## 完成信息

- 完成时间：2026-05-26 13:09:53
- 完成摘要：完成 CPMS 档案钱包证明第 1 步改造：后端保存钱包账户并验证 wuminapp 签名，前端支持签名请求/回执流程，wuminapp 支持通用身份签名文案，相关协议和模块文档已更新。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
