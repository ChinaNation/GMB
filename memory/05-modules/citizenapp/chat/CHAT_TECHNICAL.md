# CitizenApp 私密聊天技术架构

## 1. 目标边界

CitizenApp Chat 对所有拥有钱包账户的用户开放，不依赖会员。聊天账户使用钱包地址，OpenMLS 设备密钥与钱包私钥严格分离。

数据归属固定为：

- 消息明文、密文副本、会话、发送队列和附件只保存在用户设备。Chat 路由缓存不是通讯录；通讯录跨设备同步由 USER 模块端侧加密后写入 D1，Chat 和 Worker 均不能读取联系人明文或解密密钥。
- Cloudflare 不持久化消息、会话、联系人或附件，也不提供离线内容代存。
- Cloudflare 只保存设备公钥、推送 Token、一次性 KeyPackage 和防重放哈希。
- CitizenChain runtime 不参与聊天，不记录聊天字段，不收取聊天费用。

## 2. 总体架构

```text
发送设备
  -> OpenMLS 生成 ChatEnvelope
  -> 本机 Isar 写入待发送队列
  -> Worker 校验钱包 session 与登记设备
  -> Durable Object WebSocket 在当前请求内转发密文
  -> 接收设备立即 OpenMLS 解密并写入本机
```

接收设备不可达时：

```text
Worker 返回 queued
  -> 密文继续只留发送设备
  -> Worker 发送 chat_wake 无内容推送
  -> 接收设备的后台处理器建立短时 WebSocket，并向发送方发 peer_ready 信令
  -> 发送设备在线时立即重试；发送设备离线时由反向唤醒启动其本机重试
```

这不要求双方同时打开聊天页。系统允许后台执行时，两端由通用推送自动建立短时收发窗口；设备被系统完全停止、卸载或长期离线时，未送达消息等待持有本地队列的发送设备恢复。

附件链路：

```text
Worker 瞬时转发 SDP/ICE
  -> WebRTC DTLS DataChannel
  -> 附件字节直接从发送设备到接收设备
  -> 接收设备写入 App 私有目录
```

WebRTC 只使用公共 STUN 发现候选地址，不配置中继服务。NAT 环境无法建立设备直连时，附件保持在发送设备并等待后续重试；Cloudflare 不接收、转发或保存附件字节。

## 3. 身份与密钥

```text
聊天账户 = owner_account 钱包地址
会话登录 = 已绑定的硬件 P-256 设备子钥静默签名
端到端加密 = OpenMLS device_id + 设备密钥 + KeyPackage
```

聊天页的用户展示复用统一 `ProfilePresentation` / `ProfileAvatar`：调用方已有公开昵称时优先使用；名称缺失或错误传入完整/截断账户时，按对方账户稳定选择本地默认昵称。真实头像缺失时使用同一账户对应的本地默认照片；账户只显示在标题下方的账户行，不得充当昵称。

通讯录联系人三点菜单的“私信”直接复用 `openDirectChat()`：`peerAddress` 使用联系人钱包账户，标题使用统一公开昵称或稳定本地昵称。联系人私人名称只属于通讯录，不进入 Chat 路由真源；该入口不得复制聊天页面、会话创建或传输逻辑。

钱包主私钥只在创建热钱包时证明 P-256 设备子钥归属。Chat 运行态禁止读取 seed、调用用户认证签名或使用 CitizenWallet。

设备登记签名字段：

```text
owner_account
device_id
device_public_key_hex
expires_at
nonce
```

Worker 从 session 派生 `owner_account`，查询 `square_device_subkeys.p256_pubkey` 验签。签名不落库，nonce 只以 SHA-256 哈希落库。

## 4. 消息协议

协议真源：`citizenapp/chat/proto/chat_envelope.proto`。

`ChatEnvelope` 只保留：

```text
protocol_version
envelope_id
conversation_id
sender_account
recipient_account
sender_device_id
mls_wire_message
encrypted_metadata
created_at_millis
ttl_millis
mls_message_kind
ratchet_tree
```

附件控制消息类型为 `gmb_chat_attachment_v2`，只在 OpenMLS application 明文中承载附件 ID、文件名、MIME 和字节数。附件字节不进入 `ChatEnvelope`。

## 5. Cloudflare 接口

- `POST /v1/chat/devices/register`：登记设备公钥和 APNs/FCM Token。
- `POST /v1/chat/keypackages`：发布一次性 OpenMLS KeyPackage。
- `GET /v1/chat/keypackages/{owner_account}`：列出有效 KeyPackage。
- `POST /v1/chat/keypackages/consume`：原子读取后硬删除 KeyPackage。
- `POST /v1/chat/envelopes`：当前请求内转发 OpenMLS 密文。
- `POST /v1/chat/signals`：当前请求内转发 SDP、ICE 和 `peer_ready`。
- `GET /v1/chat/ws`：按钱包账户和设备建立实时连接。

Durable Object 使用 hibernatable WebSocket，不写 Storage。接收设备不存在时返回 `queued`，不会创建消息记录。

## 6. D1 最小表

```text
chat_devices
  owner_account
  device_id
  device_public_key_hex
  push_provider
  push_token
  expires_at
  created_at

chat_keypackages
  owner_account
  device_id
  key_package_id
  key_package
  cipher_suite
  created_at
  expires_at

chat_device_binding_nonces
  owner_account
  nonce_hash
  expires_at
  created_at
```

Chat 禁止使用 R2。staging 和 production 已按目标结构重建，旧聊天内容表、旧迁移登记和 `chat/` 对象均已清除。

## 7. 推送与设备直连

推送载荷固定为：

```json
{
  "kind": "chat_wake",
  "sender_account": "..."
}
```

出现任何消息、会话、附件或密文字段时客户端拒绝处理。

App 使用代码中的 Firebase 公开配置初始化，不提交 `google-services.json` 或
`GoogleService-Info.plist`。以下构建参数可覆盖默认值，用于切换独立环境：

- `FIREBASE_API_KEY`
- `FIREBASE_PROJECT_ID`
- `FIREBASE_MESSAGING_SENDER_ID`
- `FIREBASE_ANDROID_APP_ID`
- `FIREBASE_IOS_APP_ID`

当前 Firebase 项目为 `citizenapp-23542`，消息发送方 ID 为 `124593150477`；
Android/iOS 应用分别登记为 `org.citizenapp`。API key、项目 ID、发送方 ID和
App ID 是 Firebase 客户端公开标识，不属于服务端密钥。

Worker 推送 Secret：

- APNs：`APNS_KEY`、`APNS_KID`、`APNS_TEAM`、`APNS_TOPIC`；`APNS_ENV` 在 staging 使用 `sandbox`，production 使用 `production`
- FCM：`FCM_PROJECT`、`FCM_EMAIL`、`FCM_KEY`

附件直连固定使用 `stun:stun.cloudflare.com:3478` 发现候选地址。该地址不是密钥，不提供流量中继，也不产生 Cloudflare Realtime 中继用量。

FCM 服务端使用专用账号
`citizenapp-push@citizenapp-23542.iam.gserviceaccount.com`，只授予
`Firebase Cloud Messaging API Admin`。staging 与 production 使用独立 Google
服务账号密钥，私钥只保存在对应 Cloudflare Worker Secret，不写入 App、仓库或
本机长期文件。

iOS 已启用 `remote-notification` 后台模式和 `aps-environment` entitlement。Android 已声明通知权限；Firebase Messaging 插件负责 FCM service。

## 8. 本地存储

- `ChatConversationEntity`：本机会话索引。
- `ChatMessageEntity`：本机消息。
- `ChatRouteCacheEntity`：对方账户、设备公钥、安全码和近场提示。
- `ChatOutboundQueueEntity`：发送设备待重试密文；送达在线设备后立即删除。
- `ChatPendingInboundEntity`：Welcome 尚未处理时的本机 application 暂存。
- OpenMLS provider storage：设备私有 MLS 状态。
- `chat/attachments/`：设备私有附件缓存。

删除某个会话只影响当前设备。注销账户时当前设备清除全部 Chat 本地数据，同时 Worker 先关闭实时连接，再硬删除设备、KeyPackage 和防重放行。

## 9. 广场边界

Chat 与广场权限分离：

- Chat：所有钱包用户可用。
- 广场浏览：必须有钱包 session；无订阅账户每日最多返回 100 条内容，有效会员不限产品浏览量。
- 广场发布：无订阅账户禁止；有效会员按四档套餐和身份权限发布。
- 防机器人、盗链和异常用量属于第 2 步，本步骤不提前加入双轨逻辑。

## 10. 当前状态

已完成：

- Worker 瞬时密文和信令转发、无内容 APNs/FCM 发送器。
- App 本机重试队列、WebSocket 立即解密、WebRTC 附件、推送后台自动收发和 Token 刷新登记。
- Protobuf、Isar 和测试生成物中的云端内容存储字段清理。
- staging Worker 已部署为 `f8fbb3e0-b5b3-4055-bf69-d0f305f4a8bb`；Access 未登录访问返回预期 302。
- staging/production D1 目标结构重建与 R2 Chat 前缀实查清空。
- Firebase 项目、Android/iOS 应用和最小权限 FCM 服务账号已创建；staging 与
  production Worker 均已配置独立 FCM Secret，OAuth 与 FCM HTTP v1 已真实鉴权通过。
- 聊天附件已统一为 STUN 辅助的设备直连；中继接口、密钥、数据表和客户端中继分支已彻底删除，Cloudflare 控制台中 staging/production 两个 Realtime 应用也已永久删除。

外部控制台待完成：

- Apple Developer APNs Key 需要在对应开发者账户中创建并写入 Worker Secret。

## 11. 验收

- `flutter analyze` 无本任务错误。
- `flutter test test/chat` 全部通过。
- Worker typecheck 和单元测试全部通过。
- production `/api/health` 返回 200；staging 未登录请求由 Access 返回 302。
- FCM 服务账号 OAuth 返回 200；FCM HTTP v1 对故意无效 Token 返回预期的 `INVALID_ARGUMENT`，排除鉴权和 API 配置错误。
- WebRTC 配置只包含公共 STUN 地址，不包含中继 URL、用户名或凭证。
- 不传 Firebase 构建参数时 Android debug APK 构建通过。
- 未登录广场和 Chat 接口返回 401。
- 无订阅 session 返回 `browse_limit=100`，额度用尽返回 429，发布准备返回 402。
- D1 不存在任何聊天内容表，R2 `chat/` 前缀为空。
- 最终真机验收必须覆盖 Android/iOS 前台直达、推送唤醒、发送设备恢复重试、WebRTC 直连成功与直连失败后本机保留重试。

## 12. 预计修改目录

- `citizenapp/lib/chat/`：Chat 运行态、本机队列、推送和设备间附件；涉及代码、注释和残留清理。
- `citizenapp/chat/proto/`：精简 Protobuf 真源与生成物；涉及协议代码。
- `citizenapp/cloudflare/src/chat/`：瞬时信令转发、无内容推送和注销清理；涉及 Worker 代码。
- `citizenapp/cloudflare/migrations/`：目标基线，不新增迁移文件；涉及 D1 基线和旧文件删除。
- `citizenapp/android/`、`citizenapp/ios/`：通知与后台能力；涉及平台配置。
- `citizenapp/lib/8964/`：钱包 session、浏览额度显示和无会员发布拦截；涉及代码。
- `memory/`：统一架构、协议、安全和任务记录；只涉及文档与旧口径清理。
