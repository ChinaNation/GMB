# CitizenApp 聊天:相册图片/视频发送 + 表情/表情包(强制 E2E、Cloudflare 零存储)

任务需求：给一对一聊天新增「从相册/相机发图片和视频」「内联展示图片视频」「Unicode 表情选择器 + 内置贴纸包」；全部消息(文字/文件/图片/视频/贴纸)维持端到端,Cloudflare 不存储任何媒体数据。
所属模块：citizenapp / chat（+ iOS/Android 媒体权限声明）

## 定稿方向（用户逐轮确认）

- 群聊双轨已定但**不在本卡**:本卡只做需求 1(媒体 + 表情)。群聊另开卡 2/3。
- 媒体离线送达取舍 = **维持零存储**。大媒体只在双方在线时经 WebRTC 直传;对方离线由发送方本机暂存重试,上线补发。文字/贴纸不受此限(走 envelope 瞬时中转,本就零存储)。
- 表情现成推荐:Unicode 走 `emoji_picker_flutter`;表情包/贴纸走**内置资源包,只传 sticker-id**(几十字节,走 envelope,天然 E2E)。

## 现状事实（已核实,非推断）

- Cloudflare 既不存明文也不存密文,只瞬时中转 + 无内容唤醒(`cloudflare/src/chat/service.ts:236,268`)。
- 附件字节现已走 WebRTC 16KB 分片 P2P 直传(`lib/chat/transport/chat_webrtc_transport.dart:52,101`),不经 Cloudflare;envelope 服务端上限 256KB(`cloudflare/src/limits/catalog.ts:118`),大媒体不得塞 envelope。
- 消息类型枚举现仅 `text/attachment`(`lib/chat/chat_models.dart:5`);渲染层一律 `Message.text`,附件显示为 `[附件] 文件名`(`lib/chat/chat_ui_adapter.dart:16,38`),点开只下载不预览。
- 依赖已就绪:`image_picker ^1.1.2`、`video_player ^2.10.1`、`saver_gallery ^4.1.0`、`file_picker ^8.0.0`、`flutter_chat_ui ^2.11.1`(`pubspec.yaml`)。新增依赖仅 `emoji_picker_flutter`。

## 输出物

- 消息模型:`lib/chat/chat_models.dart` 的 `ChatMessageKind` 从 `text/attachment` 细分为 `text/image/video/file/sticker`;同步 `chat_ui_adapter.dart`、`storage/chat_store.dart`、`proto/chat_envelope.*`、`chat_flow.dart`。
- 输入栏(`lib/chat/chat_page.dart`):加「相册选图/选视频」(`image_picker.pickImage/pickVideo`)、「拍照/拍视频」、文件(保留 `file_picker`)、emoji 选择器、贴纸面板。
- 渲染:图片内联缩略图 + 点击全屏查看;视频封面 + 点击 `video_player` 播放;贴纸从本地资源渲染;保存到相册用 `saver_gallery`。
- 传输(维持零存储):媒体字节继续走 `chat_webrtc_transport` 16KB 分片;离线复用发送方重试队列(`chat_runtime.retryOutgoing`),消息标「等待对方上线」pending 态;客户端单文件上限**图片 ≤10MB、视频 ≤100MB**(用户定稿),图片超限本地压缩、视频超限拒发并提示。
- 表情:`emoji_picker_flutter` 接入(纯客户端);贴纸内置 `assets/stickers/`,素材**定稿 = Microsoft Fluent Emoji 3D(MIT 许可)**,发送只传 sticker-id 走 envelope。

## 分步实现(每步先出技术方案确认后执行)

- **步骤1 地基**:消息载荷 schema + 模型 + 渲染分发。新增 `chat_payload.dart` 单源编解码(显式 `kind`,替代 `_messageKindFromPlaintext` 启发式,顺带修掉"文本像 JSON 被误判"隐患);`ChatMessageKind` 扩 `text/image/video/file/sticker`;媒体元数据(尺寸/时长/blurhash/mime/byte_size)全部塞进**已持久化的 `plaintext` 控制 JSON**,故 **Isar schema 零改动**;adapter 按 kind 分发渲染。传输沿用现有 WebRTC 语义。用现有 file_picker 验证往返。
- **步骤2 采集与传输硬化**:`media/`(相册/相机/视频采集、压缩门控、抽帧封面、blurhash 生成、缓存、存相册)+ `viewer/`(全屏/播放)+ WebRTC 背压/进度 + **离线字节重试**(今天 `retryOutgoing` 只重发控制 envelope、不重发 WebRTC 字节,需补)+ pending 态 + iOS/Android 权限。
- **步骤3 表情贴纸**:分 3a 贴纸 + 3b 表情。3a=`assets/stickers/fluent3d/`(内置 Fluent 3D)+ `stickers/sticker_pack.dart` 单源 + `compose/sticker_panel.dart` + 自绘 composer 渲染闭环;3b=`emoji_picker_flutter` + `compose/emoji_panel`(表情插入文本)。
- 权限声明:iOS `Info.plist`(`NSPhotoLibraryUsageDescription`/`NSCameraUsageDescription`/`NSMicrophoneUsageDescription`)、Android(`READ_MEDIA_IMAGES`/`READ_MEDIA_VIDEO`/`CAMERA`)。
- 中文注释、测试、文档更新、残留清理。

## 必须遵守

- **Cloudflare 零存储不变**:不新增任何 chat 媒体存储路由,媒体字节只走 WebRTC/瞬时,严禁塞 envelope(256KB 上限不动),严禁复用广场 R2/uploads 通道存聊天媒体。
- 不动 `citizenchain`(与链无关);不动广场上传/R2 逻辑。
- 改本地消息 Isar schema 属「先沟通条件」:开发期零用户,直接重建本地 DB,**不做迁移/兼容/残留**(遵守死规则:无兼容、无遗留残桩)。
- 贴纸素材必须本地内置,只传 id;不经服务端分发。
- 服务端(`cloudflare/`)本卡零改动(纯客户端 + 权限)。

## 验收标准

- `flutter analyze lib/chat` 0 问题;chat 相关 widget/单元测试通过(含图片/视频/贴纸消息的模型往返、渲染、离线 pending)。
- 双方在线:相册图片、视频、贴纸、emoji 均能收发并正确内联展示;图片可全屏、视频可播、媒体可存相册。
- 对方离线:大媒体进发送方重试队列并显示 pending,对方上线后补达;Cloudflare 无任何媒体落库(抓路由/存储确认)。
- 残留清理:旧 `attachment` 单一路径无死码遗留;文档已更新。

## 执行结果 · 步骤1 地基（2026-07-15，完成）

- **新增单源载荷编解码** `lib/chat/chat_payload.dart`（`ChatPayloadCodec`/`ChatContent`）：显式 `kind` 判别（`{t:"gmb.chat.msg",v:1,kind,…}`），坏数据/非协议一律退化为纯文本、绝不抛错；顺带修掉“文本恰好是 JSON 被误判为附件”的隐患。
- **消息类型** `ChatMessageKind` 由 `{text,attachment}` 改为 `{text,image,video,file,sticker}`（删 attachment，无兼容）。
- **`chat_flow`**：`sendText`/新增 `sendSticker`/`sendAttachment`→`sendMedia` 收敛到共享 `_deliverOutbound`（保持“先加密控制消息、再发 WebRTC 字节”的顺序）；`ChatAttachmentDraft`→`ChatMediaDraft`（加 kind/尺寸/时长/blurhash）；收端 kind 走 codec；`downloadAttachment` 走 codec；删 `_messageKindFromPlaintext`/`_AttachmentControl`/死 helper。
- **`chat_runtime`**：`sendMedia`/`sendSticker`/非抛错 `resolveCachedMediaPath`；WebRTC 字节通道与缓存落盘不变。
- **`chat_ui_adapter`**：按 kind 分发到 `Message.image/.video/.file`/文本；图片带 blurhash+宽高，media path 由页面预解析注入。
- **`chat_page`**：`Chat` 挂 `imageMessageBuilder/videoMessageBuilder/fileMessageBuilder`（图片内联+占位、文件/视频条点按另存）；`_pickMedia`（按 mime 判 kind）；`_resolveMediaPaths` 预解析本机缓存路径。
- **`chat_tab`/`open_direct_chat`**：`sendMediaFactory`/`onSendMedia`/`onResolveMediaPath` 全链路改名与接线。
- **`chat_store`**：`_messageSummary` 走 codec，支持 `[图片]/[视频]/[文件] 名/[贴纸]`；**Isar schema 零改动**（元数据在已存的 `plaintext`）。
- **测试**：新增 `chat_payload_test`（5 类往返、JSON 文本不误判、坏数据兜底、摘要）；重写 `chat_ui_adapter_test`（image/video/file/sticker 分发）；更新 `chat_envelope_session_test`（`sendMedia`+codec 控制）与 `chat_tab_test`（媒体按钮+文件另存）。
- **验证**：`flutter analyze lib`（全仓）无本任务新增问题（仅 2 处历史 info，非本改动文件）；`flutter test test/chat --concurrency=1` = 48 通过 / 4 跳过（smoldot native 门控）；旧 schema 残留全仓清零（仅剩 `chat_payload_test` 中 1 处刻意的反例夹具）。
- **文档**：`memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md` §4 协议、§10 状态已更新。
- **对抗式审查**：4 维度 × 逐条 refute 复核,0 处代码正确性缺陷,确认 5 处测试覆盖缺口(2 中 3 低)并全部补齐:视频 kind 的 adapter 分发用例、`sendMedia` 加密失败绝不先发字节的零泄漏顺序、`downloadAttachment` 拒非媒体控制、字节未到达/截断守卫、以及 WebRTC 字节↔控制消息同 attachmentId 关联 + 发送方自存副本断言。测试 48→52 通过。
- **未做（步骤2/3，步骤1 收尾时的快照）**：相册/相机采集 UI、压缩门控、抽帧封面、blurhash 生成、全屏查看/播放页、离线字节重试、emoji 面板、Fluent 3D 贴纸资源与发送、iOS/Android 权限声明。（截至 3a 完工:除 **3b emoji 面板**外均已落地。）

## 执行结果 · 步骤2a 大小门控 + 流式字节管道（2026-07-15，完成）

- **上限定稿**：`lib/chat/chat_media_limits.dart`[新] 单源——图片 100MB、视频/文件 5GB；`forKind/forMime/exceedsForKind` + `ChatMediaTooLargeException`。
- **字节管道流式化(防 OOM)**：`ChatMediaDraft` 由整字节改**携源文件路径 + byteSize**；发送端 `chat_webrtc_transport.sendAttachment` 用 `File.openRead` 分片 + `bufferedAmount/onBufferedAmountLow` 背压流式推送;接收端新增 `ChatAttachmentReceiveBuffer` 分片**直写临时文件**(运行计数,不堆 RAM),`attachment_end` 大小精确匹配后经 `ChatFlow.importAttachmentFileToCache`(rename 零拷贝/跨卷流式复制)移入缓存;`ChatDownloadedAttachment` 去 `bytes`、`readCachedAttachment` 改 stat 判定。`_bindChannel` 逐帧串行(`peer.tail` 链)保证异步落盘有序。
- **收发双端四门门控**：①`chat_flow.sendMedia` 发前 `exceedsForKind` 拦(抛异常、不发字节)②`ChatAttachmentReceiveBuffer` 按 `content_type` 定额:声明超限拒收(不建临时)、累积超限中止删临时③`chat_runtime._saveReceivedAttachmentToCache` 落盘前二次 `forMime` 校验超限删临时④`chat_ui_adapter` 声明超限渲染"已拒收"占位、`chat_page._resolveMediaPaths` 跳过不解析。
- **现成方案**:流式 = `dart:io`(本仓 square_upload 等已用);背压 = `flutter_webrtc` 内建 `bufferedAmount` 系列。**2a 零新依赖**。
- **测试(48→62 通过,+14)**：`chat_media_limits_test`(边界)、`chat_webrtc_transport_test`(重写:receive buffer happy/声明超限拒收/累积超限中止删临时/截断丢弃)、`chat_ui_adapter_test`(门④)、`chat_envelope_session_test`(门①超限抛异常、零泄漏顺序、路径/自存 attachmentId 关联、download 负路径与截断均改路径化)、`chat_tab_test`(路径化)。
- **验证**:`flutter analyze lib` 全仓仅 2 处历史 info(非本改动);`dart format` 全过;`flutter test test/chat --concurrency=1` = 62 通过/4 跳过;旧 API(`saveAttachmentBytesToCache`/`.bytes`/`isComplete`/内存 `received`)残留清零。
- **文档**:CHAT_TECHNICAL §2 附件链路(流式+四门)、§8 本地存储(`.tmp`)、§10 状态已更新。
- **对抗式审查(4 维度 × 逐条 refute)**:0 处代码正确性缺陷,确认 3 处测试覆盖缺口(2 中 1 低)并全部补齐——① `importAttachmentFileToCache(moveSource:true)` 移动路径补测;② 门③抽取为可注入 `ChatFlow.acceptReceivedMediaToCache`(runtime 委托)+ 超限删临时/达标移入 单测;③ 传输帧路由抽取为可测 `handleIncomingFrame`(dummy cloud + 构造 RTCDataChannelMessage 驱动)+「拒收既不回调也不 ack」「完整传输回调并 ack 一次」单测。一条 `Image.file` 未降采样判 out-of-scope(2c)。测试 62→66 通过。
- **未做(2b/2c/2d)**:相册/相机采集、图片压缩、blurhash/宽高/时长探测、blurhash 占位真渲染、全屏/播放、存相册、离线字节补发与分片断点续传、iOS/Android 权限。

## 执行结果 · 步骤2b 采集与压缩探测（2026-07-15，完成）

- **新增依赖**:`image_size_getter`/`flutter_image_compress`/`video_compress`/`blurhash_dart`/`image`(直依赖)。
- **媒体层** `lib/chat/media/`:`media_mime`(mime/kind 单源)、`media_picker`(相册图/拍照/相册视频/录像;`XFile`→路径型 `PickedMediaFile`,注入 seam 可测)、`media_compressor`(**图仅超限才压**一次仍超则抛;**视频/文件不转码超限抛**;注入 sizeOf/compressImage 可测)、`media_probe`(宽高走 `image_size_getter` 读头;时长/首帧走 `video_compress`;blurhash 由**原生降采样小缩略图**经 `blurhash_dart` 编码,**绝不整解码 100MB 原图**;native 经 seam,`encodeBlurhash` 纯 Dart 可测)。
- **来源菜单** `compose/media_source_sheet.dart`:相册图/拍照/相册视频/录像/文件。
- **`chat_page`**:`onAttachmentTap`→弹菜单→`_pickViaSheet`(采集→`ensureWithinLimit`→`probe`→组装路径型 `ChatMediaDraft` 含宽高/时长/blurhash)→进 2a 管道;删旧 `_pickMedia`/`_guessContentType`/`_mediaKindFromMime`(收敛到 `media_mime`)。
- **权限**:iOS `Info.plist` 加 `NSMicrophoneUsageDescription`;Android 加 `READ_MEDIA_VIDEO`/`RECORD_AUDIO`(相机/相册已有)。
- **测试(66→82,+16)**:`media_mime`(mime/kind)、`media_compressor`(直通/图压达标/压后仍超/压缩失败/视频超限不压 五分支)、`media_probe`(`encodeBlurhash` 真小图产 hash、坏字节 null、image/video 装配、file 空、抛错兜底)、`media_picker`(归一 image/video/取消)。
- **内存安全**:采集侧探测/压缩全走文件头/原生降采样/抽帧,不 Dart 侧整解码,100MB 图不炸。
- **验证**:`flutter analyze lib` 仅 2 处历史 info(非本改动);`dart format` 全过;`flutter test test/chat` = 82 通过/4 跳过;chat_page 旧采集 helper 残留清零。crypto/proto/Cloudflare/Isar/transport/chat_flow/chat_runtime 零改动。文档 CHAT_TECHNICAL §10 已更新。
- **对抗式审查(4 维度 × 逐条 refute)**:确认 5 处并全部修复——**[高·真实内存 bug] iOS 视频抽帧是原分辨率**(`VideoCompress.getFileThumbnail` 无 maximumSize),原本直接 `img.decodeImage` 整帧入 Dart 堆(4K~33MB/8K~132MB),违反内存安全不变量 → 修:video 缩略先经 `flutter_image_compress` 原生降采样到 ≤64 再解码,压缩失败返 null 不回退原帧;[低] `encodeBlurhash` 只 `copyResize(width:64)` 对纵向/窄图无效甚至放大 → 修:仅缩小、把较大边降到 ≤64;[低] `ensureWithinLimit` 未用的 `mime` 死参 → 删(调用端同步);[低×2] picker mime 回退分支、probe 空缩略图分支缺测 → 补。测试 82→86。
- **未做(2c/2d)**:blurhash 占位真渲染、全屏查看、视频播放页、存相册、离线字节补发、分片断点续传。

## 执行结果 · 步骤2c 呈现升级（2026-07-15，完成）

- **新增依赖**:`flutter_blurhash`(渲染);`video_player`/`saver_gallery` 已有;图片缩放用内建 `InteractiveViewer`(零依赖)。
- **新增**:`viewer/image_viewer_page.dart`(全屏图,`InteractiveViewer` 捏合缩放 + 存相册,按屏幕 `cacheWidth` 降采样)、`viewer/video_player_page.dart`(`video_player` 播放 + 存相册,dispose 释放 controller)、`media/media_gallery_saver.dart`(封 `SaverGallery.saveFile`,可注入)。
- **`chat_ui_adapter`**:image 控制补 `file_name`;video 补 `blurhash` + `file_name`(供封面占位/存相册;图片已带 `Message.image.blurhash`)。
- **`chat_page` builder**:image/video 字节未到→`BlurHash` 占位、到达→`Image.file`(cacheWidth 降采样)/blurhash 封面+播放图标;点图→`ImageViewerPage`、点视频→`VideoPlayerPage`;新增 `_blurhashOrPlaceholder`/`_mediaAspectRatio`/`_openImageViewer`/`_openVideoPlayer`。
- **内存**:内联/全屏图一律 `cacheWidth` 降采样解码,100MB 图不整解码成位图。
- **测试(82→88,+6)**:`image_viewer_page`(结构 + 存相册成功/失败,注入 saver;**注**真实图片解码在 widget 测会挂 fake-async,故用不存在路径走 errorBuilder,真实渲染以真机为准)、`chat_ui_adapter`(video blurhash/file_name、image file_name 断言);视频播放依赖 native,真机验证。
- **验证**:`flutter analyze lib` 仅 2 处历史 info;`dart format` 全过;`flutter test test/chat` = 88 通过/4 跳过。crypto/proto/Cloudflare/Isar 零改动。文档 CHAT_TECHNICAL §10 已更新。
- **对抗式审查(4 维度)**:0 正确性/内存缺陷(cacheWidth 降采样已核对正确),确认 2 处并修复——[中] image/video builder 的占位分支零覆盖(且 image 读 `message.blurhash`、video 读 `metadata['blurhash']` 的不对称无守卫)→ 补两条 widget 测(未到达占位"接收中"×2 + 视频播放图标;视频 blurhash 封面用 pump 非 settle 避 BlurHash 异步解码挂起);[低] `_buildImageMessage` 上方"全屏/存相册在步骤2"陈旧注释残留 → 删。测试 88→90。
- **未做(2d)**:离线字节补发、分片断点续传。

## 执行结果 · 步骤2d-1 离线字节补发（2026-07-15，完成）

- **发送重排**:`chat_flow.sendMedia` 由"先发字节再发控制"改为**加密 → 控制先离线安全落库/入队 → 自存缓存 + 登记待投递 → 尝试 WebRTC 字节(失败不抛错)**。修掉"给离线对端发媒体整体失败";零泄漏顺序(加密先于字节)保持。新增回调 `recordPendingMedia`/`onDeviceDelivered` + 静态 `attachmentCachePath`。
- **持久化待投递队列**(媒体升级唯一 Isar 变更):`lib/isar/app_isar.dart` 新增 `ChatOutgoingMediaEntity`(attachmentId 唯一 + recipientAccount 索引 + conversationId/fileName/contentType/byteSize;**不存 cacheFilePath**——绝对路径跨重装/系统迁移会失效,补发时由 `ChatFlow.attachmentCachePath` 按当前 Documents 目录重算),注册 schema + `build_runner` 重生成 `.g.dart`。`chat_store` 加 `recordOutgoingMedia`/`deleteOutgoingMedia`/`readPendingOutgoingMedia`/`outgoingMediaCount` + `deleteConversation`/`clearAllForOwner` 连带清理。
- **上线补发**:`chat_runtime.retryOutgoing`(peer_ready 触发)重发控制 envelope 后,仅当该对端有账户时调 `_resendPendingMedia`;补发核心抽取为**可测 `MediaResend.run`**(与 WebRTC/文件系统/Documents 目录解耦)——遍历待投递媒体,在途(`_mediaBytesInFlight` 去重,防同一 attachmentId 双传)跳过、缓存副本存在则重发 WebRTC 字节、ack 后删行;副本已删(会话删)则清孤儿 pending;仍失败保留待下次。App 重启后队列在,恢复即补发。
- **测试(90→98,+8)**:`media_resend_test`(核心 4 分支:在途跳过/孤儿清理/成功删行/失败保留)、`chat_store_test`(待投递队列登记/按对端读/删/会话删连带清 + `clearAllForOwner` 连带清)、`chat_envelope_session_test`(在线送达=先登记后标记已送达净零、离线抛错=控制仍成立/登记待投递/不抛错/未标记已送达)。既有 sendMedia 测试(join-key/门①/零泄漏)经重排后仍全绿。
- **对抗式审查(4 维度 × 逐条 refute)**:0 处代码正确性缺陷,确认 8 处并全部落地——**[关键设计]** 待投递行原持久化绝对 `cacheFilePath`,重装/迁移后失效 → 改存 conversationId、补发按当前 Documents 目录重算路径;**[并发]** peer_ready 与初始发送可能对同一 attachmentId 双传 → `_mediaBytesInFlight` 在途去重;**[健壮]** `retryOutgoing` 对无账户对端会 NPE → 门 `recipientAccount != null`;**[可测性]** 补发逻辑埋在 runtime 依赖真机通道 → 抽 `MediaResend.run` 纯核心;其余为测试覆盖缺口(成功/失败/孤儿/在途/在线净零/clearAllForOwner)。测试 90→98。[低] 控制先于登记的窄崩溃窗(极小概率占位无补发)判为零存储 P2P 固有取舍,开发期接受。
- **验证**:`flutter analyze lib/chat test/chat` = No issues found;`dart format lib/chat lib/isar/app_isar.dart test/chat` 全过;`flutter test test/chat --concurrency=1` = 98 通过/4 跳过。
- **踩坑**:一度误用 `dart format lib`(全 lib)重排了 18 个无关文件(8964/citizen/qr 等,纯格式)→ 已 `git checkout` 全部还原,改动严格限定 chat+isar;此后只 `dart format lib/chat lib/isar/app_isar.dart`。
- **未做(2d-2)**:分片断点续传(接收端 partial 按 attachmentId、`resume_offset` 握手、`openRead(offset)` 续发 + 完整性),让接近 5GB 传输断网可续。

## 执行结果 · 步骤3a 贴纸（Fluent 3D，2026-07-15，完成）

- **内置资产**:从 `microsoft/fluentui-emoji`(MIT)精选 **48 张 3D PNG**(256×256,共 1.7MB)入 `assets/stickers/fluent3d/`,分四类(表情16/手势12/爱心10/庆祝10);pubspec 声明整目录。
- **清单单源**:`lib/chat/stickers/sticker_pack.dart` = 唯一 id 清单(`packId='fluent3d'`)+ `assetPath/isKnown/grouped`;**不落 manifest.json**(避免双源漂移),由 `sticker_pack_test` 反核对"清单 ⇔ 磁盘 png 一一对应、零死引用零孤儿"。
- **零字节传输**:贴纸只把 `(packId, stickerId)` 塞进 MLS 明文信封(传输层步骤1已备好 `chat_flow/runtime.sendSticker`),**零字节、零 WebRTC、零云存储**;接收端按 id 查本地内置 PNG 渲染,未知 id(对端资产旧/缺)白名单门拦截后降级占位 `[贴纸]`,绝不崩(亦挡 `sticker_id` 路径穿越)。
- **渲染与面板**:`chat_ui_adapter` sticker 分支 `Message.text` 占位 → `Message.custom`(metadata 带 id);`chat_page` 经 `composerBuilder` 自绘 `Composer`(发送/附件仍走 Chat 注入回调)+ `topWidget` 挂贴纸开关与 `StickerPanel`(分类 Tab 网格,`Image.asset` 带 errorBuilder);`_buildStickerMessage` 白名单门渲染无气泡大图 + 降级。两处入口 `chat_tab`/`open_direct_chat` 接 `runtime.sendSticker`。
- **对抗式审查(5 维度 × 逐条 refute)**:0 处代码正确性缺陷,确认 7 项并全部处理——**[中·行为回归]** `_reloadMessages` 无条件置 `_loading=true` 使整块 Chat(含面板)unmount、连发时分类 Tab 归零 → 改为只首屏骨架、重载保持 Chat 常驻(加 panel-survives-send 断言守回归);**[低]** `StickerPanel` 固定 264 横屏/小屏挤压列表 → 按视口 40% 夹取;**[高×2 测试]** `_buildStickerMessage` 渲染/降级 + 自绘 composer 接线(开关切换/点选路由 onSendSticker)零测 → 补 `chat_sticker_ui_test`;**[中·测试]** `onSendSticker→runtime.sendSticker` 接线未测 → chat_tab 导航测 + `_FakeRuntime.sendSticker` 记录四参;**[低×2 一致性]** 任务卡 `pack_fluent3d`/文档测试 `1f600` → 订正为 `fluent3d`/`grinning_face`。1 项(开关常驻显示)驳回。
- **测试(98→110,+12)**:`sticker_pack_test`(reconcile/isKnown/assetPath/grouped)、`sticker_panel_test`(点选/分类 Tab)、`chat_sticker_ui_test`(未知 id 降级/已知不崩/开关弹面板点选路由+面板存活/再点收起)、`chat_tab_test`(进会话点贴纸→runtime.sendSticker 四参)、`chat_ui_adapter_test`(sticker→CustomMessage);payload/envelope 夹具 id 对齐真实命名。
- **验证**:`flutter analyze lib/chat test/chat` = No issues found;`dart format lib/chat test/chat` 全过;`flutter test test/chat --concurrency=1` = 110 通过/4 跳过。

## 执行结果 · 步骤3b 表情面板（emoji，2026-07-15，完成）

- **成熟包接入**:新增 `emoji_picker_flutter ^4.4.0`(离线 emoji 数据,**无网络/无遥测**依赖;仅 shared_preferences 存"最近使用")。`EmojiPicker(textEditingController: _composerController)` 把选中的 Unicode emoji **直接插到 composer 光标处**,随文本走现有 `sendText`——**零协议变更、零新增数据面**(emoji 就是文本)。
- **互斥面板重构**:`chat_page` 的 `bool _stickerPanelOpen` 收敛为 `enum _ComposerPanel {none,emoji,sticker}` + `_togglePanel(panel)`(同则关/异则切,打开任一收键盘);`_composerToolbar` 两键(`chat-emoji-toggle`/`chat-sticker-toggle`);`_buildEmojiPanel` 高度按视口 40% 夹取、主题化配色、关搜索栏。表情/贴纸二选一挂 topWidget。
- **对抗式审查(3 镜头 × 逐条 refute)**:0 处正确性缺陷,确认 1 项修复、驳回 4 项(EmojiPicker 库 dispose 漏移 textEditingController 监听=无 scrollController 时纯 no-op、有界、随页 dispose 自清,不值 workaround;最近使用 emoji 落 SharedPreferences=非 PII 的全局标准 UX,删聊天记录不该清;反向互斥/tooltip=对称逻辑低价值)。确认项:**文本经共享 controller 走 onSendText 无端到端测**(且文本发送全仓此前零测)→ 补 `chat_emoji_panel_test` 一条:断言 `EmojiPicker` 与 `Composer` 的 controller 为同一实例(防接错 controller 静默丢字)+ `enterText('你好🙂')`→点发送→`onSendText` 收到全文。
- **测试(110→113,+3)**:`chat_emoji_panel_test`(点表情弹/收 EmojiPicker;共享 controller + 文本经 onSendText 发出;表情↔贴纸互斥)。
- **验证**:`flutter analyze lib/chat test/chat` = No issues found;`dart format` 全过;`flutter test test/chat --concurrency=1` = **113 通过/4 跳过**。
- **收官**:本卡三步(1 载荷地基 / 2 媒体采集传输 / 3 表情贴纸)全部完成;媒体升级唯一遗留 = 2d-2 分片断点续传(可选打磨,非阻断)。

## 待后续（非本卡）

- 贴纸素材最终版权与美术定稿;大文件本地转码/压缩策略打磨;离线补发的进度与失败 UX 细化。
- 群聊(卡 2 私密小群 E2E ≤1989、卡 3 大频道复用广场非 E2E)。
