# CitizenApp 广场用户主页三键：关注 / 通知 / 订阅

任务需求：用户定稿的广场（8964）用户主页头像行三个动作按钮端到端落地。本会话即
`20260721-citizenapp-profile-selfview-governance-chat-selfguard.md` 所指「关注/通知/订阅分流的独立会话」。
所属模块：citizenapp / 8964 广场 + profile。全链下（订阅门禁读链上 finalized）。

> **执行位置纪律**：一切改动只在 `/Users/rhett/GMB` 主检出（死规则 [[feedback_user_evaluates_in_main_checkout]]）。
> harness 曾把会话放进 `citizenchain/.claude/worktrees/*`，Step 1 首版误落 worktree，已在主检出重做。
> 主检出广场已被并行会话重构成 **6 分类**（recommended/following/campaign/article/photos/videos）
> + 删 `square_empty_state.dart` + 坦克水印取代空态 + 悬浮 FAB。本会话改动基于该重构版**叠加**。

## 用户定稿规格
1. **关注** = 关注用户的动态+文章进「广场-关注流」，取消关注则不显示；身份=钱包账户。后端已在 Cloudflare 实现。
2. **通知** = 关注**默认开通知**，用户可关；机制同聊天推送，内容不同（红点数字+声音，受设备系统通知开关约束）。
   - 静音不影响关注流展示，只影响红点+推送。
   - 红点双游标双徽章：广场底部 tab（进广场清）+ 关注子 tab（进关注子 tab 清）；只进广场不进关注→广场清、关注留。
   - 铃铛文案「订阅动态」→「通知」，回调 `onSubscribe`→`onNotify`。
3. **订阅** = 只有「已开通创作者订阅（有档）」的用户主页显示订阅按钮；补「创作者本人平台会员 active」fail-closed 门禁。最后做。

## 第 1 步「关注」——完成并验证（2026-07-21，主检出）
**改动**（纯 App，后端/链端零改动）：
- `citizenapp/lib/8964/pages/square_home_page.dart`：删 `_filterPosts`，改 `_composeFeed(serverPosts)`——`following` 直接
  渲染服务端 `/v1/square/feed/following`（已 JOIN `square_follows`，动态+文章都在）；其余 5 分类行为不变（对
  `merged=[本地草稿+服务端+种子]` 过滤），仅 following 排除本地/种子。
- `citizenapp/test/8964/square_home_page_test.dart`：加 `_KindFeedSource`（按 feedKind 返回）+ 用例「渲染服务端关注帖
  (动态+文章)、本地种子不混入」。
**验证**：`flutter test test/8964/square_home_page_test.dart` **4 绿**；`flutter analyze` 两文件 **No issues**；
`dart format` 过；全仓无 `_filterPosts` 残留。取关即时性走方案 A（切 tab 重载 + 下拉刷新）。

后端现状（已核实，Cloudflare 未改动）：`square_follows(owner_account,followed_account,created_at)` +
`POST/DELETE /v1/square/follows` + `GET /v1/square/feed/following`（JOIN，不按 content_format 过滤）。
已知限制（P2）：feed 端点无 keyset 游标（只 limit，还被浏览额度压缩）；取关需下拉刷新才即时。

## 第 2 步「通知」——设计已定稿并确认；分 2a/2b/2c

决策定稿：关注即默认开通知，铃铛=按用户静音/取消静音；静音不影响关注流展示，只影响红点+推送。
红点双游标双徽章：广场底部 tab（进广场清 last_seen_square_at）+ 关注子 tab（进关注子 tab 清 last_seen_following_at）；
只进广场不进关注→广场清、关注留。

### 2a 通知开关立住 —— 完成并验证（2026-07-21，主检出）
**Worker**（`citizenapp/cloudflare/`）：
- `migrations/0001_square_core.sql`：`square_follows` 建表内加 `notify_enabled INTEGER NOT NULL DEFAULT 1` 列（关注即默认开）。零用户开发期直接改建表脚本、整库重建，不走增量迁移。
- `profiles/repository.ts`：`isNotifying`（已关注且 notify_enabled=1）+ `setFollowNotify`（UPDATE，返回是否命中关注）。
- `profiles/service.ts`：`buildProfileResponse` 并发读 + 回 `is_notifying`；`types.ts` `UserProfileResponse` 加 `is_notifying`。
- `feeds/follows.ts`：`setFollowNotifyRoute`（`PUT /v1/square/follows/:account/notify {enabled}`，未关注 409 not_following）。
- `routes.ts` 加 PUT 分支；`limits/catalog.ts` 注册 `PUT /^\/v1\/square\/follows\/[^/]+\/notify$/`（否则 404）。
- 设备证明：该 PUT 命中 `/v1/square/*` 通配→需 P-256 设备签名，客户端 `_putJson` 已带（同 follow）。
**Flutter**：
- `citizen_profile.dart`：加 `isNotifying` 字段 + fromJson/toJson/copyWith。
- `square_api_client.dart` + `citizen_profile_api.dart`：`setNotify({session,followedAccount,enabled})` 走 `_putJson`。
- `profile_action_icons.dart`：铃铛「订阅动态」→「通知」，`onSubscribe`→`onNotify`，加 `isNotifying`（实心 active `notifications_active`/空心 `notifications_outlined`），兼容并行加的 `enabled`。
- `user_profile_page.dart`：`_toggleNotify`（未关注提示先关注；已关注乐观切换 setNotify）；关注/取关同步 isNotifying（关注默认开、取关清）。
**验证**：worker typecheck 干净、`vitest run` **160 绿**（profiles +5 通知用例）；`flutter test test/8964` **105 绿** + profile/contact 全绿；`flutter analyze` 改动文件 No issues；无 `onSubscribe`/「订阅动态」残留。

### 2b 双游标红点 + 双徽章 + 清零（拉模型）—— 完成并验证（2026-07-21，主检出）
**Worker**：
- `migrations/0001_square_core.sql`：`square_follows` 表后新增 `square_notify_reads(owner_account PK, last_seen_square_at, last_seen_following_at DEFAULT 0)`（同折进核心建表脚本，非增量迁移；`db:local` 执行 0001 整库重建即得）。
- `feeds/notify.ts`：`GET /v1/square/notify/unread → {square_unread,following_unread}`（`square_posts JOIN square_follows`，`notify_enabled=1` 且
  `created_at > 游标`，无已读行游标视 0）；`POST /v1/square/notify/read {scope}`（upsert 只推进对应游标到 now，另一游标不动）。
- `routes.ts` + `limits/catalog.ts` 注册两路由；均走 `/v1/square/*` 设备证明（客户端 `_getJson`/`_postJson` 已带）。
**Flutter**：
- `square_api_client.dart`：`fetchNotifyUnread`/`markNotifyRead`。
- `square_home_page.dart`：45s 轮询 `_refreshNotify`（仅 `_feedSource is SquareApiClient` 生产态开启，fake 测试跳过不触网）；
  广场数经 `onSquareUnreadChanged` 上抛，关注数留 `_followingUnread`；`selectedTab==0`→`_onSquareActivated`（清广场游标）；
  切关注子 tab→`_onFollowingActivated`（清关注游标）。
- `square_feed_tabs.dart`：关注段 `Badge`（`followingUnread`，>99 显示 99+）。
- `square_tab_page.dart` 透传；`main.dart`：`_squareNotifyCount` + 广场底部 `NavigationDestination` 包 `Badge`（仿 `_pendingVoteCount`），
  `_squarePage = SquareTab(selectedTab:_selectedTab, tabIndex:0, onSquareUnreadChanged:…)`。
**验证**：worker typecheck 干净、`vitest run` **165 绿**（+5 notify 用例：计数/静音排除/游标/进广场只清广场/scope 校验）；
`flutter test test/8964` **109 绿**（+4 SquareFeedTabs 徽章用例）；`flutter analyze` 改动文件 No issues。
拉模型闭环，红点不依赖推送。红点计数逻辑由 worker 单测覆盖；徽章渲染由 widget 测试覆盖；轮询/清零端到端待真机验。

### 2c 发帖扇出 + 可见推送（声音+横幅）+ 设备权限
用户定：含作者名 / **Cloudflare Queue 扇出（非上限截断，分页跨调用推完全部）** / 设备注册前移。

**2c-1 Queue 扇出 + 可见推送 —— 完成并验证（2026-07-21，主检出）**
- `types.ts`：`SquareNotifyJob{author_account,author_name,content_format,post_id,cursor?}` + `Env.SQUARE_NOTIFY_QUEUE?: Queue<>`。
- `wrangler.toml`：`[[queues.producers/consumers]]`（default `square-notify-fanout` / staging / production 各一队列）。**需 Workers Paid**。
- `chat/push.ts`：`sendSquarePostAlert`（APNS `apns-push-type:alert`+`aps.alert`+`sound`；FCM `notification`+`android.notification.sound`；`data.kind='square_post'`+`post_id`）；复用本文件 APNS-JWT/FCM-OAuth；`chat_wake` 一字未动。
- `feeds/notify_fanout.ts`：`fanOutPage` keyset 分页（先分页粉丝再取设备，避免多设备跨页错位）；`notify_enabled=1` + 未过期设备；满页 keyset 续跑入队；`buildAlert` 作者名+「发布了新动态/文章」。
- `posts/confirm.ts`：INSERT 后读作者名一次入队；入队失败只 log 不回滚。
- `index.ts`：`queue()` 消费者，逐条 ack/失败 retry（`max_retries=3`）。
- 验证：`tsc` 干净、`vitest` **168 绿**（+3 fanout：目标集合=未静音+未过期 / 不满页不续跑 / 满页续跑游标=末粉丝）。隐私：含作者名不含正文/媒体；静音不入扇出；全链下。

**2c-2 设备注册前移 —— 完成并验证（2026-07-21，主检出）**
- 用户定：**后台预热注册**（不阻塞首帧、广场推送对所有用户生效；代价=启动后台预热聊天 runtime，轻微违背懒加载）。
- `main.dart`：`_prewarmPushRegistration()` 首帧 `addPostFrameCallback` → `unawaited(_registerPushDeviceInBackground())` →
  `readOwnerAccount` 有账户则 `_chatRuntime.ensureReady(owner)`（注册设备 token，广场+聊天共用）；`on Object catch` 静默兜底（预热失败绝不崩主流程，进聊天 tab 仍会重试）。
- 发现记录：`_ensureDeviceRegistered` 深挂聊天就绪流、设备公钥来自 MLS 密钥包，**无「只注册 token」轻路径**，故走 `ensureReady`。

**2c-3 客户端 square_post 分支 —— 完成并验证（2026-07-21，主检出）**
- `chat_push_service.dart`：新增 `squarePostOpens` 广播流 + `_handleOpenedMessage`（点击/冷启动打开的推送：`kind=='square_post'`→发切广场信号；其余走 `chat_wake`）；`onMessageOpenedApp`+`getInitialMessage` 改走它。`wakeSenderFromData` 判定不动，`chat_wake` 未受影响。
- `chat_runtime.dart`：`Stream<void> get squarePostOpens => _pushService.squarePostOpens;`。
- `main.dart`：`_squareOpenSub` 监听 → `_openSquareTab()`（`selectedTab=0`+`_currentIndex=0`，广场页据此清广场红点）；dispose 取消。
- 前台刷红点省略（45s 轮询+进广场清已覆盖），不引入死代码。
- 可见横幅+声音由系统据 `notification` payload 展示，无需客户端渲染。

**2c-4 Android 声音渠道补强 —— 完成（2026-07-21，主检出）**
- `chat/push.ts` `sendFcmAlert`：`android.notification` 加 `channel_id:'square_posts'`（保留 `sound:'default'`）。
- `android/.../MainActivity.kt`：`companion object` 常量 `SQUARE_POST_CHANNEL_ID='square_posts'` + `ensureSquarePostNotificationChannel()`（Android 8+ 建 `IMPORTANCE_HIGH` 带声音+振动渠道，`configureFlutterEngine` 调用；已存在则跳过）。
- `AndroidManifest.xml`：FCM 默认渠道 meta-data `com.google.firebase.messaging.default_notification_channel_id=square_posts`（payload 未带 channel_id 时兜底）。
- 不引入 `flutter_local_notifications`，纯原生最小改动。iOS 无渠道概念（`aps.sound` 已生效）。
- 验证：worker `tsc`+`vitest` 168 绿；Kotlin 为标准 `NotificationChannel` API（导入/API-26 guard/常量正确），**本环境无 JDK 未跑 Gradle 编译**，随 `flutter build apk`/CI 编译。

**2c 验证**：worker `vitest` 168 绿；Flutter `flutter analyze` 改动文件 No issues；`flutter test test/chat test/8964 --concurrency=1` **257 绿**（并行跑多 Isar 目录的 23 失败是已知隔离问题 [[feedback_isar_is_community_fork]]，分开跑各自全绿）。
**部署前置**：Queue 需 Workers Paid；队列在 Cloudflare 控制台首次部署自动创建（或 `wrangler queues create`）。

## 第 3 步「订阅」——完成并验证（2026-07-21，主检出）
订阅按钮补「创作者本人平台会员 active」门禁；链端零改、Cloudflare 零改（读现有链上 Subscriptions）。
- `subscribe/creator_subscribe_service.dart`：加 `fetchPlatformSnapshot(address)` = `_rpc.fetchSubscriptionSnapshot(subscriberAddress: address)`（creatorAddress 省略=平台 IssuerKey；`fetchSubscriptionSnapshot` 语义已核实：带 creatorAddress 读创作者订阅、省略读平台会员）。
- `profile/widgets/creator_subscribe_button.dart`：`_load()` 的 `Future.wait` 多并发读 owner 平台快照存 `_ownerPlatform`（读失败→整个 `_load` 落 `on Exception` catch→按钮隐藏，fail-closed）；`build()` 门禁改 `!ownerPlatformActive` 也隐藏，`ownerPlatformActive = _ownerPlatform?.state?.isEffectiveAt(chainNowMs) == true`（同块 Timestamp.Now、禁本机时钟，与 `creator_service.load()` 同源）。
- `test/8964/profile/creator_subscribe_button_test.dart`（新增）：门禁四态——有档+平台 active→显示 / 平台过期→隐 / 快照抛 FormatException→隐(fail-closed) / 无档→隐。
- 验证：`flutter analyze` 改动文件 No issues；`flutter test test/8964` **113 绿**（+4 门禁用例）。
- 备注：并行会话此前给按钮加了 `enabled`（自看置灰），本步与其正交叠加，未冲突。

## 三键全部完成（关注/通知/订阅）
- 关注：`_composeFeed` following 渲染服务端流。
- 通知：2a 铃铛开关 + 2b 双游标红点 + 2c Queue 扇出/可见推送/设备注册前移/点击导航。
- 订阅：平台会员 active + 有档 双条件 fail-closed 门禁。

影响范围：`citizenapp/lib/8964`（广场+profile）+ `citizenapp/lib/chat`（推送/runtime）+ `citizenapp/lib/main.dart`（红点/预热/导航）+ `citizenapp/cloudflare`（通知端点/建表脚本 0001/Queue 扇出）。链端仅第 3 步读快照，无写。
部署前置：① DB schema 在 `0001`，删本地库重跑 `db:local`；② Queue 需 Workers Paid。

## 端到端联调手册（真机/双账户手验）

**部署前置**
- Cloudflare：`db:local`（或对应环境）执行 `0001` 建全表；`deploy:staging`/`deploy:production` 部署 worker（Queue `square-notify-fanout(-env)` 首次部署自动创建，需 Workers Paid）；APNS/FCM secrets 沿用聊天现有。
- App：Firebase 配置已内嵌；首启过 `AppPermissionGate` 授通知权限即触发设备注册预热。

**关注（无需推送/队列）**
1. 账户 A 进 B 主页点关注（乐观高亮，`POST /v1/square/follows`）。
2. B 发一条动态 + 一篇文章。
3. A 广场→「关注」子 tab：应见 B 的动态卡+文章卡（服务端 `GET /v1/square/feed/following` JOIN 结果）。
4. A 取关 B → 下拉刷新 → 消失。
- 直查：带 A session `GET /v1/square/feed/following` 返回 B 的已发布帖。

**通知·红点（拉模型，无需推送，最易验）**
1. A 关注 B（默认开通知）。A 停在非广场 tab。
2. B 发帖 → 约 45s 内（或手动切 tab）A 底部「广场」tab 出现红点数字。
3. A 进广场 → 广场红点清（`POST notify/read scope=square`）；此时「关注」子 tab 若有新帖仍显红点。
4. A 点「关注」子 tab → 关注红点清（scope=following）。**只进广场不进关注→广场清、关注留**。
- 直查：带 A session `GET /v1/square/notify/unread` → `{square_unread, following_unread}`；铃铛静音 B 后该计数应不含 B 的帖。

**通知·铃铛静音**
- A 在 B 主页点铃铛关 → `is_notifying` 转灰（`PUT /v1/square/follows/{B}/notify {enabled:false}`）；B 再发帖不进 A 红点/推送，但仍在关注流。未关注时点铃铛 → 409 提示先关注。

**通知·系统推送（需真机 + 队列）**
- 前置：A 真机启动过 App（预热注册设备 token）、系统通知开关开、未静音 B。
- A 后台/杀死 → B 发帖 → A 收系统横幅「B 展示名 · 发布了新动态/文章」+ 声音；点通知打开 App 切广场 tab。
- 无第二台真机验推送时：用 FCM/APNS 测试工具直接向 A 已注册 token 发 `{notification:{title,body}, data:{kind:'square_post',post_id:'x'}}`——点击应打开广场（验 2c-3 客户端分支）；扇出集合逻辑已由 worker `fanOutPage` 单测覆盖。
- Android 8+ 真机声音：已补强（见 2c-4）——App 侧 `MainActivity` 建 `square_posts` 高优先级带声音渠道，Worker payload 带 `channel_id='square_posts'`；首启创建渠道后横幅+声音生效。首次装机需启动过一次 App（建渠道）再收推送。

**订阅门禁**
- 前置：B 是创作者（「我的-创作者」设了档）+ B 平台会员 active → A 进 B 主页见「订阅 TA」。
- B 平台会员过期（或取一个设了档但会员过期的账户）→ A 进其主页订阅按钮消失。
- 断链/快照读失败 → 按钮消失（fail-closed）。
