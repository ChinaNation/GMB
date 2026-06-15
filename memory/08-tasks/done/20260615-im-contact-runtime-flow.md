# 20260615 IM 联系人包与真实远程收发联调

## 状态

done

## 任务需求

把已经完成的 IM 底座接成用户可用的远程文本消息闭环：公民“信息”Tab 支持 IM 联系人包导入/导出，联系人保存钱包聊天账户、IM 设备和私人通信全节点端点；聊天详情页默认接入 OpenMLS、Isar 消息库和私人通信全节点 RPC，实现发送文本、拉取 pending、解密入库和 ack。

## 边界

- 通信全节点只服务 owner 自己，不互为中继、不替第三方存消息、不做公共 DHT、公共 rendezvous 或 Relay。
- 钱包账户作为聊天账户和公民币收付款账户；钱包私钥只用于绑定证明和后续链上转账签名，不作为 IM 加密密钥。
- 本任务不做近场原生实现、不做附件、不做聊天窗口真实公民币转账上链。
- 不修改旧 `user_contact` 二维码协议；IM 联系人包在 IM 模块内独立解析和生成。

## 预计修改目录

- `wuminapp/lib/im/`：新增 IM 运行态编排、联系人包导入导出和信息 Tab 默认收发链路；涉及代码、中文注释和残留清理。
- `wuminapp/lib/im/crypto/`：复用 OpenMLS native 和设备身份边界生成 KeyPackage；涉及代码。
- `wuminapp/lib/im/transport/`：复用私人通信全节点 owner RPC 发送、拉取、ack 和 KeyPackage 接口；涉及代码。
- `wuminapp/test/im/`：新增联系人包和运行态收发联调测试；涉及测试代码。
- `memory/05-modules/wuminapp/im/`：更新 IM 技术文档，记录产品化远程收发入口；涉及文档。
- `memory/07-ai/`：更新统一协议和命名登记；涉及文档与残留清理。
- `memory/08-tasks/`：归档已完成 IM 任务卡，并记录本任务完成状态；涉及任务卡文档。

## 验收

- `flutter analyze`
- `flutter test --concurrency=1 test/im/im_contact_bundle_test.dart test/im/im_mls_boundary_test.dart test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart`
- `flutter test --concurrency=1 test/im/im_tab_page_test.dart test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart`
- `git diff --check`

## 完成记录

- 已新增 `wuminapp/lib/im/im_contact_bundle.dart`，定义 `GMB_IM_CONTACT_V1` IM 联系人包，独立于钱包 `user_contact` 二维码，支持扫码导入和“我的 IM 码”展示。
- 已新增 `wuminapp/lib/im/im_runtime.dart`，把钱包聊天账户、OpenMLS native、Isar、本机私人通信全节点配置、KeyPackage 发布、直连投递和 pending 同步串成产品运行态。
- 已把 `wuminapp/lib/im/im_tab_page.dart` 接入扫码、我的码、节点配置和默认真实发送/同步回调；`wuminapp/lib/main.dart` 默认向信息 Tab 注入 `ImRuntime`。
- 已扩展 OpenMLS native KeyPackage 返回 `device_public_key_hex`，并重编 Android `arm64-v8a` / `armeabi-v7a` `libsmoldot.so`。
- 已更新 IM 技术文档、统一协议和统一命名登记，并把已完成但残留在 `open/` 的 IM 任务卡归档到 `done/`。

## 验收记录

- `flutter analyze`：通过。
- `flutter test --concurrency=1 test/im/im_contact_bundle_test.dart test/im/im_mls_boundary_test.dart test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_tab_page_test.dart test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart`：通过。
- `cargo test`（`wuminapp/rust`）：通过。
- `./scripts/build-smoldot-native.sh android`（`wuminapp`）：通过。
