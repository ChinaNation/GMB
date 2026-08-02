# CitizenApp 推特式用户主页（lib/8964/profile）

## 定位
- 「我的 → 点头像」进本人主页；广场帖子点作者进他人主页。
- 用户主页寻址主键 = 永久 `cid_number`；当前绑定 `account_id` 只承担签名、授权和
  SS58 派生，主页不直接展示或复制 AccountId。`ss58_address` 只用于展示和边界
  输入输出；换绑账户不改变主页或公开昵称。
- 头像/背景/签名/公开昵称等资料是**链下数据**，用户设置值进 Cloudflare R2；链上只有发帖、交易。`display_name` 是唯一公开昵称真源，`walletName` 只是本机钱包标签，两者不再同步。公开资料缺失或图片读取失败时，App 优先按 CID 稳定选择本地内置默认昵称、头像和背景，该展示兜底不上传、不持久化，也不参与身份判断。

## 数据分层
```
链上 citizenchain   发帖索引/哈希/回执、交易                        ← 不动
R2   (Worker)        profile/{cid_number}/profile.json + 头像/背景对象 = 唯一公开资料源
D1   (Worker)        square_posts / square_follows / 计数聚合
本地 (App assets)    profile_defaults 只提供稳定展示兜底；不成为公开资料真源
本地 (SharedPrefs)   公开资料离线缓存；旧本地头像/背景迁移到 R2 后清空
本地 (Isar/AppKv)    本人已发布规范 manifest 副本 + 草稿；不保存公共 feed 或永久媒体副本
```

### 本人已发布内容本地副本

- `SquareLocalPostEntity` / `SquarePostStore` 是本人已发布内容唯一 Isar 边界；
  `cid_number` 是归属主键，`account_id` 只记录发布时签名账户事实，CID 换绑不迁移。
- 正文、标题、文章图文块和媒体声明只保存在原始 `manifest_bytes` 中，不拆第二套本地正文字段；
  写入与读取均重新验证 SHA-256、当前 schema、账户、分类和内容形态，损坏数据 fail-closed。
- 本地仅接受 Worker 已确认的 `post_state=published`；`created_at` 由 Worker 提供，仓库不读取
  设备时间。图片、视频、封面、文件路径和临时签名 URL 始终不进入本表。
- Worker 确认发布后，`SquarePublishService` 使用发布阶段保留的原始 manifest 和 Worker
  发布锚立即写本地；本地失败不回滚远端成功、不提示用户重试，只显示完成告警并调度回灌。
- `SquarePostSyncService` 在广场获得有效会话后后台启动：首次完整分页，后续按
  Worker `(created_at DESC,post_id DESC)` 扫描到上次最新事实即停止；每页原子落盘，
  全部目标页成功后才更新不含设备时间的检查点。远端内容过期删除不反向删除本地副本。
- 本人主页由上层使用永久 `cid_number` 判定本人边界，进入页面后先解析本地 manifest
  展示正文，再按 `post_id` 合并 Worker 结果；读取本人本地副本不依赖 Bearer session，
  断网、Worker 故障或会话无法建立时仍可展示。Worker 的作者资料和媒体元数据优先；
  他人主页和公共 feed 永远不读取本机副本。
- 本地不保存媒体字节，也不猜测对象键或 URL；manifest 声明过但 Worker 已无对应媒体时，
  页面明确显示“媒体已从云端清理，本机仅保留正文”。
- 单帖删除和编辑替换旧帖统一走 `SquarePostDeletionCoordinator`：归属只校验会话 CID，
  Worker 删除成功后才删同 CID 本地副本；只有精确 `404/post_not_found` 可视为远端已
  不存在并清理本地残留，权限、会话和网络错误必须保留本地副本。
- 注销挑战与确认均用 finalized 链身份复核 `session.cid_number → 当前 account_id`；
  `account_id` 只完成签名授权，Worker 唯一按 CID 删除跨历次换绑账户产生的全部内容、
  媒体、关系、设备、挑战、会话、finalized 交易最小证明和充值订单。后两类记录必须在
  创建时固化 `cid_number`，原始链交易事实由公链保存。Session token 只以 SHA-256 进入 KV 键和 D1 强一致
  CID 注销索引，禁止以 KV 前缀枚举作为完整性依据。服务端硬删除成功后，App 再按 CID 删除全部本人副本和同步
  检查点；某项本地清理失败不阻断其余资料、会话、私信和设备子钥清理，也不得把已完成的
  服务端注销误报为可重试失败。
- 注销中的媒体存储总量释放与 `square_media_assets`、帖子、上传索引删除必须由同一个
  D1 原子 batch 提交；Images、Stream、R2 或 KV 清理中途失败时不得预先扣减
  `resource_totals`，重试成功只允许释放一次。
- 本地 `CitizenProfileCache` 以 `cid_number` 为唯一缓存键；注销后的资料缓存必须调用
  `clear(cid_number)`。`account_id` 只用于清除该账户的 API Session 与 Chat 本地数据，
  禁止用它删除 CID 资料缓存。

## Worker 接口（详见 unified-protocols P-API-CITIZENAPP-002）
- `GET /square/posts/self?limit=5&cursor=...`：必须携带本人 Bearer session 与 P-256
  设备请求证明；Worker 只按 session `cid_number` 返回本人已发布内容的原始 manifest
  base64、链锚和 Worker 时间。D1 `object_keys_json` 必须与 `account_id + post_id` 生成的
  唯一规范 manifest 路径完全一致；空清单、额外键或错误路径均整页拒绝。帖子/上传/
  manifest 三组哈希或归属任一不一致时同样整页拒绝；不返回媒体字节、路径、临时 URL，
  不增加公共浏览量。
- `GET /square/users/:cid_number`：profile + 计数 + 认证 + is_following（公开可读，带 session 反映登录者视角）。
- `GET /square/users/:cid_number/posts?category=&limit=&cursor=`：按作者分页（all/normal/campaign）。
- `GET /square/users/:cid_number/follows?type=following|followers`：关注/粉丝列表分页。
- `PUT /square/profile`：本人写 display_name/bio/头像背景 key（返回与 GET 同构）。
- `POST /square/profile/assets/prepare` + `PUT /square/profile/assets`：
  每个身份固定使用 `profile/{cid_number}/avatar` 与
  `profile/{cid_number}/banner` 两个对象键，并由同域 Worker 校验实际字节、MIME、
  图片文件头、尺寸与 sha256 后覆盖写 R2；头像 512KiB/1024×1024，背景
  1536KiB/1920×720，并发上传也不可能增加对象数。内容不上链。
  `cid_number` 只从 Worker session 派生并经统一路径校验，不接受客户端上传属主、
  SS58、AccountId 或任意字符串清洗。
- `GET /square/media/<object_key>`：必须携带钱包 Bearer session，只允许读取固定头像/背景键；`Image.network` 使用 session header，服务端不要求该只读图片请求附加 P-256 签名。
- 关注/取关复用已有 `POST/DELETE /square/follows`。

## 前端结构（lib/8964/profile）
- `user_profile_page.dart`：`NestedScrollView + SliverAppBar(pinned,expandedHeight:372) + FlexibleSpaceBar + bottom:分类TabBar + TabBarView`；cache-first 加载 + session-aware is_following。展开高度为昵称、SS58、CID、签名和三项计数保留独立空间，避免窄屏与分类标签重叠。
- `widgets/collapsible_header.dart`：折叠比例驱动 `ImageFiltered` 单图层虚化（非全屏 `BackdropFilter`）+ 资料主体淡出 + 折叠标题浮现；真实 R2 背景优先，缺失/失败时显示稳定本地背景照片。
- `widgets/profile_header_card.dart`：圆角方形头像（真实 R2 图片优先、缺失/失败回落稳定本地照片）+ 身份徽章 + 公开昵称 + SS58 及唯一复制按钮 + 独立 CID 行 + 签名 + 关注/关注者/帖子三项计数 + 右上三图标槽。SS58 只从规范 `account_id` 即时派生；非法或未加载账户显示“暂不可用”且隐藏复制入口。CID 使用页面路由身份真源，不提供复制按钮。
- `widgets/profile_action_icons.dart`：本人 通知/聊天/关注；他人 关注(toggle)/消息（**图标非按钮**）。
- `widgets/profile_kebab_menu.dart`：`⋮` 二维码（→ `user_qr_page.dart` 名片码）/编辑资料(self-only)/注销用户(self-only)；产品不提供举报功能。
- `user_qr_page.dart`：主页 `⋮ → 二维码` 传入当前
  `cid_number + display_name + account_id`，生成严格
  `UserQrPage.userContact` 固定身份码并支持存相册；钱包/账户/聊天入口统一通过
  `openAccountQrPage()`，只有链上身份账户生成 `k=3`，其它账户生成五分钟 `k=4`。
  「我的」tab 原二维码图标已删。
- `widgets/profile_category_tabs.dart` + `widgets/profile_posts_list.dart`：帖子/竞选/照片/视频/文章五 Tab；照片/视频从帖子 `media_items` 客户端派生（不建表）；本人 Tab 即使没有 session 也先按页面 CID 读取本地副本，session 只用于远端请求和受保护媒体；他人 Tab 禁止访问本机副本。帖子 Tab 传 `content_format=normal` 排除文章，文章 Tab 传 `content_format=article` 拉真数据，用 `widgets/square_article_card.dart` 渲染、点开 `pages/square_article_detail_page.dart`。
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
    `{appDocs}/square_drafts/{cid_number}/{draftId}/`）、`compose_draft_store.dart`
    （AppKvEntity 前缀 `square.compose.draft.by_cid.{cid_number}.{draftId}`、按 updated_at 新→旧、
    **上限 100 淘汰最旧**）、`drafts_page.dart`
    （缩略卡、右滑删除、点击恢复）。行为：**持续防抖自动保存**（编辑中 800ms + 退出/取消 flush，空内容不存/删）、
    发布成功删草稿、发布失败保留可重发。壳持 `_draftId`、向 body 注入 `persistMedia`/`onChanged`，body 加
    `snapshot()/restore()`。旧"每人一条失败恢复草稿"（`storage/square_draft_store.dart` +
    发布服务 `_saveDraftAfterFailure`/`_deleteDraftAfterSuccess` + home `draftStore` 参数）**已彻底删除**——
    失败内容由草稿箱持续自动保存兜底；发布失败仅上抛错误消息。
- `follows_list_page.dart`：关注/粉丝列表；按分页并行补公开资料，单个资料失败时显示稳定本地昵称和头像，账户只放副标题。
- `profile_edit_page.dart`：`CitizenProfileEditPage` 公开昵称/签名/头像/背景编辑；保存上传 R2 + `PUT /profile`，不得读取或重命名本机钱包；本地旧图迁移后清空。
- `models/profile_presentation.dart`：唯一展示解析器；公开昵称只接受 `display_name`，优先以 `cid_number` 做 FNV-1a 稳定分桶，从内置词库与 `assets/profile_defaults/` 11 张照片中选择默认昵称、头像和背景。头像与背景使用不同盐值且避免同图；任何钱包名、完整或截断账户都不能成为公开昵称。
- `models/citizen_profile.dart`、`services/citizen_profile_api.dart`、`citizen_profile_cache.dart`、`profile_asset_service.dart`、`square_session_provider.dart`。
- 私聊入口共享 [`lib/chat/open_direct_chat.dart`](../../../../citizenapp/lib/chat/open_direct_chat.dart)。通讯录、聊天、广场作者、关注/粉丝列表全部进入同一个 `UserProfilePage`；联系人只保存私人名称和账户，不再维护联系人详情或公开资料副本。
- `profile_avatar.dart` 是用户主页、通讯录、广场、聊天和关注列表共用的圆角方形头像、稳定默认照片和身份徽章唯一 UI 实现，禁止各入口复制一套头像规则。

## 广场 feed 卡片（lib/8964/widgets）
- 六类（图片/视频/文章 × 普通/竞选）共用统一版式；竞选与普通**媒体布局一致**，只靠身份表达区分。
- `square_post_header.dart`（图文/文章卡共用作者头部）：**圆角方形头像（真实图片优先、缺失/失败回落稳定本地照片）+ 右下角扇贝身份勋章**（复用 `ProfileAvatar`，竞选红/投票蓝/访客金，会员=勾/仅身份=小人，布局同主页头像）；昵称后**只有竞选公民**显示红色"竞选"药丸；副标题**只有竞选公民**显示岗位（`post.campaignPosition` 有值时 `岗位 · 时间`，否则只时间）；右上更多按钮 `CrossAxisAlignment.start` 贴上边缘。
- `square_media_grid.dart`（`SquareMediaGrid` + 共享 `SquareMediaTile`）：横竖屏由 `mediaItems.first.isPortrait`（媒体原始 width/height，缺失按横屏兜底）决定。1图/视频=横屏 16:9 / 竖屏 3:4 单块；2图/3图以上=只出前两张、左右各半、**外侧圆角+中缝直角+2px 缝**（容器比例横 2:1、竖 3:2 使左右图为 1:1/3:4），3图以上第二张**右下角 `+N`**（N=总数-2）；视频叠播放键、冷归档态占位。
- `square_post_card.dart`：头部 → 正文/媒体 → `square_post_actions.dart` 互动栏。**竖屏单图/单视频=左媒体（flex2,3:4）+右正文（flex3）**；其余=正文在上、下走 `SquareMediaGrid`。
- `square_article_card.dart`：头部 → **标题(2行截断)+正文(2行截断)在上 → 强制横屏 16:9 首图在下** → 互动栏；首图=`media_items[0]`，方向恒横屏不随原始朝向。home feed 与 profile 文章 Tab 均按 `content_format==article` 分发到此卡。
- **数据链（媒体尺寸）**：横竖屏所需 width/height——Worker 全链路已带（`LimitTicket→D1→feed manifestMediaItems`），Flutter 端 `SquareMediaItem.{width,height}` + `_parseMediaItem` 读 `data['width'/'height']` 补齐；`limits` 仅做上传门控（字节+包围盒+额度），不承载显示朝向。
- **数据链（作者昵称/头像）**：feed/作者拉帖的作者 `display_name` + `avatar_object_key` 由 `social/author_signals.resolveAuthorSignals` 对去重作者并行读 `profile.json`（`readProfileDoc`，缺失软降级空名/无头像）回填，`hydrateFeedItems` 与 `listAuthorPosts` 一并 spread；Flutter `SquareAuthor.{displayName,avatarObjectKey}` + `_parsePost` 解析。头像走 `Image.network(mediaUrl(avatar_object_key), 带 viewer session Bearer)`；昵称或头像缺失/失败时统一走 `ProfilePresentation` / `ProfileAvatar` 的本地稳定兜底，账户只显示在明确的账户行。

## 关键行为
- 关注/取关：单击 + 乐观更新（粉丝数±1，失败回滚），**不逐次签名**；session 由默认热钱包静默登录一次（`signWithWalletNoAuth`）复用。session 存在的意义是防伪造他人关注（写入完整性），非内容加密。
- 认证勾以链上已确认发布携带的 `cid_number` 为真源（confirm 时写入），不信任 App/Worker 自报。
- 主页资料媒体经 `mediaUrl(object_key)` → `GET /media/<key>` 渲染 `Image.network`，并携带钱包 session header；缺失或失败回落 `assets/profile_defaults/` 的稳定照片。广场主媒体使用 Images / Stream 短期地址。
- 广场首页浏览只从 `IdentityBadgeSnapshotStore` 读取当前永久 CID 的身份徽章展示信号；
  用户进入动态/文章发布页时通过
  `SquareIdentityService.loadCurrent(readLiveChain: true)` 读取 finalized 身份，
  快照不得用于发布资格判断。
- 若轻节点已被交易、治理或发布等其他主动流程启动并进入 operational，广场首页
  通过可取消状态监听为当前永久 CID 刷新一次徽章快照；钱包换绑后继续按同一 CID
  读取，不轮询。
- 广场首页的信息流 Future 在创建时立即挂只读错误观察器，随后仍由 `FutureBuilder`
  接收同一个原始 Future 并展示失败态；这消除了 Worker 快速失败发生在首帧监听前的
  未处理时间窗，不会把前台错误转换成成功或空列表。
- 广场与关注通知红点属于后台软功能：会话建立、拉取和清读任一失败都捕获到 `Object`
  边界并静默降级，不得从 `unawaited` 任务逸出影响浏览；信息流失败仍单独显示
  “广场内容加载失败”。

## 边界 / 待续
- 文章长文分类已落地（发布/文章 Tab/详情，链端零改动，见任务卡 20260706-citizenapp-square-article）；广场推荐流暂仍按普通卡显示文章，feed 识别文章卡为后续增强。
- 当前通知只覆盖广场/关注红点游标；完整通知中心不在本模块范围。

## 关联
- 任务卡：`memory/08-tasks/open/20260706-citizenapp-user-profile-homepage.md`
- 默认资料统一：`memory/08-tasks/open/20260715-citizenapp-profile-fallback-unification.md`
- 广场总卡：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
