# wuminapp P2P IM 技术架构

## 1. 模块定位

`wuminapp` 的 P2P IM 是独立通信功能，不属于钱包、治理、交易或身份实名模块。

用户入口只有一个：手机端在“多签”Tab 与“交易”Tab 之间新增“信息”Tab。用户不选择“近场通信 / 通信全节点”模式，两类消息都在“信息”Tab 集中显示、统一发送和统一搜索。

核心目标：

- 远程通信：手机连接用户自己的通信全节点，支持对方手机不在线时收取密文消息。
- 近场通信：手机与手机直接通信，用于游行、聚会、演出、体育场等手机网络拥堵或不可用场景。
- 完全去中心化边界：不使用中心化信令服务器，不把公共节点当通信中继，聊天内容只留在 IM 通信体系内，不把同一路由器作为必要条件。
- 成熟组件优先：协议、加密、传输、近场能力优先复用成熟库或系统框架，不自研底层通信协议和加密算法。

## 2. 总体架构

```text
wuminapp 信息 Tab
├── 会话列表 / 消息详情 / 附件入口
├── 统一消息层 wuminapp/lib/im/
│   ├── 端到端加密
│   ├── 统一消息 Envelope
│   ├── 文本 / 图片 / 视频 / 文件
│   ├── 分片 / 重传 / 校验 / 去重
│   ├── 本地消息库与发送队列
│   └── TransportRouter 自动选择传输
├── 远程通信全节点传输
│   └── 手机 <-> 用户自己的通信全节点
└── 近场无网传输
    ├── Android：成熟 Nearby Connections 优先；必要时回退系统 Wi-Fi Direct
    └── iOS：Multipeer Connectivity
```

技术原则：

- “信息”Tab 是唯一用户入口；通信方式是底层能力，不做面向用户的模式选择器。
- 先定义统一消息层，再接远程通信全节点和近场传输。
- 远程通信和近场通信都只传输端到端加密后的消息 Envelope。
- Android 与 iOS 的近场能力使用成熟平台能力，Dart 只调用统一接口。
- 平台原生目录保持浅层结构：`wuminapp/android/im/` 与 `wuminapp/ios/im/`。
- `im` 目录只实现 IM 通信功能，不混入钱包、治理、交易或用户资料逻辑。

## 3. 用户体验边界

底部 Tab 目标顺序：

```text
首页 / 多签 / 信息 / 交易 / 我的
```

“信息”Tab 内部包含：

- 会话列表：所有远程通信全节点消息、近场消息统一展示。
- 消息详情：同一联系人或群组的消息按时间线展示，不暴露底层传输来源为主流程。
- 附件发送：文本、图片、视频、文件统一进入发送队列。
- 附近发现：只作为临时连接动作，例如“附近的人”或“面对面发送”，不作为全局通信模式。

设置页只在需要时增加“通信全节点”设置项，不增加“通信模式选择”设置项。

“通信全节点”设置项只负责：

- 绑定 / 更换 / 解绑我的通信全节点。
- 展示节点在线状态、PeerId、通信端点和收件箱同步状态。
- 展示设备授权状态。

## 4. 统一消息层

统一消息层位于 `wuminapp/lib/im/`，对远程通信全节点、Android 近场、iOS 近场提供同一套消息格式和状态机。

成熟组件优先选型：

- 客户端 SDK：优先评估 Dart / Flutter 生态中成熟的 Matrix SDK 能力，复用其会话、同步、加密和附件处理经验；如因自有通信全节点协议边界无法直接采用 Matrix federation，也只复用成熟数据模型与 E2EE 组件思想，不自研加密算法。
- 消息编码：优先使用成熟序列化方案，如 Protobuf 或 Matrix 事件 JSON；禁止临时拼字符串协议。
- 端到端加密：优先复用成熟实现，如 Matrix SDK 配套的 vodozemac / Olm-Megolm 类能力；确需自定义 Envelope 时也只组合成熟密码学库。
- 本地存储：继续使用 wuminapp 既有 Isar 体系；落地 schema 前必须先沟通。

统一消息对象：

```text
ImEnvelope
├── envelope_id
├── conversation_id
├── sender_device_id
├── recipient_device_ids
├── created_at
├── content_type
├── encrypted_payload
├── attachment_refs
├── chunk_refs
└── transport_hints
```

发送状态：

- `queued`：已进入本机发送队列。
- `sending`：正在通过某个传输发送。
- `sent`：已成功交给通信全节点或近场对端。
- `delivered`：对端设备或对端通信全节点已确认收取。
- `failed`：发送失败，可重试。

约束：

- `message_id` 与 `chunk_id` 必须全局去重，避免同一消息通过近场和通信全节点重复显示。
- 消息状态只属于 IM，不得复用 txHash、finalized、投票 pending 等其他业务状态。
- IM 使用独立设备身份密钥，不直接复用钱包私钥作为长期 IM 通信密钥。
- IM 设备授权只使用独立通信身份和 IM 设备密钥体系，不依赖钱包账户或其他业务状态。
- 每个设备独立授权，用户可以撤销单个设备。

## 5. 自动传输路由

`TransportRouter` 负责在用户无感知的情况下选择传输：

```text
发送消息
-> 生成并加密 ImEnvelope
-> 写入本地发送队列
-> 查询收件人通信端点和本地近场发现状态
-> 优先投递可达的通信全节点
-> 若附近发现对端设备，可并行或补充近场投递
-> 根据 ack 去重并更新统一消息状态
```

路由规则：

- 收件人配置了通信全节点且可达时，优先使用通信全节点。
- 双方设备近场可达时，允许近场直连投递，适合无公网或网络拥堵场景。
- 两种路径都可用时，允许并行发送，但以 `envelope_id` 和 ack 去重。
- 两种路径都不可用时，消息留在发送队列，等待后续重试。
- 用户不需要选择通信模式。

## 6. 通信全节点

通信全节点能力放在 `citizenchain/node/src/im/`。

定位：

- 通信全节点是桌面节点软件的一个运行模式，用于让 wuminapp 用户全天候实时在线。
- 当前 `citizenchain/node` 已固定使用 libp2p 网络后端，通信全节点只复用这个成熟网络能力，不复用其他业务、不另起一套 P2P 网络栈。
- 通信全节点只保存端到端加密后的 IM Envelope 和附件分片，不读取明文。

成熟组件优先选型：

- 节点间 P2P：复用 `citizenchain/node` 既有 libp2p 后端能力，新增 IM 专用协议名和消息处理器。
- 直接投递：优先使用 libp2p request-response 或 stream 类能力，不自研 TCP 私有协议。
- 大附件：优先评估 iroh-blobs 这类成熟内容寻址、校验、断点下载组件；若不引入 iroh，则在 IM 层只实现薄封装，底层 hash、分片和校验使用成熟库。
- 节点持久化：优先复用节点现有安全数据目录和成熟嵌入式存储方案；聊天密文、通信端点、设备公钥、PeerId、更新时间和撤销状态都只进入 IM 专属存储。

通信全节点职责：

- 保存绑定用户的密文收件箱。
- 接收其他用户投递的密文 Envelope。
- 给绑定用户的 wuminapp 提供拉取、确认和删除接口。
- 维护设备授权、通信端点、PeerId、过期时间和容量限制。
- 清理过期消息、过期附件和已确认分片。

通信全节点禁止承担：

- 不解密消息。
- 不替用户签名。
- 不处理钱包、治理、交易业务。
- 不作为公共归档全节点或普通全节点的默认职责。
- 通信端点、设备公钥、PeerId、更新时间和撤销状态只属于 IM 通信体系。

## 7. Android 近场无网通信

Android 近场能力放在 `wuminapp/android/im/`。

成熟组件优先选型：

- 第一优先：Google Nearby Connections。它提供离线点对点发现、连接和数据交换，底层组合 Bluetooth、BLE 和 Wi-Fi，并支持 bytes、files、streams。
- 第二优先：系统 Wi-Fi Direct / Wi-Fi P2P API。仅当 Nearby Connections 的 Google Play Services 依赖不符合发布边界时，才采用 BLE 发现 + Wi-Fi Direct 传输的原生组合。
- Flutter 接入：优先使用维护状态良好的 Flutter 插件；如插件不能满足安全和权限边界，再用 Kotlin 封装 Platform Channels。

Android 流程：

```text
用户进入信息 Tab 的附近发现入口
-> Android 近场能力开始 advertise / discover
-> 用户确认连接
-> 双方验证 IM 设备身份
-> 传输密文 ImEnvelope / attachment chunk
-> ack 写回统一消息层
-> 空闲后断开近场链路
```

约束：

- 近场发现必须由用户主动触发，不做后台长期扫描。
- 权限申请只在使用附近发现时触发，不在 App 启动时打扰用户。
- 若采用 Nearby Connections，必须在文档中明确 Google Play Services 依赖和数据收集边界。
- 若采用 Wi-Fi Direct，Android 13+ 必须处理 `NEARBY_WIFI_DEVICES` 权限。

## 8. iOS 近场无网通信

iOS 近场能力放在 `wuminapp/ios/im/`。

成熟组件优先选型：

- 使用 Apple Multipeer Connectivity。
- 原生语言：Swift。
- Flutter 接入：Platform Channels。

iOS 流程：

```text
用户进入信息 Tab 的附近发现入口
-> MCNearbyServiceAdvertiser / MCNearbyServiceBrowser 发现附近 wuminapp
-> 用户确认连接
-> MCSession 建链
-> 双方验证 IM 设备身份
-> 传输密文 ImEnvelope / attachment chunk
-> ack 写回统一消息层
```

约束：

- iOS 不使用 Android 风格 Wi-Fi Direct API。
- iOS 近场模式不能把同一路由器作为必要条件，必须按无公网、无中心服务器、无需同一路由器的目标做真机验证。
- iOS 权限文案必须说明附近设备发现用途，不得暗示会读取通讯录或真实身份。

## 9. 设置页

设置页只增加“通信全节点”配置，不增加“通信模式”配置。

是否需要设置项取决于通信全节点绑定方式：

- 若 wuminapp 能从本地绑定状态、二维码或用户配置自动获取通信全节点端点，设置页只显示状态与解绑。
- 若需要用户手动输入或扫码绑定通信全节点，设置页提供绑定入口。

设置页字段：

- 通信全节点状态：未绑定 / 离线 / 在线 / 同步中。
- PeerId。
- 通信端点。
- 绑定设备列表。
- 最近同步时间。

## 10. 不采用方案

- 不采用“局域网模式”作为独立技术路线，不要求同一路由器。
- 不采用中心化信令服务器。
- 不把公共全节点作为 IM 信令、中继或消息存储节点。
- 不把聊天内容、通信端点、设备公钥、PeerId、更新时间或撤销状态交给 IM 之外的系统保存。
- 不让用户选择“近场模式 / 通信全节点模式”。
- 第一版不做多跳 mesh。人群拥堵场景先做附近用户一跳直连，多跳转发后续单独设计。
- 不自研底层加密算法、P2P 传输栈、附件分片校验算法。

## 11. 预计修改目录

- `wuminapp/lib/im/`：Dart 侧 IM 统一消息层、会话状态、发送队列、传输抽象和信息 Tab 数据模型；涉及代码。
- `wuminapp/lib/im/crypto/`：IM 设备身份、端到端加密、签名校验；涉及代码，优先复用成熟 E2EE 组件。
- `wuminapp/lib/im/storage/`：手机本地消息库、附件分片缓存、失败重试队列；涉及代码，Isar schema 落地前必须先沟通。
- `wuminapp/lib/im/transport/`：远程通信全节点、Android 近场、iOS 近场三类传输接口与自动路由；涉及代码。
- `wuminapp/lib/ui/`：底部 Tab 新增“信息”入口，位于“多签”和“交易”之间；涉及代码。
- `wuminapp/android/im/`：Android Nearby Connections 或 Wi-Fi Direct 原生近场模块；涉及代码，只承载 IM 近场通信功能。
- `wuminapp/ios/im/`：iOS Multipeer Connectivity 原生近场模块；涉及代码，只承载 IM 近场通信功能。
- `wuminapp/android/app/`：后续仅做权限、Gradle/sourceSet 或 MethodChannel 接入，不承载 IM 业务主体；涉及少量平台接线代码。
- `wuminapp/ios/Runner/`：后续仅做权限文案、Xcode/Swift 桥接接入，不承载 IM 业务主体；涉及少量平台接线代码。
- `citizenchain/node/src/im/`：通信全节点收件箱、密文消息存储、设备绑定接口和 libp2p IM 协议处理；涉及代码，后续在通信全节点阶段实现。
- `citizenchain/node/frontend/settings/`：通信全节点绑定、端口、PeerId、在线状态和收件箱同步展示；涉及代码。
- `memory/01-architecture/wuminapp/`：wuminapp IM 总体架构文档；涉及文档。
- `memory/05-modules/wuminapp/im/`：wuminapp IM 模块技术文档；涉及文档。
- `memory/05-modules/citizenchain/node/`：通信全节点模式落地后的节点技术边界；涉及文档。

## 12. 实施顺序

1. 信息 Tab 壳与统一消息层：先落会话列表、消息详情、发送队列和本地状态，不接真实传输。
2. 成熟 E2EE 与设备身份：复用成熟库建立设备授权、密钥轮换和消息加密。
3. 通信全节点收件箱：在 `citizenchain/node/src/im/` 复用 libp2p 能力实现密文投递和拉取。
4. Android 近场：优先 Nearby Connections，若发布边界不接受 Google Play Services，再用 BLE + Wi-Fi Direct。
5. iOS 近场：Multipeer Connectivity。
6. 拥堵场景专项优化：发现列表、短消息优先、大文件分片、断点续传。

## 13. 当前状态

本文件保存完整技术架构；当前未创建 `wuminapp/lib/im/`、`wuminapp/android/im/`、`wuminapp/ios/im/`、`citizenchain/node/src/im/` 或业务代码。
