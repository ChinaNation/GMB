# CitizenApp 私密聊天技术架构

## 1. 目标边界

CitizenApp Chat 对所有已注册永久 `cid_number` 的用户开放，不依赖会员。CID 是消息、
会话、MLS 名册、设备池和投递的唯一身份键；当前绑定 `account_id` 只负责签名鉴权与
聊天内付款，`ss58_address` 只用于展示和边界输入。OpenMLS 设备密钥与钱包私钥严格分离。

数据归属固定为：

- 消息正文、会话摘要、搜索索引、MLS 状态和附件在用户设备静止时也必须是密文；短命
  明文只允许在当前操作内存或附件临时目录中存在并及时清理。Chat 路由缓存不是通讯录；
  通讯录跨设备同步由 USER 模块端侧加密后写入 D1，Chat 和 Worker 均不能读取联系人明文
  或解密密钥。
- Cloudflare 不持久化消息、会话、联系人或附件，也不提供离线内容代存。
- Cloudflare 只保存设备公钥、推送 Token、一次性 KeyPackage 和防重放哈希。
- CitizenChain runtime 不参与聊天，不记录聊天字段，不收取聊天费用。

## 2. 总体架构

```text
发送设备
  -> OpenMLS 生成 ChatEnvelope
  -> 本机 Isar 写入待发送队列
  -> Worker 校验 `cid_number + binding_revision + account_id` session 与登记设备
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
  -> 发送端 openRead 分片 + bufferedAmount 背压流式推送(整文件不进内存)
  -> 接收端分片直写临时文件(.part),运行字节计数做门控
  -> attachment_end 大小精确匹配后移入 App 私有缓存
```

WebRTC 只使用公共 STUN 发现候选地址，不配置中继服务。NAT 环境无法建立设备直连时，附件保持在发送设备并等待后续重试；Cloudflare 不接收、转发或保存附件字节。

**媒体大小与门控(单源 `chat_media_limits.dart`,单个文件上限按会员档,ADR-036:无订阅/自由 10MB、民主 100MB、薪火 5GB;与会员套餐 `chat_file_max_bytes` 同源,由 `ChatMediaLimits.applyMembershipLevel` 在 `SquareApiClient.fetchMembership` 时设置当前档,未知/失效 fail-closed 自由档)。** 因字节走 WebRTC、服务端不在字节路径,大小只考验用户网络与设备,不占 Cloudflare 资源;门控必须由收发两端各自强制,构成四门:①发送端发前硬拦(超限抛 `ChatMediaTooLargeException`)②接收端字节层按当前档定额,声明超限拒收、累积超限中止删临时(防谎报小 `byte_size` 狂发)③落盘前二次校验④渲染层对声明超限者显"已拒收"、不解析路径。被篡改的发送方无法把超限媒体塞给诚实接收方;收发两端都被改则是纯 P2P 私下传输,任何零存储设计都无法阻止(固有边界)。字节全程流式落盘,5GB 也不 OOM。**分界固定 100MB(`ChatMediaLimits.needsRelay`):≤100MB 走 WebRTC(永不触 Cloudflare 字节路径);>100MB(仅薪火)走 Cloudflare R2 瞬时中转**——客户端一次性内容密钥流式分块 AES-256-GCM 加密(`media/media_relay_crypto.dart`),K 只随 E2E 信封传、Cloudflare 只经手密文;`media/chat_relay_media.dart` 编排上传/下载,Worker `chat/relay.ts` 薪火+尺寸门 + R2 代理(拉完 ack 删 / 24h TTL 兜底)。>100MB 未配置中转即拒发,绝不降级 WebRTC。详见 `CHAT_GROUP_TECHNICAL.md §11`。

## 3. 身份与密钥

```text
聊天身份 = 永久 `cid_number`
签名授权 = CID 当前 finalized 绑定的 `account_id`
会话登录 = 当前绑定三元组下登记的硬件 P-256 设备子钥静默签名
端到端加密 = CID + OpenMLS device_id + 设备密钥 + KeyPackage
```

聊天页的用户展示复用统一 `ProfilePresentation` / `ProfileAvatar`：调用方已有公开昵称时
优先使用；名称缺失时按对方 CID 稳定选择本地默认昵称和照片。当前账户只允许显示在明确
标注的账户行，不得充当昵称、头像种子、会话身份或路由键。

通讯录联系人三点菜单的“私信”直接复用 `openDirectChat()`：`peerCidNumber` 使用联系人
永久 CID，标题使用统一公开昵称或按 CID 稳定生成的本地昵称。联系人私人
`contact_remark` 只属于通讯录，不进入 Chat 路由真源；该入口不得复制聊天页面、
会话创建或传输逻辑。

钱包主私钥只在创建热钱包时证明 P-256 设备子钥归属。Chat 运行态禁止读取 seed、调用用户认证签名或使用 CitizenWallet。

设备登记签名字段：

```text
cid_number
binding_revision
account_id
device_id
device_public_key_hex
expires_at
nonce
```

Worker 从 session 派生 CID、绑定版本和当前账户，先与同一 finalized 链快照中的当前
绑定三元组精确核对，再按 `cid_number + device_id` 查询
`square_device_subkeys.p256_public_key` 验签。签名不落库，nonce 只以 SHA-256 哈希落库，
防重放唯一保护 CID，不因钱包换绑重置。

换绑生命周期固定为：有当前账户和新账户同次签名时，交易提交前先旁路生成新绑定密文并
回读验证，finalized 后提交目标密文；没有当前账户签名时，不尝试解密或删除此前正文密文，
只把它留在此前 `binding_revision + account_id` 加密上下文中并从新绑定入口隐藏，同时清空
不能安全续用的出站队列、入站乱序缓冲、媒体补发、路由缓存和 MLS 群镜像。两种路径都由
finalized 新绑定驱动：先激活新账户派生上下文和新钱包设备子钥，再撤销不匹配 Session，
完整关闭此前 Chat HTTP/WebSocket/MLS 上下文，最后建立新绑定上下文。任何用途子钥使用后
立即清零，不设长期内存密钥缓存。

## 4. 消息协议

协议真源：`citizenapp/chat/proto/chat_envelope.proto`。

`ChatEnvelope` 只保留：

```text
protocol_version
envelope_id
conversation_id
sender_cid_number
recipient_cid_number
sender_device_id
mls_wire_message
encrypted_metadata
created_at_millis
ttl_millis
mls_message_kind
ratchet_tree
```

消息内容(即 OpenMLS application 明文载荷)由单一编解码 `citizenapp/lib/chat/chat_payload.dart`（`ChatPayloadCodec` / `ChatContent`）统一编码，采用显式 `kind` 判别，消息类型为 `text / image / video / file / sticker`：

```jsonc
{ "t":"gmb.chat.msg", "v":1, "kind":"text|image|video|file|sticker",
  "text":"…",                                    // text
  "attachment_id":"att-…","file_name":"…","mime":"…","byte_size":123,
  "width":1080,"height":1920,"duration_ms":4200,"blurhash":"…",  // image/video/file
  "pack_id":"fluent3d","sticker_id":"grinning_face" }  // sticker
```

- 收端按 `kind` 确定类型；任何非本协议或坏数据都退化为纯文本、绝不抛错，修掉了早期“内容恰好是 JSON 的文本被误判为附件”的隐患（取代旧的 `gmb_chat_attachment_v2` 启发式）。
- `image/video/file` 的字节走 WebRTC，以 `attachment_id` 关联；控制消息只带元数据。`sticker` 只承载内置贴纸包 `pack_id + sticker_id`，不传任何字节，本地资源渲染（素材定稿 Microsoft Fluent Emoji 3D，MIT）。
- 媒体字节与贴纸美术都不进入 `ChatEnvelope`，也不进入任何 Cloudflare 存储。媒体控制元数据随消息保存在本机 `ChatMessageEntity.plaintext`，不改动 Isar schema。

## 5. Cloudflare 接口

- `POST /v1/chat/devices/register`：登记设备公钥和 APNs/FCM Token。
- `POST /v1/chat/keypackages`：发布一次性 OpenMLS KeyPackage。
- `GET /v1/chat/keypackages/{cid_number}`：列出有效 KeyPackage。
- `POST /v1/chat/keypackages/consume`：原子读取后硬删除 KeyPackage。
- `POST /v1/chat/envelopes`：当前请求内转发 OpenMLS 密文。
- `POST /v1/chat/signals`：当前请求内转发 SDP、ICE 和 `peer_ready`。
- `GET /v1/chat/ws`：按 `cid_number + binding_revision + account_id + device_id` 建立实时连接。

Durable Object 使用 hibernatable WebSocket，不写 Storage。接收设备不存在时返回 `queued`，不会创建消息记录。

## 6. D1 最小表

```text
chat_devices
  cid_number
  binding_revision
  account_id
  device_id
  device_public_key
  push_provider
  push_token
  expires_at
  created_at

chat_keypackages
  cid_number
  binding_revision
  account_id
  device_id
  key_package_id
  key_package
  cipher_suite
  created_at
  expires_at

chat_device_binding_nonces
  cid_number
  nonce_hash
  expires_at
  created_at
```

CID 换绑后，Worker 以新 finalized 绑定三元组为准，先撤销旧 Session，再按 CID 删除
绑定版本或账户不等于新三元组的 `chat_devices`、`chat_keypackages` 和设备子钥，并精确
关闭旧三元组的 WebSocket。按 CID 命名的 `ChatRealtimeObject` 命名空间和 CID 级
`chat_device_binding_nonces` 继续保留；不得删除整个对象或重置 nonce。整身份注销才允许
关闭该 CID 的全部实时连接并清除 CID 级 nonce。
新设备子钥登记只有在旧实时连接关闭成功后才返回成功；Durable Object 关闭请求失败必须
返回 `503 chat_realtime_revoke_failed`，禁止把旧连接仍存活的状态伪装成换绑收敛完成。

Chat 的普通消息、会话和 ≤100MB 附件禁止使用 R2；>100MB 薪火会员附件只允许进入
`CHAT_RELAY` 瞬时密文桶并按领取确认或 24 小时生命周期删除。当前本地 Cloudflare
业务状态已按最终基线重建；staging 和 production 的旧 D1/KV/R2 状态仍待远端不可恢复
清理的单独确认，文档不得提前宣称远端已重建。

## 7. 推送与设备直连

推送载荷固定为：

```json
{
  "kind": "chat_wake",
  "sender_cid_number": "..."
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

- `ChatConversationEntity`：按 `ownerCidNumber + conversationId` 唯一的本机会话索引；
  `bindingRevision + accountId` 只标识其密文加密上下文，不是会话归属主键。
- `ChatMessageEntity`：按 `ownerCidNumber + envelopeId` 唯一的本机消息；信封只记录发送方和
  接收方 CID，读取和搜索只接受当前 finalized 绑定精确匹配的密文上下文。
- `ChatRouteCacheEntity`：按属主 CID 分区，保存对方永久 CID、设备公钥、安全码和近场提示。
- `ChatOutboundQueueEntity`：按属主 CID 分区的发送设备待重试密文；送达在线设备后立即删除。
- `ChatOutgoingMediaEntity`：按属主 CID 分区的待设备投递媒体；收件人路由键为永久 CID。字节留本机缓存，对方上线时按当前 Documents 目录重算缓存路径，不持久化绝对路径，收到 ack 后删行。
- `ChatPendingInboundEntity`、群镜像和乱序 Commit：全部按属主 CID 分区；MLS 名册只使用 CID。
- OpenMLS provider storage：
  `chat/by_cid/<owner_cid_number>/by_binding/<binding_revision>/<account_id>/mls/<device_id>/`。
- 媒体密文缓存：
  `chat/by_cid/<owner_cid_number>/by_binding/<binding_revision>/<account_id>/attachments/`；
  接收临时文件位于该绑定分区的短命明文目录，完整校验后封存，拒收或截断时删除。有签名
  交接提交后，已重加密文件树原子移动到新绑定目录；无签名交接时此前目录原样保留且不会
  被新账户读取。

删除某个会话只影响当前设备。注销身份时当前绑定 `account_id` 只完成授权；Worker 以
`cid_number` 为唯一删除主键，先关闭该 CID 实时连接，再按 CID 硬删除跨历次换绑账户的
全部设备、KeyPackage、防重放行和会话。会话 token 只以 SHA-256 哈希进入 KV 键与 D1
强一致 CID 注销索引，不依赖 KV 前缀枚举。当前设备随后清除全部 Chat 本地数据。

## 9. 广场边界

Chat 与广场权限分离：

- Chat：所有已注册 CID 用户可用，当前绑定账户只负责鉴权。
- 广场浏览：必须有钱包 session；无订阅账户每日最多返回 100 条内容，有效会员不限产品浏览量。
- 广场发布：无订阅账户禁止；有效会员按三档套餐发布（ADR-036，会员与身份解耦）。发帖分类权限按身份档（竞选内容须竞选身份 `candidate`）、用量额度按会员档，二者互不影响（用户 2026-07-16）。
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
- 媒体/表情三步升级第 1 步(消息载荷地基,2026-07-15)：消息类型由 `{text,attachment}` 扩为 `{text,image,video,file,sticker}`；新增单源编解码 `chat_payload.dart` 取代 `_messageKindFromPlaintext` 启发式；图片/视频/文件端到端往返与按类型内联渲染打通（`chat_ui_adapter` 分发 + `chat_page` 媒体 builder），图片内联展示、文件/视频点按另存。proto / 传输 / Cloudflare / Isar schema 零改动，字节仍只走 WebRTC。相册-相机采集、离线字节重试、emoji 面板与 Fluent 3D 贴纸在第 2、3 步。
- 媒体升级第 2a 步(大小门控 + 流式字节管道,2026-07-15)：上限定稿**图片 100MB、视频/文件 5GB**(单源 `chat_media_limits.dart`);字节管道由"整文件进内存"改为**流式落盘 + `bufferedAmount` 背压**(`ChatMediaDraft` 携路径、`ChatAttachmentReceiveBuffer` 分片直写临时文件),5GB 不 OOM;**收发双端四门门控**(发前拦/字节层拒收/落盘二次/渲染拒收占位)全部落地并有单测。用 `dart:io` 流式 + `flutter_webrtc` 背压等现成原语,2a 零新依赖。crypto/proto/Cloudflare/Isar 仍零改动。
- 媒体升级第 2b 步(采集与压缩探测,2026-07-15)：新增 `lib/chat/media/`(采集 `media_picker`/压缩门控 `media_compressor`/宽高时长 blurhash 探测 `media_probe`/mime `media_mime`)与 `compose/media_source_sheet`(相册图/拍照/相册视频/录像/文件)。策略:**图片仅超限才压缩**(正常图原样,`flutter_image_compress`)、**视频不转码超限拒**;blurhash 一律由**原生降采样小缩略图**编码(图 `flutter_image_compress`、视频 `video_compress` 抽帧 → `blurhash_dart`),宽高走 `image_size_getter` 读头,**绝不 Dart 侧整解码 100MB 原图**。新增依赖 image_size_getter/flutter_image_compress/video_compress/blurhash_dart/image;iOS 加麦克风、Android 加 READ_MEDIA_VIDEO/RECORD_AUDIO 权限。native 采集/压缩/抽帧经可注入 seam,编排与 blurhash 编码有单测。字节仍走 2a 流式管道,crypto/proto/Cloudflare/Isar 零改动。全渲染(blurhash 占位真渲染/全屏/播放)与存相册、离线补发在 2c/2d。
- 媒体升级第 2d-1 步(离线字节补发,2026-07-15)：`sendMedia` 重排——**加密 → 控制消息先离线安全落库/入队(和文字一样)→ 自存缓存 + 登记待设备投递 → 尝试 WebRTC 字节;失败(对方离线)不抛错**,给离线对端发媒体不再整体失败;零泄漏顺序(加密先于发字节)保持。新增 Isar 集合 `ChatOutgoingMediaEntity` 持久化"待设备投递"(存 conversationId 不存绝对路径,补发按当前 Documents 目录重算);`retryOutgoing`(peer_ready 触发)在重发控制 envelope 后、**仅当对端有账户时**调补发核心 `MediaResend.run`(可测纯核心,与 WebRTC/文件系统解耦)——`_mediaBytesInFlight` 在途去重(防初始发送与 peer_ready 补发对同一 attachmentId 双传),缓存副本存在则重发 WebRTC 字节、ack 后删行,副本已删则清孤儿,仍失败保留待下次,App 重启后仍能补发。Isar 是媒体升级唯一 schema 变更(dev 零用户直接重建)。crypto/proto/Cloudflare 零改动。经对抗式审查 8 项(关键:持久绝对路径失效、双传去重、无账户对端守卫、补发可测化)全部落地,`test/chat` 98 通过/4 跳过。分片断点续传见 2d-2。
- 媒体升级第 2c 步(呈现升级,2026-07-15)：blurhash 占位真渲染(`flutter_blurhash`)、图片全屏(`viewer/image_viewer_page`,`InteractiveViewer` 捏合缩放)、视频播放(`viewer/video_player_page`,`video_player`)、存系统相册(`media/media_gallery_saver` 封 `saver_gallery`)。内联图与全屏图一律按显示尺寸 `cacheWidth` 降采样解码,100MB 图不整解码位图。adapter 给 image/video 控制补 `file_name`、video 补 `blurhash` 供渲染/存相册。存相册流程可注入 saver 有 widget 测(图片真解码在 widget 测会挂 fake-async,以真机为准);视频播放依赖 native controller,真机验证。新增依赖仅 flutter_blurhash。crypto/proto/Cloudflare/Isar 零改动。离线字节补发与分片断点续传在 2d。
- 表情贴纸第 3a 步(Fluent 3D 贴纸,2026-07-15)：从 `microsoft/fluentui-emoji`(MIT)精选 48 张 3D PNG 内置 `assets/stickers/fluent3d/`(四类:表情/手势/爱心/庆祝);清单单源 `stickers/sticker_pack.dart`(`packId='fluent3d'` + `assetPath/isKnown/grouped`,**不落 manifest.json** 免双源漂移,测试反核对清单⇔磁盘)。贴纸**零字节/零 WebRTC/零云存储**:只把 `(packId,stickerId)` 塞进 MLS 明文信封(传输层步骤1已备好 `sendSticker`),收端按 id 查本地内置 PNG 渲染;未知 id 经 `isKnown` 白名单门降级占位 `[贴纸]`(亦挡 `sticker_id` 路径穿越)。`chat_ui_adapter` sticker 分支改 `Message.custom`;`chat_page` 经 `composerBuilder` 自绘 `Composer`(发送/附件仍走 Chat 注入回调)+ `topWidget` 挂贴纸开关与 `StickerPanel`(分类 Tab 网格、`Image.asset` 带 errorBuilder、高度按视口夹取);`_buildStickerMessage` 白名单门渲染无气泡大图 + 降级;两处入口 `chat_tab`/`open_direct_chat` 接 `runtime.sendSticker`。**关键取舍**:重载消息不复用全屏 `_loading` 骨架(否则整块 Chat 连贴纸面板 unmount、连发时分类 Tab 归零)。经对抗式审查 7 项(行为回归/响应式/渲染与接线测试/一致性)全部落地。零新依赖(纯 `Image.asset`)。crypto/proto/Cloudflare/Isar 零改动。emoji 表情面板 = 3b,待做。
- 表情贴纸第 3b 步(emoji 表情面板,2026-07-15)：新增 `emoji_picker_flutter ^4.4.0`(离线 emoji 数据,**无网络/无遥测**;仅 shared_preferences 存最近使用)。`EmojiPicker(textEditingController: _composerController)` 把选中 Unicode emoji 直接插到 composer 光标处,随文本走现有 `sendText`——**零协议变更、零新增数据面**(emoji 即文本)。`chat_page` 的 `_stickerPanelOpen` 布尔收敛为 `enum _ComposerPanel{none,emoji,sticker}` + `_togglePanel`(表情/贴纸互斥、打开收键盘),工具条两键(`chat-emoji-toggle`/`chat-sticker-toggle`),emoji 面板高度按视口夹取。经对抗式审查确认 1 项(补端到端:共享 controller 断言 + 文本经 onSendText 发出)、驳回 4 项(库 dispose 无害 no-op、recents 非 PII、反向互斥对称)。crypto/proto/Cloudflare/Isar 零改动。至此本升级三步(载荷/媒体/表情贴纸)全部完成。
- 媒体升级第 2d-2 步(分片断点续传,2026-07-15)：WebRTC 附件协议加一次**续传握手**——`attachment_start` 后接收端回 `attachment_resume{resume_offset}`(本地同 `attachment_id` 的 `.part` 已存字节数),发送端 `openRead(offset)` 只补缺口。接收缓冲 temp 改按 **`<attachment_id>.part`** 命名(跨尝试复用)、append 追加写、`dispose` 改为**保留 partial**(`_closeSink`,断点续传核心;拒收/超限/大小不符仍删档);启动 `sweepStalePartials` 清 mtime 超 7 天残档。完整性口径 = 精确大小校验 + attachmentId 内容不可变 + SCTP 有序可靠 + 偏移取磁盘实际大小(**用户定稿不加 sha256**)。经对抗式审查修 5 项:接收端 `onDataChannelState` Closed 收口(否则同会话补发双开 sink 污染 partial)、发送端 clamp 对端 `resume_offset` 到 `[0,byteSize]`(负值 `openRead` 抛 `RangeError` 逃逸 `on Exception`)、`_closePeer` 先 `await peer.tail` 再关流避免与 `finish` 竞态、+2 边界测;另修 `sendAttachment` 超时泄漏 peer(try/finally 幂等 `_closePeer`)。与 2d-1 `MediaResend` 衔接补缺口。crypto/proto/Cloudflare/Isar 零改动,维持零存储。`test/chat` 118 通过/4 跳过。

- 聊天页入口改造(顶栏搜索框 + 加号 5 入口,2026-07-23)：`chat_tab.dart` 顶栏由「装饰性搜索图标 + 新建群聊卡片」改为「**搜索框 + 右上角加号**」。加号弹出 5 项,分派枚举 `_ChatEntryAction`,全部复用既有链路,聊天页零重复实现:扫一扫→`openScanDispatchFlow(paymentWallet: 默认钱包)`;收付款→全 App 唯一 `UserQrPage`;发私信→`ContactBookPage(mode: pickForMessage)`;发群聊→既有 `GroupCreatePage`(原卡片职能迁入菜单);加好友→`scanAndAddContact`。可注入 `ChatEntryOpeners`(5 个 `ChatEntryOpener = Future<void> Function(BuildContext)`)供测试断言路由;**先查注入再解析钱包**,否则注入替身时仍会先摸真实 `WalletManager` 碰存储。
- 加号弹窗样式(2026-07-23)：淡深色面板(`0xF01F2A30`)+ 白色图标文案 + **顶部凸出三角,顶点对齐加号中心**。**未用 `PopupMenuButton`** —— 它的水平位置由框架按可用空间决定，拿不到确定锚点，三角只能靠猜偏移量。改为 `showGeneralDialog` 自绘：经 `Builder` 取加号按钮自身 `RenderBox` 的屏幕坐标算出中心 X，反推面板左边界并夹到屏内，再用夹取后的实际左边界回算三角横向位置，保证仍对准加号。`barrierColor` 透明(不压黑整屏)。**面板本身必须是 `Material`** —— 弹窗不在 Scaffold 之下，`InkWell` 缺 Material 祖先会直接抛异常(此坑由 widget 测试拦下)。
- 用户二维码唯一化 + 扫码场景分流(2026-07-23，2026-07-28 CID 收口)：全 App 只有
  **一张**用户二维码(`QR_V1 k=3` userContact 名片码 = 钱包 SS58 + 钱包标签)，真源
  收敛到 `8964/profile/user_qr_page.dart`。扫码结果由扫描模式决定：`contact` 模式
  只接受 userContact，先由 SS58 派生账户并链读双向绑定 CID 后加入通讯录，钱包标签
  不写入私人备注；`transfer`/`dispatch` 模式把 userContact 作为收款人进入转账。
  缺少 CID 的 userTransfer 收款码不得再兼作联系人码。**不新造带金额收款码**。
- 聊天搜索页（2026-07-23，2026-07-31 当前钱包派生订正）：一个输入框分会话、联系人、聊天记录三段结果。会话按标题/最近消息搜索；联系人只在本地匹配私人备注、CID 和 finalized 当前账户快照，打开私信时始终传联系人 CID。聊天记录由 `ChatStore.searchMessages({ownerCidNumber, currentAccountId, keyword, limit})` 查询：`ownerCidNumber` 是唯一数据分区与索引归属，`currentAccountId` 用于从当前绑定账户 child 直接派生 Chat 与索引用途子钥；候选消息解密后再以 `ChatPayloadCodec.decode(...).summary` 复验真实子串，避免截断 HMAC token 假阳性。点结果统一复用 `openGroupChat` / `openDirectChat`，聊天记录命中只打开所在会话，不另造账户身份路径。

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
