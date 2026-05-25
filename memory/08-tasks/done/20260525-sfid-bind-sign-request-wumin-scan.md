# SFID 绑定签名请求和 wuminapp 现场扫码修复

## 任务需求

1. 修复 SFID 公民绑定流程中 wuminapp 扫描签名请求后提示 `sign_request.address 必填` 的问题。
2. 删除 SFID 公民绑定弹窗“上传二维码”按钮左侧图标。
3. 将 wuminapp 电子护照现场签名扫描二维码页面的扫码框改为与其他扫码页一致的正方形。

## 建议模块

- SFID 后端公民绑定 challenge。
- SFID 前端公民绑定弹窗。
- wuminapp 电子护照现场签名页。

## 影响范围

- `sfid/backend/citizens/`
- `sfid/frontend/citizens/`
- `wuminapp/lib/my/myid/`
- `memory/05-modules/sfid/`
- `memory/05-modules/wuminapp/`

## 主要风险点

- `sign_request` 必须带当前待绑定钱包的 `address` 和 `pubkey`，否则 wuminapp 严格解析会拒绝。
- 后端绑定签名验证必须校验提交的地址与 challenge 锁定地址一致，不能只在最终提交时再信任前端传入。
- “上传二维码”只删图标，不改变上传功能。
- wuminapp 扫码框只改现场签名页，不影响通用签名回执扫描页。

## 验收标准

- SFID 绑定签名请求的 `body.address` 非空，`body.pubkey` 为对应 `0x` 公钥。
- wuminapp 扫描 SFID 绑定签名请求不再报 `sign_request.address 必填`。
- SFID 绑定弹窗“上传二维码”按钮无左侧图标。
- wuminapp 电子护照现场签名扫描页显示正方形扫码框。
- 相关构建/静态检查通过，文档同步更新，残留清理完成。

## 执行记录

- 已修复 SFID 公民绑定 challenge：生成 challenge 时接收用户 SS58 地址，解析并锁定对应 `0x` 公钥，`sign_request.body.address/pubkey` 不再为空。
- 已在最终绑定提交时校验提交地址与 challenge 锁定公钥一致，防止 challenge 被换地址复用。
- 已修复绑定弹窗 `bind_archive` 提交时误把 `account_pubkey` 当作 `user_address` 的问题，改为使用记录中的 SS58 钱包地址。
- 已删除“上传二维码”按钮左侧上传图标，按钮与“开启扫码”保持同一纯文字风格。
- 已将 wuminapp 电子护照现场签名页的相机区域改为正方形扫码框 + 四角提示。
- 已同步 SFID 公民后端文档、SFID 前端布局文档和 wuminapp 用户模块文档。

## 验证记录

- `cargo check --manifest-path sfid/backend/Cargo.toml` 通过。
- `cargo test --manifest-path sfid/backend/Cargo.toml` 通过，70 passed。
- `npm run build` 通过。
- `dart format lib/my/myid/myid_sign_page.dart` 通过。
- `flutter analyze lib/my/myid/myid_sign_page.dart` 通过。

- 状态：done

## 完成信息

- 完成时间：2026-05-25 10:23:12
- 完成摘要：已修复 SFID 绑定签名请求缺少 address/pubkey、移除上传二维码按钮图标，并将 wuminapp 现场签名扫码框改为正方形；相关验证和文档已完成。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
