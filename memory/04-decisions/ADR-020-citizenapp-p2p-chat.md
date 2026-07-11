# ADR-020 citizenapp P2P Chat 私人通信全节点方案

- 状态：Superseded
- 决议日期：2026-06-14
- 关联任务卡：`memory/08-tasks/open/20260614-chat-p2p.md`
- 替代文档：`memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`

> 2026-07-05 追加：本 ADR 的“私人通信全节点 / 已删除的节点聊天协议 / 区块链节点承载聊天”路线已被当前 Chat 技术方案替代。当前正式路线只保留 Cloudflare 密文 mailbox 互联网聊天和手机近场聊天；本 ADR 仅作为历史决策记录，不得作为新实现依据。

## 背景

citizenapp 需要在底部“多签”和“交易”之间新增“信息”Tab，提供去中心化 P2P 即时通讯能力，并在聊天窗口内支持向对方钱包账户发送公民币。

本系统的硬边界：

- 通信不上链。
- 通信不依赖 CID。
- 不使用中心化消息服务器。
- 通信全节点是私人节点，只服务自己的手机和自己的收件箱。
- 私人通信全节点之间只做点对点投递，不互为中继。

## 决议

### 1. 钱包账户作为聊天账户

citizenapp 的钱包账户是用户可见聊天账户，也是聊天窗口发送公民币时的收付款账户。

Chat 设备密钥与钱包账户分层：

- 钱包账户用于聊天身份展示、联系人绑定和链上转账签名。
- Chat 设备密钥用于 OpenMLS 端到端加密。
- 钱包私钥不得作为 Chat 消息加密密钥。
- 钱包私钥不得交给通信全节点。

### 2. OpenMLS 作为 E2EE 主路线

Chat 端到端加密采用 OpenMLS。v1 使用经典 MLS 密码套件；协议字段保留 `suite_version`，为后续 hybrid / PQC 套件留出口。

v1 经典套件不宣称抗量子。PQC 迁移仅作为密码敏捷能力预留，不把 ADR-022 的账户签名密钥直接复用为 Chat 消息加密密钥。

### 3. Protobuf 作为 Chat 外层协议

外层统一 envelope 使用 Protobuf；OpenMLS 标准 wire bytes 作为 envelope 内层字段。SCALE 继续只用于链内、runtime 或既有链上载荷边界，不作为移动端 Chat 主协议。

协议名登记为 `GMB_CHAT_V1`，节点 P2P 协议名为 已删除的节点聊天协议。

### 4. 私人通信全节点

通信全节点运行在区块链桌面节点软件中，模块目录为 `citizenchain/node/src/chat/`。

它只允许：

- 保存自己的密文 mailbox。
- 保存自己的 KeyPackage 池。
- 接收别人投递给自己的密文消息。
- 给自己的手机提供拉取、确认、删除。
- 管理自己的设备授权。
- 向对方私人通信全节点直连投递密文。

它禁止：

- 不给第三方做 Relay。
- 不做公共 rendezvous。
- 不做公共 DHT 基础设施。
- 不替别人存消息。
- 不解密消息。
- 不处理钱包、治理、交易业务。

### 5. 节点可达性

节点端点支持 IPv4、IPv6 和用户自有域名：

- `/ip4/<addr>/tcp/<port>/wss/p2p/<peer_id>`
- `/ip6/<addr>/tcp/<port>/wss/p2p/<peer_id>`
- `/dns4/<domain>/tcp/<port>/wss/p2p/<peer_id>`
- `/dnsaddr/<domain>/p2p/<peer_id>`

优先采用用户自己的可达端点，例如公网 IPv6、端口映射、UPnP / NAT-PMP、用户自有域名或用户自己控制的公网入口。不可达时消息保留在发送队列等待重试，不能退化成“借别人通信全节点中继”。

### 6. 聊天窗口发送公民币

聊天窗口内可以发起公民币转账，但到账真相仍以链上状态为准。

流程：

1. 用户在聊天窗口点击发送公民币。
2. citizenapp 使用联系人聊天账户作为收款钱包地址。
3. 用户确认金额和收款账户。
4. 本机钱包账户签名并通过 smoldot 提交链上交易。
5. 聊天中发送加密 `payment_notice`。
6. 双方 citizenapp 分别查链确认到账状态。

`payment_notice` 只是聊天提示，不是到账真相。

### 7. 近场通信

近场通信仍传输同一套密文 `ChatEnvelope`。

- Android 优先 Nearby Connections。
- Android 无 GMS 场景评估 Wi-Fi Direct / Wi-Fi Aware / BLE。
- iOS 使用 Multipeer Connectivity。
- Android 与 iOS 跨平台近场通过 BLE GATT 做发现、控制和短消息；大附件需单独真机验证。

## 后续 Spike

在真实跨节点投递前，必须先核实 CitizenChain 当前 `sc-network/libp2p` 对 Chat 的可用能力：

- request-response 是否可直接注册 已删除的节点聊天协议。
- notification 是否适合 ack / 状态推送。
- 节点端点变更如何签名与分发。
- 是否需要扩展现有网络 API。

默认不另起独立 libp2p swarm。只有 Spike 证明现有网络服务无法承载 Chat 必要能力时，才进入新决策。

## 否决方案

- 否决 Matrix / Olm / Megolm 作为主路线：授权、生态绑定和 PQC 演进口径不如 OpenMLS 适配当前目标。
- 否决把 KeyPackage 或联系人目录上链。
- 否决用 CID 做 Chat 密钥目录。
- 否决通信全节点互为中继。
- 否决用钱包私钥直接做 Chat 加密密钥。
