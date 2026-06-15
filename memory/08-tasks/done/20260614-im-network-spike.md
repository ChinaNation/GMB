# IM 私人通信全节点网络 Spike

## 状态

done

## 任务需求

在区块链节点软件和公民 wuminapp 中继续完善 `im/` 目录，先落地 P2P IM 的私人通信全节点网络 Spike 骨架，验证“全节点只服务自己”的边界：

- 节点端新增 IM 端点、设备绑定、密文信封和私人 mailbox 的最小实现。
- 公民端新增私人通信全节点端点与绑定 payload 模型。
- 协议和技术文档同步更新。
- 不接入 OpenMLS、Isar schema、真实 sc-network 拨号、近场通信和真实链上转账。

## 边界

- 通信全节点只保存 owner 自己的密文收件箱。
- 不做第三方 Relay。
- 不做公共 DHT / rendezvous。
- 不替别人保存消息。
- 钱包账户是聊天账户和转账账户；钱包私钥只用于绑定证明和链上转账签名，不做 IM 加密密钥。
- IPv4、IPv6、用户自有 `dns4` / `dnsaddr` 都是可登记端点。

## 预计修改目录

- `citizenchain/node/src/im/`：新增私人节点端点、绑定、密文信封、内存 mailbox 和 Tauri 调试命令；涉及代码和中文注释。
- `wuminapp/lib/im/`：新增私人节点传输模型和绑定 payload；涉及代码和中文注释。
- `memory/05-modules/wuminapp/im/`：更新公民 IM 技术文档；涉及文档和残留口径清理。
- `memory/05-modules/citizenchain/node/`：更新节点 IM 边界文档；涉及文档和残留口径清理。
- `memory/07-ai/`：更新统一协议登记；涉及文档和协议字段登记。

## 实施记录

- 新增 `citizenchain/node/src/im/endpoint.rs`，登记 IM 私人节点 endpoint 模型和 multiaddr 边界校验。
- 新增 `citizenchain/node/src/im/envelope.rs`，登记加密信封提交请求、ack 和状态模型。
- 新增 `citizenchain/node/src/im/binding.rs`，登记钱包聊天账户到 IM 设备和私人节点的绑定载荷。
- 新增 `citizenchain/node/src/im/mailbox.rs`，提供只服务 owner 的内存态 mailbox，拒绝第三方收件箱。
- 扩展 `citizenchain/node/src/im/commands.rs` 和桌面 Tauri 命令注册，用于本地调试绑定、提交、拉取和确认密文信封。
- 新增 `wuminapp/lib/im/crypto/im_binding_payload.dart` 和 `wuminapp/lib/im/transport/im_private_node_transport.dart`，给后续扫码绑定、节点连接和真实传输留稳定模型。
- 更新 IM、node 和统一协议文档。

## 验收记录

- `cargo test -p node im::`：通过，8 个 IM 单测全过。
- `cargo check -p node`：通过。
- `dart format wuminapp/lib/im/crypto/im_binding_payload.dart wuminapp/lib/im/transport/im_private_node_transport.dart`：通过。
- `flutter analyze`：通过。
- `flutter test --concurrency=1`：通过。
- `CITIZENCHAIN_HEADLESS=1 cargo run -p node -- --help`：通过，节点二进制 CLI 入口可启动并输出帮助。
- `git diff --check`：通过。

补充：未加 `CITIZENCHAIN_HEADLESS=1` 的 `cargo run -p node -- --help` 在 macOS 有显示环境下会进入桌面模式，并因本机已有节点占用 RocksDB `LOCK` 退出；该结果属于本机运行态环境锁，不是本次 IM 代码编译或协议边界失败。
