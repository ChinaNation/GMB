任务需求：
- 在 wuminapp-我的-我的钱包中，通过右上角加号进入“导入冷钱包”页面后，在顶部“导入冷钱包”右侧增加扫码图标。
- 点击扫码图标后调用摄像头扫描钱包二维码，把识别到的钱包账户地址填入导入冷钱包页面的账户地址输入框。
- 图标必须使用扫码图标，不得使用二维码图标。

所属模块：
- wuminapp / wallet
- wuminapp / qr

输入文档：
- memory/07-ai/unified-required-reading.md
- memory/07-ai/workflow.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/document-boundaries.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/pre-submit-checklist.md
- memory/07-ai/unified-protocols.md
- memory/07-ai/unified-naming.md
- memory/07-ai/module-definition-of-done/wuminapp.md
- memory/05-modules/wuminapp/wallet/WALLET_TECHNICAL.md
- memory/05-modules/wuminapp/qr/QR_TECHNICAL.md

必须遵守：
- 不突破钱包模块和二维码模块边界。
- 不新增第二套扫码协议，不新增二维码协议版本。
- 扫码按钮必须使用现有扫码图标资源 `assets/icons/scan-line.svg`，不得使用 `Icons.qr_code*` 或二维码图标。
- 扫码结果只填入账户地址输入框，不自动导入冷钱包。
- 不做旧协议兼容或双轨兼容；只识别当前钱包二维码地址来源和现有账户地址格式。

输出物：
- wuminapp 钱包页面代码调整
- 必要中文注释
- 模块技术文档更新
- 残留清理
- 静态检查或测试验证

验收标准：
- 导入冷钱包页面标题右侧出现扫码图标。
- 点击扫码图标进入摄像头扫码页面。
- 扫描当前钱包二维码后能把账户地址填入导入冷钱包页面输入框。
- 未使用二维码图标。
- 文档已更新，残留已清理。
- wuminapp 模块完成标准已对照。

## 执行记录

- 状态：done
- 代码：`ImportColdWalletPage` 标题右侧已接入 `assets/icons/scan-line.svg` 扫码图标，并通过 `QrScanPage(mode: QrScanMode.raw)` 调用摄像头扫码。
- 解析：扫码结果只提取当前钱包二维码地址、`gmb://account/<address>`、裸 SS58 地址，或当前导入框支持的 0x/64 hex 公钥。
- 行为：扫码只回填输入框，不自动导入冷钱包。
- 验证：`dart analyze lib test`、`flutter test test/wallet/pages/wallet_list_tile_test.dart`、`git diff --check` 已通过。
