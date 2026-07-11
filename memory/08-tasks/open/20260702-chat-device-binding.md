# Chat 钱包绑定签名并入 QR_V1

- 日期:2026-07-02
- 状态:完成
- 范围:CitizenApp Chat、CitizenWallet 离线签名、citizenchain node Chat、runtime primitives 签名常量、协议文档

## 目标

- 删除 Chat 钱包绑定旧字符串签名域。
- Chat 钱包绑定统一使用 `QR_V1/k=1` 签名请求、`a=8` 动作码和 `signing_message(OP_SIGN_CHAT_DEVICE_BIND, payload)`。
- CitizenWallet 必须能独立解码 Chat 绑定 payload 并展示可核对字段；解码失败或动作码不匹配时拒签。
- citizenchain node 必须对 Chat 绑定请求执行 sr25519 验签，不得只检查签名非空。

## 验收

- 已完成:Chat 钱包绑定旧字符串域在代码和协议文档中无生产残留。
- 已完成:Rust 与 Dart 的 Chat 绑定 payload、op_tag、签名字节规则一致。
- 已完成:node Chat 绑定单元测试覆盖有效签名和伪造签名拒绝。
- 已完成:CitizenWallet 解码与签名服务测试覆盖 Chat 钱包绑定 action。
- 已完成:相关 Rust、Flutter 测试和 analyze/check 已通过。
