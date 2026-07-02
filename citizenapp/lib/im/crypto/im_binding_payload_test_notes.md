# IM 绑定 Payload 测试说明

本文件记录当前 Spike 阶段的测试边界。

- `ImBindingPayload.signingPayloadBytes()` 必须与 `citizenchain/node/src/im/binding.rs` 的 `RegisterImDeviceRequest::signing_payload()` 保持一致。
- 当前阶段已接入 OpenMLS native KeyPackage、MLS 会话持久化和 `citizenapp/im/proto/im_envelope.proto` Dart 生成物；绑定签名域已统一为 `QR_V1/k=1/a=8` + `OP_SIGN_IM_WALLET_BINDING`。
- 后续固化钱包签名 fixture 后，必须用同一个 fixture 校验 Dart `ImBindingPayload.signingPayloadBytes()` 和 Rust `RegisterImDeviceRequest::signing_payload()`。
