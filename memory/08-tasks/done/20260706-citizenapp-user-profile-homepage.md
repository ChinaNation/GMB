# CitizenApp 推特式用户主页（我的 → 用户资料改造）

## 任务需求

- 把「我的 → 点头像右侧进入的用户资料页」从当前的「二维码 + 头像 + 昵称」三项，改造成推特式用户主页。
- 主页包含：头像、背景、钱包账户、签名、关注、关注者、帖子、竞选、照片、视频、文章分类。
- 架构要求合理、高效；按分阶段任务卡逐步实现，每完成一步输出下一步技术方案。

## 已确认设计决策（用户逐条拍板）

1. 身份唯一性：主页身份 = **默认热钱包地址**；换默认钱包 = 换身份 = 换主页。
2. 存储边界：**只有发帖、交易上链**；头像/背景/签名/展示名等一律存 **Cloudflare R2**（经 Worker）。
3. 头像统一**圆角方形**（见任务卡 UI 草图）；账户有 `cid_number` → 显示勾号认证图标。
4. 二维码、编辑资料收进右上角竖三点 `⋮`；**编辑资料仅本人可见**，别人看不到入口。
5. 头像那一行右上角（原二维码/编辑资料位置）改为 **3 个图标：通知 / 聊天 / 关注**（是图标不是按钮）。
6. 顶部背景加高；上滑时背景虚化，仅保留 `返回` + `⋮`，分类标签（帖子/竞选/照片/视频/文章）固定在虚化背景下方；两者 sticky，回顶展开。

## 所属模块

- citizenapp（`lib/8964/profile/` 新增前端主页；`lib/my/user/` 旧资料页收敛）
- cloudflare（Worker 公开资料层 + 计数 + 按作者拉帖）
- memory（架构文档、协议登记、任务卡）
- 不改 citizenchain/runtime（发帖/交易上链沿用现有 square-post pallet，本任务不新增链上字段）

## 核心边界

- 头像/背景/签名/展示名等公开资料是**链下数据**，只进 R2，链上零改动。
- 现有本地私有头像/背景（SharedPreferences `user.profile.state.v2`）改造后**真源切 R2**，本地仅离线缓存，不留私有分支（遵守零残留）。
- 认证勾号以**链上已确认发布携带的 `cid_number`** 为准（confirm 时由链上事件写入 `square_posts.cid_number`），不信任 App/Worker 自报。
- 「照片 / 视频」是**从帖子 `media_items` 派生的视图**，不建新表、不重复存储。
- 「文章」是长图文，属新内容类型；是否新增链上分类还是仅 R2 manifest 标记，留到阶段 7 单独决策，前期链上零改动。
- 主页页面区分 `isSelf`（= `ownerAccount == 默认热钱包地址`）：本人看到 编辑资料 + 通知/聊天/关注(我的关注)；别人看到 关注(toggle)/消息 + 举报。

## 目标数据分层

```text
链上 citizenchain     发帖索引/内容哈希/回执、交易                  ← 不动
R2   (Worker)          profile/{owner}/profile.json + 头像/背景对象   = 唯一公开资料源
D1   (Worker)          square_posts / square_follows / 计数聚合
本地 (SharedPrefs/Isar) 仅离线缓存 + 草稿，不再是公开资料真源
```

## R2 公开资料包契约（citizenapp.square.profile.v1）

```text
object key: profile/{sanitizeOwnerAccount(owner_account)}/profile.json
{
  schema: "citizenapp.square.profile.v1",
  owner_account,
  display_name,          # 展示名，独立字段（不再等于钱包名）
  bio,                   # 个性签名
  avatar_object_key,     # 头像 R2 key（+ content_hash）
  banner_object_key,     # 背景 R2 key（+ content_hash）
  updated_at
}
```

派生字段（不入 profile.json，响应时 join）：`cid_number`/认证（链上确认）、关注数/粉丝数/帖子数（D1 COUNT）。

## Worker 接口增量

| 接口 | 用途 | 阶段 |
|---|---|---|
| `GET /v1/square/users/:account` | profile + 计数 + 认证 + is_following | 1 |
| `GET /v1/square/users/:account/posts?category=&limit=&cursor=` | 按作者分页（category ∈ all/normal/campaign） | 1 |
| `PUT /v1/square/profile` | 本人存 display_name/bio/头像背景 key | 1 |
| 头像/背景对象上传 | 复用现有 R2 三步闭环 prepare→dev-put/put→complete | 6 |
| 关注/取关 | 已有 `POST/DELETE /v1/square/follows` | 复用 |

## 前端目标目录

```text
lib/8964/profile/
  user_profile_page.dart          # NestedScrollView 壳 + isSelf 路由
  widgets/
    collapsible_header.dart        # SliverAppBar + ImageFiltered 虚化折叠
    profile_header_card.dart       # 圆角方形头像/认证勾/地址/签名/计数
    profile_action_icons.dart      # 通知·聊天·关注 ↔ 关注·消息（图标）
    profile_kebab_menu.dart        # ⋮：二维码 + 编辑资料(self-only)
    profile_category_tabs.dart     # 帖子/竞选/照片/视频/文章
    profile_media_grid.dart        # 照片/视频九宫格（派生）
  models/citizen_profile.dart      # R2 资料包模型 + fromJson/copyWith
  services/
    citizen_profile_api.dart       # GET/PUT profile、计数、按作者拉帖
    citizen_profile_cache.dart     # 本地离线缓存
```

## 分阶段执行计划

### 阶段 0：任务卡固化（本卡）
- 固化决策、数据分层、R2 契约、接口增量、目录、阶段计划。不改代码。

### 阶段 1：Worker 公开资料层（后端）
- 新增 `GET /v1/square/users/:account`、`GET /v1/square/users/:account/posts`、`PUT /v1/square/profile`。
- R2 profile 读写 + D1 计数（following/followers/posts）+ is_following + 认证派生。
- 加 `square_posts(owner_account,...)`、`square_follows(followed_account)` 索引 migration。
- 验收：`npm run typecheck` + `npm test` 全绿；不碰 runtime、不写 secret。

### 阶段 2：前端数据层
- `CitizenProfile` 模型 + `citizen_profile_api` + `citizen_profile_cache`；单测。

### 阶段 3：前端折叠头骨架
- `UserProfilePage`（NestedScrollView + SliverAppBar 虚化 + 分类 TabBar 固定）；虚化用单图层 `ImageFiltered` 非全屏 `BackdropFilter`；widget 测试。

### 阶段 4：前端资料卡 + 三图标 + ⋮ + self/other
- 圆角方形头像 + 认证勾 + 地址 + 签名 + 计数；三图标(通知/聊天/关注)；⋮(二维码/编辑资料 self-only)。

### 阶段 5：前端分类内容 + 入口接线
- 帖子/竞选/文章列表 + 照片/视频九宫格（派生）；「我的」点头像 / 广场点作者 跳转 `UserProfilePage`。

### 阶段 6：编辑资料页
- 头像/背景上传复用 R2 三步闭环；display_name/bio 存 R2；本地旧头像/背景迁移到 R2 后仅留缓存。

### 阶段 7：文章分类 + 清理 + 文档
- 决策文章是链上新分类还是 R2 manifest 标记并落地；废旧 `ProfileEditPage` 残桩；回写架构文档 + `unified-protocols.md`；整体验收。

## 执行记录

### 阶段 0（完成）
- 已创建本任务卡并登记 `memory/08-tasks/index.md`。固化 6 项决策、数据分层、R2 契约、接口增量、目录、7 阶段计划。

### 阶段 1（完成，后端 Worker 公开资料层）
- 新增 `citizenapp/cloudflare/src/profiles/repository.ts`：R2 profile 读写（schema `citizenapp.square.profile.v1`）、D1 三项计数（following/followers/posts）、认证派生 `readLatestCidNumber`（取最近已发布帖子链上 `cid_number`）、`isFollowing`、`listAuthorPosts`（category all/normal/campaign + keyset 游标，复用 `buildFeedPostItem` 派生媒体）。
- 新增 `citizenapp/cloudflare/src/profiles/service.ts`：`getUserProfileRoute`（公开可读 + `maybeSession` 附带 is_following）、`getUserPostsRoute`、`putProfileRoute`（仅本人；owner 从 session 派生；display_name≤40 / bio≤160 超限拒绝；头像/背景 key 必须落本人 `profile/{owner}/` 前缀，防越权写他人对象）。
- 修改 `src/routes.ts`：接 `GET /v1/square/users/:account`、`GET /v1/square/users/:account/posts`、`PUT /v1/square/profile`。
- 修改 `src/types.ts`：新增 `CitizenProfileDoc` / `UserProfileCounts` / `UserProfileResponse` / `AuthorPostCategory`。
- 修改 `src/storage/r2_keys.ts`：新增 `profileObjectKey` / `profileAssetPrefix`。
- 修改 `src/shared/http.ts`：新增可复用 `maybeSession`。
- 新增 `migrations/0003_profile_indexes.sql`：`square_posts(owner_account,post_state,created_at)`、`square_follows(followed_account)` 索引。
- 新增 `test/profiles.test.ts`：8 例（R2 往返、缺失/非法 schema 回 null、计数+认证+is_following、未登录公开可读、PUT 持久化、越权 asset key 拒绝、超长拒绝、按作者 category+cursor 分页）。
- 边界：未改 `citizenchain/runtime`；未写任何 secret；认证以链上确认 `cid_number` 为真源，不信任自报。
- 验收：`npm run typecheck` 通过；`npm test` 6 文件 23 例全绿（新增 8 例）；`npm run migrate:local` 0003 应用成功（3 commands）。
- 追加：把 `PUT /v1/square/profile` 返回体统一为与 GET 相同的完整 `UserProfileResponse`（抽 `buildProfileResponse` 共用），客户端单一解析；本人视角 is_following=false。

### 阶段 2（完成，前端数据层）
- 新增 `citizenapp/lib/8964/profile/models/citizen_profile.dart`：`CitizenProfile`（镜像 `UserProfileResponse`）+ `fromJson/toJson/copyWith` + `resolvedDisplayName`（空展示名回落钱包名→截断地址）。
- 新增 `citizenapp/lib/8964/profile/services/citizen_profile_cache.dart`：`SharedPreferences` 离线缓存（键 `square.profile.cache.<owner>`；只缓存真实资料，兜底默认值不入缓存）。
- 新增 `citizenapp/lib/8964/profile/services/citizen_profile_api.dart`：门面，聚合 fetchProfile / fetchAuthorPosts / updateProfile，网络细节委托 `SquareApiClient`。
- 扩展 `citizenapp/lib/8964/services/square_api_client.dart`：新增 `fetchUserProfile` / `fetchAuthorPosts`（返回 `({posts, nextCursor})` 记录）/ `updateProfile` + `_putJson`，复用 `_parsePost`/`_getJson`/`baseUri`，不动既有 feed/发布方法。
- 关键发现：现有 feed 媒体只渲染占位图标（无 `Image.network`），object_key→URL 尚未在 App 落地；阶段 2 模型只承载 key，图片 URL 解析留到阶段 3-6 与 feed 统一。
- 新增 `citizenapp/test/8964/profile/citizen_profile_test.dart`：8 例（模型映射/展示名回落/JSON 往返、缓存往返+清除、`fetchUserProfile` 带 session、`fetchAuthorPosts` category+cursor、`updateProfile` PUT 仅传改动字段）。踩坑：`http.Response(String)` 默认 Latin1，中文夹具须显式 utf-8。
- 边界：纯数据层，无 UI、无页面接线、无链上改动。
- 验收：`dart format` 通过；`flutter analyze lib/8964` 干净；`flutter test test/8964` 20/20（新增 8 + 既有 12，无回归）；Worker `npm test` 复跑 23/23。

### 阶段 3（完成，前端折叠头骨架）
- 新增 `citizenapp/lib/8964/profile/widgets/profile_category_tabs.dart`：`ProfileTab` 枚举（帖子/竞选/照片/视频/文章）+ `ProfileCategoryTabs`（`PreferredSizeWidget`，`isScrollable` 适配窄屏，挂 SliverAppBar.bottom）。
- 新增 `citizenapp/lib/8964/profile/widgets/collapsible_header.dart`：`CollapsibleHeader`，`LayoutBuilder` 算折叠比例 → 背景层 `ImageFiltered(blur 18*t)`（单图层，非全屏 `BackdropFilter`），资料主体 `Opacity(1-t)` 淡出，折叠满浮现居中标题；背景阶段 3 用主题渐变占位。
- 新增 `citizenapp/lib/8964/profile/user_profile_page.dart`：`UserProfilePage(ownerAccount,isSelf,initialProfile?)`，`NestedScrollView + SliverOverlapAbsorber/Injector + SliverAppBar(pinned,expandedHeight:300) + FlexibleSpaceBar + bottom:ProfileCategoryTabs + TabBarView`；返回/⋮ 占位；资料主体与 Tab 内容为占位。
- 新增 `citizenapp/test/8964/profile/user_profile_page_test.dart`：3 例（5 Tab + 返回 + ⋮ 渲染、切换 Tab 显示对应占位、他人主页无异常构建）。
- 边界：纯 UI 骨架；未接真实资料加载编排/头像背景真图/Tab 真内容；无链上改动。
- 验收：`dart format` 通过；`flutter analyze lib/8964/profile test/8964/profile` 干净；`flutter test test/8964/profile` 11/11（8 数据 + 3 widget）。

### 阶段 4（完成，资料卡 + 三图标 + ⋮ + self/other + cache-first）
- 新增 `widgets/profile_header_card.dart`：圆角方形头像（`borderRadius 15`）+ 认证勾（`Icons.verified`，`isCertified` 时）+ 认证公民 pill + 展示名 + 短地址+复制 + 可选 CID + 签名 + 计数（可点）+ 右上 actions 槽。
- 新增 `widgets/profile_action_icons.dart`：本人=通知/聊天/关注三图标；他人=关注(toggle)/消息；黑色半透明圆形按钮（决策 5，是图标）。
- 新增 `widgets/profile_kebab_menu.dart`：`⋮` PopupMenu，二维码常驻、编辑资料仅本人、举报仅他人（决策 4）。
- 改 `user_profile_page.dart` 为 `StatefulWidget`：注入 `CitizenProfileApi`/`CitizenProfileCache`（可测试替身）；initState → 读缓存立即渲染 → 后台 `fetchProfile`（公开读）回刷 + 写缓存；异常 `on Exception` 兜底不覆盖。菜单/图标/计数回调阶段 4 为 snackbar stub。
- 新增 `test/8964/profile/profile_test_doubles.dart`（FakeProfileApi/FakeProfileCache/sampleProfile）+ `profile_header_test.dart`（4 例：self 三图标+编辑资料、other 关注+消息+举报、未认证隐藏勾/pill、cache-first 回刷+写缓存）；`user_profile_page_test.dart` 改注入替身。
- 边界：头像/背景真图、follow 真调用与 is_following、Tab 真内容、入口接线均留后续；无链上改动。
- 验收：`dart format` 通过；`flutter analyze lib/8964/profile` 干净；`flutter test test/8964` 27/27（15 profile + 12 既有，无回归）。

### 阶段 5a（完成，Tab 真内容 + 入口接线，公开只读）
- 新增 `widgets/profile_posts_list.dart`：`ProfilePostsTab`（游标分页 + 触底 `NotificationListener` 加载 + 空/失败态；`SliverOverlapInjector` 对齐固定头）。posts 模式渲染 `SquarePostCard`；media 模式从帖子 `mediaItems` 按 kind **客户端派生**九宫格（不建表）。
- 改 `user_profile_page.dart`：5 Tab 映射——帖子=normal / 竞选=campaign / 照片=image 派生 / 视频=video 派生 / 文章=`_EmptyTab('文章功能即将上线')`（article 分类留阶段 7）；点帖子/媒体 → `SquarePostDetailPage`。
- 改 `widgets/square_post_card.dart`：新增 `onAuthorTap`，作者头像/名区域独立点击 → 用户主页（不误触整卡 onTap）。
- 改 `pages/square_home_page.dart`：`_FeedBody` 加 `onOpenAuthor`，`_openAuthor` 用 `_identityFuture` 判 isSelf → `push UserProfilePage`。
- 改 `lib/my/user/user.dart`：我的 tab 点头像/名由「进 ProfileEditPage」改为 `_openMyProfile` → `push UserProfilePage(默认钱包,isSelf:true)`（空钱包给引导）；旧 `ProfileEditPage` 保留待阶段 6 迁移/阶段 7 清理。
- 新增测试：`profile_posts_tab_test.dart`（帖子渲染/竞选过滤/照片派生/文章空态/空帖子态 5 例）+ `square_post_card_author_test.dart`（作者点击触发 onAuthorTap 不误触整卡）；`profile_test_doubles.dart` 扩 `samplePost` + `FakeProfileApi.fetchAuthorPosts`。
- 边界：公开只读，无 session、无 follow、无链上改动。分页机制已接，游标已在阶段 2 数据层测过；媒体 Tab 从 category=null 全量帖子派生（文本多时媒体稀疏，可接受）。
- 验收：`dart format` 通过；`flutter analyze lib/8964 lib/my/user/user.dart test/8964/profile` 干净；`flutter test test/8964` 33/33（新增 6，无回归）。

### 阶段 5b（完成，follow / session / is_following）—— 口径 A（关注复用登录 session，不逐次签名）
- 事实澄清（防误解）：关注/取关**不逐次签名**；写接口带的是**登录 session token**（[`followRoute`](cloudflare/src/feeds/follows.ts) 用 `requireSession` 校验 Bearer）。session 由默认**热钱包**对登录挑战串**静默签一次**（`signWithWalletNoAuth`，无弹窗/无扫码）换取并缓存复用。冷钱包不可能是默认用户（[`getDefaultWallet`](citizenapp/lib/wallet/core/wallet_manager.dart:213) 只返回 isHotWallet），故不存在"冷钱包点关注弹窗"。session 存在的意义=防伪造他人关注（写入完整性），非内容加密。
- 后端：新增 `GET /v1/square/users/:account/follows?type=following|followers&limit=&cursor=`（`listFollows` keyset 分页）+ 接线 routes；`POST/DELETE /follows` 复用。Worker 单测 +1（following/followers 排序）。
- 前端：`SquareApiClient` 加 `followUser`/`unfollowUser`/`fetchFollows` + `_deleteJson`；`CitizenProfileApi` 透出；`CitizenProfile` 加 `SquareFollowEntry`。
- 新增 `services/square_session_provider.dart`：全 App 单例，默认热钱包静默登录换 session（`SquareApiClient` 内部按 owner 缓存）。
- 新增 `follows_list_page.dart`：关注/关注者分页列表，行短地址点击进主页。
- 改 `user_profile_page.dart`：注入 `sessionProvider`；`_load` 带 session 拉 profile → is_following 正确；他人「关注」乐观 toggle（isFollowing 翻转 + 粉丝数±1，失败回滚 + snackbar）；self「关注」图标 + 关注/关注者计数点击 → `FollowsListPage`。
- 新增/扩测试：Worker follows-list；`profile_test_doubles` 加 `FakeSessionProvider`/`fakeSession` + `FakeProfileApi.follow/unfollow/fetchFollows`（含 throwOnFollow）；header 测试 +关注乐观翻转 +失败回滚；`follows_list_page_test`（列表渲染 + 空态）。三处 page 测试注入 `FakeSessionProvider`。
- **本步暂缓（保持 stub）**：self 聊天/通知真实入口、他人「消息」→ Chat（`ChatPage`）。作为后续小步接。
- 验收：Worker `npm run typecheck` + `npm test` 6 文件 **24/24**（+1 follows）；`flutter analyze lib/8964 lib/my/user/user.dart test/8964/profile` 干净；`flutter test test/8964` **37/37**（+4，无回归）。

### 阶段 6a（完成，编辑资料：展示名 + 签名）
- 新增 `lib/8964/profile/profile_edit_page.dart`：`CitizenProfileEditPage`（避免与旧 `ProfileEditPage` 撞名）；展示名(≤40)/签名(≤160) 两字段，保存走 `_api.updateProfile`（`PUT /profile`，session 由 provider 静默换取；无热钱包给引导），成功 pop 回更新后的 `CitizenProfile`。
- 改 `user_profile_page.dart`：`⋮ 编辑资料` 由 stub 改 `_openEditProfile` → 打开编辑页，返回后 setState 回刷 + 写缓存。
- 扩测试：`FakeProfileApi.updateProfile` 记录 `lastUpdate`；`profile_edit_page_test.dart`（预填、保存发送编辑字段、无热钱包给引导且不调用）。
- 边界：仅展示名/签名；头像/背景图片编辑留 6b（需 R2 资产上传 + 图片读取通道）。
- 验收：`dart format` 通过；`flutter analyze lib/8964/profile test/8964/profile` 干净；`flutter test test/8964/profile` 28/28；`flutter test test/8964` 40/40（+3，无回归）。

### 阶段 6b-1（完成，媒体读取通道 + 头像/背景渲染）
- 后端：新增 `src/media/service.ts` `mediaRoute` → `GET /v1/square/media/<object_key>` 直出 R2 对象（content-type + 长缓存 + CORS），只允许 `square/`/`profile/` 前缀、拒 `..`；接线 routes。这条通道同时解锁广场 feed 图片显示（后续可切）。
- 前端：`SquareApiClient.mediaUrl(objectKey)`（object_key→公开 URL，逐段 encode）；`CitizenProfileApi.mediaUrl` 透出。
- 渲染：`ProfileHeaderCard` 加 `avatarUrl` → 圆角方形 `Image.network`（失败/空回落 `_AvatarPlaceholder`）；`CollapsibleHeader` 改为渐变作底 + 背景图 `Stack` 叠加（图空/失败透出渐变）；页面按 `avatar/bannerObjectKey` 经 `_api.mediaUrl` 计算 URL 下传。
- 测试：Worker `media.test.ts`（直出/404/非法前缀 3 例）；`mediaUrl` 编码断言；头像图渲染 smoke（有 key 不抛异常）。
- 验收：Worker `npm run typecheck` + `npm test` 7 文件 **27/27**（+3 media）；`flutter analyze` 干净；`flutter test test/8964` **42/42**（+2，无回归）。
- 边界：仅读取通道 + 显示；头像/背景**上传**与本地迁移留 6b-2（avatar/banner key 目前仍为空，显示占位）。

### 阶段 6b-2（完成，头像/背景上传 + 本地迁移 + feed 图片切换）
- 后端：新增 `src/profiles/assets.ts` `prepareProfileAsset`（`POST /v1/square/profile/assets/prepare`：kind avatar/banner + jpeg/png/webp + ≤15MB + sha256 校验 → object_key `profile/{owner}/{kind}_{sha}.{ext}` + 上传 URL，复用 `createUploadUrl` 生产预签名/本地 dev-put）+ `devPutProfileAsset`（仅校验本人前缀，无上传行）；接线 routes。内容不上链。
- 前端：`SquareApiClient.prepareProfileAsset` + `uploadBytesTo`（dev-put 同源带 Bearer，生产预签名 URL 不带 Authorization）；新增 `services/profile_asset_service.dart`（算 sha256 → 授权 → PUT → 返回 key/hash）。
- 编辑页：`CitizenProfileEditPage` 加背景 + 头像 `image_picker` 选图（`Image.memory` 预览），保存时上传 R2 并随 `updateProfile` 写 key/hash；**本地迁移**：打开时若 R2 无对应 key 而本地 `UserProfileState` 有旧图，预载为待迁移资产，保存成功后清本地私有副本（best-effort，异常忽略，测试无 SharedPreferences 自动跳过）。
- feed 图片切换：`SquareApiClient._parseMediaItem` 把 object_key 拼成 `mediaUrl`；`SquareMediaGrid` 与主页照片/视频 `_MediaTile` 改 `Image.network`（失败回落图标，视频叠播放键）——广场 feed 图片首次真显示。
- 测试：Worker `profile_assets.test.ts`（key/kind/content_type 校验 3 例）；`profile_asset_service_test.dart`（上传返回 key/hash + 校验 sha256/byte_size/Bearer、PUT 失败抛异常 2 例）。
- 验收：Worker `npm run typecheck` + `npm test` 8 文件 **30/30**（+3）；`flutter analyze lib/8964 test/8964/profile` 干净；`flutter test test/8964` **44/44**（+2，无回归）。

### 阶段 5c（完成，Chat/通知接实）
- 新增 `lib/chat/open_direct_chat.dart`：`openDirectChat(context, {peerAddress, title})` + `DirectChatOpener` typedef——抽取联系人详情的私聊拼装为共享助手（默认热钱包 sender，复用 `ChatRuntime`/`ChatPage`，空钱包引导）。
- DRY：`lib/my/user/user.dart` `_ContactDetailPage._openMessage` 改调 `openDirectChat`，删去重复 Chat 拼装 + 冗余 `chat_page`/`chat_runtime` 导入。
- `user_profile_page.dart` 三图标接实：他人「消息」→ `openDirectChat(peer=ownerAccount, title=展示名)`；本人「聊天」→ push `ChatTab()`；本人「通知」→ push `_NotificationsPlaceholderPage`（通知系统未建，占位）。加可选 `onOpenDirectChat` 注入点便于测试。
- 测试：他人页点「消息」→ spy 收到 `(ownerAccount, 展示名)`；本人页点「通知」→ 显示占位页。
- 边界：只接线，复用现有 Chat，不改 Chat 协议/加密/传输；通知系统不新建。
- 验收：`dart format` 通过；`flutter analyze lib/8964/profile lib/chat/open_direct_chat.dart lib/my/user/user.dart test/8964/profile` 干净；`flutter test test/8964` **46/46**（+2，无回归）。

### 阶段 7a（完成，清理 + 文档）
- 删残桩：`lib/my/user/user.dart` 旧 `ProfileEditPage`/`_ProfileEditPageState`（426–732 行，已被新主页 + `CitizenProfileEditPage` 取代，无人引用）整段删除；`_MyQrCodePage`/`_HollowQrPainter`/`_SquareAvatar`/`_HeaderBackground` 仍被 我的 tab 使用，保留。分析无残留未用导入。
- 修注释：`user_profile_page.dart` 顶部类注释由「阶段 4…」更新为完整功能口径。
- 文档：`memory/07-ai/unified-protocols.md` P-API-CITIZENAPP-002 追加 profile 六接口（users/:account、/posts、/follows、PUT /profile、assets prepare/dev-put、/media）+ R2 `profile.json` 契约；新增 `memory/05-modules/citizenapp/8964/PROFILE_TECHNICAL.md`。
- 验收：`dart format` 通过；`flutter analyze lib/my lib/8964/profile lib/chat/open_direct_chat.dart` 干净；`flutter test test/8964 test/chat` **84 passed / 4 skipped**（native smoldot 跳过，无回归）。

### 阶段 7a 追加（二维码归属，用户拍板落地）
- 决定：删「我的」tab 右上角二维码图标；主页 `⋮ → 二维码` 显示该主页用户名片码（钱包账户 + 昵称）。
- 实现：`_MyQrCodePage`/`_HollowQrPainter` 从 user.dart 提取为公开 `lib/8964/profile/user_qr_page.dart` `UserQrPage(contactName, address)`（复用 UserContactBody 名片码 + 存相册）；删 user.dart 里旧 QR 图标 Positioned、`_openMyQrPage`、旧私有 QR 类与 painter，清 3 个未用导入（dart:ui/rendering/saver_gallery）。主页 `⋮ 二维码` → `UserQrPage(名字, ownerAccount)`（self=本人码，other=对方码）。
- 测试：`⋮ → 二维码` 打开 `UserQrPage`（+1）。验收 `flutter test test/8964` **47/47**。

## 整卡状态：主页功能完整交付；遗留两项（另行处置）
1. 文章长文分类未落地（占位「即将上线」）——已拆独立任务卡（链零改动走 R2 manifest 标记）。
2. 通知系统未建（占位页）。
- 全链路测试累计：Worker 30/30、Flutter 8964+im 84 passed/4 skipped（8964 现 47）。
