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
- `POST /v1/square/profile/assets/prepare` + `PUT .../dev-put`：头像/背景上传授权（生产 R2 预签名 / 本地 dev-put），内容不上链。
- `GET /v1/square/media/<object_key>`：公开媒体读取通道（square/、profile/ 前缀），供 `Image.network`。
- 关注/取关复用已有 `POST/DELETE /v1/square/follows`。

## 前端结构（lib/8964/profile）
- `user_profile_page.dart`：`NestedScrollView + SliverAppBar(pinned,expandedHeight:300) + FlexibleSpaceBar + bottom:分类TabBar + TabBarView`；cache-first 加载 + session-aware is_following。
- `widgets/collapsible_header.dart`：折叠比例驱动 `ImageFiltered` 单图层虚化（非全屏 `BackdropFilter`）+ 资料主体淡出 + 折叠标题浮现；渐变作底、背景图叠加。
- `widgets/profile_header_card.dart`：圆角方形头像（`Image.network` + 占位回落）+ 认证勾（有 `cid_number`）+ 展示名/地址/签名/计数 + 右上三图标槽。
- `widgets/profile_action_icons.dart`：本人 通知/聊天/关注；他人 关注(toggle)/消息（**图标非按钮**）。
- `widgets/profile_kebab_menu.dart`：`⋮` 二维码（→ `user_qr_page.dart` 名片码）/编辑资料(self-only)/举报(other-only)。
- `user_qr_page.dart`：`UserQrPage(contactName, address)` 名片二维码（UserContactBody + 存相册），主页 ⋮ 二维码进入；「我的」tab 原二维码图标已删。
- `widgets/profile_category_tabs.dart` + `widgets/profile_posts_list.dart`：帖子/竞选/照片/视频/文章五 Tab；照片/视频从帖子 `media_items` 客户端派生（不建表）；帖子 Tab 传 `content_format=normal` 排除文章，文章 Tab 传 `content_format=article` 拉真数据，用 `widgets/square_article_card.dart` 渲染、点开 `pages/square_article_detail_page.dart`。
- 文章发布：广场发布入口分流（`_openCompose` 底部弹层 动态/文章）；`pages/square_article_compose_page.dart`（标题10-50+首图必填+正文≤19890+正文图≤64）；链上仍发 Normal、manifest 标 `content_format=article`+`title`（首图=media_items[0]）。签名器/媒体草稿构造抽 `services/square_compose_signers.dart`、`services/square_media_draft.dart` 与发动态共用。
- `follows_list_page.dart`：关注/粉丝列表。
- `profile_edit_page.dart`：`CitizenProfileEditPage` 展示名/签名/头像/背景编辑；保存上传 R2 + `PUT /profile`；本地旧图迁移后清空。
- `models/citizen_profile.dart`、`services/citizen_profile_api.dart`、`citizen_profile_cache.dart`、`profile_asset_service.dart`、`square_session_provider.dart`。
- 私聊入口共享 [`lib/chat/open_direct_chat.dart`](../../../../citizenapp/lib/chat/open_direct_chat.dart)（联系人详情与主页共用）。

## 关键行为
- 关注/取关：单击 + 乐观更新（粉丝数±1，失败回滚），**不逐次签名**；session 由默认热钱包静默登录一次（`signWithWalletNoAuth`）复用。session 存在的意义是防伪造他人关注（写入完整性），非内容加密。
- 认证勾以链上已确认发布携带的 `cid_number` 为真源（confirm 时写入），不信任 App/Worker 自报。
- feed/主页媒体经 `mediaUrl(object_key)` → `GET /media/<key>` 渲染 `Image.network`（失败回落图标）。
- 广场首页浏览只从 `IdentityBadgeSnapshotStore` 读取当前默认钱包的身份徽章展示信号，不启动 smoldot；用户进入动态/文章发布页时才通过 `SquareIdentityService.loadCurrent(readLiveChain: true)` 读取 finalized 身份，快照不得用于发布资格判断。
- 若轻节点已被交易、治理或发布等其他主动流程启动并进入 operational，广场首页通过可取消状态监听为当前钱包刷新一次徽章快照；切换默认钱包后按新账户隔离读取，不轮询。

## 边界 / 待续
- 文章长文分类已落地（发布/文章 Tab/详情，链端零改动，见任务卡 20260706-citizenapp-square-article）；广场推荐流暂仍按普通卡显示文章，feed 识别文章卡为后续增强。
- 通知系统未建（占位页）。

## 关联
- 任务卡：`memory/08-tasks/open/20260706-citizenapp-user-profile-homepage.md`
- 广场总卡：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
