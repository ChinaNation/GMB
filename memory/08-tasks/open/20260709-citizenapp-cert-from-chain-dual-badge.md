# CitizenApp 认证真源改链上 + 投票蓝/竞选红双色徽章

## 任务需求

用户主页（及全端）的"是否认证/认证投票公民还是竞选公民"唯一来源必须是**链上身份**，不再是"有没有发帖"。链上有投票身份 → 认证投票公民；链上有竞选身份 → 认证竞选公民。用户主页认证徽章改两种：**投票=蓝、竞选=红**（访客=无徽章）。用户已定：①范围=四处认证展示点全统一；②只徽章分色，名字旁不加文字；③生产链 RPC 地址由用户提供，我配置。

## 背景（已实证，见 wf 分析）

- 现状 bug：主页 `is_certified = cidNumber!==null`，cidNumber 来自 D1「最近一条带 cid 的已发布帖」（service.ts:76 + repository.ts:94 `readLatestCidNumber`）→ 没发帖=未认证，且不反映护照过期/吊销。
- 链上真源现成：`fetchChainIdentityState`（chain/identity.ts:24）已同时读 `VotingIdentityByAccount`+`CandidateIdentityByAccount`、校验护照窗口+status=normal，返回 `identity_level`(visitor/voting/candidate)。会员校验已在用。
- 读链成本：每次 2 个 RPC、单端点 `env.SQUARE_CHAIN_RPC_URL`（secret，未配→503）、无缓存无超时。
- 徽章色：`AppTheme.voting=0xFF3B82F6`(蓝)、`AppTheme.danger=0xFFEF4444`(红)；现单色 `0xFF007A74`。
- 四处认证展示点：①主页头像(profile_header_card，BFF)②我的tab头像(user.dart，MyIdService链直读，仅投票)③广场帖子作者(square_post_card，feed cid)④广场首页顶栏(square_home_page，SquareIdentityService链直读，仅投票)。当前全只认投票表、单色。

## 分步骤技术方案

### Phase 1 · Worker（本轮）
- rpc.ts：`fetchChainStorage` 的 fetch 加 `AbortSignal.timeout(3000)`，超时→504。
- identity.ts：新增 `fetchChainIdentityStateCached`——KV `square_identity:<acct>` 45s 短缓存 + 读链失败**软降级为访客**（不抛错）。
- profiles/service.ts `buildProfileResponse`：`readLatestCidNumber`→`fetchChainIdentityStateCached`；`is_certified = identity_level!=='visitor'`；响应加 `identity_level`。
- types.ts `UserProfileResponse` 加 `identity_level`。
- repository.ts 删死代码 `readLatestCidNumber`。
- 验收：`npm run typecheck` + `npm test`（更新 profiles.test.ts 认证桩为链上桩）。

### Phase 2 · 客户端 point1 主页徽章（本轮）
- citizen_profile.dart 加 `identityLevel`（解析 `identity_level`）。
- 新增共享 `identityBadgeColor(level)`（voting→蓝/candidate→红/其它→null）helper 或 CitizenBadge，收敛四处。
- profile_header_card.dart 头像徽章按 level 出蓝/红，visitor 不显示。
- 验收：flutter analyze + test/8964/profile。

### Phase 3 · 客户端 point2/point4（我的tab + 广场顶栏，链直读，本轮或次轮）
- myid_service.dart 补读 `CandidateIdentityByAccount`，MyIdState 加 identityLevel(none/voting/candidate)。
- user.dart _SquareAvatar 徽章分色；_identityLevelName 文案改"认证投票公民/认证竞选公民"。
- square_identity_state.dart + square_chain_service.dart 补读候选表；square_home_page 顶栏分色。

### Phase 4 · 广场帖子作者徽章 point3（次轮，有成本权衡）
- feed 逐作者实时读链太贵（N×2 RPC）。改**投影**：post confirm 时把 identity_level 落进 square_posts（新增列 + migration），feed 读 D1 投影。SquareAuthor 加 identityLevel，square_post_card 两处徽章分色。

## 主要风险点

- ⚠️ 生产 Worker secrets 为空，`SQUARE_CHAIN_RPC_URL` 未配 → 部署后主页读链软降级为"全员未认证"（比现状发帖投影更严格）。**必须等用户给 RPC 地址、配好 secret 再部署**，避免认证空窗。
- is_certified 语义变更 + 新增 identity_level = 响应契约变更，Dart 端解析需同步（Phase 2）。
- 读链无缓存会拖垮主页首屏 → Phase 1 的 KV 缓存 + 超时 + 软降级是硬要求。
- 四处口径统一，抽共享 helper，避免各改各的漂移。

## 是否需要先沟通

- 否。三决策用户已定。RPC 地址待用户提供后配 secret + 部署。

## 输出物 / 验收

- Worker typecheck+test 绿；Flutter analyze+profile test 绿；四处徽章分色一致；无死代码残留。
- 真机：链上投票公民→蓝徽章、竞选公民→红徽章、访客→无；不依赖发帖。

## 完成记录（2026-07-09）

### Phase 1 · Worker —— 完成
- rpc.ts：state_getStorage 加 `AbortSignal.timeout(3000)`，超时→504。
- identity.ts：新增 `fetchChainIdentityStateCached`（FEED_CACHE `square_identity:<acct>` 45s 缓存 + 读链失败软降级为访客，不抛错）。
- profiles/service.ts `buildProfileResponse`：`readLatestCidNumber`→`fetchChainIdentityStateCached`；`is_certified=identity_level!=='visitor'`；响应加 `identity_level`。
- types.ts `UserProfileResponse` 加 `identity_level`。repository.ts 删死代码 `readLatestCidNumber`。
- 测试：profiles.test.ts 认证桩改链上（往 FEED_CACHE 塞 `square_identity:`）+ 新增竞选/访客软降级用例。typecheck 干净、`npm test` **65 项全绿**。
- **未部署**：等用户给 `SQUARE_CHAIN_RPC_URL` 生产地址，配 secret + 部署后链上认证生效；未配前主页软降级为未认证（不崩）。

### Phase 2 · 客户端主页徽章 point1 —— 完成
- citizen_profile.dart 加 `identityLevel`（未知 fail-closed→visitor）。
- 新增 `lib/ui/identity_badge.dart`：`identityBadgeColor`(voting→蓝/candidate→红/其它→null)+`identityBadgeLabel`。
- profile_header_card.dart 头像徽章按 level 出蓝/红勾（visitor 不显）+Tooltip/semanticLabel。
- 测试：新增「voting=蓝/candidate=红」两用例；`flutter test test/8964/profile` **40 项全绿**、analyze 干净。
- profile_header_card/user_profile_page 有并行线程 fallbackName 改动，本次只动徽章。

### 待续
- Phase 3：我的 tab（user.dart+myid_service 补候选表）、广场顶栏（square_identity_state+square_chain_service 补候选表）分色。
- Phase 4：广场帖子作者——走 post confirm 投影（square_posts 加 identity_level 列+migration）。
- 部署 + 配 RPC secret（待用户给地址）。

## 徽章重新设计（2026-07-09 晚，用户定稿）

规则升级为**双信号**：颜色=链上身份档（访客橙/投票蓝/竞选红）；**勾=购买的会员档匹配身份档且有效**。用户三决策：①会员公开、全端显勾；②纯访客（无身份无会员）无徽章；③一次做全四处。矩阵：visitor+访客会员→橙无勾；voting+无/低/过期→蓝空心环；voting+投票会员→蓝勾；candidate+无/低/过期→红空心环；candidate+竞选会员→红勾。visitor 永不带勾。

### 已完成（Worker + 主页 point1，已验证）
- Worker：`UserProfileResponse` 加 `membership_level`+`membership_active`；`buildProfileResponse` Promise.all 加 `getMembership`（+1 D1 读），`membership_active=subscriptionIsActive`。typecheck 干净、`npm test` **67 项全绿**（新增会员低于身份档/过期两用例，FakeDb 加 square_memberships 桩）。
- 客户端主页：`app_theme` 加 identityVisitor(gold)/identityVoting/identityCandidate；`identity_badge.dart` 重写为 `identityBadgeStyle`(颜色+checked) + 共享两态 `CitizenBadge`(带勾=实心+白勾/不带勾=彩色空心环)；`CitizenProfile` 加 membershipLevel/membershipActive；profile_header_card `_Avatar` 用 CitizenBadge；缓存前缀 bump v2。`flutter test test/8964/profile` **42 项全绿**（新增橙/蓝/红×带勾/空心环矩阵用例）、analyze 干净。

### 更正：feed 作者身份色无需投影/迁移（用户纠正，已复核）
链上身份**统一经 Cloudflare 读国储会节点 RPC**，主页和 feed 同源。feed 只是把主页单作者扩成"本页去重作者集"：`resolveAuthorSignals`（新文件 src/social/author_signals.ts）对去重 owner 并发 `fetchChainIdentityStateCached`（KV 缓存+软降级）+ `batchMemberships` 一条 IN() 批量读会员。M≤50→最坏 100 RPC，远低于付费 subrequest 限（10000），6 连接自动节流。**零 schema 变更、零迁移**。

### 已完成（Worker 全部 + 客户端 point1，已验证）
- Worker 主页：identity_level + membership_level + membership_active（buildProfileResponse）。
- Worker feed 作者：`resolveAuthorSignals` + `batchMemberships`；`hydrateFeedItems`(posts/repository) 与 `listAuthorPosts`(profiles/repository) 均去重作者读信号回填；`SquarePostFeedItem` 加 identity_level/membership_level/membership_active。typecheck 干净、`npm test` **67 全绿**。
- 客户端 point1 主页：双态 CitizenBadge（橙/蓝/红 + 勾/空心环），42 profile 测试全绿。

### 客户端四处全部完成（2026-07-09，已验证）
- point3 帖子作者：SquareAuthor 加 identityLevel/membershipLevel/membershipActive；_parsePost 解析；square_post_card 头像中性化 + 名字旁 CitizenBadge。
- point2 我的tab：myid_service 补读 CandidateIdentityByAccount → MyIdState.identityLevel；user.dart _loadState 非阻塞拉会员(_refreshMembership)；_SquareAvatar 换 CitizenBadge。
- point4 广场顶栏：square_chain_service 加 fetchIdentity(投票+候选)；SquareIdentityState.identityLevel；square_home_page 非阻塞拉会员 + 顶栏 CitizenBadge。
- 验收：`flutter analyze lib` 干净；`flutter test test/8964` 65 全绿、myid/square_chain 全绿；Worker `npm test` 67 全绿。会员一律非阻塞加载（身份先渲染，勾稍后补）。
- 注：test/wallet/wallet_manager_test 6 失败属并行线程 seed 迁移（biometric），非本卡改动。
- **仍缺**：生产 `SQUARE_CHAIN_RPC_URL` secret（待用户给地址）+ 部署；未配前主页/feed 颜色软降级为访客→无徽章。

### 徽章样式二次定稿（2026-07-09 晚，用户拍板，已实现）
推翻我擅自改的"实心圆/空心环"，改成**推特式扇贝勋章**（`CitizenBadge` 用 `_RosetteBadgePainter` CustomPaint 画：8 花瓣+中心圆，白底盘）。规则最终简化：
- 颜色 = 链上身份档：访客橙 `identityVisitor` / 投票蓝 `identityVoting` / 竞选红 `identityCandidate`。
- 内符号：**有生效会员 → 白色对勾**；**只有身份/纯访客 → 白色小人（头+肩）**。
- **全端统一、人人都有徽章**（纯访客=橙+小人）；`identityBadgeStyle` 恒返回非空，判定 `checked=membershipActive`（任意生效会员即带勾，不再看档位匹配）。
- 四处调用点未改（沿用 CitizenBadge），profile 测试更新两条（纯访客现有橙人徽章；竞选+任意会员现带勾）。`flutter test test/8964` 65 全绿、analyze 干净。
- 教训：徽章样式属用户设计决策，不得擅自改；改前先出可视稿获批。
- 像素表现（勋章瓣形/小人姿势）需真机目测；Worker 数据不变、无需重部署此项。

### （历史）待续项已并入上节
- point3 广场帖子作者：`square_models.dart` SquareAuthor 加 identityLevel/membershipLevel/membershipActive；`square_api_client _parsePost` 解析扁平字段 data['identity_level'] 等；`square_post_card.dart` 两处徽章换 CitizenBadge（identityBadgeStyle）。
- point2 我的tab：user.dart _loadState 加 fetchMembership + myid_service 补读 CandidateIdentityByAccount 给 identityLevel；_SquareAvatar 换 CitizenBadge。
- point4 广场顶栏：square_identity_state 补候选表 + 引 fetchMembership；顶栏换 CitizenBadge。
- 部署 + 配 RPC secret（待用户给地址；主页/feed 颜色靠链读，未配 RPC 前软降级为访客→无徽章）。
