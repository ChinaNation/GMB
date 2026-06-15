# 20260615 IM OpenMLS FFI 与 Protobuf 生成闭环

## 状态

done

## 任务需求

在公民 IM 模块中继续推进真实端到端加密基础：从 `wuminapp/im/proto/im_envelope.proto` 生成 Dart Protobuf 类型，并在现有 `wuminapp/rust` native 库中新增 OpenMLS 最小 FFI 边界，验证 KeyPackage 生成与两方 OpenMLS 加密/解密 round-trip。

## 预计修改目录

- `wuminapp/im/proto/`：继续作为 GMB_IM_V1 Protobuf schema 真源；不移动到仓库根目录。
- `wuminapp/lib/im/proto/`：新增 Dart Protobuf 生成物；只放生成代码，不手写业务逻辑。
- `wuminapp/lib/im/crypto/`：新增 Dart native OpenMLS 调用边界；钱包账户仍只作为聊天账户/收款账户，不作为 IM 密钥来源。
- `wuminapp/rust/`：扩展现有 `libsmoldot` C ABI，新增 `gmb_im_mls_*` OpenMLS FFI 函数；不新增第二套 native 库。
- `wuminapp/test/im/`：新增 Protobuf round-trip 和 native OpenMLS smoke 测试。
- `memory/05-modules/` 与 `memory/07-ai/`：更新技术文档、协议登记和命名登记，清理旧口径残留。

## 边界

- 不复用钱包私钥、SFID 身份或链上账户密钥做 IM 加密密钥。
- 不在 Dart 中自研密码学；OpenMLS 逻辑必须在 Rust 侧调用 OpenMLS 库。
- 本任务只做 OpenMLS FFI 最小闭环，不承诺持久化 MLS group/session 状态。
- Protobuf 真源固定在 `wuminapp/im/proto/im_envelope.proto`。

## 验收

- `cargo test`
- `cargo build --release`
- `flutter analyze`
- `flutter test --concurrency=1 test/im/im_envelope_proto_test.dart test/im/im_mls_native_test.dart`
- `git diff --check`

## 完成记录

- 已从 `wuminapp/im/proto/im_envelope.proto` 生成 Dart Protobuf 类型到 `wuminapp/lib/im/proto/`，并新增 `ImEnvelope` / `ImKeyPackage` round-trip 测试。
- 已在现有 `wuminapp/rust` native 库中新增 `wuminapp/rust/src/im_mls.rs`，通过 OpenMLS 0.8 生成真实 KeyPackage，并执行两方 MLS application message 加解密 smoke。
- 已扩展 `wuminapp/rust/src/lib.rs`、`wuminapp/rust/build.rs` 和 `wuminapp/native/smoldot.h`，把 `gmb_im_mls_create_key_package_json`、`gmb_im_mls_two_party_smoke_json` 暴露到现有 `libsmoldot` C ABI。
- 已新增 `wuminapp/lib/im/crypto/im_mls_native.dart`，Dart 侧可加载现有 native 库调用 OpenMLS KeyPackage 生成和两方 round-trip smoke。
- 已补齐 `protobuf`、`protoc_plugin`、`ffi`、`path`、`fixnum` 依赖登记。
- 已更新 IM 技术文档、统一协议和统一命名登记，清理 OpenMLS native 与 Protobuf 生成链路的旧状态描述。

## 验收记录

- `cargo test`（`wuminapp/rust`）：通过，2 个 OpenMLS native 单测全过。
- `cargo build --release`（`wuminapp/rust`）：通过，生成可供 Dart 测试加载的 release native 库。
- `flutter analyze`（`wuminapp`）：通过。
- `flutter test --concurrency=1 test/im/im_envelope_proto_test.dart test/im/im_mls_native_test.dart`：通过，4 个 IM 测试全过。
- 旧口径残留搜索：通过，未发现根目录 proto、旧 OpenMLS FFI 未接入、旧联系人包字段等残留。
- `git diff --check`：通过。
