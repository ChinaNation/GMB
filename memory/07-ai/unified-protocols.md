# GMB 统一协议文件

## 1. 定位

本文件是 GMB AI 编程系统的统一协议入口。

以后任何设计、修改、删除下列内容之前，必须先查本文件：

- 扫码协议
- 二维码 `kind` / `body` / `payload` 结构
- 链上交易 call data 字段顺序
- SCALE 编码载荷格式
- CID / CitizenApp / citizenchain 之间的 API 契约
- 签名、验签、防重放、nonce、era、fixture 规则
- storage key、subject id、action、pallet/call index 等跨端字段契约

本文件负责统一“协议名称、边界、字段、规则、真源、测试”。详细技术文档可以继续放在 `memory/01-architecture/` 或 `memory/05-modules/`，但必须从本文件登记和跳转。

## 2. 强制规则

1. 不允许在代码、文档、测试里直接发明新协议名。新协议名必须先登记到本文件。
2. 不允许把“内层交易载荷格式”说成“新增扫码协议”。扫码协议和载荷格式必须分层命名。
3. 修改字段顺序、字段名、编码类型、签名 payload、nonce、era、pallet/call index 前，必须先更新本文件对应条目。
4. 每个协议条目必须写清楚：名称、类型、唯一真源、生产者、消费者、字段、编码、验收测试。
5. 详细协议文档自称“唯一事实源”时，必须在本文件有对应登记；否则不得自称唯一事实源。
6. 废弃协议不得直接删除，必须先在本文件标记 `废弃`，写清替代协议和清理范围。

## 3. 统一术语

| 术语 | 含义 | 是否扫码协议 |
|---|---|---|
| 扫码协议 | 二维码外层 envelope 和 `k` 流向规则 | 是 |
| 签名请求 | `QR_V1` 下的 `k = 1` | 否，属于扫码协议中的一种流向 |
| 交易载荷格式 | `b.d` 中某个链上 call data 的字段顺序和编码 | 否 |
| 接口契约 | HTTP / Tauri command / app API 的路径、字段和错误规则 | 否 |
| 凭证载荷 | CID 等系统签发给链端验签的 payload 字段 | 否 |
| storage 契约 | pallet storage 名称、key 类型、读取方和写入方规则 | 否 |

死规则：

```text
扫码协议只有一个：QR_V1。
b.d 里可以有很多不同交易载荷格式，但它们都不是新的扫码协议。
```

## 4. 协议登记模板

新增或修改协议时，按这个模板登记：

```text
### 编号：协议名称

- 状态：当前 / 草案 / 废弃
- 类型：扫码协议 / 交易载荷格式 / 接口契约 / 凭证载荷 / storage 契约
- 唯一真源：
- 详细文档：
- 生产者：
- 消费者：
- 字段：
- 编码：
- 签名/验签规则：
- 禁止兼容：
- 禁止事项：
- 必跑测试：
```

## 5. 当前协议登记

### P-CID-001：CID_NUMBER_V1

- 状态：当前
- 类型：接口契约 / 编码协议
- 唯一真源：`citizenchain/onchina/src/cid/validator.rs`
- 详细文档：`memory/05-modules/citizenchain/onchina/DATA_SECURITY_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/cid/generator.rs`
- 消费者：`citizenchain/onchina`、`citizenapp`、`citizenwallet`、`citizenchain`
- 字段：
  1. `R5`:省码 2 位 + 市码 3 位
  2. `K3`:主体属性 `K1` + 机构类型 `T2`
  3. `P1`:盈利属性
  4. `C1`:校验位
  5. `N9`:9 位稳定散列序列
  6. `D4`:年份
- 编码：`R5-K3P1C1-N9-D4`,示例 `LN001-NRC0G-944805165-2026`
- 签名/验签规则：本协议只定义身份号码格式;链上或二维码签名按对应协议条目执行。
- 禁止兼容：不兼容历史格式,不保留历史字段别名。
- 禁止事项：
  - 禁止在 OnChina 内部继续使用身份字段别名
  - 禁止恢复独立历史主体属性段
  - 禁止跳过 `C1` 校验
- 必跑测试：`cargo test --manifest-path citizenchain/onchina/Cargo.toml number::`

### P-API-ONCHINA-001：OnChina 管理员登录态工作台契约

- 状态：当前
- 类型：接口契约
- 唯一真源：
  - 后端：`citizenchain/onchina/src/auth/login/model.rs`
  - 工作台清单：`citizenchain/onchina/src/workspace/model.rs`
  - 前端：`citizenchain/onchina/frontend/auth/types.ts`、`citizenchain/onchina/frontend/workspace/types.ts`
- 详细文档：
  - `memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
  - `memory/05-modules/citizenchain/onchina/FRONTEND_TECHNICAL.md`
- 生产者：OnChina 登录、扫码登录轮询、`/api/v1/admin/auth/check`、`/api/v1/admin/auth/identify`、`/api/v1/admin/own-institution`
- 消费者：OnChina 前端 `AuthContext` 和 `workspace/WorkspaceRouter`
- 字段：
  - 登录态继续携带 `institution_code`、`cid_number`、`cid_full_name`、`cid_short_name`、`admin_name`、`capabilities`
  - `workspace`
  - `workspace.workspace_kind`: `registry` / `judicial` / `legislation` / `generic`
  - `workspace.workspace_title`
  - `workspace.workspace_sections[]`
  - `workspace_sections[].workspace_section`: `operations` / `display` / `records`
  - `workspace_sections[].workspace_section_title`
  - `workspace_sections[].workspace_actions[]`
  - `workspace_actions[].workspace_action`
  - `workspace_actions[].workspace_action_title`
  - `workspace_actions[].workspace_action_enabled`
  - `/api/v1/admin/own-institution` 返回 `InstitutionDetailOutput`: `institution`、`accounts`、`created_by_name`、`created_by_role`
- 编码：HTTP JSON,字段统一 snake_case;前端类型保持 snake_case,不另造 lowerCamelCase API 别名。
- 签名/验签规则：本契约只描述登录态返回;管理员身份仍由 QR_V1 登录签名、节点绑定和链上 active admins 校验决定。
- 禁止兼容：不得恢复“注册局根 UI + 非注册局只塞一个 tab”的旧口径;不得新增第二套 `dashboard` / `console` / `tenant` 同义字段。
- 禁止事项：
  - 禁止把 `workspace` 作为管理员授权真源。
  - 禁止前端根据本地硬编码越过后端 `capabilities` 显示受限操作。
  - 禁止非注册局机构复用注册局业务 UI。
- 必跑测试：
  - `cargo check --manifest-path citizenchain/onchina/Cargo.toml`
  - `npm --prefix citizenchain/onchina/frontend run build`
  - 真实本地 OnChina 服务的 `/api/v1/admin/auth/check` 和真实页面验收

### P-QR-001：QR_V1

- 状态：当前
- 类型：扫码协议
- 唯一真源：`memory/01-architecture/qr/qr-protocol-spec.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenapp`、`citizenchain/node`、`citizenchain/onchina`
- 消费者：`citizenwallet`、`citizenapp`、`citizenchain/onchina`
- 字段：顶层只允许 `p/k/i/e/b`;具体字段以 `qr-protocol-spec.md` 为准
- 编码：紧凑 JSON envelope
- 签名/验签规则：按 `k` 和 `b.a + b.d` 执行;签名响应只带 `u/s`
- 禁止兼容：开发期不做旧协议兼容
- 禁止事项：
  - 禁止新增 `QR_V2`
  - 禁止新增第二套扫码协议字符串
  - 禁止把某个 `b.d` 的交易载荷格式称为新扫码协议
- 必跑测试：QR fixture、citizenwallet/citizenapp QR 解析测试

### P-API-CITIZENAPP-001：CitizenApp 公权机构链上投影查询契约

- 状态：当前
- 类型：接口契约
- 唯一真源：
  - 后端：`citizenchain/onchina/src/citizenapp/public_institution.rs`
  - 移动端：`citizenapp/lib/citizen/public/data/public_institution_api.dart`
  - 快照生成器：`citizenapp/tools/generate_public_institution_bundle.mjs`
- 详细文档：
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
  - `memory/05-modules/citizenapp/rpc/RPC_TECHNICAL.md`
- 生产者：OnChina 链上公权机构投影 BFF、CitizenApp 公权机构创世快照生成器
- 消费者：CitizenApp 公权机构目录、Isar 本地缓存和快照导入服务
- 字段：
  - `GET /api/v1/app/public-institutions` 请求字段：`province_name`、`city_name`、`since_version`、`after_cid`、`page_size`
  - `GET /api/v1/app/public-institutions` 响应分页字段：`items`、`page_size`、`next_cursor`、`has_more`、`manifest_version`、`catalog_status`
  - `items[]` 字段：`cid_number`、`cid_full_name`、`cid_short_name`、`status`、`category`、`p1`、`province_code`、`city_code`、`town_code`、`institution_code`、`has_legal_personality`、`legal_rep_name`、`parent_cid_number`、`account_count`、`custom_account_names`、`created_at`
  - `GET /api/v1/app/public-institutions/version` 响应字段：`province_name`、`city_name`、`manifest_version`、`chain_genesis_hash`、`chain_block_hash`、`chain_block_number`、`synced_at`、`count`
  - CitizenApp 快照 manifest 字段：`schema_version`、`chain_id`、`snapshot_block_number`、`snapshot_block_hash`、`genesis_hash`、`state_root`、`chainspec_hash`、`admin_division_root`、`public_institution_root`、`version`、`generated_at`、`shard_hashes`、`provinces`
- 编码：HTTP JSON,字段统一 snake_case;快照分片为 UTF-8 JSON,分片 hash 为 sha256 hex。
- 版本语义：`manifest_version` 必须由链投影 finalized anchor 生成,至少包含 `chain_genesis_hash`、`chain_block_hash`、`chain_block_number` 与投影数量;不得仅因本地 `synced_at` 改变而推进。
- 签名/验签规则：本契约只读公开公权机构投影,不携带签名;关键操作仍必须通过链上读取或 QR_V1 签名流程校验。
- 禁止兼容：不得把 CitizenApp 内置快照、Isar 缓存或 OnChina PostgreSQL 投影升级为公权机构真源;不得恢复链下公权机构目录真源。
- 禁止事项：
  - 禁止 CitizenApp 全量扫链枚举公权机构。
  - 禁止 OnChina 从 `china.sqlite` 运行态重新生成公权机构。
  - 禁止接口下发行政区名称副本;行政区名称仍由行政区字典按 code join。
- 必跑测试：
  - `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`
  - `flutter test test/citizen/public/public_institution_bundle_loader_test.dart test/citizen/public/public_institution_sync_test.dart`

### P-API-CITIZENAPP-002：CitizenApp Square Worker / R2 契约

- 状态：草案（阶段 3 Worker / R2 本地服务已落地；阶段 5 App 上传与上链闭环已落地；阶段 6 Worker 链上事件确认和正式 feed 入库已落地；阶段 10 已改为链上扣费入块后再上传媒体，并增加本地草稿保护；阶段 11 已完成 staging 部署和 prepare 远端 smoke；阶段 12 曾用已废弃的单一 RPC Secret 通过 R2 上传 + Worker 链确认负向 smoke；2026-07-10 起 Worker 改为 Access 三项 Secret，新的 staging 私有链路尚待部署验收；阶段 13 的 runtime metadata 阻塞结论保留；2026-07-07 起第 1 步会员系统已接入官网 Stripe webhook、三档会员计划和 Worker 侧链上身份资格读取；2026-07-08 起第 2 步广场主媒体迁移为 Cloudflare Images / Stream，R2 只保留 manifest 与头像/背景等资料资产；2026-07-08 起官网 `/membership` 可调用 Worker 创建 Stripe subscription Checkout，CitizenApp「我的」Tab 已新增只读会员入口；2026-07-08 起上传 prepare/complete 已按三档会员强制校验动态/文章额度和竞选会员权限；2026-07-08 起帖子删除只清 Cloudflare 数据并保留链上记录，修改统一视为新发布成功后删除旧 Cloudflare 数据）
- 类型：接口契约 / storage 契约
- 唯一真源：
  - 方案任务卡：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
  - CitizenApp 架构：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
  - 落地后实现真源：`citizenapp/cloudflare/` 与 `citizenapp/lib/8964/`
- 详细文档：
  - `memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
  - `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/8964/`：钱包签名登录、上传请求、发布请求、发布确认请求、feed 请求、推荐信号上报
  - `citizenapp/cloudflare/`：会员、上传回执、链上事件确认、feed、关注关系、推荐信号
- 消费者：
  - `citizenapp/lib/8964/`
  - Cloudflare R2 bucket `citizenapp-square-media`
  - Cloudflare D1 database `citizenapp-square-db`
  - Cloudflare KV namespace `citizenapp-square-feed-cache`
- HTTP API 字段：
  - `GET /health` 响应：`ok`、`service`、`storage_backend`、`content_on_chain`
  - `POST /v1/square/auth/challenge` 请求：`owner_account`；响应：`challenge_id`、`owner_account`、`signing_payload`、`expires_at`
  - `POST /v1/square/auth/session` 请求：`owner_account`、`challenge_id`、`signature`；响应：`session_token`、`owner_account`、`expires_at`
  - `GET /v1/square/membership` 请求：Bearer `session_token`；响应：`plans[]`、`identity`、`identity_error`、`eligible_levels[]`、`membership`、`subscription_active`、`active`、`inactive_code`、`inactive_message`。`plans[]` 中三档会员只按美元计价，字段包含 `price_currency = usd`、`price_usd_cents`、`price_usd_monthly`；`active` 是支付状态与链上身份资格同时满足后的最终权益状态。
  - `POST /v1/square/membership/stripe/checkout` 请求：`owner_account`、`membership_level`(`visitor`/`voting`/`candidate`)，可选 Bearer `session_token`；响应：`checkout_session_id`、`checkout_url`、`membership_level`。官网可不带 Bearer，Worker 会校验 `owner_account` 是合法 SS58；带 Bearer 时必须和 session owner 一致。投票/竞选会员创建 Checkout 前必须读取链上身份，不满足则拒绝；访客会员不读链。正式授权仍只以 Stripe subscription webhook 写入 `square_memberships` 为准。
  - `POST /v1/square/membership/stripe/webhook` 请求：Stripe webhook 原始 JSON + `Stripe-Signature`；响应：`event_id`、`event_type`、`action`。本接口不需要 Bearer，必须用 `STRIPE_WEBHOOK_SECRET` 校验签名；`customer.subscription.created/updated` 根据 subscription metadata `owner_account`、`membership_level` 写入会员状态，写入前必须校验 subscription item 的 Stripe Price 为 `usd` 且金额匹配会员等级；`customer.subscription.deleted` 标记失效，`checkout.session.completed` 只观察不直接授予权益。
  - `POST /v1/square/uploads/prepare` 请求：Bearer `session_token`，`post_category`、`content_format`(`normal`/`article`)、`title_length`、`text_length`、`media_items[]`、`manifest_hash`；响应：`upload_id`、`post_id`、`storage_receipt_id`、`expires_at`、`estimated_bytes`、`manifest_object_key`、`manifest_upload_url`、`media_items[]`。Worker 在本接口按当前有效 `membership_level` 强制校验普通 / 竞选分类、动态 / 文章形态、标题 / 正文长度和图片 / 视频数量；本接口只固定链上索引、manifest R2 短期上传授权和 Images/Stream 一次性上传授权，不写媒体对象。
  - `media_items[]` 响应字段：`media_kind`、`content_type`、`byte_size`、`provider`(`cloudflare_images`/`cloudflare_stream`)、`provider_asset_id`、`upload_method`(`direct_form`/`tus`)、`asset_state`、`delivery_url`、`playback_hls_url`、`playback_dash_url`、`thumbnail_url`、`upload_url`。
  - `PUT /v1/square/uploads/dev-put` 请求：Bearer `session_token`，query `upload_id`、`object_key`；响应：`object_key`、`byte_size`；仅 `SQUARE_DEV_UPLOAD_PROXY=1` 的本地开发模式可用，生产环境禁用。
  - `POST /v1/square/uploads/dev-media` 请求：Bearer `session_token`，query `upload_id`、`media_index`；响应：`upload_id`、`media_index`、`asset_state`；仅 `SQUARE_DEV_UPLOAD_PROXY=1` 的本地开发模式可用，用于模拟 Images/Stream 直传。
  - `POST /v1/square/uploads/complete` 请求：Bearer `session_token`，`upload_id`、`manifest_hash`、`content_hash`；响应：`upload_id`、`post_id`、`content_hash`、`storage_receipt_id`、`storage_state`(`completed`/`processing`)。Worker 读取 R2 manifest 并复核 manifest hash、owner、`post_category`、`content_format`、标题 / 正文长度、媒体数量和 `square_media_assets` 一致性；返回的 `storage_receipt_id` 必须等于 prepare 阶段预生成的回执；视频上传已接收但 Stream 尚未 ready 时返回 `processing`。
  - `POST /v1/square/uploads/stream/webhook` 请求：Cloudflare Stream webhook 原始 JSON + `Webhook-Signature`；响应：`action`、`provider_asset_id`、`asset_state`。本接口不需要 Bearer，必须用 `CLOUDFLARE_STREAM_WEBHOOK_SECRET` 校验签名。
  - `storage_until` 当前不由上传完成接口返回；CitizenApp 发布交易使用 `GET /v1/square/membership` 的 `membership.expires_at` 作为链上 `storage_until`。
  - `POST /v1/square/posts/confirm` 请求：Bearer `session_token`，`post_id`、`block_hash`、可选 `tx_hash`；响应：`post`
  - `DELETE /v1/square/posts/{post_id}` 请求：Bearer `session_token`；响应：`post_id`、`post_state = deleted`、`cleanup{deleted_media_assets,deleted_r2_objects,reclaimed_storage_bytes}`。仅作者本人可调用；Worker 删除 Cloudflare Images / Stream provider asset、R2 manifest 和 D1 媒体索引，清空 D1 `title/text` 并把 `square_posts.post_state` 置为 `deleted`；链上 `SquarePosts`、发布事件和 1 元发布费记录不改写。重复删除不得重复回收 `storage_used_bytes`。
  - `GET /v1/square/feed/recommended` 请求：可选 Bearer `session_token`、`limit`；响应：`posts[]`
  - `GET /v1/square/feed/following` 请求：可选 Bearer `session_token`、`limit`；响应：`posts[]`
  - `GET /v1/square/feed/campaign` 请求：可选 Bearer `session_token`、`limit`；响应：`posts[]`
  - `POST /v1/square/follows` 请求：Bearer `session_token`、`followed_account`
  - `DELETE /v1/square/follows/{followed_account}` 请求：Bearer `session_token`
  - `POST /v1/square/signals` 请求：Bearer `session_token`、`post_id`、`signal_type`
  - `POST /v1/square/reports` 请求：Bearer `session_token`、`post_id`
  - `GET /v1/square/users/{owner_account}` 请求：可选 Bearer `session_token`；响应：`profile`（`owner_account`、`display_name`、`bio`、`avatar_object_key`、`banner_object_key`、`cid_number`、`is_certified`、`counts{following,followers,posts}`、`is_following`、`updated_at`）。公开可读；带 session 时 `is_following` 反映登录者视角；认证以链上已确认发布携带的 `cid_number` 为真源。
  - `GET /v1/square/users/{owner_account}/posts` 请求：可选 `category`（all/normal/campaign）、`content_format`（all/normal/article）、`limit`、`cursor`；响应：`posts[]`、`next_cursor`（按 `created_at` keyset 游标）。帖子 Tab 传 `content_format=normal` 排除文章，文章 Tab 传 `content_format=article`。
  - `GET /v1/square/users/{owner_account}/follows` 请求：`type`（following/followers）、`limit`、`cursor`；响应：`accounts[{owner_account,created_at}]`、`next_cursor`
  - `PUT /v1/square/profile` 请求：Bearer `session_token`，可选 `display_name`(≤40)、`bio`(≤160)、`avatar_object_key`、`avatar_content_hash`、`banner_object_key`、`banner_content_hash`（头像/背景 key 必须落本人 `profile/{owner}/` 前缀）；响应：与 GET `users/{owner_account}` 同构的完整 `profile`。
  - `POST /v1/square/profile/assets/prepare` 请求：Bearer `session_token`，`kind`(avatar/banner)、`content_type`(jpeg/png/webp)、`byte_size`(≤15MB)、`sha256`；响应：`object_key`(`profile/{owner_account}/{kind}_{sha256}.{ext}`)、`content_hash`、`upload_url`（生产 R2 预签名 PUT / 本地 dev-put）。头像/背景等公开资料只进 R2，不上链。
  - `PUT /v1/square/profile/assets/dev-put` 请求：Bearer `session_token`，query `object_key`（须属本人 `profile/{owner_account}/` 前缀）；响应：`object_key`、`byte_size`；仅 `SQUARE_DEV_UPLOAD_PROXY=1` 本地开发可用。
  - `GET /v1/square/media/{object_key}` 公开读取 R2 头像/背景等 profile 资料资产（只允许 `profile/` 前缀，拒 `..`）；广场主图片 / 视频不再从 R2 读取，改由 Images / Stream URL 承载。
- 用户公开资料 R2 契约：`profile/{sanitize(owner_account)}/profile.json`（schema `citizenapp.square.profile.v1`：`owner_account`、`display_name`、`bio`、`avatar_object_key`、`avatar_content_hash`、`banner_object_key`、`banner_content_hash`、`updated_at`）。计数与认证不入 profile.json，响应时由 D1/链上派生。头像/背景对象落 `profile/{owner_account}/{kind}_{sha256}.{ext}`。
- Feed item 字段：
  - `post_id`
  - `owner_account`
  - `cid_number`
  - `post_category`: `normal` / `campaign`
  - `content_format`: `normal` / `article`
  - `title`
  - `text`
  - `content_hash`
  - `storage_receipt_id`
  - `manifest_url`
  - `cover_url`
  - `media_items[]`
  - `created_at`
  - `chain_block`
  - `author_state`
- Feed media item 字段：
  - `media_kind`: `image` / `video`
  - `object_key`
  - `url`
  - `provider`
  - `provider_asset_id`
  - `asset_state`
  - `playback_hls_url`
  - `playback_dash_url`
  - `content_type`
  - `byte_size`
  - `sha256`
  - `duration_seconds`
  - `width`
  - `height`
- R2 object key：
  - `square/{owner_account}/posts/{post_id}/manifest.json`
  - `profile/{owner_account}/profile.json`
  - `profile/{owner_account}/{avatar|banner}_{sha}.{jpg|png|webp}`
- R2 manifest 字段（阶段 5 App 端实际生成的规范化内容清单）：
  - `schema`: 固定为 `citizenapp.square.post.v1`
  - `owner_account`
  - `post_category`
  - `content_format`（可选，`normal`/`article`；**仅文章写入**，普通帖不带 → 默认 normal，保持旧 manifest 形状与哈希）
  - `title`（可选，文章标题 10–50 字；普通帖不带）
  - `text`（动态正文 ≤300 字；文章正文按会员计划校验，访客 20000 字，投票 / 竞选 30000 字）
  - `media_items[]`（动态最多 9 张图 + 1 个视频；文章 `[0]`=首图，`[1..]`=正文图，访客正文图最多 50 张，投票 / 竞选最多 100 张）
  - `media_items[].media_kind`: `image` / `video`
  - `media_items[].file_name`
  - `media_items[].content_type`
  - `media_items[].byte_size`
  - `media_items[].sha256`
- 内容形态：链上 `post_category` 仍只发 normal/campaign（不扩四类、不重新创世）；`content_format`/`title` 只落链下 R2 manifest + D1（confirm 时从 manifest 写入 `square_posts.content_format`(默认 normal)/`title` 两列）。普通文章使用 `post_category=normal + content_format=article`，竞选文章使用 `post_category=campaign + content_format=article`；`content_hash` 覆盖整份 manifest（含 content_format/title），上链防篡改。
- R2 manifest 边界：
  - `manifest_hash` 和链上 `content_hash` 均取该规范化内容清单的 sha256 hex。
  - `post_id`、`manifest_object_key` 由 Worker 在 `uploads/prepare` 后生成，真源在 D1 `square_uploads.object_keys_json`；Images/Stream 的 `provider_asset_id`、上传方式、播放地址和处理状态真源在 D1 `square_media_assets`，不要求 App 在 prepare 前写入 manifest。
- D1 表字段：
  - `square_login_challenges`: `challenge_id`、`owner_account`、`signing_payload`、`expires_at`、`used_at`
  - `square_memberships`: `owner_account`、`membership_level`(`visitor`/`voting`/`candidate`)、`storage_quota_bytes`、`storage_used_bytes`、`expires_at`、`updated_at`、`subscription_source`、`stripe_customer_id`、`stripe_subscription_id`、`stripe_price_id`、`subscription_status`、`current_period_start`、`current_period_end`、`cancel_at_period_end`、`identity_level`、`identity_checked_at`
  - `square_uploads`: `upload_id`、`post_id`、`owner_account`、`post_category`、`manifest_hash`、`content_hash`、`storage_receipt_id`、`estimated_bytes`、`object_keys_json`、`status`、`created_at`、`completed_at`
  - `square_media_assets`: `upload_id`、`post_id`、`owner_account`、`media_index`、`media_kind`、`provider`、`provider_asset_id`、`upload_method`、`content_type`、`byte_size`、`asset_state`、`delivery_url`、`playback_hls_url`、`playback_dash_url`、`thumbnail_url`、`duration_seconds`、`width`、`height`、`error_code`、`created_at`、`updated_at`、`ready_at`
  - `square_posts`: `post_id`、`owner_account`、`cid_number`、`post_category`、`text`、`content_hash`、`storage_receipt_id`、`chain_block`、`created_at`、`post_state`
  - `square_follows`: `owner_account`、`followed_account`、`created_at`
  - `square_user_signals`: `owner_account`、`post_id`、`signal_type`、`weight`、`created_at`
- Worker 环境变量：
  - `CITIZENAPP_SQUARE_API_BASE_URL`：CitizenApp 编译期 define，用于显式覆盖广场 Worker API 根地址；默认直连 production Worker，本地调试可显式传 `http://127.0.0.1:8787`。
  - `CITIZEN_CHAIN_RPC_URL`：Access 保护的链 RPC HTTPS 地址，只允许作为 Cloudflare 远端 Secret，不写入仓库和 CitizenApp。
  - `CITIZEN_CHAIN_RPC_ACCESS_CLIENT_ID`、`CITIZEN_CHAIN_RPC_ACCESS_CLIENT_SECRET`：Worker 调用 Access 应用的服务令牌，必须与 URL 成套配置为远端 Secret；当前代码只允许 `state_getStorage` 与 `author_submitExtrinsic` 两个内部固定方法，不提供通用代理。
  - `SQUARE_DEV_UPLOAD_PROXY`：本地开发上传代理开关；生产环境不得开启。
  - `R2_ACCOUNT_ID`、`R2_ACCESS_KEY_ID`、`R2_SECRET_ACCESS_KEY`、`R2_BUCKET_NAME`：Worker 生成 R2 S3 预签名上传 URL 所需变量；只允许配置在 Cloudflare 远端变量或 secret，不得下发到 CitizenApp，不得写入仓库。
  - `CLOUDFLARE_ACCOUNT_ID`、`CLOUDFLARE_API_TOKEN`：Worker 调用 Cloudflare Images / Stream API 生成一次性上传 URL 所需变量；API token 必须使用 Cloudflare secret，不得下发到 CitizenApp，不得写入仓库。
  - `CLOUDFLARE_IMAGES_DELIVERY_BASE_URL`：Cloudflare Images delivery URL 前缀，不含 asset id 和 variant。
  - `CLOUDFLARE_STREAM_CUSTOMER_SUBDOMAIN`：Cloudflare Stream playback URL 前缀。
  - `CLOUDFLARE_STREAM_WEBHOOK_SECRET`：Stream webhook 签名 secret；必须用 Cloudflare secret/变量配置，不得写入仓库和 CitizenApp。
  - `STRIPE_SECRET_KEY`：Worker 创建 Stripe Checkout Session 所需 secret key；必须用 Cloudflare secret 配置，不得写入仓库、官网前端或 CitizenApp。
  - `STRIPE_WEBHOOK_SECRET`：Stripe webhook endpoint secret；必须用 Cloudflare secret/变量配置，不得写入仓库和 CitizenApp。
  - `STRIPE_DEV_CHECKOUT_PROXY`：本地开发 Checkout 代理开关；只允许 Miniflare / wrangler dev 验证时设为 `1`，生产环境必须保持 `0`。
  - `STRIPE_PRICE_VISITOR`、`STRIPE_PRICE_VOTING`、`STRIPE_PRICE_CANDIDATE`：官网 Stripe 美元价格 ID，可作为 subscription metadata 缺失时的会员等级映射；不是密钥，但仍只作为部署配置维护。Stripe Checkout 可允许本地法币或 USDC 支付，会员权益系统不保存本地法币展示金额、换汇结果或 USDC 支付流水，USDT 不作为目标支付方式。
  - `CITIZENAPP_MEMBERSHIP_SUCCESS_URL`、`CITIZENAPP_MEMBERSHIP_CANCEL_URL`：Stripe Checkout 成功 / 取消回跳地址，由部署环境配置到官网 `/membership`，不得从客户端任意传入以免开放重定向。
  - `VITE_CITIZENAPP_SQUARE_API_BASE_URL`：官网构建时可选 Worker API 根地址；未设置时使用 production Worker 默认地址。
- CitizenApp 本地缓存字段：
  - `SquareDraft`: `owner_account`、`post_category`、`text`、`media_drafts[]`、`draft_state`、`updated_at_millis`、`last_error`、可选 `upload_id/post_id/content_hash/storage_receipt_id/storage_until/tx_hash/block_hash_hex`；当前落地复用 `AppKvEntity`，不新增 Isar schema。
  - `SquareUploadTask`: 当前不落独立 Isar schema；发布中的上传状态由 `SquareDraft` 和 Worker `square_uploads.status` 表达。
  - `SquarePostCache`: `post_id`、`owner_account`、`cid_number`、`post_category`、`content_hash`、`storage_receipt_id`、`manifest_url`、`cover_url`、`cached_at`
  - `SquareFeedCursor`: `owner_account`、`feed_kind`、`cursor`、`updated_at`
  - `SquareUserSignalCache`: `owner_account`、`post_id`、`signal_type`、`created_at`、`synced`
- 编码：HTTP JSON 字段统一 snake_case；R2 manifest 为 UTF-8 JSON；hash 字段为 sha256 hex；Worker 阶段 3 已落地字段的时间统一使用毫秒时间戳。
- CitizenApp 阶段 5/6 实现真源：
  - `citizenapp/lib/8964/services/square_api_client.dart`
  - `citizenapp/lib/8964/services/square_upload_service.dart`
  - `citizenapp/lib/8964/services/square_publish_service.dart`
  - `citizenapp/lib/8964/pages/square_compose_page.dart`
  - `citizenapp/lib/8964/pages/square_home_page.dart`
  - `citizenapp/lib/my/user/user.dart`
- Worker 阶段 6 实现真源：
  - `citizenapp/cloudflare/src/membership/checkout.ts`
  - `citizenapp/cloudflare/src/membership/stripe.ts`
  - `citizenapp/cloudflare/src/membership/service.ts`
  - `citizenapp/cloudflare/src/uploads/service.ts`
  - `citizenapp/cloudflare/src/media/cloudflare_assets.ts`
  - `citizenapp/cloudflare/src/chain/rpc.ts`
  - `citizenapp/cloudflare/src/chain/square_event.ts`
  - `citizenapp/cloudflare/src/posts/confirm.ts`
  - `citizenapp/cloudflare/src/posts/repository.ts`
- 官网会员订阅实现真源：
  - `citizenweb/src/pages/Membership.tsx`
- 签名/验签规则：
  - Worker session 必须由钱包账户对 `signing_payload` 签名获得。
  - Worker 只能把钱包签名证明用于登录和上传授权，不得托管钱包私钥。
  - manifest R2 上传授权与 Images/Stream 一次性上传 URL 必须短期有效，且绑定 `owner_account`、`upload_id`、媒体 provider asset 和容量校验结果。
  - CitizenApp 必须先用 finalized 余额确认钱包至少保留 `2.11 元`（ED 1.11 元 + 发布费 1 元），余额不足不得进入 Worker prepare 或媒体上传。
  - CitizenApp 必须在链上扣费交易入块后才上传 manifest 与主媒体；链上未入块不得占用 R2 / Images / Stream 存储，只能保存本地草稿。
  - 生产环境 manifest 上传必须使用 Worker 基于 R2 S3 凭证签发的短期 PUT URL；广场图片/视频必须使用 Images/Stream 一次性上传 URL；本地开发代理 `dev-put` / `dev-media` 不得部署为生产上传入口。
  - `post_category = campaign` 的最终权限真源在链上发布交易，不以 Worker 自报为准。
  - `POST /v1/square/posts/confirm` 只能在指定 `block_hash` 的 `System.Events` 中存在字段完全匹配的 `SquarePostPublished` 事件时写入正式 feed。
  - Worker 确认发布时必须同时校验 session 钱包、D1 上传记录、链上事件、R2 manifest 和 `square_media_assets`；任一不一致不得入库为 `published`。
- 禁止兼容：开发期不兼容旧广场入口壳数据格式，不保留旧“提案广场”feed 作为个人动态广场接口。
- 禁止事项：
  - 禁止把 R2 API key、Cloudflare token、Images/Stream API token、D1 凭证写入 CitizenApp 或仓库。
  - 禁止要求 App 用户直接注册 Cloudflare 账户或直接向 R2 / Images / Stream 付费。
  - 禁止把 R2/D1/KV 描述成公民链节点、全节点或链上存储。
  - 禁止媒体内容、正文附件、封面和 manifest 上链。
  - 禁止 Worker 成为钱包资金托管方。
  - 禁止链上交易未入块或未找到匹配事件时把动态加入正式 feed。
- 必跑测试：
  - Worker API 单元测试
  - Worker 链上事件解码与发布确认测试
  - Worker 本地启动测试
  - R2 manifest 上传/读取本地模拟或真实测试
  - Images / Stream direct upload 本地 dev-media smoke 与 Stream webhook 签名测试
  - CitizenApp 广场 API adapter 测试
  - App 真机或模拟器广场浏览与发布流程验收

### P-API-CITIZENAPP-003：CitizenApp Chat Cloudflare Mailbox

- 状态：当前（阶段 IM-2 已落地 Worker mailbox API、D1 迁移、设备绑定验签 helper、KeyPackage、密文 envelope、pending 拉取与 ack；阶段 IM-3 已落地 CitizenApp 自动 Worker session、设备绑定签名、KeyPackage 发布、发送前 KeyPackage 拉取/消费和聊天页打开自动 pending 同步；阶段 IM-4 已落地信息 Tab 15 秒前台轮询、聊天页 8 秒前台轮询和失败 30 秒退避；阶段 IM-5 已补齐 mailbox pending 拉取、解密落库和 ack 的端到端状态机回归，并新增 native OpenMLS mailbox 闭环用例；阶段 IM-6 已完成 macOS host OpenMLS native 真实执行验收；阶段 IM-7 已落地加密附件 prepare/upload/complete、R2 密文对象校验和 App 侧加密附件发送底座；阶段 IM-8 已落地聊天页文件选择、附件下载授权、密文下载、本地校验解密和私有缓存保存；阶段 IM-9 已落地 WebSocket 新密文通知与 App 轮询兜底；阶段 IM-10 已将实时连接迁入账户级 Durable Objects fanout；阶段 IM-11 已将 Cloudflare mailbox 收紧为临时密文投递队列，ack 后删除 D1 envelope 和对应 R2 加密附件对象；近场真机和多分片续传待后续阶段）
- 类型：接口契约 / 密文 mailbox 契约
- 唯一真源：
  - 方案任务卡：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
  - IM 技术文档：`memory/05-modules/citizenapp/im/IM_TECHNICAL.md`
  - 落地后实现真源：`citizenapp/cloudflare/src/chat/` 与 `citizenapp/lib/im/transport/`
- 生产者：
  - `citizenapp/lib/im/`：OpenMLS 加密、KeyPackage、密文 envelope、ack 删除确认。
  - `citizenapp/cloudflare/`：临时密文 mailbox、KeyPackage 池、WebSocket 新密文通知。
- 消费者：
  - `citizenapp/lib/im/`
  - Cloudflare D1 database `citizenapp-square-db`
  - Cloudflare R2 bucket `citizenapp-square-media`
  - Cloudflare Durable Objects / WebSocket。当前 IM-10 按 `owner_account` 路由到 `ChatRealtimeObject`，由账户级对象管理在线设备连接。
- HTTP API 字段：
  - `POST /v1/chat/devices/register` 请求：Bearer `session_token`，`owner_account`、`device_id`、`device_public_key_hex`、`binding_signature`、`expires_at`、`nonce`
  - `POST /v1/chat/keypackages` 请求：Bearer `session_token`，`owner_account`、`device_id`、`device_public_key_hex`、`key_package_id`、`key_package`、`cipher_suite`、`created_at`、`expires_at`
  - `GET /v1/chat/keypackages/{owner_account}` 请求：Bearer `session_token`，`limit`
  - `POST /v1/chat/keypackages/consume` 请求：Bearer `session_token`，`owner_account`、`key_package_id`、`requester_account`
  - `POST /v1/chat/envelopes` 请求：Bearer `session_token`，`envelope_id`、`conversation_id`、`sender_account`、`sender_device_id`、`recipient_account`、`recipient_device_id`、`mls_message_kind`、`envelope`、`attachment_manifest_key`、`created_at`、`expires_at`
  - `GET /v1/chat/envelopes/pending` 请求：Bearer `session_token`，`owner_account`、`device_id`、`limit`
  - `POST /v1/chat/envelopes/ack` 请求：Bearer `session_token`，`owner_account`、`device_id`、`envelope_id`；成功后删除 `chat_envelopes` 行，并删除该 envelope 关联的 R2 加密附件 manifest/chunk 对象。
  - `POST /v1/chat/attachments/prepare` 请求：Bearer `session_token`，`owner_account`、`device_id`、`conversation_id`、`attachment_id`、`manifest_byte_size`、`chunks`；返回 manifest/chunk 的 R2 object key、upload_url、content_type。
  - `PUT /v1/chat/attachments/dev-put` 请求：开发代理上传入口，仅在 `SQUARE_DEV_UPLOAD_PROXY=1` 时启用；生产不得依赖该入口。
  - `POST /v1/chat/attachments/complete` 请求：Bearer `session_token`，`conversation_id`、`attachment_id`、`manifest_object_key`、`chunk_object_keys`；Worker 只确认 R2 对象存在，不保存附件密钥。
  - `POST /v1/chat/attachments/download` 请求：Bearer `session_token`，`owner_account`、`device_id`、`conversation_id`、`attachment_id`、`manifest_object_key`、`manifest_hash`、`chunk_refs`；Worker 通过 `chat_envelopes.attachment_manifest_key` 确认当前钱包是发送方或接收方后返回短期下载 URL。
  - `GET /v1/chat/attachments/dev-get` 请求：开发代理下载入口，仅在 `SQUARE_DEV_UPLOAD_PROXY=1` 时启用，仍必须带 Bearer session；生产不得依赖该入口。
  - `GET /v1/chat/ws` 请求：Bearer `session_token`，query `owner_account`、`device_id`；Worker 完成 session 和 active device 校验后转发到账户级 `ChatRealtimeObject`；WebSocket 成功后先返回 `gmb_im_ws_ready_v1`，随后只推送 `gmb_im_new_envelope_v1` 新密文通知，字段为 `envelope_id`、`conversation_id`、`recipient_account`、`recipient_device_id`、`mls_message_kind`、`created_at`。
  - Durable Object binding：`CHAT_REALTIME`，class `ChatRealtimeObject`，对象名称固定为 `owner_account`。
- D1 表字段：
  - `chat_devices`: `owner_account`、`device_id`、`device_public_key_hex`、`binding_signature`、`expires_at`、`created_at`、`revoked_at`
  - `chat_keypackages`: `owner_account`、`device_id`、`key_package_id`、`key_package`、`cipher_suite`、`created_at`、`expires_at`、`consumed_at`、`consumed_by_account`
  - `chat_envelopes`: `envelope_id`、`conversation_id`、`sender_account`、`sender_device_id`、`recipient_account`、`recipient_device_id`、`mls_message_kind`、`encrypted_payload`、`attachment_manifest_key`、`created_at`、`expires_at`。该表只保存未 ack 且未过期的临时密文投递项，不保存聊天历史。
- R2 object key：
  - `chat/{owner_account}/conversations/{conversation_id}/attachments/{attachment_id}/manifest.enc`
  - `chat/{owner_account}/conversations/{conversation_id}/attachments/{attachment_id}/chunk_001.bin`
- 编码：
  - HTTP JSON 字段统一 snake_case。
  - `envelope` 载荷承载 `GMB_IM_V1 / ImEnvelope` Protobuf bytes 的 base64url 表示。
  - `key_package` 承载 OpenMLS KeyPackage bytes 的 base64url 表示。
  - R2 附件必须在 CitizenApp 本地加密后上传；manifest 和分片均为 `AES-GCM-256` 密文。
  - 附件内容密钥、nonce、mac、manifest_hash 和 chunk_hash 只允许进入 OpenMLS application 明文，再由 `mls_wire_message` 端到端加密传输。
  - 附件下载必须先下载密文对象、校验 sha256，再在 CitizenApp 本机用 OpenMLS 控制消息中的 AES-GCM 参数解密；接收端必须在本机缓存附件后再 ack 删除 Cloudflare 临时副本。
- 签名/验签规则：
  - Worker session 必须由钱包账户签名获得。
  - 设备绑定必须证明 `device_id` / `device_public_key_hex` 属于 `owner_account`。
  - 钱包私钥只用于设备绑定证明，不得用于 OpenMLS 消息加密。
  - Cloudflare Worker 不得生成、保存或恢复 OpenMLS 私钥。
  - Cloudflare Worker 不得生成、保存或恢复附件内容密钥。
  - Worker 只允许临时保存未 ack / 未过期的私聊或群聊密文和必要投递元数据；ack 或过期后必须删除对应 envelope，附件 envelope 还必须删除对应 R2 加密对象。
- 禁止兼容：不兼容区块链节点通信节点 mailbox、不兼容 `/gmb/im/1` 作为正式互联网聊天链路。
- 禁止事项：
  - 禁止保存私聊或群聊明文。
  - 禁止把私密聊天写入广场公开评论表。
  - 禁止把 Cloudflare mailbox 描述成公民链节点、区块链节点或全节点聊天能力。
  - 禁止要求用户安装或开启区块链软件后才能使用互联网聊天。
- 必跑测试：
  - Worker chat API 单元测试。
  - 设备绑定验签测试。
  - KeyPackage 发布/拉取/消费测试。
  - 密文 envelope 投递/拉取/ack 测试。
  - 加密附件 prepare/upload/complete/download/dev-get 测试。
  - CitizenApp `ImCloudflareTransport` 测试。
  - CitizenApp 附件文件选择、下载解密和私有缓存保存测试。
  - OpenMLS 1:1 和群聊密文 round-trip 测试。

### P-API-CITIZENAPP-004：CitizenApp Chain Bootstrap Manifest

- 状态：当前（2026-07-08 第 2 步已落地 Cloudflare Worker `GET /v1/chain/bootstrap`；第 3 步已接入 CitizenApp 轻节点初始化；第 4 步已接入已签名交易受控广播 path）
- 类型：接口契约
- 唯一真源：
  - ADR：`memory/04-decisions/ADR-032-citizenapp-chain-edge-architecture.md`
  - Worker：`citizenapp/cloudflare/src/chain/bootstrap.ts`
  - 路由：`citizenapp/cloudflare/src/routes.ts`
  - App：`citizenapp/lib/rpc/chain_bootstrap_api.dart`、`citizenapp/lib/rpc/smoldot_client.dart`
- 详细文档：
  - `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
  - `memory/08-tasks/open/20260708-citizenapp-chain-edge-architecture.md`
- 生产者：
  - Cloudflare Worker `GET /v1/chain/bootstrap`
  - 部署环境变量 `CITIZEN_CHAIN_*`
- 消费者：
  - CitizenApp 轻节点连接状态机
  - CitizenApp 广场和聊天服务发现
- HTTP API 字段：
  - `GET /v1/chain/bootstrap` 响应顶层字段：`ok`、`schema`、`generated_at`、`cache_ttl_seconds`、`chain`、`light_client`、`p2p`、`services`、`security`、`degradation`
  - `schema`: 固定 `citizenapp.chain.bootstrap.v1`
  - `chain`: `chain_id`、`chain_name`、`chain_type`、`protocol_id`、`genesis_hash`、`state_root`、`ss58_format`、`token_symbol`、`token_decimals`
  - `light_client`: `mode`、`truth_source`、`api_is_truth`、`bundled_assets_required`、`checkpoint`
  - `light_client.mode`: 固定 `smoldot`
  - `light_client.truth_source`: 固定 `p2p_finalized_storage`
  - `light_client.api_is_truth`: 固定 `false`
  - `light_client.checkpoint`: `source`、`light_sync_state_url`、`light_sync_state_sha256`
  - `p2p`: `bootnodes`、`bootnodes_source`、`min_peer_count_hint`
  - `services`: `square_base_url`、`chat_base_url`、`media_base_url`、`signed_extrinsic_relay`
  - `services.signed_extrinsic_relay`: `enabled`、`path`；默认 `enabled=false/path=null`，仅当 Worker 显式配置 `CHAIN_EXTRINSIC_RELAY_ENABLED=1` 且服务节点 RPC 已配置时返回 `enabled=true/path=/v1/chain/extrinsics/relay`
  - `security`: `exposes_rpc_url`、`rpc_proxy`、`exposes_private_key_material`、`validator_rpc_public`，全部固定 `false`
  - `degradation`: `p2p_unavailable`、`chain_success_source`
- Worker 环境变量：
  - `CITIZEN_CHAIN_BOOTNODES`: 公开 bootnode multiaddr 列表，允许换行、逗号或分号分隔；不是密钥。
  - `CITIZEN_CHAIN_BOOTSTRAP_TTL_SECONDS`: 启动清单 HTTP 缓存秒数。
  - `CITIZEN_CHAIN_GENESIS_HASH`: 当前链 genesis hash。
  - `CITIZEN_CHAIN_STATE_ROOT`: 当前轻形态 chainspec genesis `stateRootHash`。
  - `CITIZEN_CHAIN_LIGHT_SYNC_STATE_URL`: 可选公开 `light_sync_state.json` HTTPS 地址；为空时 App 使用本地打包资产。
  - `CITIZEN_CHAIN_LIGHT_SYNC_STATE_SHA256`: `assets/light_sync_state.json` 的 sha256 hex。
  - `CHAIN_EXTRINSIC_RELAY_ENABLED`: 已签名交易受控广播开关，默认 `0`。
  - `CHAIN_EXTRINSIC_RELAY_MAX_BYTES`: relay 接受的 signed extrinsic 最大字节数。
  - `CHAIN_EXTRINSIC_RELAY_MAX_PER_MINUTE`: relay 每分钟按请求 IP hash 限流数量。
  - `CITIZEN_CHAIN_RPC_URL`: Access 保护的私有链 RPC HTTPS 地址，只放远端 Secret。
  - `CITIZEN_CHAIN_RPC_ACCESS_CLIENT_ID`、`CITIZEN_CHAIN_RPC_ACCESS_CLIENT_SECRET`: Access 服务令牌，必须成套放入远端 Secret；缺失任一项时 relay 固定关闭。
- 编码：HTTP JSON，字段统一 snake_case；时间统一毫秒时间戳；hash 字段为 hex；`bootnodes` 元素为 Substrate multiaddr 字符串。
- 签名/验签规则：本接口不携带用户签名，不接受交易载荷；只声明受控广播 path 是否可用，广播协议见 `P-API-CITIZENAPP-005`。
- 禁止兼容：不兼容 API-only 链连接方案；不得把本接口演化成通用 JSON-RPC fallback。
- 禁止事项：
  - 禁止响应中返回 `CITIZEN_CHAIN_RPC_URL`、两项 `CITIZEN_CHAIN_RPC_ACCESS_*`、Validator RPC、Archive RPC 或任何私密 RPC 完整 URL。
  - 禁止 Cloudflare Worker 接触、保存或下发用户私钥、助记词、keystore、签名种子。
  - 禁止把 `GET /v1/chain/bootstrap` 的响应当作链上状态真源。
  - 禁止把 `signed_extrinsic_relay.enabled=true` 解读为链上成功；该字段只表示可提交完整 signed extrinsic 到受控广播接口。
  - 禁止把 `bootnodes` 连接失败直接判定为 DNS 故障；轻节点是否可用以 peer 和 best/finalized 推进为准。
- 必跑测试：
  - `npm --prefix citizenapp/cloudflare run typecheck`
  - `npm --prefix citizenapp/cloudflare test -- chain_bootstrap.test.ts`
  - 完整 Worker 测试：`npm --prefix citizenapp/cloudflare test`
  - `flutter test test/rpc/chain_bootstrap_api_test.dart`
  - `flutter analyze lib/rpc/chain_bootstrap_api.dart lib/rpc/smoldot_client.dart test/rpc/chain_bootstrap_api_test.dart`

### P-API-CITIZENAPP-005：CitizenApp Signed Extrinsic Relay

- 状态：当前（2026-07-08 第 4 步已落地 Worker 受控广播、D1 审计表和 App submit-only 兜底）
- 类型：接口契约
- 唯一真源：
  - Worker：`citizenapp/cloudflare/src/chain/extrinsic_relay.ts`
  - D1 迁移：`citizenapp/cloudflare/migrations/0007_chain_extrinsic_relay.sql`
  - App：`citizenapp/lib/rpc/signed_extrinsic_relay_api.dart`、`citizenapp/lib/rpc/chain_rpc.dart`
- 详细文档：
  - `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
  - `memory/08-tasks/open/20260708-citizenapp-chain-edge-architecture.md`
- 生产者：
  - CitizenApp 本地完成签名后的 submit-only 兜底逻辑。
  - Cloudflare Worker `POST /v1/chain/extrinsics/relay`。
- 消费者：
  - CitizenChain RPC service node 的 `author_submitExtrinsic`。
  - D1 表 `chain_extrinsic_relays`。
- HTTP API 字段：
  - 请求：`signed_extrinsic_hex`，完整 signed extrinsic hex，必须以 `0x` 开头。
  - 响应：`ok`、`schema=citizenapp.chain.extrinsic_relay.v1`、`relay_id`、`relay_status=broadcast`、`deduplicated`、`tx_hash`、`accepted_at`、`chain_success_source=finalized_runtime_storage_or_events`。
- 编码：HTTP JSON，字段统一 snake_case；`tx_hash` 为 32 字节 hex；Worker 不保存原始 extrinsic body，只保存 `extrinsic_sha256`。
- 签名/验签规则：App 在本地完成交易签名；Worker 不接触私钥、不生成签名、不修改交易载荷，只把完整 signed extrinsic 交给服务节点 RPC。
- 禁止兼容：不得演化成通用 JSON-RPC proxy；不得新增兼容旧 RPC URL 下发字段。
- 禁止事项：
  - 禁止请求体携带 `private_key`、`mnemonic`、`seed`、`secret`、`keystore`、`password`、`recovery_phrase` 等密钥材料。
  - 禁止响应中返回 `CITIZEN_CHAIN_RPC_URL`、两项 `CITIZEN_CHAIN_RPC_ACCESS_*`、Validator RPC、Archive RPC 或任何私密 RPC 完整 URL。
  - 禁止把 relay 返回 `tx_hash` 当成链上成功；业务成功必须继续以 finalized runtime storage 或事件为准。
  - 禁止 App 在交易本身已被判定为 invalid / bad proof / stale / future / payment 类错误时再走 relay 兜底。
- 必跑测试：
  - `npm --prefix citizenapp/cloudflare run typecheck`
  - `npm --prefix citizenapp/cloudflare test -- chain_bootstrap.test.ts chain_extrinsic_relay.test.ts`
  - `flutter analyze lib/rpc/chain_bootstrap_api.dart lib/rpc/chain_rpc.dart lib/rpc/signed_extrinsic_relay_api.dart test/rpc/chain_bootstrap_api_test.dart test/rpc/signed_extrinsic_relay_api_test.dart`
  - `flutter test test/rpc`
  - `npm --prefix citizenapp/cloudflare run migrate:local`

### P-IM-002：CitizenApp Nearby Chat Transport

- 状态：草案（2026-07-05 方案冻结；待后续阶段实现）
- 类型：近场传输契约
- 唯一真源：
  - IM 技术文档：`memory/05-modules/citizenapp/im/IM_TECHNICAL.md`
  - 落地后实现真源：`citizenapp/android/im/`、`citizenapp/ios/im/`、`citizenapp/lib/im/transport/`
- 生产者：`citizenapp/lib/im/transport/ImNearbyTransport`
- 消费者：`citizenapp/lib/im/`
- 传输方式：
  - Android：Nearby Connections，后续补 Wi-Fi Aware / BLE fallback。
  - iOS：Multipeer Connectivity。
  - Android / iOS 跨平台：BLE 发现 + Wi-Fi / 热点数据通道，或二维码交换会话信息后 Wi-Fi 直连。
- Wire 载荷：
  - `nearby_session_id`
  - `sender_account`
  - `sender_device_id`
  - `recipient_account`
  - `recipient_device_id`
  - `envelope_id`
  - `envelope`
- 编码：
  - `envelope` 承载 `GMB_IM_V1 / ImEnvelope` Protobuf bytes。
  - 近场 transport 不改变 OpenMLS 会话、不改变 `ImEnvelope`。
- 签名/验签规则：
  - 近场初次通信必须显示安全码或二维码校验入口。
  - 钱包地址只作为聊天身份；OpenMLS 设备密钥负责端到端加密。
- 禁止事项：
  - 禁止近场依赖 Cloudflare、链 RPC 或区块链节点通信节点。
  - 禁止近场传输明文私聊/群聊内容。
  - 禁止为近场另建一套消息格式。
- 必跑测试：
  - `ImNearbyTransport` envelope 去重测试。
  - Android 真机近场收发测试。
  - iOS 真机近场收发测试。
  - Android / iOS 跨平台近场 smoke。

### P-TX-013：Square.publish_square_post

- 状态：草案（阶段 4 runtime 发布索引已落地；阶段 5 CitizenApp 交易编码和提交入块闭环已落地；阶段 6 Worker 链上事件确认和正式 feed 入库已落地）
- 类型：交易载荷格式 / storage 契约
- 唯一真源：
  - 方案任务卡：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
  - 落地后实现真源：`citizenchain/runtime/otherpallet/square-post/`
- 详细文档：
  - `memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
  - `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/8964/chain/`
  - `citizenapp/lib/rpc/`
- 消费者：
  - `citizenchain/runtime/otherpallet/square-post/`
  - `citizenapp/cloudflare/src/chain/`
- 交易字段：
  1. `post_id`: `Vec<u8>`，runtime 约束为非空且最多 64 字节。
  2. `post_category`: `SquarePostCategory::Normal` / `SquarePostCategory::Campaign`。
  3. `content_hash`: `[u8; 32]`，必须非全 0。
  4. `storage_receipt_id`: `Vec<u8>`，runtime 约束为非空且最多 96 字节。
  5. `storage_until`: `u64`，必须大于 0。
- 派生字段：
  - `owner_account`：由 signed origin 派生，App 不得作为参数伪造。
  - `cid_number`：由 runtime 按 `owner_account` 读取链上公民身份绑定派生；未认证用户为空。
  - `created_block`：由 runtime 当前区块派生。
- 编码：
  - SCALE call data。
  - pallet index：`36`。
  - call index：`0`。
  - `post_category` 枚举只允许 `Normal = 0` / `Campaign = 1`。
  - `content_hash` 固定为内容 manifest 规范化后的 hash。
  - `storage_receipt_id` 固定引用 P-API-CITIZENAPP-002 的 Worker 上传回执。
  - CitizenApp 编码实现真源：`citizenapp/lib/8964/chain/square_chain_service.dart`。
- 链上 storage / event：
  - `SquarePosts[post_id] -> SquarePost`：保存 `post_id`、`owner_account`、可空 `cid_number`、`post_category`、`content_hash`、`storage_receipt_id`、`storage_until`、`created_block`。
  - `PublishedPostCountByAccount[owner_account] -> u64`：保存账户累计成功发布数量。
  - `SquarePostPublished` 事件：Worker 只能根据入块事件确认发布并写入正式 feed。
- 签名/验签规则：
  - 外层链交易必须由 `owner_account` 对应钱包账户签名。
  - runtime 对每条发布交易扣 1 元发布费。
  - 发布费按既定 8:1:1 规则支付到矿工、国储会费用账户、安全基金账户。
  - `post_category = normal` 时所有钱包账户可发布。
  - `post_category = campaign` 时 runtime 必须确认 `owner_account` 在 `CitizenIdentity::VotingIdentityByAccount` 中有 `citizen_status = Normal` 的 `cid_number`。
  - Worker 同步 feed 只能信任已 finalized 或已确认入块的链上事件。
- 禁止兼容：不兼容任何“媒体内容上链”或“链下假发布成功”的旧流程。
- 禁止事项：
  - 禁止把动态正文、图片、视频、封面、manifest 写入 runtime storage。
  - 禁止 runtime 依赖 Worker 判断竞选发布资格。
  - 禁止非认证用户通过传入 `cid_number` 伪造竞选发布。
  - 禁止链上交易未入块时把动态作为正式内容进入 feed。
- 必跑测试：
  - runtime 普通发布成功测试。
  - runtime 未认证用户发布 `campaign` 失败测试。
  - runtime 认证用户发布 `campaign` 成功测试。
  - runtime `SquarePost` 费用分类为 `VoteFlat` 测试；实际 1 元和 8:1:1 分账复用现有 `OnchainFeeRouter` 测试。
  - CitizenApp 发布交易编码测试：阶段 5 已由 `citizenapp/test/8964/square_chain_service_test.dart` 覆盖。
  - Worker 链上事件解码与发布确认测试：阶段 6 已由 `citizenapp/cloudflare/test/chain_confirm.test.ts` 覆盖。

### P-QR-002：QR_V1 / k=1 sign_request

- 状态：当前
- 类型：扫码协议内签名请求流向
- 唯一真源：`memory/01-architecture/qr/qr-signing-recognition.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenapp`、`citizenchain/node`、`citizenchain/onchina`
- 消费者：`citizenwallet`
- 字段：
  - `b.a`:业务动作码
  - `b.g`:签名算法码,当前 `1 = sr25519`
  - `b.u`:32B 签名者公钥,base64url 无填充
  - `b.d`:payload bytes,base64url 无填充
- 编码：外层 JSON；`b.d` 内部是具体链上 call data 或已登记的链下业务载荷
- 签名/验签规则：
  - `b.a` 必须已登记
  - `b.d` 必须能被扫码端 decoder 按对应交易载荷格式完整解码
  - `b.a` 必须和 decoder 得到的 action 一致
  - 用户确认页只展示 decoder 产出的 `reviewFields`;左侧分类名必须由统一映射翻译为中文，禁止直接渲染机器 key
  - 用户确认页的账户字段必须展示 SS58 地址，禁止把原始公钥 hex 当作普通用户确认字段展示
  - `activate_admin_account` 载荷中的 `institution_code` 必须用共享机构码编码，禁止各端手写第二套字节映射。
  - **onchina 控制台链写动作码(`b.d`=裸 SCALE call data,冷钱包解码核对后冷签 origin 由 CitizenWallet 提交)**:链交易统一 `a=(pallet<<8)|call`(禁止扁平小整数,会撞非链动作码 1..8)。机构创建=公权 `0x2005`(PublicManage 32/call 5)/私权 `0x2105`(PrivateManage 33/call 5,见 P-TX-001);公民投票身份注册=`0x0a00`(CitizenIdentity 10/call 0,见 P-TX-011);公民参选身份上链=`0x0a01`(CitizenIdentity 10/call 1,见 P-TX-011);管理员集合=CREG `0x0c01`(`federal_set_city_registry_admins`)/FRG `0x0c00`(`propose_admin_set_change`,见 P-TX-007);非链文本治理 `a=3 = ACTION_ONCHINA_ADMIN / QR_ACTION_ONCHINA_ADMIN`(onchina_admin_governance JSON);IM 钱包绑定 `a=8 = QR_ACTION_IM_WALLET_BINDING`。动作码由 `onchina/src/core/institution_call.rs::chain_action_code(pallet,call)` 与 call data 同源派生,非链常量在 `core/qr/mod.rs`,runtime 注释真源在 `primitives::sign`,均与 `qr-action-registry.md` 同步。
  - Substrate 交易 payload 长度 >256B 时必须签 `blake2_256(payload)`
- 禁止兼容：开发期严格模式，不做别名兼容
- 禁止事项：
  - 禁止恢复 `display` / `summary` / `fields`
  - 禁止未登记的 `a` 进入生产
  - 禁止把内部哈希、nonce、原始公钥 hex 当作普通用户确认字段展示
- 必跑测试：`citizenwallet/test/signer/payload_decoder_test.dart`、QR sign request 测试

### P-QR-003：QR_V1 / k=5 im_node_pairing

- 状态：已删除（2026-07-05 聊天方案改为 Cloudflare 互联网聊天 + 近场聊天；区块链节点通信节点聊天方式不再作为正式路线）
- 类型：扫码协议内固定码
- 唯一真源：无当前代码真源；旧实现文件已删除
- 详细文档：
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`
  - `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`
- 生产者：无；桌面节点不再生成 IM 配对二维码。
- 消费者：无；CitizenApp 扫到 `k=5` 按未知类型拒绝。
- 字段：
  - 无当前字段；旧 `b.node_peer_id`、`b.node_multiaddr`、`b.endpoint_kind` 已删除。
- 编码：无当前编码；`QR_V1/k=5` 不再是合法扫码流向。
- 签名/验签规则：正式聊天不再扫描区块链软件通信节点二维码。
- 禁止兼容：不得恢复旧联系人码、旧 IM 联系人 bundle、旧 `communication` 模式字段或通信节点配对流程。
- 禁止事项：
  - 禁止用本二维码添加联系人。
  - 禁止把本二维码作为交易、转账、治理或 CID 身份码处理。
  - 禁止恢复通信节点配对、桌面通信节点二维码、节点 IM mailbox 或 `/gmb/im/1`。
- 删除验收：已删除 `citizenapp/lib/qr/bodies/im_node_pairing_body.dart`、`citizenapp/lib/im/im_node_settings_page.dart`、桌面通信节点二维码生成和相关测试残留；`test/qr/qr_router_test.dart` 覆盖 `k=5` 拒绝。

### P-CRED-003：CitizenIdentity VotingIdentityPayload

- 状态：当前
- 类型：凭证载荷 / 交易载荷内层结构
- 唯一真源：`citizenchain/runtime/otherpallet/citizen-identity/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- 消费者：
  - `citizenwallet/lib/signer/payload_decoder.dart`
  - `citizenwallet/lib/signer/qr_signer.dart`
  - `citizenchain/runtime/otherpallet/citizen-identity`
- 字段：
  1. `cid_number`
  2. `wallet_account`
  3. `citizen_age_years`
  4. `passport_valid_from`
  5. `passport_valid_until`
  6. `citizen_status`
  7. `residence_province_code`
  8. `residence_city_code`
  9. `residence_town_code`
- 编码：SCALE `VotingIdentityPayload<AccountId>`;字符串字段为 bounded `Vec<u8>`,账户字段为 `AccountId32`。
- 签名/验签规则：
  - `QR_V1` 非链动作 `a=2 citizen_identity` 的签名字节为 `blake2_256(GMB || 0x10 || payload_bytes)`。
  - runtime 通过 `primitives::sign::OP_SIGN_CITIZEN_IDENTITY` 验证目标公民钱包签名。
  - `citizen_age_years` 必须大于等于 16;OnChina 和 runtime 都必须校验。
- 禁止兼容：不兼容旧 `citizen-identity-v1|...` 文本载荷,不保留旧签原文规则。
- 禁止事项：
  - 禁止本地新增公民阶段要求钱包账户。
  - 禁止未满 16 周岁公民推送链上身份。
  - 禁止二维码携带展示摘要或字段别名。
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/qr_signer_test.dart`
  - `cargo test --manifest-path citizenchain/Cargo.toml -p citizen-identity`

### P-TX-011：CitizenIdentity.register_voting_identity / upgrade_to_candidate_identity

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/otherpallet/citizen-identity/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- 消费者：
  - `citizenchain/runtime/otherpallet/citizen-identity`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `registrar_account`
  2. `VotingIdentityPayload` 或 `CandidateIdentityPayload`
  3. `citizen_signature`
- 编码：
  - SCALE call data
  - `0x0a00 register_voting_identity` 仅携带投票身份字段。
  - `0x0a01 upgrade_to_candidate_identity` 携带投票身份字段 + `birth_province_code / birth_city_code / birth_town_code / citizen_full_name / citizen_sex`；该交易同时写入投票身份和参选身份。
  - pallet index：`10`
  - call index：`0`
  - 前两个字节固定为 `[0x0a, 0x00]`
  - 动作码：`a=0x0a00`
- 签名/验签规则：
  - 外层链交易由当前注册局管理员公民钱包签名并提交。
  - 内层 `citizen_signature` 必须来自目标公民钱包对 P-CRED-003 的签名。
  - runtime 校验注册局管理范围、CID 唯一性、公民签名和 16 周岁年龄门槛。
- 禁止兼容：不兼容旧无年龄字段的 `VotingIdentityPayload`,不保留旧字段顺序。
- 禁止事项：
  - 禁止绕过 `citizen-identity` 在业务模块内自建投票身份。
  - 禁止前端或 OnChina 伪造已上链状态。
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `cargo test --manifest-path citizenchain/Cargo.toml -p citizen-identity`

### P-TX-012：LegislationYuan 法律案提案载荷

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/public/legislation-yuan/src/lib.rs`
- 详细文档：
  - `memory/04-decisions/ADR-027-legislation-yuan.md`
- 生产者：
  - `citizenchain/onchina/src/domains/legislation/law/chain_propose.rs`
- 消费者：
  - `citizenchain/runtime/public/legislation-yuan`
  - `citizenwallet` 法律案扫码 decoder
- 字段：
  - `propose_enact_law`: `[pallet=27, call=0] + tier + scope_code + houses + proposer_body + executive + legislature + vote_type + title + title_en + chapters + effective_at`
  - `propose_amend_law`: `[pallet=27, call=1] + law_id + proposer_body + executive + legislature + vote_type + title + title_en + chapters + effective_at`
  - `propose_repeal_law`: `[pallet=27, call=2] + law_id + proposer_body + executive + legislature + vote_type`
- 编码：
  - 裸 SCALE call data
  - `tier`/`vote_type` 为单字节枚举序号
  - `scope_code` 为 `u32`
  - `law_id` 为 `u64`
  - `houses` / `proposer_body` / `executive` / `legislature` 使用 `(InstitutionCode[4], AccountId32)`
  - `chapters` 为 `章 > 节 > 条 > 款` 的 SCALE 结构
  - `effective_at` 为 `u64` 毫秒时间戳，不是块号
  - 动作码：`0x1b00` / `0x1b01` / `0x1b02`
- 签名/验签规则：
  - 外层链交易由当前立法/提案机构管理员冷钱包签名并提交。
  - 业务投票、计票、签署和守卫流程统一归投票引擎与 legislation-vote，不得由 OnChina 或客户端复刻。
- 禁止兼容：不兼容旧区块高度生效载荷，不保留旧字段顺序。
- 禁止事项：
  - 禁止前端显示或让用户填写旧区块高度生效字段。
  - 禁止未登记动作码进入冷钱包 decoder。
- 必跑测试：
  - `cargo test -p onchina --manifest-path citizenchain/Cargo.toml law`
  - `cargo test -p legislation-yuan --manifest-path citizenchain/Cargo.toml`

### P-IM-001：GMB_IM_V1

- 状态：当前（统一消息/加密格式；正式传输只保留 Cloudflare 密文 mailbox 和近场聊天）
- 类型：接口契约 / 编码协议 / 端到端加密消息外层
- 唯一真源：`citizenapp/im/proto/im_envelope.proto`
- Dart 生成物：`citizenapp/lib/im/proto/im_envelope.pb.dart`、`citizenapp/lib/im/proto/im_envelope.pbenum.dart`、`citizenapp/lib/im/proto/im_envelope.pbjson.dart`
- 详细文档：
  - `memory/04-decisions/ADR-020-citizenapp-p2p-im.md`
  - `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/im/`：OpenMLS 加密、会话状态机、消息队列。
  - `citizenapp/cloudflare/src/chat/`：Cloudflare 密文 mailbox 接口，不生成明文。
  - `citizenapp/android/im/`：Android 近场 transport。
  - `citizenapp/ios/im/`：iOS 近场 transport。
- 消费者：
  - `citizenapp/lib/im/`
  - `citizenapp/cloudflare/src/chat/`
  - `citizenapp/android/im/`
  - `citizenapp/ios/im/`
- 字段：
  - `ImEnvelope.protocol_version`
  - `ImEnvelope.envelope_id`
  - `ImEnvelope.conversation_id`
  - `ImEnvelope.sender_chat_account`
  - `ImEnvelope.recipient_chat_account`
  - `ImEnvelope.sender_device_id`
  - `ImEnvelope.mls_wire_message`
  - `ImEnvelope.encrypted_metadata`
  - `ImEnvelope.attachment_manifest_hash`
  - `ImEnvelope.chunk_refs`
  - `ImEnvelope.created_at_millis`
  - `ImEnvelope.ttl_millis`
  - `ImEnvelope.ack_policy`
  - `ImEnvelope.mls_message_kind`
  - `ImEnvelope.ratchet_tree`
  - `ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_UNSPECIFIED`
  - `ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_WELCOME`
  - `ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION`
  - `ImRouteRecord.proto`
  - `ImRouteRecord.wallet_chat_account`
  - `ImRouteRecord.route_display_name`
  - `ImRouteRecord.im_device_id`
  - `ImRouteRecord.im_device_pubkey_hex`
  - `ImRouteRecord.safety_number`
  - `ImRouteRecord.cloudflare_mailbox_id`
  - `ImRouteRecord.nearby_peer_hint`
  - `ImRouteRecord.created_at_millis`
  - `ImRouteRecord.expires_at_millis`
  - `ImEnvelopeAck.envelope_id`
  - `ImEnvelopeAck.state`
  - `ImKeyPackage.protocol_version`
  - `ImKeyPackage.owner_wallet_account`
  - `ImKeyPackage.device_id`
  - `ImKeyPackage.device_public_key_hex`
  - `ImKeyPackage.key_package_id`
  - `ImKeyPackage.key_package`
  - `ImKeyPackage.cipher_suite`
  - `ImKeyPackage.created_at_millis`
  - `ImKeyPackage.expires_at_millis`
  - `ImKeyPackage.consumed_at_millis`
  - `PublishImKeyPackageRequest.owner_wallet_account`
  - `PublishImKeyPackageRequest.device_id`
  - `PublishImKeyPackageRequest.device_public_key_hex`
  - `PublishImKeyPackageRequest.key_package_id`
  - `PublishImKeyPackageRequest.key_package`
  - `PublishImKeyPackageRequest.cipher_suite`
  - `PublishImKeyPackageRequest.created_at_millis`
  - `PublishImKeyPackageRequest.expires_at_millis`
  - `FetchImKeyPackagesRequest.owner_wallet_account`
  - `FetchImKeyPackagesRequest.requester_chat_account`
  - `FetchImKeyPackagesRequest.limit`
  - `ConsumeImKeyPackageRequest.owner_wallet_account`
  - `ConsumeImKeyPackageRequest.key_package_id`
  - `ConsumeImKeyPackageRequest.requester_chat_account`
- 验收接口：
  - 互联网聊天只走 `P-API-CITIZENAPP-003：CitizenApp Chat Cloudflare Mailbox`。
  - 近场聊天只走 `P-IM-002：CitizenApp Nearby Chat Transport`。
  - 区块链节点通信节点、`/gmb/im/1`、节点 mailbox、节点 KeyPackage 池和 `im_node_pairing` 已删除，不再作为正式验收入口。
- 编码：外层 Protobuf；OpenMLS 标准 wire bytes 放入 `mls_wire_message`；链内 SCALE 不作为 IM 主协议。
- 当前实现状态：Dart Protobuf 生成与 `ImEnvelope` / `ImKeyPackage` / `ImRouteRecord` round-trip 已通过；`ImEnvelope` 已承载 MLS message kind 与 Welcome ratchet tree；OpenMLS native 边界通过现有 `libsmoldot` C ABI 调用 Rust OpenMLS，可生成真实 KeyPackage、返回设备签名公钥、完成两方 round-trip smoke、创建持久化 MLS 会话、处理 Welcome、解密 application，并在 App 重启后恢复同一会话；macOS host 调试库构建已禁用 release strip，避免 dyld `mis-aligned LINKEDIT string pool`，native OpenMLS mailbox 闭环已在本机真实执行通过；公民端已有 Isar 消息库、消息流状态机、IM 路由缓存、联系人详情消息入口和信息 Tab 会话列表；`ImCloudflareTransport` 已接入 Worker chat HTTP API，支持设备登记、KeyPackage 发布/拉取/消费、密文投递、pending 拉取、ack 删除、WebSocket 新密文通知、加密附件 prepare/upload/complete 和加密附件 download/dev-get；Worker 已新增账户级 `ChatRealtimeObject`，`POST /v1/chat/envelopes` 落库成功后按接收钱包调用 DO，DO 只向在线设备推送新密文索引通知；Worker mailbox 已收紧为临时队列，ack 后删除 `chat_envelopes` 行和对应 R2 加密附件对象，提交/拉取时顺手清理过期 envelope；`ImRuntime` 已在发送或同步前自动复用广场 Worker session、签名登记 IM 设备、发布本设备 KeyPackage，并在首次会话自动拉取/消费对方 KeyPackage；`ImRuntime.sendAttachment` 已支持发送端先保存本机附件缓存，再本地 AES-GCM 加密 manifest/chunk、上传 R2 密文对象、投递 OpenMLS 附件控制消息；`ImRuntime.downloadAttachment` 已支持优先读取本机附件缓存，未命中时下载密文 manifest/chunk、校验 sha256、本地 AES-GCM 解密并保存到 App 私有缓存；`ImRuntime.deleteLocalConversation` 已支持删除某个本机会话的 Isar 消息、队列、pending 入站记录和附件缓存目录；`ImRuntime.startRealtimeSync` 已支持 WebSocket 通知后触发 pending 同步；聊天页已接入现成附件按钮文件选择、点击附件下载和右上角更多菜单删除本机聊天记录；信息 Tab 和聊天窗口打开后会先同步 pending，再优先使用 WebSocket 新密文通知，连接不可用或断开时分别回退到 15 秒 / 8 秒前台轮询，失败统一退避到 30 秒，轮询成功后重试 WebSocket，页面销毁或 App 退后台即停止；信息 Tab 会话列表已支持左滑删除本机聊天记录；IM-5/IM-6 已覆盖 A/B 两端经 mailbox pending 拉取、接收端解密落库和 ack 清空 mailbox 的闭环状态机；IM-7/IM-8 已覆盖附件密文上传、附件消息落库、下载授权、下载解密和 UI 触发；IM-9/IM-10 已覆盖信息 Tab/聊天页实时通知优先、轮询兜底和账户级 DO 路由；IM-11 已覆盖 ack 删除 Cloudflare envelope/R2 加密附件、附件先落本机再 ack、删除本机会话不误删其他会话；IM-12 已覆盖信息 Tab 左滑删除、聊天页菜单删除和删除后返回上一页；旧“我的 -> 设置 -> 设置通信节点”、桌面通信节点功能、节点 IM mailbox 和 `/gmb/im/1` 已删除。
- 签名/验签规则：
  - `ImRouteRecord` 是 IM 模块内部路由缓存，不是第二套通讯录，不得替代“我的通讯录”联系人详情。
  - 公民端发消息必须读取用户资料中的通信账户；未设置通信账户不得发送。
  - 钱包账户只对 IM 设备、公钥、过期时间和 nonce 做绑定签名。
  - 绑定签名请求固定为 `QR_V1/k=1/a=8 im_wallet_binding`；`b.d` 是 `wallet_account, im_device_id, im_device_pubkey, expires_at_millis, nonce` 的 SCALE bytes。
  - 签名字节固定为 `signing_message(OP_SIGN_IM_WALLET_BINDING=0x1A, b.d)`；Cloudflare mailbox 登记设备前必须用 `wallet_account` 解出的 32 字节公钥验签。
  - 钱包私钥只用于绑定证明，不作为 IM 消息加密密钥。
  - KeyPackage 由 IM 设备密钥管理，必须具备 TTL、一次性消费或租约消费、防重放和撤销清理。
  - 首次 MLS 会话发送会产生 Welcome + application 两条 wire message；Welcome 必须通过 `ImEnvelope.ratchet_tree` 伴随传递 ratchet tree bytes。
  - Cloudflare mailbox 只接受 `recipient_chat_account == ImEnvelope.recipient_chat_account` 的密文信封。
  - 附件密钥材料只允许出现在 OpenMLS application 载荷中，不得放入 `ImEnvelope.encrypted_metadata`、D1 字段或 R2 manifest 明文。
  - 附件下载授权只允许发送方或接收方钱包账户获取，且必须与 `chat_envelopes.attachment_manifest_key` 匹配。
  - 近场 transport 只传输同一个 `ImEnvelope`，不得另建明文近场消息格式。
- 存储边界：
  - CitizenApp 本地保存明文消息、OpenMLS provider storage、发送队列和路由缓存。
  - 删除聊天记录只删除当前设备本地会话、消息、发送队列、pending 入站记录和附件缓存，不删除联系人，不影响其他设备或对方设备。
  - Cloudflare D1 保存设备绑定、KeyPackage，以及未 ack / 未过期的临时密文 envelope。
  - Cloudflare R2 只临时保存 CitizenApp 本地加密后的附件 manifest 和附件分片；对应 envelope ack 或过期后删除。
  - Android / iOS 近场 transport 不做长期服务端存储。
- 禁止兼容：开发期不兼容未登记字段、未登记协议名、旧 Matrix / Olm / Megolm 主协议口径、旧通信节点 mailbox、旧 `/gmb/im/1`、旧 `im_node_pairing`。
- 禁止事项：
  - 禁止把 CID 号码、实名信息、身份档案字段写入 IM 协议。
  - 禁止把 IM 路由缓存做成第二套通讯录。
  - 禁止复用钱包私钥作为 IM 端到端加密密钥。
  - 禁止把私聊或群聊明文写入 Cloudflare、链、节点或广场公开表。
  - 禁止恢复区块链节点通信节点聊天、`/gmb/im/1`、节点 mailbox 或节点 KeyPackage 池作为正式路线。
- 必跑测试：`cargo test`（`citizenapp/rust`）、`flutter test --concurrency=1 test/im/im_route_cache_store_test.dart`、`flutter test --concurrency=1 test/im/im_tab_page_test.dart test/im/im_envelope_proto_test.dart test/im/im_mls_native_test.dart`、`flutter test --concurrency=1 test/im/im_mls_session_test.dart test/im/im_mls_native_session_test.dart`、`flutter test --concurrency=1 test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart`、Worker chat API 测试、Worker `/v1/chat/ws` smoke、`ImCloudflareTransport` 测试、加密附件 prepare/upload/complete/download/dev-get 测试、聊天页附件按钮/点击下载测试、聊天记录删除 UI 测试、`ImRuntime` 自动 mailbox 准备测试、信息 Tab/聊天页 WebSocket 通知优先与前台轮询兜底测试、`ImNearbyTransport` 测试、Protobuf 跨端 round-trip、OpenMLS 加解密测试、KeyPackage 防重放测试、Cloudflare mailbox 只存密文检查。
- 运行态 smoke：后续以 CitizenApp + Cloudflare Worker 本地/预发环境验证密文投递、拉取、ack、附件密文上传/下载授权、WebSocket 通知和近场真机收发；不得再以 `citizenchain/scripts/im-two-node-smoke.sh` 或 `/gmb/im/1` 作为正式 smoke。

### P-TX-001：PublicManage/PrivateManage.propose_create_{public,private}_institution

- 状态：当前(机构管理已拆分公权/私权两 pallet,取代旧 `OrganizationManage.propose_create_institution`)
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/entity/public-manage/src/lib.rs`(`propose_create_public_institution` call 5)
  - `citizenchain/runtime/entity/private-manage/src/lib.rs`(`propose_create_private_institution` call 5)
  - 两 call 参数形态完全相同(下 16 字段),仅 pallet 前缀不同
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：
  - `citizenchain/onchina/src/core/institution_call.rs`(注册局录入,按机构码路由公权/私权 pallet 前缀)
  - `citizenapp/lib/transaction/...`(机构创建,具体路径随 runtime 拆分对齐)
- 消费者：
  - `citizenchain/runtime/entity/public-manage` / `citizenchain/runtime/entity/private-manage`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `cid_number`
  2. `cid_full_name`
  3. `cid_short_name`（公权/私权机构统一上链）
  4. `town_code`（镇级公权机构运行期注册时必填;非镇级机构为空）
  5. `accounts`
  6. `institution_code`
  7. `admins_len`
  8. `admins`（A2 起 = `Vec<AdminProfile>`，逐人 account+admin_cid_number+name+admin_role+term_start+term_end+source；投票快照只取 `.account`，编码同 P-TX-007 机构布局的 AdminProfile）
  9. `threshold`
  10. `register_nonce`
  11. `signature`
  12. `issuer_cid_number`
  13. `issuer_main_account`
  14. `signer_pubkey`
  15. `scope_province_name`
  16. `scope_city_name`
- 编码：
  - SCALE call data
  - pallet index：公权机构=`32`(PublicManage),私权机构=`33`(PrivateManage);由 `institution_code` 经 `primitives::cid::code::is_private_legal_code` 派生(onchina `create_institution_pallet_index` 单源)
  - call index：`5`(两 pallet 同)
  - 前两个字节:公权=`[0x20, 0x05]`(动作码 `0x2005`)、私权=`[0x21, 0x05]`(动作码 `0x2105`)
- 签名/验签规则：
  - `register_nonce / signature / issuer_cid_number / issuer_main_account / signer_pubkey / scope_*` 由 CID 机构注册信息凭证提供
  - runtime 通过 `issuer_main_account` 查询 `admins-change::AdminAccounts`,确认 `signer_pubkey` 属于该机构 `admins` 后验签
  - `accounts.account_name` 顺序必须与 CID `/registration-info.account_names` 一致
  - 名称分档：runtime 用 `primitives::cid::code::is_public_legal_code(institution_code)` 判定;公权/私权机构均必须带非空 `cid_full_name`+`cid_short_name` 并上链
  - CID 注册凭证签名覆盖 `cid_full_name`、`cid_short_name` 和 `town_code`;镇级机构不能复用非镇级凭证,简称也不能被改包篡改。
- 禁止兼容：开发期不兼容旧 `call_index=0`、不兼容旧 `OrganizationManage(17).propose_create_institution`
- 禁止事项：
  - 禁止把本交易载荷称为新增扫码协议
  - 禁止继续使用已删除的 `OrganizationManage(17)` / `[0x11,0x05]` 编码机构创建
  - 禁止在本载荷末尾追加 `subject_property / private_type / partnership_kind / parent_cid_number`
  - 禁止用裸非法人机构码（`SFGT/SFGP/UNIN`）直接创建机构账户；非法人必须由 CID 上层明确归属后走对应管理员模块
  - 禁止 CitizenWallet decoder 解码后仍有剩余字节
- 必跑测试：
  - `cargo test -p onchina`(institution_call 跨真类型对拍 + 公权/私权前缀分支)
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `cargo check -p public-manage -p private-manage`

### P-TX-010：AddressRegistry address payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/otherpallet/address-registry/src/lib.rs`
  - `citizenchain/onchina/src/domains/address/chain_call.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/ADDRESS_REGISTRY_TECHNICAL.md`
  - `memory/05-modules/citizenchain/onchina/ADDRESS_TECHNICAL.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenchain/onchina/src/domains/address/chain_call.rs`
- 消费者：`citizenchain/runtime/otherpallet/address-registry`
- 字段：
  - `set_catalog_version(35.0)`：`registrar_account`, `catalog_version`, `catalog_hash`
  - `set_address_name(35.1)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_name`
  - `remove_address_name(35.2)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`
  - `set_address(35.3)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail`
  - `remove_address(35.4)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail`
- 编码：
  - SCALE 裸 call data
  - pallet index：`35`
  - call index：`0..4`
  - 前两个字节：`[0x23, call_index]`
  - 动作码：`a=(35<<8)|call_index`,即 `0x2300..0x2304`
- 签名/验签规则：
  - `origin` 必须是 `registrar_account` 对应注册局的有效管理员。
  - FRG 省级组只能更新本省地址。
  - CREG 只能更新本市地址。
  - `catalog_version` 与 `catalog_hash` 由 OnChina 当前 `china.sqlite` 派生或由调用方显式传入。
- 禁止兼容：不兼容旧地址全量上链、旧墓碑表、旧变更日志表和旧地址字段。
- 禁止事项：
  - 禁止把地址库全量上链。
  - 禁止在链上保存旧地址历史或墓碑。
  - 禁止绕过 `AddressUpdateAuthority` 直接在 pallet 内复制 FRG/CREG 权限。
- 必跑测试：
  - `cargo check --manifest-path citizenchain/Cargo.toml -p address-registry`
  - `cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`
  - `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`

### P-TX-002：JointVote.cast_referendum

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/src/lib.rs`
  - `citizenchain/runtime/votingengine/joint-vote/src/lib.rs`
- 详细文档：
  - `memory/06-quality/fixtures/step2d_credential_payload.json`
  - `memory/08-tasks/done/20260507-p0-5-step2d-fixture.md`
- 生产者：
  - `CitizenApp` 联合公投签名请求流程
  - Step2D fixture
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `approve`
- 编码：
  - SCALE call data
  - pallet index：`23`
  - call index：`1`
  - 前两个字节固定为 `[0x17, 0x01]`
  - call data 长度为 11 字节，后接标准签名尾部
- 签名/验签规则：
  - runtime 按交易签名账户读取链上公民身份。
  - 联合公投资格和作用域由 `citizen-identity` 判定。
- 禁止兼容：开发期不兼容旧 `VotingEngine(9).call_index=2`
- 禁止事项：
  - 禁止 Step2D fixture 中继续出现 `cast_referendum` 的 `pallet_index=9 / call_index=2`
  - 禁止 `cast_referendum` fixture 继续使用 `0x0902` 前缀
  - 禁止 `CitizenWallet` 与 `CitizenApp` 各自维护重复 Step2D fixture
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/pallet_registry_test.dart`
  - `citizenapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-CRED-001：OnChina subject registration-info credential

- 状态：当前
- 类型：凭证载荷 / 接口契约
- 唯一真源：`citizenchain/onchina/src/subjects/chain_multisig_info.rs`
- 详细文档：`memory/05-modules/citizenchain/node/offchain-transaction/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/subjects/chain_multisig_info.rs`
- 消费者：
  - `citizenchain/onchina/src/institution/subjects/registration_call.rs`(注册局录入构造 call data)
  - `citizenchain/runtime/entity/public-manage` / `citizenchain/runtime/entity/private-manage`(链端验签)
- 字段：
  - 外层业务字段：`cid_number`、`cid_full_name`、`account_names`
  - 凭证字段：`credential.register_nonce`、`credential.issuer_cid_number`、`credential.issuer_main_account`、`credential.signer_pubkey`、`credential.scope_province_name`、`credential.scope_city_name`、`credential.signature`
- 编码：
  - HTTP JSON 响应
  - runtime 验签 payload 按 OnChina 后端 `build_institution_registration_info_credential` 的 SCALE tuple 顺序
- 签名/验签规则：
  - OnChina 后端用签发机构管理员密钥签发。
  - 链端用 `issuer_main_account` 读取 `admins-change::AdminAccounts`，确认 `signer_pubkey` 属于该机构 `admins` 后验签。
  - `scope_province_name / scope_city_name` 只表示业务作用域，不表示签发人身份。
- 禁止兼容：不把 `subject_property / private_type / partnership_kind / parent_cid_number` 纳入链端注册凭证
- 禁止事项：
  - 禁止用普通机构详情接口替代 `/registration-info`
  - 禁止 CitizenApp 自己拼 `register_nonce / signature / issuer_cid_number / issuer_main_account / signer_pubkey / scope_*`
- 必跑测试：OnChina 后端 registration-info 测试、P-TX-001 双端编码/解码测试

### P-SIGN-001：Citizenchain signed extrinsic era

- 状态：当前
- 类型：签名 / extrinsic 协议
- 唯一真源：
  - `citizenchain/node/src/governance/signing.rs`
  - `citizenapp/lib/rpc/signed_extrinsic_builder.dart`
- 详细文档：
  - `memory/08-tasks/done/20260507-p0-4-immortal-era.md`
- 生产者：
  - `citizenchain/node`
  - `citizenapp`
  - `citizenwallet` 公民钱包提交链路
- 消费者：
  - `citizenchain runtime` signed extension 验签
- 字段：
  - `eraPeriod = 0`
  - `era bytes = 0x00`
  - `blockNumber = 0`
  - `SigningPayload.blockHash = genesisHash`
  - `ExtrinsicPayload.blockNumber = 0`
- 编码：
  - signed extension `CheckEra` 使用 immortal era 单字节 `0x00`
  - `CheckEra` additional signed hash 使用创世块哈希，即 `block_hash(0)`
- 适用范围：
  - 本协议仅约束 **sr25519 外层签名**的 signed extrinsic；PQC(ML-DSA-65)交易不走本协议，见下方 ADR-022 注与 P-TX-008/009。
- 签名/验签规则：
  - 签名前 payload 与最终 extrinsic body 必须使用同一份 immortal era 字节
  - 使用 polkadart 时必须传 `eraPeriod: 0`
  - `SigningPayload.blockHash` 必须传 `genesisHash`，不得传最新块 hash
  - 抗量子升级(ADR-022):PQC 交易改走 General Transaction(无外层 sr25519 签名),由自定义 `GmbPqcAuth` TransactionExtension 携带 ML-DSA-65 签名(proof 在扩展 extra),验签后把 origin 转 `Signed(account)`;未绑定账户首次走 bootstrap(post_dispatch 写 `AccountPqcKey`)无感绑定;AccountId 仍为原 sr25519 锚点。详见 P-TX-008/009。
- 禁止兼容：开发期不兼容热钱包 mortal era
- 禁止事项：
  - 禁止业务 service 自己保留 `_eraPeriod = 64`
  - 禁止 signed extrinsic 构造路径调用 `fetchLatestBlock()` 参与 era 计算
  - 禁止把最新块 hash 写入 immortal era 的 signing payload
- 必跑测试：
  - `citizenapp/test/rpc/signed_extrinsic_builder_test.dart`
  - `flutter test test/transaction/multisig-transfer test/proposal test/trade`

### P-TX-003：InternalVote.cast

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/votingengine/internal-vote/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `citizenwallet/lib/signer/pallet_registry.dart`
- 生产者：`citizenapp`、`citizenchain/node`
- 消费者：
  - `citizenchain/runtime/votingengine/internal-vote`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `approve`
- 编码：
  - SCALE call data
  - pallet index：`22`
  - call index：`0`
  - 前两个字节固定为 `[0x16, 0x00]`
- 签名/验签规则：
  - 管理员投票统一走 `InternalVote::cast`
  - 业务 pallet 不再承载 `vote_*` wrapper
- 禁止兼容：开发期不兼容旧 `VotingEngine(9)` 投票入口
- 禁止事项：
  - 禁止恢复业务 pallet 内的投票 wrapper
  - 禁止把内部投票编码回 `VotingEngine(9)`
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/pallet_registry_test.dart`

### P-TX-004：JointVote.cast_admin

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/votingengine/joint-vote/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `citizenwallet/lib/signer/pallet_registry.dart`
- 生产者：`citizenapp`、`citizenchain/node`
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `account_id`
  3. `approve`
- 编码：
  - SCALE call data
  - pallet index：`23`
  - call index：`0`
  - 前两个字节固定为 `[0x17, 0x00]`
- 签名/验签规则：
  - 联合投票的机构管理员阶段走 `JointVote::cast_admin`
  - `account_id` 底层类型为 `AccountId`
- 禁止兼容：开发期不兼容旧 `VotingEngine(9)` 投票入口
- 禁止事项：
  - 禁止恢复旧联合投票 wrapper
  - 禁止把 `account_id` 注释成当前 `InstitutionPalletId`
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/pallet_registry_test.dart`

### P-CRED-002：PopulationScopeSnapshot

- 状态：当前
- 类型：链上人口作用域
- 唯一真源：`citizenchain/runtime/otherpallet/citizen-identity`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
  - `memory/05-modules/citizenchain/runtime/otherpallet/citizen-identity/CITIZEN_IDENTITY_TECHNICAL.md`
- 生产者：链上交易调用者
- 消费者：
  - `citizenchain/runtime/votingengine`
  - `citizenapp`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `PopulationScope::Country`
  2. `PopulationScope::Province(province_code)`
  3. `PopulationScope::City(province_code, city_code)`
  4. `PopulationScope::Town(province_code, city_code, town_code)`
- 编码：
  - SCALE call data
  - `JointVote.prepare_joint_population_snapshot(scope)` 使用 pallet `23` / call `2`
  - `LegislationVote.prepare_population_snapshot(scope)` 使用 pallet `28` / call `0`
- 签名/验签规则：
  - 交易只按标准链上账户签名。
  - runtime 从 `citizen-identity` 读取作用域人口分母。
- 禁止兼容：开发期不兼容任何链下签发人口证明格式
- 禁止事项：
  - 禁止前端或 OnChina 伪造人口分母。
  - 禁止业务模块自行获取或透传人口证明；人口快照只属于投票引擎及其投票流程。
  - 禁止跳过 runtime 链上人口读取。
- 必跑测试：
  - `citizenchain/runtime/src/tests/cases.rs` 中 population snapshot 相关测试
  - `citizenapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-TX-003：ResolutionIssuance.propose_resolution_issuance

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/issuance/resolution-issuance/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/issuance/resolution-issuance/RESOLUTIONISSUANCE_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`citizenapp`
  - `citizenchain/node/src/transaction/multisig_transfer/`
  - `citizenchain/node/frontend/transaction/multisig-transfer/`
  - `citizenapp/lib/transaction/multisig-transfer/`
- 消费者：
  - `citizenchain/runtime/issuance/resolution-issuance`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `reason`
  2. `total_amount`
  3. `allocations`
- 编码：
  - SCALE call data
  - pallet index：`8`
  - call index：`0`
  - 前两个字节固定为 `[0x08, 0x00]`
- 签名/验签规则：
  - 本交易载荷只包含发行内容,不内嵌人口快照字段。
  - 联合提案人口快照由 `JointVote.prepare_joint_population_snapshot(scope)` 单独准备并读取链上公民身份人口。
- 禁止兼容：开发期不兼容继续把人口快照字段塞回本载荷的旧格式
- 禁止事项：
  - 禁止节点或前端把人口快照字段或旧链下人口证明字段混入本交易载荷
  - 禁止把发行金额显示口径和链端 `u128` 分单位混用
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenchain/runtime/src/tests/cases.rs`

### P-TX-005：MultisigTransfer proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/transaction/multisig-transfer/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/transaction/multisig-transfer/MULTISIG_TRANSFER_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`citizenapp`
- 消费者：
  - `citizenchain/runtime/transaction/multisig-transfer`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- CI 同步：
  - `.github/workflows/citizenwallet-ci.yml` 必须从 `MultisigTransfer` / `multisig-transfer` 同步 `citizenwallet/lib/signer/pallet_registry.dart`
- 字段：
  - `propose_transfer(19.0)`：`org`、`account_id`、`beneficiary`、`amount`、`remark`
  - `propose_safety_fund_transfer(19.1)`：`beneficiary`、`amount`、`remark`
  - `propose_sweep_to_main(19.2)`：`account_id`、`amount`
- 编码：
  - SCALE call data
  - pallet index：`19`
  - call index：`0 / 1 / 2`
- 签名/验签规则：
  - 业务提案创建由对应管理员签名
  - 投票不走本 pallet，统一走 `P-TX-003`
  - `QR_V1 / k=1` 必须使用 `a + payload` 解码展示 `institution / beneficiary / amount_yuan / remark`，禁止 node 在 QR 中塞展示字段
- 禁止兼容：`call_index=3 / 4 / 5` 留洞不复用
- 禁止事项：
  - 禁止恢复 `execute_transfer / execute_safety_fund / execute_sweep` wrapper
  - 禁止把 `account_id` 注释成当前 `InstitutionPalletId`
  - 多签转账业务唯一归口 `citizenapp/lib/transaction/multisig-transfer/`(公私个共用);禁止在 `citizenapp/lib/citizen/institution/`(机构管理只读)、`citizenchain/node/src/governance/` 或 `citizenchain/node/src/transaction/offchain_transaction/`(链下结算)中另实现多签转账业务
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `cargo test --manifest-path citizenchain/runtime/transaction/multisig-transfer/Cargo.toml`

### P-TX-006：PersonalManage proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/private/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/private/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：
  - `citizenapp/lib/transaction/personal-manage/personal_manage_service.dart`
- 消费者：
  - `citizenchain/runtime/private/personal-manage`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  - `propose_create(7.0)`：`account_name`、`admins`、`regular_threshold`、`amount`
  - `propose_close(7.1)`：`account`、`beneficiary`
- 编码：
  - SCALE call data
  - pallet index：`7`
  - call index：`0 / 1`
- ProposalData：
  - `MODULE_TAG = b"per-mgmt"`
  - `ACTION_CREATE = 0`：`account`、`proposer`、`amount`、`fee`
  - `ACTION_CLOSE = 1`：`account`、`beneficiary`、`proposer`
- 签名/验签规则：
  - 个人多签独立使用 `PersonalManage(7)` 与 `MODULE_TAG = b"per-mgmt"`
  - 投票统一走 `P-TX-003`
- 禁止兼容：不兼容旧 `OrganizationManage(17).propose_create_personal`，不兼容缺少 `regular_threshold` 的旧 `PersonalManage(7).propose_create`
- 禁止事项：
  - 禁止恢复 `OrganizationManage(17).call_index=3`
  - 禁止混用机构多签和个人多签 action 编号
  - 禁止 CitizenApp / CitizenWallet 保留旧个人多签创建交易载荷解析分支
- 必跑测试：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `flutter test test/governance/personal-manage/personal_manage_service_test.dart test/governance/personal-manage/personal_manage_storage_codec_test.dart`

### P-TX-007：AdminsChange.propose_admin_set_change

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/admins/{personal-admins,public-admins,private-admins}/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
  - `memory/05-modules/citizenapp/admins-change/ADMINS_CHANGE_CITIZENAPP_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart`
- 消费者：
  - `citizenchain/runtime/admins/{personal-admins,public-admins,private-admins}`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `institution_code`
  2. `account_id`
  3. `admins`（A2 起:**机构(public/private)= `Vec<AdminProfile>`**;个人多签 personal = `Vec<AccountId32>`,不带 profile）
  4. `new_threshold`
- 编码：
  - SCALE call data
  - pallet index：个人多签 `7`，公权机构与固定治理机构 `29`，私权机构 `30`
  - call index：个人多签 `3`，公权/私权 `0`，联邦注册局省级组 `2`
  - 前两个字节按 `AdminAccount.kind` 和主体类型选择，不再固定为 `[0x0c, 0x00]`
  - 个人多签布局：`institution_code:[u8;4] + account_id:[u8;32] + Compact<Vec<AccountId32>> + new_threshold:u32_le`
  - 机构布局：`institution_code:[u8;4] + account_id:[u8;32] + Compact<Vec<AdminProfile>> + new_threshold:u32_le`；`AdminProfile = account:[u8;32] + admin_cid_number:Compact<Vec<u8>>(≤32) + name:Compact<Vec<u8>>(≤128) + admin_role:Compact<Vec<u8>>(≤128) + term_start:u32_le + term_end:u32_le + source:u8`(0..=4=创世/注册局/内部投票/互选/普选)。account_id 为 `AccountId32`=32 字节裸(onchina `institution_call.rs::encode_admin_set_call` 跨真类型对拍锁定;旧文档误记 48)
  - **关联调用 `PublicAdmins.propose_federal_registry_province_admin_set_change`(pallet 29 / call 2,前缀 `[0x1d,0x02]`)**:联邦注册局省级 5 人组管理员集合更换,布局为 `province_code + Compact<Vec<AdminProfile>> + threshold`。
- 签名/验签规则：
  - `new_threshold` 是管理员更换通过后写入投票引擎的目标动态阈值。
  - 内置治理机构只允许固定制度阈值，App 不展示阈值输入框。
  - 个人多签和机构账户阈值必须满足 `threshold * 2 > admins_len && threshold <= admins_len`。
  - 非法人机构码不能决定 public/private；必须由 CID 注册归属或链上 `AdminAccount.kind` 显式路由到 `PublicAdmins` 或 `PrivateAdmins`。
- 禁止兼容：不兼容缺少 `new_threshold` 的旧载荷。
- 禁止事项：
  - 禁止 CitizenApp 继续生成旧 `[org:u8, account_id, admins]` 载荷。
  - 禁止 CitizenWallet 公民钱包解码旧载荷或忽略尾部多余字节。
  - 禁止在 CitizenApp / CitizenWallet 内实现投票、计票或通过判定。
- 必跑测试：
  - `citizenapp/test/governance/admins-change/admins_change_codec_test.dart`
  - `citizenwallet/test/signer/payload_decoder_test.dart`

### P-STORAGE-001：Admins.AdminAccounts

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/admins/{personal-admins,public-admins,private-admins}/src/lib.rs` + `citizenchain/runtime/admins/admin-primitives/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
  - `memory/05-modules/citizenapp/admins-change/ADMINS_CHANGE_CITIZENAPP_TECHNICAL.md`
- 生产者：各管理员 pallet 生命周期接口与机构/个人创建流程
- 消费者：
  - `citizenchain/runtime`
  - `citizenchain/node`
  - `citizenapp/lib/citizen/proposal/admins-change/services/admin_account_service.dart`
  - `citizenapp/lib/citizen/shared/admin_account_storage_codec.dart`
- 字段：
  - key：`account_id`（机构=main_account=derive(cid_number,主账户);A2 不改键,main_account 即机构身份的确定性像）
  - value：`institution_code`、`kind`、`admins`、`creator`、`created_at`、`updated_at`、`status`
  - `admins`（A2 起）：**机构 public/private = `BoundedVec<AdminProfile>`**(每人 account+admin_cid_number+name+admin_role+term_start+term_end+source);**personal = `BoundedVec<AccountId32>`**(不带 profile)。`AdminAccountQuery::active_account_admins` 仍出 `Vec<AccountId>`(抽 `.account`)→投票/多签/阈值零改;`active_account_admin_profiles` 出完整资料供展示。固定治理机构 profile 由创世写入,source=Genesis。
- 编码：
  - storage key：`twox128(pallet_name) ++ twox128("AdminAccounts") ++ blake2_128_concat(account_id)`
  - `pallet_name` 按 `AdminAccount.kind` 选择：`PublicAdmins / PrivateAdmins / PersonalAdmins`
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自链上授权流程
- 禁止兼容：不兼容旧 `AdminsChange::Subjects / AdminsChange::Institutions` 当前路径
- 禁止事项：
  - 禁止恢复 `AdminsChange` 单 pallet 当前真源叙述
  - 禁止只凭 `UNIN/SFGT/SFGP` 自动选择 `PrivateAdmins`
- 必跑测试：
  - admins-change 单测
  - CitizenApp 多签发现相关测试

### P-STORAGE-002：PublicManage/PrivateManage.InstitutionAccounts

- 状态：当前(机构生命周期已拆 PublicManage(idx32)/PrivateManage(idx33),storage 名不变但前缀随 pallet 名变;取代旧 `OrganizationManage`)
- 类型：storage 契约
- 唯一真源：
  - `citizenchain/runtime/entity/public-manage/src/lib.rs`(公权机构)
  - `citizenchain/runtime/entity/private-manage/src/lib.rs`(私权机构)
- 详细文档：
  - `memory/05-modules/citizenchain/node/offchain-transaction/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`public-manage` / `private-manage`
- 消费者：
  - `citizenchain/node/src/transaction/offchain_transaction/institution_read/chain.rs`(按机构码路由 PublicManage/PrivateManage 前缀)
  - `citizenapp` 机构读共享核心 storage codec(C 阶段三分后,按机构码路由前缀)
- 字段：
  - key1：`cid_number`
  - key2：`account_name`
  - value：机构账户信息，以 runtime 类型为准
- 同 pallet 的 `Institutions[cid_number] → InstitutionInfo`（2026-07-04 创世到市 + 镇级运行期注册）：
  - value 字段仅 `cid_full_name`、`cid_short_name`、`town_code`、`institution_code`、`created_at`、`status`（6 项）
  - 已删 `main_account`/`fee_account`/`admins`/`admins_len`/`threshold`/`creator`/`account_count`：主/费账户由派生且在 InstitutionAccounts;管理员真源 admins 模块;阈值真源 internal-vote
  - 消费方镜像须按机构码路由 PublicManage/PrivateManage 前缀(node 已切;citizenapp 待 C 阶段)
- 编码：
  - double map storage key(前缀 = `twox_128(PublicManage|PrivateManage)` ++ `twox_128(InstitutionAccounts|Institutions)`)
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自机构创建和账户治理流程
- 禁止兼容：不兼容旧 `Accounts` mirror、不兼容旧 `OrganizationManage` 前缀
- 禁止事项：
  - 禁止活跃代码继续读取已删的 `OrganizationManage::Institutions/InstitutionAccounts`
  - 禁止把机构账户当个人多签账户读取
- 必跑测试：
  - `cargo check -p node`(institution_read 前缀路由)
  - `public-manage` / `private-manage` 单测

### P-STORAGE-003：PersonalManage.PersonalAccounts

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/private/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/private/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
- 生产者：`personal-manage`
- 消费者：
  - `citizenapp/lib/transaction/personal-manage/personal_manage_storage_codec.dart`
  - `citizenapp/lib/transaction/personal-manage/personal_manage_service.dart`
- 字段：
  - key：`personal_account`
  - value：`Account { creator, account_name, created_at, status }`
- 编码：
  - storage map key
  - value：SCALE
- 签名/验签规则：storage 本身不签名；创建和关闭由 `PersonalManage` 提案流程约束
- 禁止兼容：不兼容旧 `OrganizationManage` 个人多签路径
- 禁止事项：
  - 禁止恢复 `OrganizationManage(17).propose_create_personal`
  - 禁止恢复已删除的个人多签反向索引 storage
  - 禁止把个人多签查询落回机构账户 storage
- 必跑测试：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`
  - `flutter test test/governance/personal-manage/personal_manage_service_test.dart test/governance/personal-manage/personal_manage_storage_codec_test.dart`

### P-STORAGE-004：Account-level internal admin account

- 状态：当前（已按分类管理员 pallet 落地）
- 类型：storage 契约 / subject id 契约
- 唯一真源：`memory/04-decisions/ADR-015-account-admin-internal-vote.md`
- 详细文档：
  - `memory/04-decisions/ADR-015-account-admin-internal-vote.md`
  - `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
  - `memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
- 生产者：
  - `citizenchain/runtime/admins/admin-primitives`
  - `citizenchain/runtime/admins/{public-admins,private-admins,personal-admins}`
  - `citizenchain/runtime/entity/personal-manage`
  - `citizenchain/runtime/entity/public-manage`
  - `citizenchain/runtime/entity/private-manage`
- 消费者：
  - `citizenchain/runtime/votingengine/internal-vote`
  - `citizenchain/runtime/transaction/*`
  - `citizenapp`
  - `citizenwallet`
- 字段：
  - `account_id`
  - `account_id`
  - `admins`
  - `admins_len`
  - `threshold`
  - `status`
- 生命周期事件：
  - `AdminAccountPendingCreated { subject, org, kind, creator, admins_len, threshold }`
  - `AdminAccountActivated { subject, org }`
  - `AdminAccountPendingRemoved { subject, org }`
  - `AdminAccountClosed { subject, org }`
- 编码：
  - 治理机构账户继续映射到既有 `AdminAccountKind::Builtin`
  - 注册个人账户继续映射到既有 `AdminAccountKind::PersonalAccount`
  - 注册机构账户使用账户级 `AdminAccountKind::InstitutionAccount = 0x05`，payload 为账户 `AccountId` 前 32 字节并右填零
  - `注册机构归属关系 = 0x02` 保留为 CID 机构归属/检索 ID，不作为新增账户级管理员主体
- 签名/验签规则：
  - 一人一票一笔交易，投票资格由创建提案时锁定的账户级管理员快照决定
  - 注册创建和注销关闭阈值为全员
  - 普通动态账户提案阈值由管理员数量派生
  - Pending 主体清理必须命中既有 Pending 主体，不存在时返回 `InvalidInstitution`
- 禁止兼容：开发期彻底切换，不保留机构级管理员旧分支
- 禁止事项：
  - 禁止省储行永久质押账户进入内部投票
  - 禁止注册机构账户继续复用机构级管理员池
  - 禁止动态账户由用户自由输入阈值
  - 禁止把管理员增加、删除、更换、改阈值拆成四套提案
- 必跑测试：
  - `cargo test -p admins-change --lib`
  - `cargo test -p primitives --lib`
  - `cargo test -p internal-vote --lib`
  - `cargo test -p personal-manage --lib`
  - `cargo test -p public-manage --lib`
  - `cargo test -p private-manage --lib`

### P-TX-008：GmbPqcAuth bootstrap（未绑定账户首次无感绑定+执行）

- 状态：草案（ADR-022，待实现）
- 类型：交易载荷格式（General Transaction + `GmbPqcAuth` 扩展 `extra`）
- 唯一真源：`GmbPqcAuth` TransactionExtension + `account-keys` pallet（待实现）
- 详细文档：`memory/04-decisions/ADR-022-unified-pqc-crypto.md`
- 生产者：`citizenapp`、`citizenwallet`　消费者：`GmbPqcAuth` 扩展 + `account-keys`、`citizenwallet` decoder
- 字段（扩展 extra）：`account`、`pqc_pubkey`(ML-DSA-65,~1952B)、`alg`(0x02)、`key_version`、`nonce`、`sr25519_bootstrap_signature`、`ml_dsa_signature`（业务 call 是普通 General Transaction call）
- 编码：General Transaction + `GmbPqcAuth` 扩展 `extra`（**非 pallet call**）
- payload `GMB_PQC_BOOTSTRAP_V1`（域标签 `DOMAIN_BOOTSTRAP=b"GMB_PQC_BOOTSTRAP_MLDSA65_V1"` 进 preimage，字段集与 GMB_PQC_TX_V1 对齐）：`genesis_hash`、`spec_version`、`transaction_version`、`account`、`pqc_pubkey_hash`、`key_version`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`following_extensions_hash`
- 规则（验序钉死，hash 全 `blake2_256`）：
  - ① `blake2_256(body.pqc_pubkey) == payload.pqc_pubkey_hash`
  - ② `sr25519_bootstrap_signature = sr25519_sign(blake2_256(DOMAIN_BOOTSTRAP ++ SCALE(genesis_hash,spec_version,transaction_version,account,pqc_pubkey_hash,key_version,nonce,call_hash,following_extensions_hash)))`——**sr25519 必须覆盖 pqc_pubkey_hash**（防 body 公钥替换），`account`=sr25519 公钥派生的当前 AccountId
  - ③ `ml_dsa_signature` 验交易 payload + `call_hash==blake2_256(body.call)`，且**反向覆盖 `blake2_256(sr25519_bootstrap_signature)`**（双向交叉绑定）
  - 三验过 → origin 转 `Signed(account)` → nonce/扣费/业务 dispatch；**绑定写 `AccountPqcKey` 在 `post_dispatch`**
  - 失败语义：绑定在 post_dispatch（nonce/扣费已跑），**内层 call 失败绑定仍保留、内层失败照常收费**；🔴 **post_dispatch 绝不返回 Err**（否则作废整区块），冲突（已绑定不同值）判定前移 validate 拒
  - 🔴 bootstrap 账户须 providers/sufficients>0（否则 CheckNonce 以 Payment 先拒）；body 长度上限硬校验 + 未绑定按 (account,source) 限速
  - 已绑定账户拒绝再次 sr25519 覆盖（first-bind-wins）
  - extrinsic body 携带完整 ML-DSA 公钥（~1952B）+ sr25519 bootstrap 签名（64B）+ ML-DSA 交易签名（~3309B）
- 禁止：扩 `MultiSignature`；用 PQC 公钥/hash 派生新 AccountId；CID 托管助记词/私钥
- 必跑测试：bootstrap 双签成功/拒绝、已绑定拒覆盖、写表+派发原子性

### P-TX-009：GmbPqcAuth PQC 交易（已绑定账户）

- 状态：草案（ADR-022，待实现）
- 类型：交易载荷格式（General Transaction + `GmbPqcAuth` 扩展 `extra`）
- 唯一真源：`GmbPqcAuth` TransactionExtension（待实现）
- 详细文档：`memory/04-decisions/ADR-022-unified-pqc-crypto.md`
- 生产者：`citizenapp`、`citizenwallet`　消费者：`GmbPqcAuth` 扩展、`citizenwallet` decoder
- 字段（扩展 extra）：`account`、`sig`(ML-DSA-65；公钥由链端按 account 从 `AccountPqcKey` 读，交易不带公钥)、`auth_mode`、`key_version`（业务 call 是普通 General Transaction call）
- 编码：General Transaction + `GmbPqcAuth` 扩展 `extra`（**非 pallet call**）
- payload `GMB_PQC_TX_V1`（域标签 `DOMAIN_TX=b"GMB_PQC_TX_MLDSA65_V1"`（含算法标识）进 preimage）：`genesis_hash`、`spec_version`、`transaction_version`、`account`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`key_version`、`following_extensions_hash`（`ss58_format` 为纯展示字段，链上无对应 implicit，不参与一致性比对）
- 规则（路线 A 定稿）：
  - `GmbPqcAuth` 读 `AccountPqcKey[account].pubkey` 验 ML-DSA 签名 + `call_hash==blake2_256(body.call)` + `alg==AccountPqcKey.alg`（防降级） → **把 origin 转 `Signed(account)`** → 后续 `CheckNonce`/`ChargeTransactionPayment` 走系统标准逻辑
  - 🔴 `following_extensions_hash` = SDK `inherited_implication` **精确递归编码**（`ImplicationParts{base,explicit,implicit}`，非扁平拼接；嵌套 tuple 下与链端 `inherited_implication.encode()` 逐字节对拍 `mod.rs:712-869`），覆盖 CheckGenesis/CheckMortality(immortal→genesis)/CheckNonce/ChargeTransactionPayment/CheckMetadataHash(Disabled→None)/WeightReclaim 等 implicit
  - 🔴 **tuple 12 上限**：嵌套 `(GmbPqcAuth, AuthorizeCall)` 占第一项槽位，不加第 13 项；`GmbPqcAuth` 兼管"已绑定拒 sr25519"；`extra=None` 透明放行原 origin 给 AuthorizeCall
  - txpool `provides=(account,nonce)` 由 CheckNonce 自动产生（GmbPqcAuth 不重复设）；**era 默认 immortal**（CheckMortality.implicit 仍 genesis，纳入 hash）
  - `weight()` 按 extra 变体路由 card1 benchmark 常量（禁读 state）；PqcPolicy 缺失 fail-open（不冻结全链）；`validate` 轻量无副作用
- 禁止：跳过 `nonce`/`genesis_hash` 域隔离；decoder 解码后仍有剩余字节
- 必跑测试：authorize 成功/拒绝、nonce 防重放、`citizenwallet` decoder

### P-STORAGE-005：account-keys.AccountPqcKey

- 状态：草案（ADR-022，待实现）
- 类型：storage 契约　唯一真源：account-keys pallet（待实现）
- 详细文档：`memory/04-decisions/ADR-022-unified-pqc-crypto.md`
- 生产者：`GmbPqcAuth`（bootstrap `post_dispatch` 写）+ `account-keys`（轮换 call 写）　消费者：`GmbPqcAuth`（PQC 交易验签读）、`offchain-transaction`（批签取公钥）
- **pallet_index=27**（契约真源；当前 runtime 最高 idx=26，27 空闲）
- 字段：
  - key：`AccountId`（sr25519 锚点）
  - value：`alg:u8`(0x02)、`key_version:u32`、`pubkey:BoundedVec<u8,ConstU32<2048>>`(完整 ML-DSA-65 公钥 ~1952B)、`bound_at:BlockNumber`（**删 bootstrap_mode**）
  - 另有 `PqcPolicy` storage（phase/bootstrap_deadline/reject_sr25519_when_bound/allow_bootstrap_unbound，安全默认 phase=B/reject=false/allow=true/deadline=None）
- 编码：SCALE，`StorageMap<Blake2_128Concat, AccountId, AccountPqcKeyRecord>`
- 规则：存完整公钥（非 hash）；first-bind-wins（冲突在 validate 拒）；**轮换双签**：当前 PQC 私钥授权 + 新私钥对 `(新公钥+key_version+account+genesis)` 自签 PoP，两签过才 `key_version++`；**账户不派生 ML-KEM**（决策3）；绑定后无 sr25519 回退恢复（决策1/2）
- 禁止：存私钥；用 PQC 公钥（或其 hash）当 AccountId；给账户加签名算法 state 字段（阶段策略在链层 A/B/C/D 治理，不做 per-account 状态切换）
- 必跑测试：`account-keys` pallet 单测、`offchain-transaction` 批签集成测试

## 6. 登记维护要求

新增或修改协议时，必须在本文件按编号登记；无法确认字段时必须先向用户报告，不得把未确认字段写成当前协议。
