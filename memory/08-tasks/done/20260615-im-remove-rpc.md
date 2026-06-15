# 20260615 IM 通信节点移除 RPC 依赖

- 任务编号：20260615-im-remove-rpc
- 状态：done
- 所属模块：wuminapp / citizenchain node
- 当前负责人：Codex
- 创建时间：2026-06-15

## 任务需求

移除 IM 通信节点功能对节点 RPC 的依赖。RPC 只适合本机前后端或受控局域网连接，公民手机会随身移动，不能要求手机始终和家里电脑处于同一局域网；通信节点二维码也不能继续携带 RPC URL。

## 本轮执行范围

- 删除公民端通信节点配对配置中的旧 RPC URL 字段依赖。
- 删除通信节点二维码 body 中的旧 RPC URL 字段和对应校验。
- 删除节点端旧手机节点 RPC 注册与运行态开关。
- 清理桌面设置页通信节点状态中的 RPC 展示。
- 将通信节点二维码收口为固定节点信息二维码，不再携带有效期。
- 将通信节点设置页扫描动作图标统一为公民常用 `scan-line.svg`。
- 更新 IM 技术文档、节点技术文档、统一协议和统一命名登记。

## 必须遵守

- 不修改 `citizenchain/runtime/`。
- 不把通信节点能力接回链 RPC 或本机 RPC。
- 通信节点只服务自己的手机和钱包聊天号，不做公共中继。
- 本轮可以保留后续 P2P IM 通道占位，但不得继续通过 RPC 发送、同步或发布 KeyPackage。

## 验收标准

- 通信节点二维码不再包含旧 RPC URL 字段。
- wuminapp 扫码配对不再保存或连接 RPC。
- 节点正式 RPC module 不再注册 IM 手机连接接口。
- 设置页不再显示通信节点 RPC 卡片。
- 区块链软件设置页通信节点二维码为固定节点信息二维码，不再显示有效期或刷新二维码。
- 相关测试与残留扫描通过。

## 实施记录

- 已删除公民端配对配置中的旧 RPC URL 字段、旧偏好键保存逻辑和扫码后立即授权当前通信账户的 RPC 调用。
- 已删除通信节点二维码 body 中的旧 RPC URL、临时随机值、创建时间和过期时间字段，并将 `QrKind.imNodePairing` 改为固定码。
- 已删除正式 IM 手机连接 RPC 注册入口、旧运行态开关和旧 IM RPC 模块；`core/rpc.rs` 不再注册 IM RPC。
- 已将 `ImPrivateNodeTransport` 中的 HTTP JSON-RPC 客户端删除；未接入专用 P2P 通道前，发送/同步明确失败，不再退回 RPC。
- 已将桌面通信节点设置页压缩为状态标签、PeerId/端点摘要、小二维码缩略图和点击放大弹窗；开启/关闭使用简单二次确认。
- 已将公民“设置通信节点”页面和“扫码签名”按钮的扫描图标改为 `assets/icons/scan-line.svg`，删除通信节点详情中的 RPC 行。
- 已同步 `IM_TECHNICAL.md`、`NODE_TECHNICAL.md`、`unified-protocols.md`、`unified-naming.md`。

## 验证记录

- `flutter test --concurrency=1 test/qr/im_node_pairing_body_test.dart test/im/im_node_settings_page_test.dart`：通过。
- `flutter analyze --no-fatal-warnings --no-fatal-infos`：通过。
- `cd citizenchain/node/frontend && npm run build`：通过。
- `cargo check --manifest-path citizenchain/node/Cargo.toml`：通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml settings::communication_node -- --nocapture`：通过。
- `git diff --check`：通过。
- 桌面前端浏览器 mock 验收：已启动 Vite dev server；in-app 浏览器只读执行环境禁止注入 `__TAURI_INTERNALS__`，无法在普通浏览器中模拟 Tauri invoke，未为验收增加临时代码。

## 残留扫描

- 通信节点代码路径未发现二维码 RPC 字段、公民端 RPC URL 字段或旧手机节点 RPC 方法残留。
- 旧 RPC URL 仅剩非通信节点模块文档和测试中确认二维码不含该字段的断言。
