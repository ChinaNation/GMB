# wuminapp P2P IM 技术选型方案

## 1. 模块定位

`wuminapp` 的 P2P IM 是独立通信功能，不属于链上 RPC、钱包、治理或交易模块。

核心目标：

- 远程通信：手机连接用户自己的通信全节点，支持对方手机不在线时收取密文消息。
- 近场通信：手机与手机直接通信，用于游行、演出、体育场等手机网络拥堵或不可用场景。
- 完全去中心化边界：不使用中心化信令服务器，不把公共全节点当通信中继，不把聊天内容写入区块链，不把同一路由器作为必要条件。

## 2. 总体技术路线

```text
wuminapp P2P IM
├── 统一消息层
│   ├── 端到端加密
│   ├── 统一消息 Envelope
│   ├── 文本 / 图片 / 视频 / 文件
│   ├── 分片 / 重传 / 校验 / 去重
│   └── 与具体传输方式解耦
├── 远程通信层
│   └── 手机 <-> 用户自己的通信全节点
└── 近场无网通信层
    ├── Android：BLE 发现 + Wi-Fi Direct 高速传输
    └── iOS：Multipeer Connectivity 近场 P2P
```

技术选型原则：

- 先定义统一消息层，再接远程通信全节点和近场通信。
- Android 与 iOS 的近场能力使用平台原生实现，Dart 只调用统一接口。
- 平台原生目录保持浅层结构：`wuminapp/android/im/` 与 `wuminapp/ios/im/`。
- `im` 目录只实现 IM 近场通信功能，不混入钱包、治理、链上 RPC 或用户资料逻辑。

## 3. 统一消息层

统一消息层位于 `wuminapp/lib/im/`，对远程通信全节点、Android 近场、iOS 近场提供同一套消息格式。

建议选型：

- 消息编码：Protobuf。
- 消息封装：`WuminMessageEnvelope`。
- 消息内容：文本、图片、视频、文件统一走 envelope。
- 大文件：按 chunk 分片，chunk 使用 hash 校验。
- 去重：使用 `message_id` 与 `chunk_id` 去重。
- 发送状态：本机记录待发送、发送中、已送达、失败，不把 txHash 或链上状态当 IM 消息状态。

身份与加密：

- IM 使用独立设备身份密钥，不直接复用钱包私钥作为长期 IM 通信密钥。
- 用户链上账户只用于绑定或授权 IM 设备身份。
- 消息端到端加密；通信全节点和近场传输层都只处理密文。
- 每个设备独立授权，用户可以撤销单个设备。

## 4. 远程通信层

远程通信用于日常跨地域聊天和离线消息。

选型：

- 通信节点：用户自己的通信全节点。
- 节点实现：Rust。
- 网络：IPv6 优先，QUIC/TCP 传输。
- P2P 框架：后续节点间能力优先考虑 libp2p。
- 存储：通信全节点只保存端到端加密 envelope，不读取明文。

远程流程：

```text
发送方 wuminapp
-> 读取收件人通信端点和公钥
-> 连接收件人自己的通信全节点
-> 上传密文 WuminMessageEnvelope
-> 收件人 wuminapp 上线
-> 连接自己的通信全节点
-> 拉取密文消息
-> 手机本机解密
```

边界：

- 发送方不需要运行自己的全节点。
- 收件方要离线收消息，需要自己的通信全节点在线。
- 公共归档全节点、普通全节点不存消息、不转发消息、不承担信令服务器职责。
- 链上最多记录通信端点、公钥、PeerId、更新时间，不记录聊天内容。

## 5. Android 近场无网通信

Android 近场能力放在 `wuminapp/android/im/`。

选型：

- 发现：BLE 广播 / 扫描。
- 高速传输：Wi-Fi Direct。
- 原生语言：Kotlin。
- Flutter 接入：Platform Channels。

Android 官方 Wi-Fi Direct 文档说明，支持设备可以不经过中间 access point 直接通过 Wi-Fi 连接，适合照片分享等高速数据场景：

- `https://developer.android.com/develop/connectivity/wifi/wifip2p`

流程：

```text
用户主动开启近场模式
-> BLE 广播本机临时发现 ID
-> 扫描附近 wuminapp 用户
-> 用户选择或确认连接
-> BLE 交换身份挑战和 Wi-Fi Direct 参数
-> 建立 Wi-Fi Direct 链路
-> 双方验证 IM 设备身份
-> 传输密文 WuminMessageEnvelope / chunk
-> 空闲后断开 Wi-Fi Direct
```

约束：

- 近场模式必须由用户主动开启，不做后台长期自动扫描。
- 蓝牙只用于发现和小握手，不承载图片、视频、大文件主传输。
- Wi-Fi Direct 内部可能有 Group Owner 角色，这是协议建链角色，不等于依赖外部中心化路由器。

## 6. iOS 近场无网通信

iOS 近场能力放在 `wuminapp/ios/im/`。

选型：

- Apple 原生框架：Multipeer Connectivity。
- 原生语言：Swift。
- Flutter 接入：Platform Channels。

Apple 文档说明 Multipeer Connectivity 支持附近设备服务发现，并支持消息、流、资源等通信；`MCNearbyServiceBrowser` 可通过 infrastructure Wi-Fi、peer-to-peer Wi-Fi、Bluetooth 等发现附近服务：

- `https://developer.apple.com/documentation/multipeerconnectivity`

流程：

```text
用户主动开启近场模式
-> MCNearbyServiceAdvertiser / MCNearbyServiceBrowser 发现附近 wuminapp
-> 用户确认连接
-> MCSession 建链
-> 双方验证 IM 设备身份
-> 传输密文 WuminMessageEnvelope / chunk
```

约束：

- iOS 不使用 Android 风格 Wi-Fi Direct API。
- iOS 近场模式不能把同一路由器作为必要条件，必须按无公网、无中心服务器、无需同一路由器的目标做真机验证。

## 7. Flutter 接入

Flutter 层保留统一接口，平台差异收口在 Android/iOS 原生 `im` 目录。

Flutter 官方 Platform Channels 支持 Dart 与 Android Kotlin / iOS Swift 平台代码异步通信：

- `https://docs.flutter.dev/platform-integration/platform-channels`

Dart 层接口建议：

```text
ImTransport
├── start()
├── discover()
├── connect(peer)
├── send(envelope)
├── receive()
└── disconnect()
```

实现分层：

```text
RemoteNodeTransport
AndroidNearbyTransport
IosNearbyTransport
```

## 8. 不采用方案

- 不采用“局域网模式”作为独立技术路线，不要求同一路由器。
- 不采用中心化信令服务器。
- 不把公共全节点作为 IM 信令、中继或消息存储节点。
- 不把聊天内容上链。
- 不把 Google Nearby Connections 作为核心路线；它可后续作为 Android 备用实验，但不作为 wuminapp 自主可控的主路径。
- 第一版不做多跳 mesh。人群拥堵场景先做附近用户一跳直连，多跳转发后续单独设计。

## 9. 预计修改目录

- `wuminapp/lib/im/`：Dart 侧 IM 统一消息层、会话状态、发送队列、传输抽象；涉及代码。
- `wuminapp/lib/im/crypto/`：IM 设备身份、端到端加密、签名校验；涉及代码。
- `wuminapp/lib/im/storage/`：手机本地消息库、附件分片缓存、失败重试队列；涉及代码。
- `wuminapp/lib/im/transport/`：远程通信全节点、Android 近场、iOS 近场三类传输接口；涉及代码。
- `wuminapp/android/im/`：Android BLE + Wi-Fi Direct 原生近场模块；涉及代码，只承载 IM 近场通信功能。
- `wuminapp/ios/im/`：iOS Multipeer Connectivity 原生近场模块；涉及代码，只承载 IM 近场通信功能。
- `wuminapp/android/app/`：后续仅做 Gradle/sourceSet 或 MethodChannel 接入，不承载 IM 业务主体；涉及少量平台接线代码。
- `wuminapp/ios/Runner/`：后续仅做 Xcode/Swift 桥接接入，不承载 IM 业务主体；涉及少量平台接线代码。
- `citizenchain/node/src/communication/`：通信全节点收件箱、密文消息存储、设备绑定接口；涉及代码，后续在通信全节点阶段实现。
- `citizenchain/node/frontend/settings/`：通信全节点模式、端口、IPv6 连通性、设备绑定展示；涉及代码，后续在通信全节点阶段实现。
- `memory/01-architecture/wuminapp/`：wuminapp IM 总体架构文档；涉及文档。
- `memory/05-modules/wuminapp/im/`：wuminapp IM 模块技术文档；涉及文档。
- `memory/05-modules/citizenchain/node/`：通信全节点模式落地后的节点技术边界；涉及文档。

## 10. 实施顺序

1. 统一消息层与加密身份。
2. 通信全节点远程收件箱。
3. Android 近场无网：BLE + Wi-Fi Direct。
4. iOS 近场无网：Multipeer Connectivity。
5. 拥堵场景专项优化：发现列表、短消息优先、大文件分片、断点续传。

## 11. 当前状态

本文件仅保存技术选型方案；当前未创建 `wuminapp/android/im/`、`wuminapp/ios/im/` 或业务代码。
