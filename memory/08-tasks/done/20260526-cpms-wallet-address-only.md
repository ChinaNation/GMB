任务需求：
第 1 步改造 CPMS 系统和 wuminapp：CPMS 线下只扫码识别 wuminapp 钱包账户地址并保存到公民档案，生成携带钱包地址/公钥的档案码；CPMS 不再生成钱包签名请求、不保存钱包签名证明。wuminapp 电子护照页提供钱包地址二维码。同步修改 `cpms/cpms.sh`，启动时默认保留现有数据库，不再每次运行都重新初始化 CPMS。

所属模块：
- CPMS
- wuminapp
- Architect

预计修改目录：
- `cpms/backend/db/`：简化钱包字段和迁移，删除钱包签名 challenge 表设计，涉及数据库。
- `cpms/backend/src/dangan/`：调整 ARCHIVE 载荷和签名原文，只覆盖钱包地址/公钥，涉及代码。
- `cpms/backend/src/operator_admin/`：删除钱包签名 challenge/verify 接口，新增扫码保存钱包地址接口，涉及代码。
- `cpms/frontend/web/src/operator/`：钱包绑定弹窗改为扫描 wuminapp 钱包账户二维码并直接保存，涉及代码。
- `wuminapp/lib/my/myid/`：电子护照页提供钱包地址二维码入口，CPMS 阶段不签名，涉及代码。
- `cpms/cpms.sh`：默认保留数据库，仅显式传入重置参数时重建，涉及脚本。
- `memory/05-modules/`：更新 CPMS/wuminapp 流程文档，涉及文档。
- `memory/07-ai/unified-protocols.md`：更新 CPMS 与 SFID 钱包签名职责边界，涉及协议文档。

执行记录：
- 2026-05-26：创建任务卡，开始执行 CPMS 钱包地址扫码保存与 wuminapp 地址二维码改造。
- 2026-05-26：CPMS 后端删除钱包签名 challenge/verify 流程，改为 `POST /api/v1/archives/:archive_id/wallet` 保存钱包地址并解析钱包公钥。
- 2026-05-26：CPMS ARCHIVE 载荷和签名原文改为只携带并覆盖 `wallet_address / wallet_pubkey / wallet_sig_alg`，不再包含钱包证明原文和钱包签名。
- 2026-05-26：CPMS 前端档案详情页的钱包绑定/更新弹窗改为扫描 wuminapp `user_contact` 钱包二维码并直接保存。
- 2026-05-26：wuminapp 电子护照页选择钱包后展示钱包地址二维码；CPMS 阶段不再签名，SFID 阶段再做钱包签名确认。
- 2026-05-26：`cpms/cpms.sh` 改为默认保留现有数据库；仅 `CPMS_RESET=1 ./cpms.sh` 或 `./cpms.sh --reset` 时重建数据库。
- 2026-05-26：更新 CPMS、wuminapp 与统一协议文档，清理 CPMS 钱包签名证明残留说明。
- 2026-05-26：完成验证：cpms/backend `cargo fmt && cargo check && cargo test` 通过；cpms/frontend/web `npm run build` 通过；wuminapp `dart format ... && flutter analyze` 通过；`git diff --check` 通过。

- 状态：done

## 完成信息

- 完成时间：2026-05-26 18:19:39
- 完成摘要：完成 CPMS 钱包地址扫码保存与 wuminapp 地址二维码第 1 步改造：CPMS 不再做钱包签名，只保存钱包地址并签出档案码；wuminapp 电子护照页展示钱包地址二维码；cpms.sh 默认保留数据库。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
