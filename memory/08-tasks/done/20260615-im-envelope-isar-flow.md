# 20260615 IM Envelope 正式化与 Isar 消息收发闭环

## 状态

done

## 任务需求

把已经接入的 OpenMLS native 会话能力接入正式 IM 消息管线：补齐 `ImEnvelope` 对 Welcome/application 和 ratchet tree 的承载，新增公民端 Isar 本地消息库，并实现远程节点收发状态机的基础闭环。

## 预计修改目录

- `wuminapp/im/proto/`：更新 GMB_IM_V1 外层 Protobuf 真源，正式登记 MLS 消息类型和 ratchet tree 字段；涉及协议文件。
- `wuminapp/lib/im/proto/`：重新生成 Dart Protobuf 类型；涉及生成代码。
- `wuminapp/lib/isar/`：新增 IM 会话、消息、出站队列、待处理入站消息 collection，并注册到统一 Isar schema；涉及代码和生成物。
- `wuminapp/lib/im/storage/`：新增 IM Isar 仓库，封装会话、消息、出站队列和 pending 入站读写；涉及代码。
- `wuminapp/lib/im/`：新增 IM 消息收发状态机，把 OpenMLS、Envelope、transport 和本地库串成闭环；涉及代码和中文注释。
- `wuminapp/lib/im/crypto/`：补充 MLS wire message 与 Protobuf Envelope 的转换边界；涉及代码。
- `wuminapp/lib/im/transport/`：复用私人通信全节点传输，适配正式 Protobuf envelope bytes；涉及代码。
- `wuminapp/test/im/`：新增协议、Isar 存储和收发状态机测试；涉及测试代码。
- `memory/05-modules/wuminapp/im/`：更新公民 IM 技术文档；涉及文档。
- `memory/07-ai/`：更新协议和命名登记；涉及文档。

## 边界

- 本任务只做远程私人通信全节点链路的消息管线，不做近场原生实现。
- 钱包账户继续只作为聊天账户和公民币收付款账户，不作为 OpenMLS 加密密钥。
- 节点继续只存密文 envelope bytes，不解密、不互为中继。
- 本任务不接聊天窗口真实转账签名，只保留 `payment_notice` 消息类型边界。

## 验收

- `PATH="$HOME/.pub-cache/bin:$PATH" protoc --dart_out=lib/im/proto -I im/proto im/proto/im_envelope.proto`
- `dart run build_runner build --delete-conflicting-outputs`
- `flutter analyze`
- `flutter test --concurrency=1 test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart`
- `flutter test --concurrency=1 test/im/im_mls_session_test.dart test/im/im_mls_native_session_test.dart`
- `git diff --check`

## 完成记录

- 已在 `wuminapp/im/proto/im_envelope.proto` 正式登记 `ImMlsWireMessageKind`、`ImEnvelope.mls_message_kind` 和 `ImEnvelope.ratchet_tree`，并重新生成 Dart Protobuf 类型。
- 已在 `wuminapp/lib/isar/wallet_isar.dart` 新增 IM 会话、消息、出站队列、待处理入站 envelope 四个 collection，并把 schema version 提到 5。
- 已新增 `wuminapp/lib/im/storage/im_isar_store.dart`，封装 IM 本地会话、消息、出站队列和 pending 入站读写。
- 已新增 `wuminapp/lib/im/im_message_flow.dart`，实现文本消息发送、Welcome/application envelope 投递、application 早于 Welcome 时 pending 入库、Welcome 后 pending 重放。
- 已更新 IM 技术文档、统一协议登记和统一命名登记。

## 验收记录

- `PATH="$HOME/.pub-cache/bin:$PATH" protoc --dart_out=lib/im/proto -I im/proto im/proto/im_envelope.proto` 通过。
- `dart run build_runner build --delete-conflicting-outputs` 通过。
- `flutter analyze` 通过。
- `flutter test --concurrency=1 test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart` 通过。
- `flutter test --concurrency=1 test/im/im_mls_session_test.dart test/im/im_mls_native_session_test.dart` 通过。
- `git diff --check` 通过。
