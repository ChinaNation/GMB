# 20260615 IM 聊天 UI 与联系人收发入口

## 状态

done

## 任务需求

在公民“信息”Tab 中接入现成聊天 UI，新增联系人本地模型，并把会话列表、聊天详情、文本发送和手动同步入口接到已经完成的 `ImIsarStore` 边界；真实 `ImMessageFlow`、`ImPrivateNodeTransport` 发送链路等待本机通信全节点绑定、设备身份和 KeyPackage 数据齐备后注入，当前 UI 不伪造发送成功。

## 预计修改目录

- `wuminapp/pubspec.yaml`：新增 `flutter_chat_ui`、`flutter_chat_core` 依赖；涉及配置。
- `wuminapp/lib/isar/`：新增 `ImContactEntity` 并重新生成统一 Isar schema；涉及代码和生成物。
- `wuminapp/lib/im/`：新增聊天详情页、会话列表真实数据接入、UI 控制器和消息映射；涉及代码和中文注释。
- `wuminapp/lib/im/storage/`：扩展 `ImIsarStore`，新增联系人读写和消息查询方法；涉及代码。
- `wuminapp/lib/im/transport/`：本任务不改传输层；聊天页仅预留发送/同步回调边界，真实私人通信全节点链路留到本机绑定数据齐备后接入。
- `wuminapp/test/im/`：新增联系人存储、聊天 UI 适配和信息 Tab widget 测试；涉及测试代码。
- `memory/05-modules/wuminapp/im/`：更新公民 IM 技术文档；涉及文档。
- `memory/07-ai/`：更新命名登记；涉及文档。

## 边界

- 本任务只接文本聊天 UI 和手动同步，不做近场原生、后台常驻同步、图片/附件、语音、已读回执。
- 不接 Firebase、Stream、SFID、链上目录或中心服务器。
- 钱包账户继续只作为聊天账户和公民币收付款账户，不作为 OpenMLS 加密密钥。
- 聊天窗口公民币转账入口只保留后续扩展位置，本任务不实现真实转账签名。

## 执行记录

- 已新增 `flutter_chat_ui`、`flutter_chat_core` 依赖。
- 已新增 `ImContactEntity`、`ImContactRecord` 和联系人读写方法，schema 升级到 6。
- 已新增 `ImChatPage`，用现成聊天 UI 展示本地消息，并通过回调接入后续真实发送/同步链路。
- 已新增 `im_chat_ui_adapter.dart`，把 `ImStoredMessage` 映射为 `flutter_chat_core.Message`。
- 已改造 `ImTabPage`，使用活跃钱包账户作为聊天账户，联系人和会话来自本地 store，新增联系人写入 store。
- 已更新 `main.dart` 中的信息 Tab 构造方式。
- 已补充联系人 store、聊天 UI adapter 和信息 Tab widget 测试。

## 验收

- 通过：`flutter pub get`
- 通过：`dart run build_runner build --delete-conflicting-outputs`
- 通过：`flutter analyze`
- 通过：`flutter test --concurrency=1 test/im/im_contact_store_test.dart test/im/im_chat_ui_adapter_test.dart test/im/im_tab_page_test.dart`
- 通过：`flutter test --concurrency=1 test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart`
- 通过：`git diff --check`
- 未计为通过：`flutter run -d 3C071JEKB09000 --debug` 已在 Pixel 8a 构建、安装并启动；真机截图停在空白页面，退出时触发既有 smoldot FFI `Callback invoked after it has been deleted` SIGABRT，页面级 smoke 需单独修复启动白屏/FFI 生命周期后复验。
