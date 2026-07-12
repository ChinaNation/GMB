# CitizenApp 会员订阅与广场媒体额度

## 任务需求

- 在公民 App「我的」Tab 中，在钱包和通讯录之间新增会员入口。
- 会员页展示四档会员介绍和当前订阅状态；App 按状态提供“订阅会员 / 取消订阅 / 续订会员”，命令统一打开官网，不在 App 内嵌支付。
- 订阅优先走官网 Stripe，用户用钱包账户订阅后，iPhone / Android / GitHub Android 版均通过同一钱包账户显示会员状态。
- 四档会员体系统一为：
  - 自由会员：访客身份可订阅，2.99 美元 / 月。
  - 民主会员：访客身份可订阅，9.99 美元 / 月，媒体额度对齐投票公民会员。
  - 投票公民会员：必须存在有效 `VotingIdentityByAccount`。
  - 竞选公民会员：必须存在有效 `CandidateIdentityByAccount`。
- 普通动态 / 普通文章四档会员都可发布，但按会员等级限制文字、图片、视频时长、清晰度和数量。
- 竞选动态 / 竞选文章只有竞选公民会员可发布。
- 图片走 Cloudflare Images，视频走 Cloudflare Stream；公民 App 不自建传统服务端，服务端能力统一放在 Cloudflare Worker。
- 链上只放身份真源、发布唯一记录、必要哈希和必要回执；Cloudflare 放 App 用户快速访问的数据；设备本地放钱包私钥、草稿、缓存、上传队列等本地状态。
- 删除帖子时删除 Cloudflare 中的正文、媒体和 Feed 展示数据，链上发布记录不改写。
- 修改帖子视为发布新内容，重新支付发布费、生成新的 `post_id` / `content_hash` / `storage_receipt_id`，新发布成功后删除旧 Cloudflare 数据。
- 开发流程分步骤执行；每一步先输出方案，用户确认后再执行。

## 所属模块

- citizenapp
- citizenapp/cloudflare
- citizenchain/runtime（默认只读复查；任何修改必须二次确认）
- memory

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/agent-rules.md`
- `memory/07-ai/chat-protocol.md`
- `memory/07-ai/requirement-analysis-template.md`
- `memory/07-ai/thread-model.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/workflow.md`
- `memory/07-ai/context-loading-order.md`
- `memory/07-ai/document-boundaries.md`
- `memory/07-ai/definition-of-done.md`
- `memory/07-ai/pre-submit-checklist.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`

## 新增任务卡确认记录

- 用户已在当前任务明确允许创建本任务卡。
- 新增路径：`memory/08-tasks/open/20260707-citizenapp-membership.md`
- 用途：记录公民 App 会员订阅、Cloudflare Images / Stream 媒体额度、广场修改删除流程的分步骤实施计划。
- 原因：这是正式开发任务，仓库规则要求必须先有 `memory/08-tasks/` 下任务卡。
- 是否会被 Git 跟踪：是。

## 第 2 步新增文件确认记录

- 用户已在当前任务明确允许创建以下第 2 步文件：
  - `citizenapp/cloudflare/migrations/0006_media_provider_assets.sql`
    - 用途：新增 `square_media_assets` 表，记录 Images / Stream provider asset、上传方式、状态、播放地址、时长和尺寸。
    - 原因：旧 `square_uploads.object_keys_json` 只适合 R2 object key，不适合作为 Images / Stream 状态真源。
    - 是否会被 Git 跟踪：是。
  - `citizenapp/cloudflare/src/media/cloudflare_assets.ts`
    - 用途：封装 Cloudflare Images Direct Creator Upload、Cloudflare Stream direct upload / tus、状态刷新和播放 URL 生成。
    - 原因：避免 Images / Stream API 逻辑散落在 uploads service。
    - 是否会被 Git 跟踪：是。
  - `citizenapp/cloudflare/test/media_assets.test.ts`
    - 用途：覆盖 Direct Upload URL 生成、provider asset、Stream tus 和 webhook 状态映射。
    - 原因：广场主媒体存储迁移是新主流程，必须有单元测试。
    - 是否会被 Git 跟踪：是。

## 第 4 步新增文件确认记录

- 用户已在当前任务明确允许创建以下第 4 步文件：
  - `citizenapp/cloudflare/src/uploads/quota.ts`
    - 用途：集中实现会员等级、普通 / 竞选分类、动态 / 文章形态、标题 / 正文 / 图片 / 视频数量的 Worker 强制校验。
    - 原因：额度校验必须在 Cloudflare Worker 侧强制执行，不能只依赖 App UI。
    - 是否会被 Git 跟踪：是。
  - `citizenapp/cloudflare/test/uploads_quota.test.ts`
    - 用途：覆盖四档会员额度、竞选会员限制和 manifest / 媒体资产一致性校验。
    - 原因：额度校验是发布入口的安全边界，必须有独立单元测试。
    - 是否会被 Git 跟踪：是。

## 核心边界

- 公民 App 没有自建传统服务端；会员、订阅回调、上传签发、额度校验、Feed 状态统一放 Cloudflare Worker。
- 官网订阅先只接 Stripe；RevenueCat 仅在后续同时接 Apple IAP / Google Play Billing 时再引入。
- App Store / Google Play 版本会员页展示会员状态和四档介绍，并按状态提供打开官网的订阅、取消、续订命令；支付仍完全发生在官网 Stripe Checkout。
- 会员套餐只按美元计价和展示；Stripe Checkout 可让用户使用本地法币或 USDC 支付，但本地币种和 USDC 只属于支付呈现 / 结算能力，不改变会员套餐价格。
- USDC 是目标虚拟货币支付方式；USDT 不作为目标支付方式。
- Stripe secret、Cloudflare API token、webhook secret 不得进入 App 或仓库明文。
- Cloudflare Worker 是会员和发布额度的强制校验点；App 端校验只作为交互提示。
- 图片不得继续按旧 R2 主存储方案扩展，目标态改为 Cloudflare Images。
- 视频不得继续按旧 R2 主存储方案扩展，目标态改为 Cloudflare Stream。
- Runtime 不存正文、文章、图片、视频、支付订阅信息和会员额度。
- `post_category` 继续保持 `Normal` / `Campaign` 精简分类；动态 / 文章由 Cloudflare manifest 的 `content_format` 表达。
- 普通内容四档会员均可发；竞选内容只能竞选公民会员发。
- 删除只清 Cloudflare 展示与媒体数据；链上发布记录不可改写。
- 修改视为新发布；新发布成功后旧 Cloudflare 数据删除，失败则旧内容保留。
- 涉及 `citizenchain/runtime/` 的任何修改，执行前必须单独列完整路径、预计改动内容和原因，并取得用户二次确认。

## 目标会员额度

### 自由会员

- 价格：官网订阅 2.99 美元 / 月，会员页只展示美元价格。
- 动态：文字 300 字；标清图片 9 张；横屏或竖屏标清视频 1 分钟。
- 文章：正文 20000 字；标清图片 50 张；高清首图 1 张；标题 10-50 字。

### 民主会员

- 价格：官网订阅 9.99 美元 / 月，会员页只展示美元价格。
- 资格：没有投票或竞选链上身份的访客钱包账户。
- 动态：文字 300 字；高清图片 9 张；横屏或竖屏高清视频 30 分钟。
- 文章：正文 30000 字；高清图片 100 张；高清首图 1 张；标题 10-50 字。

### 投票公民会员

- 价格：官网订阅 9.99 美元 / 月，会员页只展示美元价格。
- 资格：有效 `VotingIdentityByAccount`。
- 动态：文字 300 字；高清图片 9 张；横屏或竖屏高清视频 30 分钟。
- 文章：正文 30000 字；高清图片 100 张；高清首图 1 张；标题 10-50 字。

### 竞选公民会员

- 价格：官网订阅 99.99 美元 / 月，会员页只展示美元价格。
- 资格：有效 `CandidateIdentityByAccount`。
- 动态：文字 300 字；高清图片 9 张；横屏或竖屏高清视频 3 小时。
- 文章：正文 30000 字；高清图片 100 张；高清首图 1 张；标题 10-50 字。
- 只有本会员可发布竞选动态和竞选文章。

## 分阶段计划

1. **第 0 步：任务卡与现状复查**
   - 创建任务卡。
   - 只读复查发布费的真实实现位置；当前目标态按最低链上费用扣 0.1 元公民币。
   - 只读复查现有会员、上传、身份读取、发布确认实现。
   - 输出第 1 步 Cloudflare 会员系统方案。

2. **第 1 步：Cloudflare 会员系统**
   - D1 建立官网订阅、会员等级、资格快照和权益状态。
   - Stripe webhook 写入 Cloudflare Worker。
   - Worker 查询链上 `VotingIdentityByAccount` / `CandidateIdentityByAccount` 强制校验可订阅等级。
   - App 查询会员状态，订阅 / 取消 / 续订命令只打开官网，不在 App 内处理支付。

3. **第 2 步：Cloudflare Images / Stream 上传**
   - 图片改走 Cloudflare Images Direct Creator Upload。
   - 视频改走 Cloudflare Stream Direct Creator Upload。
   - Worker 签发一次性上传 URL，回调后复查尺寸、数量、时长和清晰度。
   - 清理旧 R2 媒体主流程残留。

4. **第 3 步：发帖与文章额度强校验**
   - Worker 按 `membership_level`、`post_category`、`content_format` 校验动态和文章。
   - 普通动态 / 普通文章允许四档会员发布。
   - 竞选动态 / 竞选文章只允许竞选公民会员发布。
   - manifest 写入 `content_format`，链上继续只传 `post_category`。

5. **第 4 步：公民 App 会员 UI**
   - 「我的」Tab 钱包和通讯录之间新增会员入口。
   - 新增会员页，展示当前会员状态、四档介绍和订阅来源说明，并按状态打开官网订阅 / 取消 / 续订。
   - App 发布页按 Worker 返回额度提示用户。

6. **第 5 步：修改与删除流程**
   - 删除调用 Cloudflare 清理正文、图片、视频和 Feed 展示数据。
   - 修改走新发布流程，重新支付发布费，新发布成功后删除旧 Cloudflare 数据。
   - 新发布失败时旧内容保持不动。

7. **第 6 步：真实验收与文档收口**
   - 使用真实本地 Worker、D1、HTTP 接口和 App 页面验收。
   - 验证四档会员、三档身份资格、普通 / 竞选权限、图片 / 视频限制、订阅 / 取消 / 续订、修改、删除。
   - 更新技术文档、协议登记、任务卡执行记录。
   - 清理旧 R2 媒体主流程、旧会员口径、旧 UI 文案和测试残留。

## 必须遵守

- 不可突破模块边界。
- 不可绕过既有链上身份真源。
- 不可把 Cloudflare 或 App 自报身份当作认证真源。
- 不可在 App、仓库或日志中泄露 Stripe / Cloudflare secret。
- 不可保留旧 R2 媒体主流程作为兼容分支，除非用户在当前任务明确要求兼容。
- 不清楚逻辑时先沟通。
- runtime 任何修改必须先取得单独二次确认。

## 输出物

- Cloudflare Worker 代码。
- D1 迁移。
- Flutter UI 和发布流程代码。
- 中文注释。
- 测试。
- 技术文档更新。
- 残留清理记录。
- 真实运行态验收记录。

## 验收标准

- 官网 Stripe 订阅能写入 Cloudflare D1 会员状态。
- App 同一钱包账户在不同平台能读取同一会员状态。
- 访客 / 投票公民 / 竞选公民三档权限和额度由 Worker 强制校验。
- 竞选动态 / 竞选文章只有竞选公民会员可发布。
- 图片走 Cloudflare Images，视频走 Cloudflare Stream。
- 删除能清理 Cloudflare 展示和媒体数据，不改写链上记录。
- 修改能作为新发布重新扣费并生成新链上记录，新发布成功后旧 Cloudflare 数据被删除。
- 代码测试通过。
- 真实本地服务、真实 D1、真实 HTTP 接口和真实 App 页面验收通过。
- 文档已更新。
- 残留已清理。
- 模块级完成标准已对照。

## 执行记录

### 第 0 步（完成，任务卡与现状复查）

- 已创建任务卡。
- 发布费目标态不是 `square-post` pallet 内部直接扣款，而是 `RuntimeFeeKindClassifier` 将 `RuntimeCall::SquarePost(_)` 归类为 `FeeChargeKind::OnchainAmount(0)`；`OnchainChargeAdapter` 按 `ONCHAIN_MIN_FEE = 10` 分收取 0.1 元。
- `OnchainChargeAdapter` 会计算、预检查、扣费并交给 `OnchainFeeRouter` 按 8:1:1 分账；runtime 测试覆盖 `SquarePost` 归类和最低链上费用，`VOTE_FLAT_FEE = 100` 分保持不变。
- 只读复查确认：`square-post` 链上仍只记录 `post_id`、`owner_account`、可空 `cid_number`、`post_category`、`content_hash`、`storage_receipt_id`、`storage_until`、`created_block`。
- 只读复查确认：当前 runtime 的 `Campaign` 发布只要求有效 `VotingIdentityByAccount`，不要求 `CandidateIdentityByAccount`；本任务先由 Cloudflare 强制竞选公民会员才能发布竞选内容，默认不改 runtime。
- 只读复查确认：Cloudflare 当前会员表只有 `owner_account`、`membership_level`、存储容量、已用容量、到期时间；没有 Stripe webhook、官网订阅、三档资格、权益快照。
- 只读复查确认：Cloudflare 当前媒体主流程仍是 R2 预签名上传和 R2 读取；尚未接入 Cloudflare Images / Stream。
- 只读复查确认：Worker 当前只有链上事件确认读取能力，没有通用 `VotingIdentityByAccount` / `CandidateIdentityByAccount` 查询服务。
- 只读复查确认：App 当前只读 `VotingIdentityByAccount` 作为 `isCertified`；没有 Candidate 身份状态。
- 只读复查确认：动态发布页和文章发布页当时仍是旧硬编码，未按会员额度驱动。
- 只读复查确认：当前没有广场帖子删除 / 修改正式接口；后续需新增 Cloudflare 删除接口，修改按新发布流程处理。

### 第 1 步（完成，Cloudflare 会员系统）

- 新增 D1 迁移 `citizenapp/cloudflare/migrations/0005_membership_subscriptions.sql`：`square_memberships` 增加官网 Stripe 订阅字段、订阅状态、周期字段、链上身份等级快照和索引。
- `citizenapp/cloudflare/src/membership/plans.ts`：四档会员 `freedom` / `democracy` / `voting` / `candidate` 的价格、身份要求、动态额度和文章额度收口为单一配置。
- 新增 `citizenapp/cloudflare/src/chain/identity.ts`：Worker 通过 `state_getStorage` 读取 `CitizenIdentity::VotingIdentityByAccount` 与 `CandidateIdentityByAccount`；投票身份需状态 normal 且护照有效期覆盖当前日期，竞选身份必须在有效投票身份基础上存在 Candidate storage。
- 更新 `citizenapp/cloudflare/src/chain/rpc.ts`：抽出通用 `fetchChainStorage`，既服务链上发布事件确认，也服务会员身份资格读取。
- 更新 `citizenapp/cloudflare/src/membership/service.ts`：`GET /v1/square/membership` 返回四档计划、链上身份状态、可订阅等级、订阅是否有效和最终权益是否 active；`requireActiveMembership` 改为支付状态与链上身份资格同时满足才放行。
- `citizenapp/cloudflare/src/membership/webhook.ts`：实现 Stripe webhook 签名校验、subscription created/updated/deleted 处理、checkout session 观察但不直接授权；所有档位按链上身份精确匹配，不满足则记录 `identity_required`，不授予权益。
- `citizenapp/cloudflare/src/routes.ts`：Stripe webhook 唯一路径为 `POST /v1/square/membership/webhook`。
- 更新 `citizenapp/cloudflare/src/types.ts` 和 `wrangler.toml`：登记 Stripe webhook secret、price id、会员订阅字段；secret 不写仓库。
- 新增 `citizenapp/cloudflare/test/membership.test.ts`：覆盖会员查询 Candidate 权益、Candidate 身份不足、Stripe webhook 写入 voting、visitor 不读链、身份不足记录 `identity_required` 和签名失败。
- 更新 `memory/07-ai/unified-protocols.md` 与 `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`：同步 API、D1 字段、环境变量和会员架构边界。
- 验收：
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run typecheck` 通过。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test` 通过：9 个测试文件，37 个测试。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run migrate:local` 通过，`0005_membership_subscriptions.sql` 本地 D1 迁移成功。
  - 真实本地 Worker HTTP 验收通过：`wrangler dev --local --port 8787 --var DEV_UPLOAD_PROXY:1 --var STRIPE_HOOK_SECRET:whsec_test` 启动后，带 Stripe 签名的 freedom subscription webhook 返回 200，D1 查询确认 `sub_http` 写入 `membership_level=freedom`、`subscription_status=active`、`identity_level=visitor`。
- 边界：第 1 步未修改 CitizenApp Flutter UI，未切换 Images / Stream，未改 runtime；竞选公民会员的业务权限已由 Worker 会员系统具备强制校验基础，发布侧接入留到第 3 步。

### 第 1 步补充（完成，美元计价与 USDC 支付口径）

- 根据后续确认，会员套餐统一为美元计价：自由会员 `$2.99/month`、民主会员 `$9.99/month`、投票公民会员 `$9.99/month`、竞选公民会员 `$99.99/month`。
- Stripe Checkout 可让用户选择本地法币或 USDC 支付；人民币、港币、台币等只作为支付呈现 / 换汇结果，不进入会员权益判断；USDT 不作为目标支付方式。
- Worker 会员计划配置补充 `price_currency = usd` 与 `price_usd_cents`，作为 webhook 校验和 App 展示的套餐真源。
- Stripe subscription webhook 在写入 D1 前强制校验 subscription item 的 Price 为 `usd` 且金额匹配会员等级；币种或金额不匹配时拒绝授权。
- 验收：
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run typecheck` 通过。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test -- membership.test.ts` 通过：8 个测试。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test` 通过：10 个测试文件，42 个测试。
  - 真实本地 Worker HTTP 验收通过：`sub_usd_validation` 使用 `usd/299` 的 freedom subscription webhook 返回 200 并写入本地 D1；`sub_hkd_reject` 使用 `hkd/2400` 的 freedom subscription webhook 返回 400 `stripe_price_currency_mismatch`，D1 查询确认未入库。

### 第 2 步（完成，Cloudflare Images / Stream 上传）

- 新增 D1 迁移 `citizenapp/cloudflare/migrations/0006_media_provider_assets.sql`：建立 `square_media_assets` 表和 provider asset / post / state 索引。
- 新增 `citizenapp/cloudflare/src/media/cloudflare_assets.ts`：封装 Cloudflare Images Direct Creator Upload、Cloudflare Stream basic direct upload、Stream tus direct upload、Images / Stream 状态刷新和播放 URL 生成；本地 `DEV_UPLOAD_PROXY=1` 时使用同源 `dev-media` 端点验证完整流程。
- 更新 `citizenapp/cloudflare/src/uploads/service.ts`：prepare 只为 manifest 生成 R2 上传 URL，图片生成 `cloudflare_images` 上传授权，视频生成 `cloudflare_stream` 上传授权；200MB 以上视频走 tus；complete 校验 manifest 与 provider asset 状态，视频转码未完成时返回 `storage_state=processing`；新增 Stream webhook 签名校验并更新视频 ready/error 状态。
- 更新 `citizenapp/cloudflare/src/posts/confirm.ts` 和 feed hydrate：feed 媒体项从 `square_media_assets` 读取 provider、asset id、状态、Images delivery URL、Stream playback URL、缩略图、时长、宽高。
- 更新 `citizenapp/cloudflare/src/media/service.ts`：公开 R2 读取通道收窄为 `profile/` 资料资产；广场主媒体不再从 R2 公开读取。
- 更新 `citizenapp/lib/8964/services/square_api_client.dart`、`square_upload_service.dart`、`square_models.dart`：App 区分 manifest PUT、Images/Stream multipart direct upload、Stream tus PATCH；展示端直接使用 Worker 返回的 Images / Stream URL。
- 更新测试：`media_assets.test.ts`、`uploads.test.ts`、`r2_keys.test.ts`、`chain_confirm.test.ts`、`media.test.ts`、`square_feed_service_test.dart`、`square_publish_service_test.dart`、`square_draft_store_test.dart`。
- 更新文档：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`、`memory/07-ai/unified-protocols.md` 同步 Images / Stream 主媒体、R2 manifest、D1 字段、API 字段、环境变量和安全边界。
- 验收：
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test -- media_assets.test.ts uploads.test.ts r2_keys.test.ts chain_confirm.test.ts media.test.ts` 通过：5 个测试文件，12 个测试。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test` 通过：12 个测试文件，51 个测试。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run migrate:local` 通过，`0006_media_provider_assets.sql` 本地 D1 迁移成功。
  - `flutter analyze lib/8964/services/square_api_client.dart lib/8964/services/square_upload_service.dart lib/8964/models/square_models.dart test/8964/square_publish_service_test.dart test/8964/square_feed_service_test.dart` 通过。
  - `flutter test --concurrency=1 test/8964/square_publish_service_test.dart test/8964/square_feed_service_test.dart` 通过。
  - 真实本地 Worker HTTP 验收通过：真实 sr25519 登录拿 session，Stripe visitor webhook 写会员，`uploads/prepare` 返回 `cloudflare_images` / `cloudflare_stream` provider，manifest 走 R2 `dev-put`，图片/视频走 `dev-media`，`complete` 返回 `storage_state=processing`，签名 Stream webhook 后 D1 中视频 asset 更新为 `ready`、`duration_seconds=4.5`、`width=1280`、`height=720`。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run typecheck` 当前被未跟踪的 `citizenapp/cloudflare/src/chain/extrinsic_relay.ts` 中 `Uint8Array<ArrayBufferLike>` / `BufferSource` 类型错误阻塞；该文件不属于本步骤新增或修改范围，本步骤未改动。
- 边界：第 2 步未修改 `citizenchain/runtime/`；当时尚未接入真实 Cloudflare 远端 Images / Stream 配置，当前已统一使用 `CF_ACCOUNT_ID`、`CF_API_TOKEN`、`IMAGES_URL`、`STREAM_URL`、`STREAM_HOOK_SECRET`。

### 第 3 步（完成，官网 Checkout 与 App 会员官网入口）

- `citizenapp/cloudflare/src/membership/subscribe.ts`：官网先下发钱包签名挑战，验签并精确核验链上身份后创建 Stripe subscription Checkout Session；请求绑定 `owner_account` 与四档 `membership_level`。
- `citizenapp/cloudflare/test/membership_subscribe.test.ts`：覆盖四档订阅挑战、candidate 身份通过/不足、session owner mismatch、Stripe 错误透传。
- `citizenapp/cloudflare/src/routes.ts`、`src/types.ts`、`wrangler.toml`：订阅接口统一为 `POST /v1/square/membership/subscribe/challenge` 与 `POST /v1/square/membership/subscribe`；配置使用 `STRIPE_API_KEY`、`STRIPE_DEV_PROXY`、`CHECKOUT_SUCCESS_URL`、`CHECKOUT_CANCEL_URL`，secret 只放部署环境。
- `citizenweb/src/pages/Membership.tsx`：官网 `/membership` 展示四档美元价格和权益，钱包扫码签名后调用 Worker 完成订阅 / 取消 / 续订；官网不保存会员状态，不接触 Stripe secret。
- `citizenapp/lib/8964/services/square_api_client.dart`：`fetchMembership` 解析 `plans[]`、链上身份、订阅状态、未生效原因，供 App 会员页展示。
- `citizenapp/lib/my/user/user.dart` 与 `lib/my/membership/membership_page.dart`：在「我的」Tab 钱包和通讯录之间提供“会员”入口，展示当前状态、三档身份与四档权益，按状态显示订阅 / 取消 / 续订按钮并打开官网。
- 更新文档：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`、`memory/05-modules/citizenweb/CITIZENWEB_TECHNICAL.md` 同步 Checkout API、官网会员页、App 会员入口、环境变量和安全边界。
- 验收：
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test -- membership_subscribe.test.ts membership.test.ts` 通过：订阅与会员核心测试通过。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test` 通过：13 个测试文件，56 个测试。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run typecheck` 通过。
  - `npm --prefix /Users/rhett/GMB/citizenweb run build` 通过。
  - `flutter analyze citizenapp/lib/8964/services/square_api_client.dart citizenapp/lib/my/user/user.dart` 通过。
  - `flutter test --concurrency=1 test/8964/square_publish_service_test.dart test/8964/square_feed_service_test.dart` 通过：6 个测试。
  - 真实本地 Worker HTTP 验收通过：`wrangler dev --local --port 8787 --var STRIPE_DEV_PROXY:1 ...` 启动后，官网订阅挑战经钱包签名确认，`POST /v1/square/membership/subscribe` 返回 `checkout_session_id=cs_dev_visitor` 与官网成功回跳 URL。
  - `git diff -- citizenchain/runtime` 为空，未修改 runtime。
- 清理：未写入 Stripe / Cloudflare secret；官网不持久化支付状态；App 不内嵌支付，仅按会员状态打开官网。

### 第 4 步（完成，发帖与文章额度强校验）

- 新增 `citizenapp/cloudflare/src/uploads/quota.ts`：集中实现四档会员动态 / 文章额度、普通 / 竞选分类权限、标题 / 正文长度、图片 / 视频数量和 manifest / 媒体资产一致性校验。
- 新增 `citizenapp/cloudflare/test/uploads_quota.test.ts`：覆盖 visitor 动态超字数、candidate 动态 9 图 + 1 视频、非 candidate 发布竞选内容拒绝、visitor 文章正文图超限、candidate 竞选文章通过、manifest 与 `square_media_assets` 一致性校验。
- 更新 `citizenapp/cloudflare/src/uploads/service.ts`：`prepare` 新增 `content_format`、`title_length`、`text_length` 声明字段并按当前有效会员计划先拒绝超额请求；`complete` 读取 R2 manifest，复核 manifest hash、owner、`post_category`、`content_format`、正文 / 标题长度、媒体数量和 provider asset 记录，防止绕过 App 伪造声明。
- 更新 `citizenapp/cloudflare/src/uploads/validation.ts`：基础媒体数量上限从旧 9 个调整为 101 个，具体动态 / 文章数量由会员额度校验决定。
- 更新 `citizenapp/lib/8964/services/square_api_client.dart`：`prepareUpload` 发送 `content_format/title_length/text_length`；会员计划解析补齐 `dynamicMaxVideos`，并提供当前生效计划读取方法。
- 更新 `citizenapp/lib/8964/services/square_upload_service.dart`：App 在取到会员状态后按 Worker 返回的当前计划做前置提示校验；竞选内容必须是竞选公民会员，文章必须有首图且不得含视频。
- 更新 `citizenapp/lib/8964/pages/square_compose_page.dart`：动态发布页改为 300 字，媒体选择改为最多 9 张图片 + 1 个视频。
- 更新 `citizenapp/lib/8964/pages/square_article_compose_page.dart`：文章发布页改为标题 10-50 字、正文 UI 上限 30000 字、正文图 UI 上限 100 张，并支持普通文章 / 竞选文章；链上仍只写 `post_category`。
- 更新测试：`citizenapp/test/8964/square_article_test.dart`、`citizenapp/test/8964/square_feed_service_test.dart`、`citizenapp/test/8964/square_publish_service_test.dart`。
- 更新文档：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`、`memory/07-ai/unified-protocols.md` 和本任务卡同步 `content_format`、额度强校验、App 发布页限制和 R2/Images/Stream 边界。
- 验收：
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run typecheck` 通过。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test` 通过：14 个测试文件，63 个测试。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test -- uploads_quota.test.ts uploads.test.ts media_assets.test.ts membership.test.ts membership_subscribe.test.ts` 通过：上传额度与会员订阅核心测试通过。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run migrate:local` 通过：本地 D1 无待应用迁移。
  - `flutter analyze lib/8964/services/square_api_client.dart lib/8964/services/square_upload_service.dart lib/8964/pages/square_compose_page.dart lib/8964/pages/square_article_compose_page.dart lib/8964/services/square_publish_service.dart lib/my/user/user.dart test/8964/square_article_test.dart test/8964/square_feed_service_test.dart test/8964/square_publish_service_test.dart` 通过。
  - `flutter test test/8964/square_article_test.dart test/8964/square_feed_service_test.dart test/8964/square_publish_service_test.dart` 通过：13 个测试。
  - 真实本地 Worker HTTP 验收通过：`wrangler dev --local --port 8787` 启动后，真实 sr25519 钱包完成 `/auth/challenge -> /auth/session`，签名 Stripe visitor webhook 写入会员，`uploads/prepare` 对 301 字动态声明返回 `dynamic_text_too_long`，伪造 300 字声明但上传 301 字 R2 manifest 时 `complete` 同样返回 `dynamic_text_too_long`，300 字 + 9 图 + 1 视频完整 dev 上传后 `complete` 返回 `storage_state=processing`。
  - `git diff -- citizenchain/runtime` 为空，未修改 runtime。
- 清理：已清理本轮触达的 `citizenapp/cloudflare/src/.DS_Store` 残留；未写入 Stripe / Cloudflare secret；未新增未确认目录；未保留旧内容形态字段、旧文章额度和旧动态字数口径残留。

### 第 5 步（完成，修改与删除流程）

- 更新 `citizenapp/cloudflare/src/media/cloudflare_assets.ts`：新增 Images / Stream provider asset 删除封装；生产环境调用 Cloudflare API 删除真实媒体，`DEV_UPLOAD_PROXY=1` 本地验收环境跳过外部 API。
- 更新 `citizenapp/cloudflare/src/posts/confirm.ts`：新增 `deletePostRoute` / `deletePostCloudflareData`；仅作者本人可删除，删除顺序为 provider asset、R2 manifest、D1 媒体索引、上传任务和帖子行；链上发布索引保持不变。
- 更新 `citizenapp/cloudflare/src/routes.ts`：新增 `DELETE /v1/square/posts/{post_id}`。
- 更新 `citizenapp/lib/8964/services/square_api_client.dart`：新增 `SquarePostDeletionService` 与 `deletePost`，App 调 Worker 删除接口。
- 更新 `citizenapp/lib/8964/services/square_publish_service.dart`：发布服务新增 `replacePostId`；修改内容按新发布走完整“余额校验、链上扣费入块、媒体上传、Worker 确认 feed”，新帖确认成功后再删除旧帖 Cloudflare 数据；旧帖删除失败只返回警告，不把已成功的新帖误存为失败草稿。
- 更新 `citizenapp/lib/8964/pages/square_compose_page.dart`、`square_article_compose_page.dart`：支持修改入口预填正文、标题和分类，并把旧 `post_id` 传入发布服务；远端旧媒体不回填为本地草稿，修改时需要重新选择媒体。
- 更新 `citizenapp/lib/8964/pages/square_post_detail_page.dart`、`square_article_detail_page.dart`：详情页右上角新增“修改 / 删除”；删除前确认，App 端先校验默认热钱包登录态和作者一致，服务端再强制校验。
- 更新 `citizenapp/lib/8964/pages/square_home_page.dart`、`profile/user_profile_page.dart`：详情页返回删除或新版帖子后刷新列表；个人主页通过版本 key 重新挂载帖子列表。
- 更新测试：`citizenapp/cloudflare/test/chain_confirm.test.ts` 覆盖 Cloudflare 数据删除、正文清空、R2 manifest 删除、媒体索引清理、存储用量只回收一次；`citizenapp/test/8964/square_feed_service_test.dart` 覆盖 App DELETE 路径和 Bearer；`citizenapp/test/8964/square_publish_service_test.dart` 覆盖修改发布确认成功后再删除旧帖的顺序。
- 更新文档：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`、`memory/07-ai/unified-protocols.md` 和本任务卡同步删除接口、修改即新发布、Cloudflare / 链上边界。
- 验收：
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare run typecheck` 通过。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test -- chain_confirm.test.ts` 通过：3 个测试。
  - `npm --prefix /Users/rhett/GMB/citizenapp/cloudflare test` 通过：14 个测试文件，64 个测试。
  - `flutter test test/8964/square_feed_service_test.dart` 通过：4 个测试。
  - `flutter test test/8964/square_publish_service_test.dart` 通过：5 个测试。
  - `flutter test test/8964/square_article_detail_test.dart` 通过：1 个测试。
  - `dart analyze lib/8964/pages/square_post_detail_page.dart lib/8964/pages/square_article_detail_page.dart lib/8964/pages/square_home_page.dart lib/8964/profile/user_profile_page.dart lib/8964/pages/square_compose_page.dart lib/8964/pages/square_article_compose_page.dart lib/8964/services/square_api_client.dart lib/8964/services/square_publish_service.dart test/8964/square_feed_service_test.dart test/8964/square_publish_service_test.dart test/8964/square_article_detail_test.dart` 通过。
  - 真实本地 Worker HTTP 删除验收通过：`wrangler dev --local --port 8787 --var DEV_UPLOAD_PROXY:1` 启动后，本地 KV 写入测试 session，本地 D1/R2 写入测试帖子；`DELETE /v1/square/posts/{post_id}` 返回 `post_state=deleted`、`deleted_media_assets=1`、`deleted_r2_objects=1`，D1 查询确认帖子、媒体索引和上传任务均已硬删除。
  - `flutter analyze` 当前仅报无关既有提示 `lib/transaction/onchain-transaction/onchain_payment_service.dart:43:13 prefer_const_constructors`；该文件无本轮 diff，本轮未修改无关交易模块。
  - `git diff -- citizenchain/runtime` 为空，未修改 runtime。
- 清理：本轮未新增文件或目录；未写入 Stripe / Cloudflare secret；未保留“旧帖原地修改”兼容分支；删除接口不改链上记录、不接触 runtime；真实 HTTP smoke 后已删除 `sqp_delete_smoke` / `squ_delete_smoke` / `sqs_delete_smoke` 测试数据和临时 R2 manifest。

### 第 6 步（完成，可执行范围内真实验收与文档收口）

- 四档会员最终口径：自由 `freedom` `$2.99`、民主 `democracy` `$9.99`、投票公民 `voting` `$9.99`、竞选公民 `candidate` `$99.99`；三档身份继续是 `visitor` / `voting` / `candidate`，订阅资格精确匹配。
- App 最终交互：会员页按状态显示“订阅会员 / 取消订阅 / 续订会员”，统一打开 `https://www.crcfrcn.com/membership`；裸域 `crcfrcn.com` 当前无法解析，因此不再作为默认入口。App 不内嵌支付、不保存 Stripe 凭证。
- Worker 最终接口：订阅使用 `/membership/subscribe/challenge` 与 `/membership/subscribe`，取消使用 `/membership/cancel/challenge` 与 `/membership/cancel`，Stripe 回调唯一使用 `/membership/webhook`；已清除重复 `/webhook/webhook` 和旧 checkout/stripe 路径口径。
- 测试补强：`membership.test.ts` 增加 `democracy` 独立 `$9.99 USD` Price webhook 覆盖，并通过 `routeRequest` 验证 webhook 正式路径；Fake D1 实现 `all()`，续订视频回灌路径不再产生被吞掉的测试告警。
- 完整验收：
  - Worker `npm run typecheck` 通过；`npm test` 通过，18 个测试文件、112 个测试。
  - Flutter 会员页 7 个测试通过；会员相关 4 个文件 `flutter analyze` 无问题。
  - 官网 `npm run build` 与 `npm run lint` 通过。
  - `wrangler deploy --dry-run --env staging` 通过，Worker bundle 与 D1 / KV / R2 / Durable Object 绑定解析正常。
  - 真实本地 Worker HTTP：`GET /health` 返回 200；重复 webhook 路径返回 404；正式 `/membership/webhook` 接收签名 Stripe `democracy` 事件返回 200，D1 确认写入 `membership_level=democracy`、`subscription_status=active`、`stripe_price_id=price_democracy`、`identity_level=visitor`。
  - 真实本地官网页面：桌面端四档计划与自由/民主切换正常；390px 移动视口无横向溢出；订阅 / 取消 / 续订入口文案齐全；浏览器控制台无 warning/error。
- 远端边界：第 6 步当时只执行 dry-run；当前已由第 7 步完成 staging/production 迁移与部署，并在 production 完成真实 Stripe、R2、Images、Stream 配置和运行态验收。
- 清理：已删除真实 HTTP smoke 写入的 `sub_step6_http` 本地 D1 数据、临时文档/类型下载、`.DS_Store`，并关闭本地链 mock、Worker、Vite 服务；未新增文件或目录，未写入真实 secret，未修改 `citizenchain/runtime/`。

### 第 7 步（完成，统一短名与远端发布）

- 用户已确认四档会员值改为 `freedom / democracy / voting / candidate`；身份档继续使用 `visitor / voting / candidate`，会员与身份不得混名。
- 用户已确认 Worker、官网和 App 构建变量按 `memory/07-ai/unified-naming.md` 第 5.5 节统一为最多三段的短名，不保留旧环境变量 fallback。
- staging 与 production `square_memberships` 只读查询均无旧会员记录，因此 D1 不保留旧会员值兼容；Stripe 产品和 Price 已核对并按四档短名映射。
- 用户确认聊天设备绑定可纳入本任务；该独立迁移路线随后已被目标基线彻底重建取代。
- 本步骤复用现有文件，不新建目录、任务卡、迁移或测试文件；不修改 `citizenchain/runtime/`。
- 完成条件：代码、配置、测试、注释、当前文档和远端 secret 全部只保留短名；本地真实 HTTP/页面、staging、production、Stripe/Stream 回调和聊天接口完成运行态验收；清理所有旧字段和临时数据。

#### 第 7 步执行记录（2026-07-11）

- 代码已统一四档会员值为 `freedom / democracy / voting / candidate`；身份档继续为 `visitor / voting / candidate`，源代码、测试和构建配置中旧会员值精确扫描为 0。
- Worker、CitizenWeb、CitizenApp 的当前配置已改为 `memory/07-ai/unified-naming.md` 第 5.5 节短名；App 共用 Worker 编译变量补充登记为 `SQUARE_API_URL`，不保留旧变量 fallback。
- Worker `typecheck` 和 18 个测试文件共 112 项测试通过；staging / production dry-run 均通过。Flutter 目标文件 analyze 无问题，会员页 7 项测试通过；CitizenWeb lint / build 通过。
- 本地真实 Worker HTTP 验收通过：`GET /health` 返回 200，会员和聊天未登录接口均返回 401；带 `STRIPE_HOOK_SECRET` 签名的 `democracy` subscription webhook 返回 200，D1 写入 `membership_level=democracy`、`subscription_status=active`、`stripe_price_id=price_democracy`、`identity_level=visitor`，随后测试记录已删除。
- staging 和 production 后续均已按目标基线彻底重建；只保留设备、一次性 KeyPackage、防重放摘要和短期 TURN 表，不存在聊天内容表。
- staging 与 production 的链、R2、Cloudflare 媒体、Stripe Secret 已统一为短名；远端旧 Secret 已删除。production 已配置 `STREAM_HOOK_SECRET`，密钥只存在 Cloudflare Secret，不落盘、不进仓库。
- Cloudflare Images/Stream Starter 套餐已启用；staging/production `IMAGES_URL`、`STREAM_URL` 已指向真实交付域名。Stream 账户唯一 webhook 已指向 production `/v1/square/uploads/stream/webhook`。
- staging 发布版本 `b0d3e5c3-20ec-4b90-823c-863fdd8a6730`；production 最终发布版本 `58572220-5154-4fda-8520-3d2b5cf1df5c`。两环境 health/bootstrap 返回 200，会员和聊天未登录返回 401，开发上传入口返回 404；production Stripe/Stream 未签名回调返回 400。
- production 真实媒体签发验收通过：临时钱包登录和 freedom 会员记录经 Worker 取得 R2 manifest、Images、Stream 三类上传授权，公共交付 URL 与账户配置一致；真实 Stream 签名 webhook 返回 200。验收后删除临时 provider asset、D1 会员/上传/媒体/挑战记录和 KV 会话，Cloudflare Stream/Images 资产均为 0。
- production 已启用退订满 90 天视频冷归档，远端绑定定时扫描返回 200；staging 和本地保持关闭。代码、配置、注释、文档和运行态均已收口，未修改 `citizenchain/runtime/`，未推送 GitHub。

- 状态：done

## 完成信息

- 完成时间：2026-07-11 17:11:03
- 完成摘要：完成四档会员、官网 Stripe 订阅、Cloudflare Images/Stream 媒体链路、staging/production 部署、真实生产验收与残留清理
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
