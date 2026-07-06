# CitizenApp 私密聊天技术架构

## 1. 模块定位

CitizenApp 聊天是公民端独立通信功能。用户可见聊天身份统一使用钱包地址；消息内容、附件明文和 OpenMLS 设备密钥只允许存在于 CitizenApp 本机。Cloudflare 只作为临时密文投递队列，未送达前短期保存 `ImEnvelope` 和加密附件对象；接收端落本机并 ack 后必须删除 Cloudflare 临时副本。近场链路只传输同一种密文 `ImEnvelope`，不得接触私聊或群聊明文。

正式聊天方式只保留两种：

```text
互联网聊天 = Cloudflare 密文 mailbox
近场聊天 = 蓝牙 / Wi-Fi 手机直连
```

已删除旧能力：

- 区块链节点通信节点聊天方式。
- 桌面端“通信节点功能”聊天入口。
- CitizenApp“设置通信节点”入口。
- `im_node_pairing` 固定码。
- `citizenchain/node/src/im/` 中的聊天 mailbox、KeyPackage 池和 `/gmb/im/1` 投递。

公开广场互动不是私密聊天：

- 点赞、评论、关注、举报、隐藏、不感兴趣等公开互动走 Cloudflare Worker + D1 明文业务数据。
- 私聊和群聊必须走 OpenMLS 端到端加密。

## 2. 总体架构

互联网聊天：

```text
CitizenApp
  -> OpenMLS 生成密文 ImEnvelope
  -> Cloudflare Worker 校验钱包 session / 设备绑定
  -> D1 临时保存未 ack / 未过期密文 envelope 与 KeyPackage
  -> R2 临时保存未 ack / 未过期加密附件对象
  -> Durable Objects / WebSocket 推送“有新密文”
  -> 对方 CitizenApp 拉取密文、本地解密、保存到本机
  -> 对方 CitizenApp ack 后 Worker 删除 D1 envelope 和对应 R2 加密附件
```

近场聊天：

```text
CitizenApp
  -> 蓝牙发现
  -> Wi-Fi Direct / Nearby Connections / Multipeer Connectivity 数据通道
  -> 传输同一种 OpenMLS ImEnvelope
  -> 对方 CitizenApp 本地解密
```

统一抽象：

```text
ImTransport
├── ImCloudflareTransport   # 互联网聊天
└── ImNearbyTransport       # 无互联网近场聊天
```

## 3. 身份模型

```text
聊天账户 = 钱包地址 owner_account / SS58 地址
设备身份 = CitizenApp 本地 OpenMLS device_id + device_key
加密身份 = OpenMLS 设备公钥 / KeyPackage
```

钱包私钥只用于证明“这个 IM 设备属于这个钱包地址”，不得用于：

- OpenMLS 消息加密。
- 每条聊天消息签名。
- Cloudflare 存储。
- 近场链路会话密钥。

设备绑定使用钱包签名：

```text
wallet_account
im_device_id
im_device_pubkey
expires_at_millis
nonce
signature
```

绑定签名后续统一登记到 Cloudflare mailbox 和近场安全码校验流程；不得再绑定区块链节点 PeerId 或通信节点端点。

## 4. 消息模型

统一消息格式继续使用 `GMB_IM_V1` / `ImEnvelope`：

```text
ImEnvelope
  protocol_version
  envelope_id
  conversation_id
  sender_chat_account
  recipient_chat_account
  sender_device_id
  mls_wire_message
  encrypted_metadata
  attachment_manifest_hash
  chunk_refs
  created_at_millis
  ttl_millis
  ack_policy
  mls_message_kind
  ratchet_tree
```

`mls_wire_message` 承载 OpenMLS 标准 wire bytes。链内 SCALE 不作为聊天主协议。

1:1 私聊和群聊都使用 OpenMLS group：

```text
1:1 私聊 = 两人 MLS group
群聊 = 多人 MLS group
```

## 5. 互联网聊天

互联网聊天由 Cloudflare Worker 提供不可信投递服务。

Worker 允许：

- 钱包签名登录。
- IM 设备绑定验签。
- KeyPackage 上传、拉取、消费。
- 密文 envelope 临时投递、拉取、ack 后删除。
- 加密附件 R2 上传授权。
- WebSocket 推送新密文通知。当前 IM-10 已使用账户级 Durable Object 管理在线设备连接，Worker 实例不再保存 socket 表。

Worker 禁止：

- 解密消息。
- 保存私聊或群聊明文。
- 保存 OpenMLS 私钥。
- 托管钱包私钥。
- 把私聊内容写入广场公开评论表。

D1 当前表：

```text
chat_devices
  owner_account
  device_id
  device_public_key_hex
  binding_signature
  expires_at
  revoked_at

chat_keypackages
  owner_account
  device_id
  key_package_id
  key_package
  cipher_suite
  created_at
  expires_at
  consumed_at

chat_envelopes
  envelope_id
  conversation_id
  sender_account
  sender_device_id
  recipient_account
  recipient_device_id
  mls_message_kind
  encrypted_payload
  attachment_manifest_key
  created_at
  expires_at
```

`chat_envelopes` 只保存未 ack 且未过期的临时密文投递项，不作为聊天历史表。`ack` 成功后删除对应行；提交和拉取 mailbox 时会顺手清理过期 envelope。

R2 加密附件路径：

```text
chat/{owner_account}/conversations/{conversation_id}/attachments/{attachment_id}/manifest.enc
chat/{owner_account}/conversations/{conversation_id}/attachments/{attachment_id}/chunk_001.bin
```

加密附件边界：

- CitizenApp 本地使用 `AES-GCM-256` 加密附件 manifest 和分片。
- 附件内容密钥、manifest nonce/mac/hash、分片 object key、分片 nonce/mac/hash 只写入 OpenMLS application 明文；该明文随后进入 `mls_wire_message`，Cloudflare 只能看到 OpenMLS 密文。
- `ImEnvelope.attachment_manifest_hash` 保存加密 manifest 的 sha256 hex；`ImEnvelope.chunk_refs` 只保存 manifest/chunk 的 R2 object key，便于接收端后续下载密文对象。
- Worker 只负责上传 `prepare -> upload -> complete` 和下载授权 `download -> signed GET`，签发 R2 短期目标并确认对象存在；不新增聊天附件 D1 表，不保存附件密钥。
- 下载授权只能在对应 `chat_envelopes` 临时行仍存在时签发；ack 删除后不得再依赖 Cloudflare 找回附件。
- 发送端发附件前会把明文附件写入本机私有缓存；接收端处理附件控制消息时必须先下载密文 manifest/chunk、校验 sha256、本地 AES-GCM 解密并保存到 App 私有目录，再 ack 删除 Cloudflare 临时副本。
- IM-8 阶段先实现单分片附件的文件选择、发送、下载、解密和本机保存；多分片续传、缩略图、系统打开文件和下载状态持久化放到后续阶段。

## 6. 近场聊天

近场聊天用于无互联网场景，必须不依赖 Cloudflare、不依赖区块链节点、不依赖链 RPC。

目标链路：

- Android：Nearby Connections，后续补 Wi-Fi Aware / BLE fallback。
- iOS：Multipeer Connectivity。
- Android / iOS 跨平台：BLE 发现 + Wi-Fi / 热点数据通道，或二维码交换会话信息后 Wi-Fi 直连。

近场只替换 transport，不改变身份、加密和消息格式：

```text
同一个钱包地址身份
同一个 OpenMLS 会话
同一个 ImEnvelope
不同传输通道
```

## 7. 本地存储

CitizenApp 本地继续负责明文和会话状态：

- `ImConversationEntity`：会话索引。
- `ImMessageEntity`：本机明文消息索引。
- `ImRouteCacheEntity`：钱包地址、设备公钥、安全码和 Cloudflare / 近场路由缓存。
- `ImOutboundQueueEntity`：互联网或近场待发送队列。
- `ImPendingInboundEntity`：Welcome 未到前的 application 暂存。
- OpenMLS native provider storage：App 私有 MLS 状态目录。
- App 私有附件缓存目录：发送端和接收端的附件明文副本。

互联网和近场都进入同一消息库，通过 `envelope_id` 去重。
用户在某台设备删除聊天记录时，该设备会删除本地会话、消息、待发送/待处理队列和附件缓存；其他聊天成员设备上的本地副本不受影响。

## 8. 发送流程

互联网 1:1 文本消息：

```text
打开 ImChatPage
-> ImRuntime 读取用户资料通信账户
-> 查询对方钱包地址的设备和 KeyPackage
-> OpenMLS 创建或恢复 MLS group
-> 生成 Welcome / application ImEnvelope
-> ImCloudflareTransport 投递密文
-> 对方 App WebSocket 收到新密文通知或主动同步
-> 对方 App 拉取密文、本地解密、保存本机、ack 删除 Cloudflare 临时副本
```

近场文本消息：

```text
打开近场聊天
-> 蓝牙 / Wi-Fi 发现对方设备
-> 交换钱包地址、设备公钥、安全码
-> OpenMLS 创建或恢复 MLS group
-> ImNearbyTransport 传输 ImEnvelope
-> 对方 App 本地解密并写入同一会话库
```

## 9. 当前代码状态

已存在能力：

- `citizenapp/lib/im/`：信息 Tab、聊天详情页、OpenMLS native 边界、Protobuf、Isar 消息库、消息流状态机、Cloudflare mailbox 自动运行态。
- `citizenapp/im/proto/im_envelope.proto`：`GMB_IM_V1` 外层 Protobuf 真源。
- `citizenapp/lib/im/crypto/`：OpenMLS 设备密钥、KeyPackage、会话处理和本地状态目录。
- `citizenapp/cloudflare/src/chat/`：Cloudflare 临时密文 mailbox API 已落地，包含设备绑定验签、KeyPackage 发布/拉取/消费、密文 envelope 投递、pending 拉取、ack 删除、过期清理、加密附件上传准备、开发代理上传、上传完成确认、下载授权、开发代理下载和 `/v1/chat/ws` 新密文通知；实时通知由 `ChatRealtimeObject` 按钱包账户聚合在线设备。
- `citizenapp/android/im/`、`citizenapp/ios/im/`：近场原生能力预留目录。
- `citizenapp/lib/im/im_runtime.dart`：发送或同步前自动复用广场 Worker 钱包 session，自动登记本机 IM 设备，自动发布本设备 KeyPackage，首次会话自动拉取并消费对方 KeyPackage；已开放 `sendAttachment`、`downloadAttachment`、`startRealtimeSync` 和 `deleteLocalConversation`。
- `citizenapp/lib/im/im_tab_page.dart`：信息 Tab 打开后自动同步 pending 密文；优先连接 WebSocket 新密文通知，连接不可用或断开时回退到前台每 15 秒轻量轮询，失败后退避到 30 秒，轮询成功后重试 WebSocket，离开页面或 App 退后台即停止；会话列表支持左滑删除本机聊天记录。
- `citizenapp/lib/im/im_chat_page.dart`：聊天窗口打开后自动同步 pending 密文；优先连接 WebSocket 新密文通知，连接不可用或断开时回退到前台每 8 秒轮询，失败后退避到 30 秒，轮询成功后重试 WebSocket，离开页面或 App 退后台即停止；已使用现成聊天 UI 的附件按钮选择文件发送，点击附件消息下载解密并保存到本机私有缓存；右上角更多菜单支持删除本机聊天记录，删除后从列表进入的聊天页会返回信息 Tab。
- `citizenapp/lib/im/transport/im_cloudflare_transport.dart`：互联网聊天 transport 已支持真实 HTTP 调用 Worker mailbox API、WebSocket 新密文通知、加密附件上传 API 和加密附件下载授权 API；未注入 Worker session 的测试/占位运行态仍返回明确失败状态。
- `citizenapp/lib/im/im_message_flow.dart`：已支持文本消息、加密附件发送和附件下载解密；附件发送先保存发送端本机明文缓存，再本地加密 manifest/chunk、上传 R2 密文对象，最后用 OpenMLS application 消息投递附件控制信息；接收端处理附件控制消息时先下载密文对象、校验 hash、解密保存到本机缓存，再 ack 删除 Cloudflare 临时副本；后续点击附件优先读取本机缓存。
- `citizenapp/lib/im/im_chat_ui_adapter.dart`：附件类本地消息在聊天列表中显示安全占位文案 `[附件] 文件名`，不把 OpenMLS 控制 JSON 原样露出给用户；附件控制消息只放在本机 UI metadata，供点击下载使用。
- `citizenapp/test/im/im_envelope_session_test.dart`：已覆盖 A/B 两端经 mailbox pending 拉取、接收端解密落库、附件先落本机缓存、ack 后 mailbox 清空的闭环状态机。
- `citizenapp/test/im/im_isar_store_test.dart`：已覆盖删除某个本机会话时清理消息、会话、待发送队列和 pending 入站记录，且不误删其他会话。
- `citizenapp/test/im/im_tab_page_test.dart`：已覆盖信息 Tab 左滑删除确认、聊天页更多菜单删除确认和删除后返回上一页。
- `citizenapp/scripts/build-smoldot-native.sh`：macOS host 调试库已禁用 release strip，避免 dyld `mis-aligned LINKEDIT string pool` 导致 Dart FFI 无法加载。
- `citizenapp/test/im/im_mls_native_session_test.dart`：native OpenMLS mailbox 闭环用例已在本机 host `libsmoldot.dylib` 下真实执行通过；没有 host 库时仍按 helper 跳过，真机/APK 集成构建中继续执行。

已删除旧能力：

- `citizenapp/lib/im/im_node_settings_page.dart`
- `citizenapp/lib/im/transport/im_private_node_transport.dart`
- `citizenapp/lib/qr/bodies/im_node_pairing_body.dart`
- `citizenchain/node/src/im/`
- `citizenchain/node/src/settings/communication-node/`
- `citizenchain/node/frontend/settings/communication-node/`
- `citizenchain/scripts/im-two-node-smoke.sh`

第 1 步已完成旧通信节点聊天路线删除和 Cloudflare transport 骨架接入；第 IM-2 步已完成 Worker chat mailbox API 和 App transport HTTP 调用；第 IM-3 步已完成 CitizenApp 自动 Worker session、设备绑定签名、KeyPackage 发布和聊天窗口打开自动同步；第 IM-4 步已完成信息 Tab 与聊天窗口前台自动收信轮询；第 IM-5 步已完成互联网私聊 mailbox 拉取、解密落库和 ack 闭环回归；第 IM-6 步已完成 macOS host OpenMLS native 真实执行验收；第 IM-7 步已完成加密附件发送底座、R2 密文上传接口和附件消息占位显示；第 IM-8 步已完成聊天页文件选择、附件下载授权、密文下载、本地校验解密和私有缓存保存；第 IM-9 步已完成基础 WebSocket 新密文通知与轮询兜底；第 IM-10 步已完成账户级 Durable Objects 生产级 fanout；第 IM-11 步已完成 Cloudflare 临时 mailbox 最小化存储、ack 删除 D1 envelope/R2 加密附件、本机附件缓存优先和本机会话删除底座；第 IM-12 步已完成信息列表和聊天页删除聊天记录 UI。后续继续落地近场真机链路、多分片续传和附件下载状态持久化。

## 10. 预计修改目录

- `citizenapp/lib/im/`：统一聊天运行态、聊天页、消息状态机、加密附件编排和 Cloudflare / 近场传输抽象；涉及代码、中文注释和残留清理。当前已完成 Cloudflare mailbox 自动 session/设备绑定/KeyPackage 发布、前台自动收信轮询、WebSocket 新密文通知、pending 拉取解密 ack 删除闭环回归、加密附件发送、下载解密、本机缓存优先和删除本机聊天记录 UI。
- `citizenapp/lib/im/transport/`：删除节点传输，新增 `ImCloudflareTransport` 和 `ImNearbyTransport`；涉及代码。当前 `ImCloudflareTransport` 已支持密文 envelope、KeyPackage、pending/ack、WebSocket 通知、附件 prepare/upload/complete 和附件 download/dev-get。
- `citizenapp/im/proto/`：继续作为 `GMB_IM_V1` 真源；如 Cloudflare mailbox 字段不足，再按确认后的协议补字段。
- `citizenapp/lib/im/crypto/`：继续承载 OpenMLS、设备密钥、KeyPackage 和安全码；涉及代码。当前 native OpenMLS KeyPackage、两方 smoke、持久化会话恢复和 mailbox 闭环均已在 host 调试库下真实执行通过。
- `citizenapp/lib/im/storage/`：会话、消息、路由缓存、发送队列、pending 入站 envelope 和本机会话删除；涉及代码。
- `citizenapp/lib/8964/`：广场作者页、动态卡片、评论作者私信入口和 Worker session 复用；涉及代码。当前 `SquareApiClient` 已向 IM 运行态开放 Worker base URI 和登录 session 复用能力。
- `citizenapp/cloudflare/`：新增临时密文 mailbox、KeyPackage、设备绑定、ack 删除、过期清理、加密附件上传授权、下载授权和 WebSocket 推送；涉及代码和配置。当前 IM-11 已将 ack 改为删除 D1 envelope，并删除对应 R2 加密附件对象；WebSocket 在线连接由账户级 `ChatRealtimeObject` 管理，不推送明文或密文正文。
- `citizenapp/android/im/`：Android 近场聊天；涉及原生代码。
- `citizenapp/ios/im/`：iOS 近场聊天；涉及原生代码。
- 旧 `citizenchain/node/src/im/`：已删除区块链节点聊天实现；后续不得恢复。
- 旧 `citizenchain/node/frontend/settings/communication-node/`：已删除通信节点设置面板；后续不得恢复。
- `memory/05-modules/citizenapp/im/`：维护本技术文档；涉及文档。
- `memory/07-ai/`：登记 Cloudflare mailbox 和近场传输协议，删除通信节点聊天口径；涉及文档。

## 11. 验收要求

第 1 步验收：

- 正式聊天方式只剩 Cloudflare 互联网聊天和近场聊天。
- 区块链节点聊天方式、桌面通信节点设置和 `im_node_pairing` 已从当前代码中删除。
- Cloudflare 明确只临时保存未 ack / 未过期私聊或群聊密文和必要投递元数据；接收端落本机并 ack 后不得保留聊天记录副本。
- 广场公开评论、点赞等公开互动允许 Worker 明文处理。
- `git diff --check` 通过。

后续实现验收按近场真机、多分片续传、附件缩略图/打开文件和完整端到端真机阶段分别确认；自动收信当前已支持 Durable Objects WebSocket 新密文通知，连接不可用或断开时必须保留前台轮询兜底。macOS 本机验收可先执行 `./scripts/build-smoldot-native.sh macos` 生成 host `libsmoldot.dylib`，再运行 native OpenMLS 测试；没有 host 库时 native 用例会按现有规则跳过，真机/APK 集成构建仍必须覆盖真实 OpenMLS 执行。加密附件验收必须确认 Worker/R2 只看到密文 manifest、密文分片和 object key，附件内容密钥不得出现在 Cloudflare D1、R2 或公开日志中；ack 后必须确认 D1 envelope 与对应 R2 加密附件对象已删除；删除聊天记录验收必须确认只删除本机记录和附件缓存，不删除联系人、不影响对方和其他设备。
