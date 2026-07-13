# CitizenApp 推特式用户主页（lib/8964/profile）

## 定位
- 「我的 → 点头像」进本人主页；广场帖子点作者进他人主页。
- 身份 = **默认热钱包地址**（[`getDefaultWallet`](../../../../citizenapp/lib/wallet/core/wallet_manager.dart) 只返回最靠前热钱包）；换默认钱包 = 换身份 = 换主页。冷钱包不可能成为默认用户。
- 头像/背景/签名/展示名等公开资料是**链下数据**，只进 Cloudflare R2；链上只有发帖、交易。

## 数据分层
```
链上 citizenchain   发帖索引/哈希/回执、交易                        ← 不动
R2   (Worker)        profile/{owner}/profile.json + 头像/背景对象     = 唯一公开资料源
D1   (Worker)        square_posts / square_follows / 计数聚合
本地 (SharedPrefs)   仅离线缓存 + 草稿；旧本地头像/背景已迁移到 R2 后清空
```

## Worker 接口（详见 unified-protocols P-API-CITIZENAPP-002）
- `GET /v1/square/users/:account`：profile + 计数 + 认证 + is_following（公开可读，带 session 反映登录者视角）。
- `GET /v1/square/users/:account/posts?category=&limit=&cursor=`：按作者分页（all/normal/campaign）。
- `GET /v1/square/users/:account/follows?type=following|followers`：关注/粉丝列表分页。
- `PUT /v1/square/profile`：本人写 display_name/bio/头像背景 key（返回与 GET 同构）。
- `POST /v1/square/profile/assets/prepare` + `PUT /v1/square/profile/assets`：每个账户固定使用 `profile/{owner}/avatar` 与 `profile/{owner}/banner` 两个对象键，并由同域 Worker 校验实际字节、MIME、图片文件头、尺寸与 sha256 后覆盖写 R2；头像 512KiB/1024×1024，背景 1536KiB/1920×720，并发上传也不可能增加对象数。内容不上链。
- `GET /v1/square/media/<object_key>`：必须携带钱包 Bearer session，只允许读取固定头像/背景键；`Image.network` 使用 session header，服务端不要求该只读图片请求附加 P-256 签名。
- 关注/取关复用已有 `POST/DELETE /v1/square/follows`。

## 前端结构（lib/8964/profile）
- `user_profile_page.dart`：`NestedScrollView + SliverAppBar(pinned,expandedHeight:300) + FlexibleSpaceBar + bottom:分类TabBar + TabBarView`；cache-first 加载 + session-aware is_following。
- `widgets/collapsible_header.dart`：折叠比例驱动 `ImageFiltered` 单图层虚化（非全屏 `BackdropFilter`）+ 资料主体淡出 + 折叠标题浮现；渐变作底、背景图叠加。
- `widgets/profile_header_card.dart`：圆角方形头像（`Image.network` + 占位回落）+ 认证勾（有 `cid_number`）+ 展示名/地址/签名/计数 + 右上三图标槽。
- `widgets/profile_action_icons.dart`：本人 通知/聊天/关注；他人 关注(toggle)/消息（**图标非按钮**）。
- `widgets/profile_kebab_menu.dart`：`⋮` 二维码（→ `user_qr_page.dart` 名片码）/编辑资料(self-only)/注销用户(self-only)；产品不提供举报功能。
- `user_qr_page.dart`：`UserQrPage(contactName, address)` 名片二维码（UserContactBody + 存相册），主页 ⋮ 二维码进入；「我的」tab 原二维码图标已删。
- `widgets/profile_category_tabs.dart` + `widgets/profile_posts_list.dart`：帖子/竞选/照片/视频/文章五 Tab；照片/视频从帖子 `media_items` 客户端派生（不建表）；帖子 Tab 传 `content_format=normal` 排除文章，文章 Tab 传 `content_format=article` 拉真数据，用 `widgets/square_article_card.dart` 渲染、点开 `pages/square_article_detail_page.dart`。
- 广场发布已合并为**统一发布页** `lib/8964/compose/`（home `_openCompose` 直进，不再底部分流）：
  - `compose_page.dart` 壳：顶栏 取消/草稿/发布（去中间标题）、头像+**类型下拉**（普通 2 项动态/文章、
    认证公民 4 项加竞选动态/竞选文章）、IndexedStack 挂 动态/文章子编辑区、底部会员额度、发布协调
    （按类型取 `collect()` 载荷→门禁→`SquarePublishService.publish`；编辑经 initial* + replacePostId 预填）。
  - `post/post_compose_body.dart` 动态：正文 + 媒体计数 `[＋]`，**图片/视频由第一次选中的类型锁定**
    （先图=图片动态≤9、先视频=视频动态×1），发布页视频预览恒横屏 16:9。
  - `article/article_compose_body.dart` 文章：**标题+正文计数固定顶部**，下方图文块可滚，`插入`在焦点文本块后
    插入横屏图片块；紧凑首图（计数右侧小加号→小缩略✕），不显示大封面。校验/常量/拍平在
    `article/article_blocks.dart`（`buildArticleManifest`：内联图追加到首图后、块以 media_index 引用）。
  - 链上仍发 normal/campaign；manifest 标 `content_format=article`+`title`+**`content_blocks`**（文章正文图文块）。
    签名器/媒体草稿构造仍共用 `services/square_compose_signers.dart`、`services/square_media_draft.dart`。
  - **内联图文全链路**：manifest `content_blocks` → Worker `posts/confirm.ts`（`buildFeedPostItem` 从 manifest
    读出回传，无 DB 迁移）→ Flutter `SquarePost.contentBlocks`（`parseArticleContentBlocks`）→ 文章详情
    `square_article_detail_page` 按块渲染（内联图恒横屏），旧文章无块降级纯文本+扁平配图。
  - **草稿箱** `compose/drafts/`：全类型（图/视频/文章及竞选）本地持久化。`compose_draft.dart`（模型+JSON，
    含文章 content_blocks + 持久媒体路径）、`compose_draft_media.dart`（picker 临时文件选中即复制到
    `{appDocs}/square_drafts/{draftId}/`）、`compose_draft_store.dart`（AppKvEntity 前缀
    `square.compose.draft.{owner}.{draftId}`、按 updated_at 新→旧、**上限 100 淘汰最旧**）、`drafts_page.dart`
    （缩略卡、右滑删除、点击恢复）。行为：**持续防抖自动保存**（编辑中 800ms + 退出/取消 flush，空内容不存/删）、
    发布成功删草稿、发布失败保留可重发。壳持 `_draftId`、向 body 注入 `persistMedia`/`onChanged`，body 加
    `snapshot()/restore()`。旧"每人一条失败恢复草稿"（`storage/square_draft_store.dart` +
    发布服务 `_saveDraftAfterFailure`/`_deleteDraftAfterSuccess` + home `draftStore` 参数）**已彻底删除**——
    失败内容由草稿箱持续自动保存兜底；发布失败仅上抛错误消息。
- `follows_list_page.dart`：关注/粉丝列表。
- `profile_edit_page.dart`：`CitizenProfileEditPage` 展示名/签名/头像/背景编辑；保存上传 R2 + `PUT /profile`；本地旧图迁移后清空。
- `models/citizen_profile.dart`、`services/citizen_profile_api.dart`、`citizen_profile_cache.dart`、`profile_asset_service.dart`、`square_session_provider.dart`。
- 私聊入口共享 [`lib/chat/open_direct_chat.dart`](../../../../citizenapp/lib/chat/open_direct_chat.dart)（联系人详情与主页共用）。

## 广场 feed 卡片（lib/8964/widgets）
- 六类（图片/视频/文章 × 普通/竞选）共用统一版式；竞选与普通**媒体布局一致**，只靠身份表达区分。
- `square_post_header.dart`（图文/文章卡共用作者头部）：**圆角方形头像（昵称首字占位）+ 右下角扇贝身份勋章**（复用 `ui/identity_badge.dart` 的 `IdentityBadge`，竞选红/投票蓝/访客金，会员=勾/仅身份=小人，布局同主页头像）；昵称后**只有竞选公民**显示红色"竞选"药丸；副标题**只有竞选公民**显示岗位（`post.campaignPosition` 有值时 `岗位 · 时间`，否则只时间）；右上更多按钮 `CrossAxisAlignment.start` 贴上边缘。
- `square_media_grid.dart`（`SquareMediaGrid` + 共享 `SquareMediaTile`）：横竖屏由 `mediaItems.first.isPortrait`（媒体原始 width/height，缺失按横屏兜底）决定。1图/视频=横屏 16:9 / 竖屏 3:4 单块；2图/3图以上=只出前两张、左右各半、**外侧圆角+中缝直角+2px 缝**（容器比例横 2:1、竖 3:2 使左右图为 1:1/3:4），3图以上第二张**右下角 `+N`**（N=总数-2）；视频叠播放键、冷归档态占位。
- `square_post_card.dart`：头部 → 正文/媒体 → `square_post_actions.dart` 互动栏。**竖屏单图/单视频=左媒体（flex2,3:4）+右正文（flex3）**；其余=正文在上、下走 `SquareMediaGrid`。
- `square_article_card.dart`：头部 → **标题(2行截断)+正文(2行截断)在上 → 强制横屏 16:9 首图在下** → 互动栏；首图=`media_items[0]`，方向恒横屏不随原始朝向。home feed 与 profile 文章 Tab 均按 `content_format==article` 分发到此卡。
- **数据链（媒体尺寸）**：横竖屏所需 width/height——Worker 全链路已带（`LimitTicket→D1→feed manifestMediaItems`），Flutter 端 `SquareMediaItem.{width,height}` + `_parseMediaItem` 读 `data['width'/'height']` 补齐；`limits` 仅做上传门控（字节+包围盒+额度），不承载显示朝向。
- **数据链（作者真名/真头像）**：feed/作者拉帖的作者 `display_name` + `avatar_object_key` 由 `social/author_signals.resolveAuthorSignals` 对去重作者并行读 `profile.json`（`readProfileDoc`，缺失软降级空名/无头像）回填，`hydrateFeedItems` 与 `listAuthorPosts` 一并 spread；Flutter `SquareAuthor.{displayName,avatarObjectKey}` + `_parsePost` 解析。头像走 `Image.network(mediaUrl(avatar_object_key), 带 viewer session Bearer)`（`/media` 同域可读任意作者头像），缺失/失败回落身份色淡底+昵称首字。页面（home feed `_FeedBody`、`profile_posts_list`）据 `avatarObjectKey`+session 生成 url/headers 传卡片。

## 关键行为
- 关注/取关：单击 + 乐观更新（粉丝数±1，失败回滚），**不逐次签名**；session 由默认热钱包静默登录一次（`signWithWalletNoAuth`）复用。session 存在的意义是防伪造他人关注（写入完整性），非内容加密。
- 认证勾以链上已确认发布携带的 `cid_number` 为真源（confirm 时写入），不信任 App/Worker 自报。
- 主页资料媒体经 `mediaUrl(object_key)` → `GET /media/<key>` 渲染 `Image.network`，并携带钱包 session header（失败回落图标）；广场主媒体使用 Images / Stream 短期地址。
- 广场首页浏览只从 `IdentityBadgeSnapshotStore` 读取当前默认钱包的身份徽章展示信号，不启动 smoldot；用户进入动态/文章发布页时才通过 `SquareIdentityService.loadCurrent(readLiveChain: true)` 读取 finalized 身份，快照不得用于发布资格判断。
- 若轻节点已被交易、治理或发布等其他主动流程启动并进入 operational，广场首页通过可取消状态监听为当前钱包刷新一次徽章快照；切换默认钱包后按新账户隔离读取，不轮询。

## 边界 / 待续
- 文章长文分类已落地（发布/文章 Tab/详情，链端零改动，见任务卡 20260706-citizenapp-square-article）；广场推荐流暂仍按普通卡显示文章，feed 识别文章卡为后续增强。
- 通知系统未建（占位页）。

## 关联
- 任务卡：`memory/08-tasks/open/20260706-citizenapp-user-profile-homepage.md`
- 广场总卡：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
