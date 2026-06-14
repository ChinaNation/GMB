# IM 绑定 Payload 测试说明

本文件记录当前 Spike 阶段的测试边界。

- `ImBindingPayload.canonicalPayload()` 必须与 `citizenchain/node/src/im/binding.rs` 的 `RegisterImDeviceRequest::canonical_payload()` 保持一致。
- 当前阶段尚未接入真实钱包签名、OpenMLS 设备密钥和 Protobuf 生成物，因此不新增 Dart 单测文件，避免把临时字符串载荷误固化为最终协议。
- 下一阶段固化 `proto/im/im_envelope.proto` 后，必须新增 Dart/Rust round-trip fixture，并用同一个 fixture 校验签名 payload。
