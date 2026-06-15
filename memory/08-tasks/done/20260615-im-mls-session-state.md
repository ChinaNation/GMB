# 20260615 IM MLS 会话持久化与真实加解密

## 状态

done

## 任务需求

把公民 IM 已跑通的 OpenMLS native smoke 升级为可持久化的会话状态机：`NativeImMlsCrypto.encrypt` 能使用对方 KeyPackage 创建首次 MLS 会话并输出 Welcome + application wire bytes，`decrypt` 能处理 Welcome、解密 application，并在 App 重启后继续使用同一会话。

## 预计修改目录

- `wuminapp/rust/`：扩展现有 `libsmoldot` C ABI，增加 MLS 会话创建、Welcome 处理、application 加解密和 OpenMLS provider storage 持久化；涉及代码和中文注释。
- `wuminapp/lib/im/crypto/`：新增会话模型和状态存储边界，接入 `NativeImMlsCrypto.encrypt/decrypt`；涉及代码和残留清理。
- `wuminapp/test/im/`：新增 MLS 会话模型、本地 pending 队列和 native 会话重启恢复测试；涉及测试代码。
- `memory/05-modules/wuminapp/im/`：更新公民 IM 技术文档，记录 MLS 会话持久化状态；涉及文档。
- `memory/07-ai/`：更新协议和命名登记，记录 MLS 会话、Welcome/application wire message 和 state store 命名；涉及文档。

## 边界

- 钱包账户继续只作为聊天账户和公民币收付款账户，不作为 OpenMLS 加密密钥。
- OpenMLS provider storage 由上游 `openmls_memory_storage` persistence 保存，不自研 MLS 状态格式。
- 节点仍只保存密文 `ImEnvelope`，不解密、不互为节点、不做 Relay。
- 本任务不改近场原生能力、不改 Isar schema、不改链上交易。

## 验收

- `cargo test`
- `cargo build --release`
- `flutter analyze`
- `flutter test --concurrency=1 test/im/im_mls_session_test.dart test/im/im_mls_native_session_test.dart`
- `flutter test --concurrency=1 test/im/im_envelope_proto_test.dart test/im/im_mls_native_test.dart`
- `git diff --check`

## 完成记录

- 已在 `wuminapp/rust/src/im_mls.rs` 接入持久化 OpenMLS provider storage，新增 `gmb_im_mls_encrypt_json` / `gmb_im_mls_decrypt_json`。
- 已在 `wuminapp/lib/im/crypto/` 新增 MLS wire message、出入站消息、状态目录和 pending inbound 队列模型。
- 已把 `NativeImMlsCrypto.encrypt/decrypt` 接到真实 native OpenMLS：首次发送生成 Welcome + application，已有会话只生成 application，接收端可处理 Welcome 并在重启后解密后续消息。
- 已更新 IM 技术文档、统一协议登记和统一命名登记。

## 验收记录

- `cargo test`（`wuminapp/rust`）通过。
- `cargo build --release`（`wuminapp/rust`）通过。
- `flutter test --concurrency=1 test/im/im_mls_session_test.dart test/im/im_mls_native_session_test.dart` 通过。
- `flutter test --concurrency=1 test/im/im_envelope_proto_test.dart test/im/im_mls_native_test.dart` 通过。
- `flutter analyze` 通过。
