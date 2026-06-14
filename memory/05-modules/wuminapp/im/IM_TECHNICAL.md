# wuminapp P2P IM 技术架构

## 1. 模块定位

`wuminapp` 的 P2P IM 是独立通信功能，不属于钱包、治理、交易或身份实名模块，但用户可见聊天账户使用 wuminapp 钱包账户。

底部 Tab 目标顺序：

```text
公民 / 多签 / 信息 / 交易 / 我的
```

“信息”Tab 是唯一入口。用户不选择“近场通信 / 通信全节点”模式；远程通信全节点消息和近场消息都在“信息”Tab 集中显示、统一发送和统一搜索。

核心目标：

- 远程通信：手机连接自己的私人通信全节点，自己的节点再直连对方私人通信全节点投递密文。
- 近场通信：手机与手机直接通信，用于游行、聚会、演出、体育场等手机网络拥堵或不可用场景。
- 钱包聊天账户：钱包账户是聊天账户和公民币收付款账户。
- 私人节点边界：通信全节点只服务自己，不互为中继，不替别人存消息。
- 完全去中心化：不上链、不进 SFID、不使用中心化消息服务器。

## 2. 总体架构

```text
我手机（公民）
  -> 我家通信全节点（只服务我，只存我的密文队列）
  -> 你家通信全节点（只服务你，只收发给你的密文）
  -> 你手机（公民）

近场旁路：
我手机 <-> 你手机
```

手机端：

```text
wuminapp/lib/im/
├── 信息 Tab / 会话列表 / 聊天详情
├── Protobuf ImEnvelope 外层协议
├── OpenMLS 标准 wire bytes 内层消息
├── 钱包账户绑定 IM 设备密钥
├── 聊天窗口发送公民币 payment_notice
├── 本地消息库、发送队列、附件缓存
└── TransportRouter 统一路由远程节点和近场链路
```

节点端：

```text
citizenchain/node/src/im/
├── 私人密文 mailbox
├── KeyPackage 池
├── 设备授权
├── /gmb/im/1 P2P 协议边界
└── IPv4 / IPv6 / dns4 / dnsaddr 端点
```

## 3. 身份模型

IM 身份分层：

```text
聊天账户 = 钱包账户 / SS58 地址 / AccountId
IM 设备身份 = 独立 OpenMLS 设备密钥
转账账户 = 同一个钱包账户
```

钱包私钥只用于：

- 证明某个 IM 设备属于该钱包账户。
- 在聊天窗口中发送公民币时签链上转账。

钱包私钥禁止用于：

- OpenMLS 消息加密。
- 每条聊天消息签名。
- 通信全节点存储。
- 节点间投递鉴权。

绑定凭证字段：

```text
wallet_account
im_device_id
im_device_pubkeys
communication_node_peer_id
node_endpoints
expires_at
nonce
wallet_signature
```

当前 Spike 固定绑定签名 payload：

```text
GMB_IM_WALLET_BINDING_V1|wallet_account|im_device_id|im_device_pubkey|node_peer_id|node_endpoints|expires_at_millis|nonce
```

`wallet_signature` 必须覆盖上述 payload。节点端当前只做字段和边界校验，真实钱包验签会在签名协议 fixture 固化后接入。

## 4. E2EE 与协议编码

端到端加密主选 OpenMLS。

- v1 使用经典 MLS 密码套件。
- `suite_version` 保留 hybrid / PQC 迁移字段。
- v1 不宣称抗量子；MLS 密码敏捷性只作为未来切换空间。
- KeyPackage 以设备为单位发布到自己的通信全节点。
- KeyPackage 必须具备 TTL、一次性消费或租约消费、防重放和撤销清理。

外层协议使用 Protobuf：

```text
GMB_IM_V1 / ImEnvelope
├── protocol_version
├── envelope_id
├── conversation_id
├── sender_chat_account
├── recipient_chat_account
├── sender_device_id
├── mls_wire_message
├── encrypted_metadata
├── attachment_manifest_hash
├── chunk_refs
├── created_at
├── ttl
└── ack_policy
```

联系人包：

```text
ImContactBundle
├── chat_account
├── display_name
├── node_peer_id
├── node_endpoints
├── device_public_keys
├── keypackage_endpoint
├── safety_code
├── expires_at
└── wallet_signature
```

禁止把 SFID 号码、实名信息、身份档案字段写入 IM 协议。

## 5. 通信全节点

通信全节点能力放在 `citizenchain/node/src/im/`。

通信全节点只允许：

- 保存自己的密文 mailbox。
- 保存自己的 KeyPackage 池。
- 接收别人投递给自己的密文消息。
- 给自己的手机提供拉取、确认、删除。
- 管理自己的设备授权。
- 向对方私人通信全节点直连投递密文。

通信全节点禁止：

- 不给第三方做 Relay。
- 不做公共 rendezvous。
- 不做公共 DHT 基础设施。
- 不替别人存消息。
- 不解密消息。
- 不处理钱包、治理、交易业务。

节点协议名：

```text
/gmb/im/1
```

真实跨节点投递前必须先做 `sc-network/libp2p` 能力 Spike，核实 request-response、notification、端点签名和节点可达性 API。

当前网络 Spike 已落地的本地调试命令：

```text
get_im_private_node_policy
validate_im_node_endpoint
register_im_owner_device
submit_im_encrypted_envelope
fetch_im_pending_envelopes
ack_im_envelope
get_im_direct_network_capability
validate_im_direct_delivery_request
submit_im_direct_encrypted_envelope
```

当前 sc-network Spike 结论：

- 已注册 `/gmb/im/1` request-response 协议。
- 已启动 incoming handler，收到 `SubmitEnvelope` 后复用 owner-only mailbox 校验。
- 已提供 `submit_im_direct_encrypted_envelope` Tauri 调试命令。
- 已新增 `GMB_IM_DEBUG_RPC=1` 条件 debug RPC，供 headless 双节点运行态验收调用；正式节点默认不注册。
- 已新增 `citizenchain/scripts/im-two-node-smoke.sh`，使用两个临时 `base-path` 启动 A/B 节点，完成 A→B 密文投递、B owner 拉取、ack 和第三方 mailbox 拒绝验证。
- outbound helper 会把联系人包里的显式 `PeerId + multiaddr` 写入 sc-network 地址簿，再用 `NetworkService::request(..., IfDisconnected::TryConnect)` 发起请求。
- 不使用公共 DHT、公共 rendezvous 或 Relay。
- 本机双节点真实运行态 smoke 已通过；后续产品化重点转为持久化 mailbox、OpenMLS wire bytes、Protobuf schema 和 wuminapp 到私人节点的正式传输。

## 6. 可达性与 IPv6

IM 支持 IPv4、IPv6 和用户自有域名，不做 IPv6-only。

支持 multiaddr：

```text
/ip4/<addr>/tcp/<port>/wss/p2p/<peer_id>
/ip6/<addr>/tcp/<port>/wss/p2p/<peer_id>
/dns4/<domain>/tcp/<port>/wss/p2p/<peer_id>
/dnsaddr/<domain>/p2p/<peer_id>
```

优先级：

1. 局域网直连。
2. IPv6 直连。
3. IPv4 端口映射 / UPnP / NAT-PMP。
4. 用户自有域名。
5. 用户自己控制的公网入口。

如果不可达，消息进入发送队列等待重试。禁止借别人通信全节点中继。

## 7. 聊天窗口发送公民币

聊天详情页内提供“发送公民币”入口。

流程：

```text
点击发送公民币
-> 默认填入对方聊天账户的钱包地址
-> 用户输入金额
-> 显示链上转账确认
-> 钱包账户签名
-> 通过 smoldot 提交链上交易
-> 聊天中发送加密 payment_notice
-> 双方各自查链确认状态
```

`payment_notice` 只是聊天提示，不是到账真相。到账真相必须以链上交易和余额查询为准。

## 8. 近场通信

近场不经过通信全节点，仍传输同一套密文 `ImEnvelope`。

Android：

- 第一优先 Nearby Connections。
- 无 Google Play Services 场景评估 Wi-Fi Direct / Wi-Fi Aware / BLE。

iOS：

- Multipeer Connectivity。
- CoreBluetooth / BLE 用于跨平台控制通道。

Android 与 iOS：

- BLE GATT 做发现、控制、短消息。
- 大附件不承诺 BLE 高速传输，需单独真机验证。

## 9. 存储

wuminapp 本地：

- Isar 保存会话、消息索引、联系人、发送队列、附件索引。
- 明文消息建议本地加密后入库。
- 本地加密密钥放手机安全存储。
- 落 Isar schema 前必须先沟通。

通信全节点：

- 只保存密文 envelope。
- 只保存密文附件分片。
- 保存 KeyPackage 池。
- 保存设备授权和 TTL 元数据。
- 容量、TTL、附件大小必须有限额。

## 10. 当前代码状态

- `wuminapp/lib/im/` 已新增基础模型和“信息”Tab 壳。
- `wuminapp/lib/im/crypto/im_binding_payload.dart` 已新增钱包聊天账户到 IM 设备、私人通信全节点的绑定 payload。
- `wuminapp/lib/im/transport/im_private_node_transport.dart` 已新增私人通信全节点端点和传输骨架，当前只入队不做真实网络发送。
- `wuminapp/android/im/` 已新增 Android 近场模块占位文档。
- `wuminapp/ios/im/` 已新增 iOS 近场模块占位文档。
- `citizenchain/node/src/im/` 已新增私人通信全节点策略、端点校验、设备绑定、密文信封、内存 mailbox、`/gmb/im/1` request-response 接入、incoming handler、显式端点直连投递 helper、Tauri 调试命令和条件 debug RPC。
- `citizenchain/scripts/im-two-node-smoke.sh` 已新增本机双节点真实运行态验收脚本，验证 A→B 直连投递、owner 拉取/ack 和第三方 mailbox 拒绝。
- 真实 OpenMLS、Protobuf 生成、Isar schema、持久化 mailbox、wuminapp 到私人节点正式传输和近场原生能力尚未接入，需按任务卡继续拆分。

## 11. 预计修改目录

- `wuminapp/lib/im/`：公民 IM 统一消息层、会话、联系人、消息状态、发送队列；涉及代码。
- `wuminapp/lib/im/crypto/`：OpenMLS、设备密钥、KeyPackage、安全码、钱包账户绑定；涉及代码。
- `wuminapp/lib/im/payment/`：聊天窗口发送公民币、`payment_notice`、链上确认状态；涉及代码。
- `wuminapp/lib/im/storage/`：Isar 消息库、联系人库、附件缓存；涉及代码，schema 前需确认。
- `wuminapp/lib/im/transport/`：远程节点传输、近场传输、自动路由、去重；涉及代码。
- `wuminapp/android/im/`：Android Nearby、Wi-Fi fallback、BLE；涉及代码。
- `wuminapp/ios/im/`：iOS Multipeer、BLE；涉及代码。
- `citizenchain/node/src/im/`：私人通信全节点 mailbox、KeyPackage、设备授权、IM P2P 协议；涉及代码。
- `memory/04-decisions/`：IM 架构 ADR；涉及文档。
- `memory/05-modules/wuminapp/im/`：wuminapp IM 技术文档；涉及文档。
- `memory/05-modules/citizenchain/node/`：通信全节点边界文档；涉及文档。
- `memory/07-ai/`：`GMB_IM_V1` 协议和命名登记；涉及文档。
