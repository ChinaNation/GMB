# IM 节点间直连投递 Spike

## 状态

done

## 任务需求

把 `/gmb/im/1` 从本机 mailbox 调试命令推进到 `sc-network` request-response 协议接入 Spike，验证区块链节点软件中的私人通信全节点能否承载“显式 PeerId + multiaddr 直连投递密文信封”的网络层。

## 边界

- 只允许显式联系人端点直连，不使用公共 DHT、公共 rendezvous 或 Relay。
- 通信全节点只服务 owner 自己的 mailbox。
- 不做第三方中继，不替别人存消息。
- 不接 OpenMLS、Protobuf、Isar、近场和聊天窗口公民币转账。
- Wire 编码 Spike 阶段用 JSON bytes，后续 `proto/im/im_envelope.proto` 固化后替换。

## 预计修改目录

- `citizenchain/node/src/im/`：新增 `/gmb/im/1` 协议配置、incoming handler、直连投递请求模型；涉及代码和中文注释。
- `citizenchain/node/src/core/`：在节点 service 网络配置中注册 IM request-response 协议；涉及代码。
- `citizenchain/node/src/desktop/`：注册本地调试命令；涉及代码。
- `memory/05-modules/wuminapp/im/`：更新 IM 技术文档，记录 sc-network Spike 结论；涉及文档。
- `memory/05-modules/citizenchain/node/`：更新节点技术文档；涉及文档。
- `memory/07-ai/`：补充协议和命名登记；涉及文档。

## 实施记录

- 新增 `citizenchain/node/src/im/network.rs`，实现 `/gmb/im/1` request-response 协议配置、JSON wire 编解码、incoming handler、registered network handle 和直连投递 helper。
- 新增 `citizenchain/node/src/im/direct.rs`，实现 `ImDirectDeliveryRequest` 和 `ImDirectNetworkCapability`，固定显式 `PeerId + multiaddr`、owner-only mailbox、不走公共发现/中继的边界。
- 在 `citizenchain/node/src/core/service.rs` 中注册 `/gmb/im/1` request-response 协议，启动 incoming handler，并把当前 `NetworkService` 注册给 IM 调试命令使用。
- 在 `citizenchain/node/src/im/commands.rs` 和 `citizenchain/node/src/desktop/mod.rs` 中新增 `get_im_direct_network_capability`、`validate_im_direct_delivery_request`、`submit_im_direct_encrypted_envelope`。
- 在 `citizenchain/node/Cargo.toml` 中显式加入 `async-channel = "1.9"`，用于接收 sc-network request-response inbound queue。
- 更新 IM、node、wuminapp 架构、统一协议和统一命名文档。

## 验收记录

- `cargo test -p node im::`：通过，11 个 IM 单测全过。
- `cargo check -p node`：通过。
- `CITIZENCHAIN_HEADLESS=1 cargo run -p node -- --help`：通过，节点二进制 CLI 入口可启动并输出帮助。
- `git diff --check`：通过。

## 当前结论

- 可行：现有 `sc-network` 可以注册 `/gmb/im/1` request-response 协议，可以处理 incoming request，也可以通过 `NetworkService::request` 做 outbound。
- 已补齐关键直连语义：outbound 前先把联系人包中的显式 `PeerId + multiaddr` 写入 sc-network 地址簿，再用 `IfDisconnected::TryConnect` 发请求。
- 未完成：尚未用两个真实节点做运行态互投；下一步必须启动两个不同 `base-path`、不同端口、不同 owner 的节点，通过 `submit_im_direct_encrypted_envelope` 验证 A→B 密文进入 B 的 owner-only mailbox。
