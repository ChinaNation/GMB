# IM 私人通信全节点持久化 mailbox 与公民端传输接入

## 状态

done

## 任务需求

把当前 IM 私人通信全节点的内存态 mailbox 改造成可落盘、可重启恢复的 owner-only mailbox，并把公民 `wuminapp/lib/im/transport/ImPrivateNodeTransport` 从“只入队占位”推进到可调用自己私人通信全节点 JSON-RPC 的正式传输骨架。

## 边界

- 不上链，不碰 SFID，不把 IM 身份或消息目录写入链上。
- 通信全节点只服务自己的 owner 钱包聊天账户，不互为节点、不做公共中继、不替第三方存消息。
- 本阶段不接 OpenMLS、不生成 Protobuf、不改 Isar schema、不做近场原生实现。
- 持久化 mailbox 先使用节点 `base-path/im/mailbox.json` 单文件快照，满足重启恢复和真实 smoke；后续可替换为专属 RocksDB/column family。
- 公民端传输只负责调用自己节点 RPC；聊天窗口 UI、真实设备签名、OpenMLS 密钥管理后续任务继续拆分。

## 预计修改目录

- `citizenchain/node/src/im/`：把内存态 mailbox 改为可落盘、可重启恢复的 owner-only mailbox；补正式 `im_*` RPC；涉及代码和中文注释。
- `citizenchain/node/src/core/`：节点启动时初始化 IM mailbox 存储路径；涉及代码。
- `citizenchain/scripts/`：升级双节点 smoke，覆盖 B 节点重启后 pending 不丢、ack 后重启不重复出现；涉及脚本。
- `wuminapp/lib/im/transport/`：把 `ImPrivateNodeTransport` 从占位 queued 改为可调用自家节点 RPC 的传输层；涉及代码。
- `memory/05-modules/wuminapp/im/`：更新 IM 技术文档，记录持久化 mailbox 和公民端传输路径；涉及文档。
- `memory/05-modules/citizenchain/node/`：更新节点文档，记录 IM 落盘、TTL/容量、重启恢复验收；涉及文档。
- `memory/07-ai/`：补充正式 `im_*` RPC 命名和验收脚本登记；涉及文档。

## 实施记录

- 更新 `citizenchain/node/src/im/mailbox.rs`，把 mailbox 从纯内存结构升级为 `base-path/im/mailbox.json` 单文件快照，保存 owner、设备绑定、pending envelope 和 ack tombstone，并补容量、TTL、重复 ack 防护。
- 更新 `citizenchain/node/src/im/commands.rs` 与 `citizenchain/node/src/core/service.rs`，节点启动时初始化 IM mailbox 存储路径，Tauri 命令、RPC 和 `/gmb/im/1` incoming handler 共享同一份持久化 mailbox。
- 更新 `citizenchain/node/src/im/rpc.rs` 与 `citizenchain/node/src/core/rpc.rs`，新增 `GMB_IM_OWNER_RPC=1` 条件注册的正式 `im_*` owner RPC，同时保留 `GMB_IM_DEBUG_RPC=1` debug RPC。
- 更新 `wuminapp/lib/im/transport/im_private_node_transport.dart`，新增 owner RPC 客户端、设备登记、直连投递、待收拉取和 ack 方法，`sendEncryptedEnvelope` 不再返回占位 queued。
- 升级 `citizenchain/scripts/im-two-node-smoke.sh`，改用正式 `im_*` RPC，并覆盖 B 节点重启恢复 pending、ack 后重启不重复、第三方 mailbox 拒绝和 ack 后重复投递不入队。
- 更新 IM、node、统一协议和统一命名文档，清理“持久化 mailbox / 正式传输未接入”的旧口径。

## 验收记录

- `cargo test -p node im::`：通过，14 个 IM 单测全过。
- `cargo check -p node`：通过。
- `flutter analyze`：通过。
- `flutter test --concurrency=1`：通过。
- `citizenchain/scripts/im-two-node-smoke.sh`：通过，两个真实 headless 节点完成 A→B 密文投递、B 重启恢复 pending、B owner 拉取、ack、ack 后重启不重复、第三方 mailbox 拒绝和 ack 后重复投递不入队。
