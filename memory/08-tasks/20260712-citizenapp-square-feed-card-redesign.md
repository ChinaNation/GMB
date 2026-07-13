# CitizenApp 广场动态卡片 UI 重构:六类 × 横竖屏媒体规则 + 方形圆角头像勋章

任务需求：按用户定稿的广场卡片设计，重构 CitizenApp 广场 feed 的图片/视频/文章及各自竞选变体显示。
所属模块：citizenapp / 8964（广场）

## 定稿设计（用户逐轮确认）

- 身份表达：方形圆角头像 + 右下角扇贝勋章（复用 `ui/identity_badge.dart` 的 `IdentityBadge`，
  竞选红/投票蓝/访客金；会员=勾、仅身份=小人）。小圆角"竞选"药丸**只有竞选用户显示**（红色，
  贴名字后）；投票/访客不显示药丸（靠勋章）。竞选岗位**只有竞选用户**显示（`post.campaignPosition`
  有值才显示，否则只显示时间）。右上角三点上移贴近上边缘。
- 媒体规则（横竖屏由每个媒体的 width/height 判定）：
  - 横屏：1图/视频=整块 16:9 长方形；2图=左右各半、外侧圆角、中缝直角、2px 缝；3图以上=只出前两张、
    第二张右下角 `+N`（N=总数-2）；文章=首图强制 16:9。
  - 竖屏：同数量逻辑，照片换 3:4。**竖屏单图/单视频=左媒体(约40%,3:4)+右文字**；2图/3图以上=文字在上、
    下面左右并排竖图（同横屏规则，tile 3:4）。
  - 文章：标题(2行截断)+正文(2行截断)在上，**首图强制横屏 16:9 在下**。

## 数据链现状（关键，已核实）

- Worker 侧 width/height **已全链路打通**：`uploads/service.ts` 从 `LimitTicket` 落 D1，
  feed 走 `listFeedPosts → buildFeedPostItem → manifestMediaItems`，后者回传 `width/height`
  （`posts/confirm.ts:349-350`；`types.ts SquareFeedMediaItem` 已含）。**Worker 零改动。**
- 缺口只在 Flutter：`SquareMediaItem` 没 width/height 字段、`square_api_client._parseMediaItem` 没解析。

## 必须遵守

- Worker `cloudflare/src/limits` 是**上传门控层**（字节+包围盒+额度），不放显示逻辑、本次不动。
- 横竖屏是显示决策，来自媒体 width/height，不进 limits。
- 复用 `IdentityBadge`，不重画勋章。
- 头像图 URL 当前 feed 作者信号不含，占位=昵称首字（真头像另属数据补充，非本次）。

## 输出物

- 模型/解析：`SquareMediaItem` +`width/height`+`isPortrait/aspectRatio`；`_parseMediaItem` 解析。
- 组件：`square_post_header.dart`（新，共享头像勋章行）；`square_media_grid.dart`（重写媒体规则）；
  `square_post_card.dart`（横竖屏感知：竖屏单图/视频走左右布局）；`square_article_card.dart`
  （标题正文在上、强制 16:9 首图在下、复用共享头部）。
- 测试：媒体布局（横竖 × 1/2/3+）、竞选药丸/岗位仅竞选显示、文章版式、作者点击回调保留。
- 文档更新、中文注释、残留清理。

## 验收标准

- `flutter analyze` 三组件 + 模型 0 问题；相关 widget 测试通过。
- 保留 `square_post_card_author_test` 的作者点击语义。
- limits/Worker 未被误改。

## 执行结果（2026-07-12）

- **数据链（Worker 零改动）**：核实 Worker 侧 width/height 已全链路带到 feed
  （`LimitTicket→square_media_assets(D1)→listFeedPosts→buildFeedPostItem→manifestMediaItems`）。
  只补 Flutter：`SquareMediaItem.{width,height}` + `isPortrait` 派生；`_parseMediaItem` 读
  `data['width'/'height']`（向后兼容，缺失=null=横屏兜底）。
- **组件**：新增 `widgets/square_post_header.dart`（方形圆角头像+扇贝勋章、竞选药丸/岗位仅竞选、
  更多按钮贴上边缘、保留作者点击 GestureDetector）、`widgets/square_post_actions.dart`（共享互动栏）；
  重写 `widgets/square_media_grid.dart`（抽 `SquareMediaTile`，横竖屏+1/2/3+、外圆内直 2px 缝、
  右下 `+N`）、`widgets/square_post_card.dart`（竖屏单图/单视频左右布局）、
  `widgets/square_article_card.dart`（标题2行/正文2行在上、强制 16:9 首图在下、共享头部+互动栏）。
- **分发**：home feed（`square_home_page.dart`）与 profile 文章 Tab 均按 `content_format==article`
  分发到文章卡（原 home feed 全走图文卡，已修）。
- **注释**：`campaignPosition` 模型注释改为"竞选卡头部有值时展示"。
- **测试**：新增 `test/8964/widgets/square_feed_cards_test.dart`（12 用例：朝向/数量/+N、竞选药丸与
  岗位仅竞选、竖屏单图左右、文章版式）。`flutter analyze lib/8964` 0 问题；
  8964 全测试 73 通过（profile 44 + widgets 12 + 根级 17），含原作者点击测试。
- **文档**：`memory/05-modules/citizenapp/8964/PROFILE_TECHNICAL.md` 增"广场 feed 卡片"节。
- **边界**：`cloudflare/src/limits` 与 Worker 未动。

### 真数据对接（2026-07-12 续，用户追加"真实数据也要对接上"）
- **作者真名 + 真头像已接**：`social/author_signals.resolveAuthorSignals` 并行读作者 `profile.json`
  （`readProfileDoc`，`.catch→null` 软降级），回填 `display_name`+`avatar_object_key`；
  `hydrateFeedItems`（feed）与 `listAuthorPosts`（作者主页）一并 spread；`types.SquarePostFeedItem` +两字段。
  Flutter：`SquareAuthor.{displayName,avatarObjectKey}` + `_parsePost` 解析；`SquarePostHeader` 头像走
  `Image.network(mediaUrl(key), viewer session Bearer)`，缺失/失败回落身份色淡底+昵称首字；
  home feed（`_FeedBody` 存 `_feedSessionToken`）与 `profile_posts_list` 据 key+session 生成 url/headers 传卡。
- 验证：Worker `tsc` 0 + `vitest` 124 全过；Flutter feed 解析测试断言 display_name/avatar/isPortrait；
  8964 相关测试全过。

### 仍待后续（非本次）
- 互动栏为无计数图标（点赞/评论/收藏）；计数与交互待后端计数字段。
- `campaignPosition` Worker 暂未回传，竞选副标题当前只显时间，待公民身份上链落地。
- 作者 profile.json 每页 N 次 R2 读，量大后可加 KV 缓存（当前并行读，可接受）。
