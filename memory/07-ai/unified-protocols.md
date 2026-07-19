# GMB 统一协议文件

## 0.1 机构、岗位与管理员当前契约（2026-07-17）

runtime 已完成机构信息、岗位任职与机构 admins 钱包集合拆分；机构类 `AdminProfile` 内嵌布局和公权/私权机构管理员集合变更 call 已删除，不得恢复兼容分支。

- 机构法定代表人任免生效后，`InstitutionInfo` 必须公开 `legal_representative_name`、`legal_representative_cid_number`、`legal_representative_account`；创世时没有真实任免资料的机构保持“尚未任命”，不得伪造法定代表人或回退到 `admins[0]`。
- 机构管理员集合 `admins` 的目标值为 `InstitutionAdmin { admin_name, admin_account }` 列表；姓名只展示，账户是唯一授权字段，不内嵌公民 CID、岗位、任期和来源。
- 机构岗位 `InstitutionRole` 归 entity，只保存岗位身份和制度事实，不设置 `role_permissions` 或通用权限表；具体职责和操作权限由对应业务模块依据“机构 + 有效岗位 + 业务动作”的硬规则判定。
- 机构管理员任职 `InstitutionAdminAssignment` 归 entity，字段为 `cid_number`、`admin_account`、`role_code`、`term_start`、`term_end`、`assignment_source`、`assignment_source_ref`、`assignment_status`。
- `assignment_source` 只允许 `Genesis`、`Registry`、`PopularElection`、`MutualElection`、`NominationAppointment`、`InstitutionGovernance`；由创世、注册局、对应投票/选举引擎结果或本机构内部治理结果写入。本机构内部治理 call 只能写 `InstitutionGovernance` 来源，不能伪装成普选、互选或任命结果。
- 所有已完成业务统一通过 `InstitutionGovernanceResult` 交给 entity；一个结果可同时包含动态岗位定义变化、多个岗位的完整目标任职集合，以及可选的法定代表人三字段整体设置或清空。
- 每条目标任职独立保存管理员钱包、`term_start`、`term_end`、`assignment_source`、`assignment_source_ref` 和状态；未出现在结果中的岗位保持不变，出现的岗位整体替换目标任职集合。
- `term_start`、`term_end` 统一为自纪元起 `u32` 天；entity 校验任职账户属于既有 admins 后，只提交岗位、任职和法定代表人，绝不反向生成或覆盖管理员集合。
- 每个机构默认且唯一存在 `LR / 法定代表人` 岗位；首次登记允许空缺，不得从管理员首位推导法定代表人。
- 五类固定创世机构只允许依法轮换岗位任职，不允许运行期业务修改固定岗位定义或法定席位；动态机构允许新增、改名、停用岗位，停用岗位必须在同一结果中清空任职。
- 动态机构沿用既有 Active 多签阈值；固定治理机构使用代码级固定阈值。任职结果不得携带或修改阈值。
- 任职不保存 `creator`；来源由 `assignment_source + assignment_source_ref` 唯一表达。
- 一个公民 `cid_number` 只能绑定一个钱包账户，一个钱包账户也只能绑定一个公民 CID。
- 一个管理员钱包账户可在多个机构任职；一个机构可有多个管理员。
- 个人多签及 `personal-admins` 不使用本机构岗位契约。

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
- 生产者：OnChina 登录、扫码登录轮询、`/api/v1/admin/auth/check`、`/api/v1/admin/auth/identify`、`/api/v1/admin/own-institution`、平台会员模块
- 消费者：OnChina 前端 `AuthContext` 和 `workspace/WorkspaceRouter`
- 字段：
  - 登录态统一携带 `institution_cid_number`、`institution_code`、`cid_number`、`cid_full_name`、`cid_short_name`、`admin_name`、`capabilities`
  - `workspace`
  - `workspace.workspace_kind`: `registry` / `private` / `judicial` / `legislation` / `public` / `unincorporated`
  - `workspace.workspace_title`
  - `workspace.workspace_modules[]`: 当前实例被授权的模块；平台技术公司可包含 `platform_membership_price`
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
- 平台会员 API：`GET /api/v1/membership/platform-prices` 读取同一 finalized 区块的 `PlatformCidNumber` 与 `PlatformPrice`；`POST /api/v1/membership/platform-prices/propose` 生成一次 `propose_set_platform_price` 签名请求；`POST /api/v1/admin/chain/submit` 是所有 OnChina 链交易唯一响应二维码回扫提交入口。
- 一次签名流程：OnChina 展示请求二维码 → CitizenWallet 只签名一次并展示响应二维码 → OnChina 回扫、验签、dry-run、提交并等待进块。prepare 与 submit 都重新核对节点绑定、准确机构 CID 和链上 active `admins`。
- 禁止兼容：不得恢复“注册局根 UI + 非注册局只塞一个 tab”的旧口径;不得新增第二套 `dashboard` / `console` / `tenant` 同义字段。
- 禁止事项：
  - 禁止把 `workspace` 作为管理员授权真源。
  - 禁止前端根据本地硬编码越过后端 `capabilities` 显示受限操作。
  - 禁止非注册局机构复用注册局业务 UI。
  - 禁止前端或机构码把平台会员模块授权给非 `PlatformCidNumber` 机构。
  - 禁止业务模块另建链提交 URL、直接钱包广播或第二套签名响应处理。
- 必跑测试：
  - `cargo check --manifest-path citizenchain/onchina/Cargo.toml`
  - `npm --prefix citizenchain/onchina/frontend run build`
  - 真实本地 OnChina 服务的 `/api/v1/admin/auth/check` 和真实页面验收

### P-QR-001：QR_V1

- 状态：当前
- 类型：扫码协议
- 唯一真源：
  - 当前协议冻结文档：`memory/01-architecture/qr/qr-protocol-spec.md`
  - action registry 代码真源：`citizenchain/crates/qr-protocol/registry/*`
- 详细文档：
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenapp`、`citizenchain/node`、`citizenchain/onchina`
- 消费者：`citizenwallet`、`citizenapp`、`citizenchain/onchina`
- 字段：顶层只允许 `p/k/i/e/b`;`k=1` 的 `b.d` 是 `review_payload`,除 Runtime 升级 hash-only 外必须完整可解码和可中文展示;具体字段以 `qr-protocol-spec.md` 为准
- 编码：紧凑 JSON envelope
- 签名/验签规则：按 `k` 和 `b.a + b.d(review_payload)` 执行;普通链交易由扫码端按 Substrate 规则从 `review_payload` 计算 `signing_bytes`;签名响应只带 `u/s`
- 平台调价动作：`propose_set_platform_price`，pallet/call 为 `SquarePost/propose_set_platform_price`；必审字段为 `actor_cid_number`、`membership_level`、`new_price_fen`。CitizenWallet 必须严格中文解码并输出标准签名响应二维码，OnChina 负责回扫提交。
- 禁止兼容：开发期不做旧协议兼容
- 禁止事项：
  - 禁止新增 `QR_V2`
  - 禁止新增第二套扫码协议字符串
  - 禁止把某个 `b.d` 的交易载荷格式称为新扫码协议
  - 禁止各端手写第二套 action registry、中文动作名、字段中文名或签名判定分支
  - 禁止普通链交易把 32B hash/signing bytes 放进 `b.d` 冒充可审阅 payload
  - 禁止扫码签名出现正常/拒绝以外的第三状态
  - 禁止未中文翻译的 action、字段或枚举值继续签名
- 必跑测试：QR fixture、citizenwallet/citizenapp QR 解析测试

### P-CHAIN-CITIZENAPP-001：CitizenApp 公权机构 finalized 快照契约

- 状态：当前
- 类型：链快照契约
- 唯一真源：`PublicManage::Institutions` 与 `PublicManage::InstitutionAccounts`
- 实现：
  - 生成器：`citizenapp/tools/generate_public_institution_bundle.mjs`
  - 移动端载入：`citizenapp/lib/citizen/public/data/public_institution_bundle_loader.dart`
- 生产者：连接目标链节点的发布期快照生成器
- 消费者：CitizenApp 公权机构目录、Isar 本地缓存和快照载入服务
- 链读取：`state_getKeysPaged` 与 `state_queryStorageAt` 必须钉在同一个 finalized 块哈希。
- 机构分片字段：`cid_number`、`cid_full_name`、`cid_short_name`、`status`、`province_code`、`city_code`、`town_code`、`institution_code`、`account_count`、`custom_account_names`、`created_at_block`。
- manifest 字段：`schema_version`、`chain_id`、`snapshot_block_number`、`snapshot_block_hash`、`genesis_hash`、`state_root`、`chainspec_hash`、`public_institution_root`、`version`、`shard_hashes`、`provinces`。
- 编码：快照和分片均为 UTF-8 JSON，业务字段统一 snake_case，分片和机构根使用 sha256 hex。
- 权限边界：快照和 Isar 只用于目录查询；身份、绑定、付款和权限操作必须精确读取当前 finalized storage。
- 禁止兼容：机构目录只允许 finalized 链快照和精确链读取，不得建立第二套查询真源。
- 禁止事项：CitizenApp 运行时不得全量扫描 49,593 条机构；行政区名称仍由行政区字典按 code join。
- 必跑测试：`flutter test test/citizen/public/public_institution_bundle_loader_test.dart test/citizen/public/public_institution_dto_test.dart`

### P-API-CITIZENAPP-002：CitizenApp Square Worker / R2 契约

- 状态：当前（广场媒体、feed、资料、通讯录密文和会员镜像共用本契约；平台与创作者订阅付款统一为公民币链上交易，Worker 只保存 finalized 镜像和创作者展示资料，不承载支付、续费或价格真源）
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
  - `POST /v1/square/auth/session` 请求：`owner_account`、`challenge_id`、`signature`；响应：`session_token`、`owner_account`、`expires_at`。会话只验证已登记 P-256 设备子钥对挑战的签名及钱包归属，不读取 `System.Account`，不要求链上账户已经存在，也不以余额或存在性存款作为 Cloudflare 登录门禁。
  - `GET /v1/square/membership` 请求：Bearer `session_token`；响应：`plans[]`、`membership`、`subscription_active`、`active`。`plans[]` 只返回权益配额和平台档位标识，不返回价格；平台价格由 CitizenApp 直接读取 finalized `PlatformPrice`。`membership` 镜像字段至少包含 `membership_level`、`subscription_status`、`last_charged_price_fen`、`last_charged_at`、`paid_until`、`pending_plan`、`updated_at`。
  - `POST /v1/square/membership/confirm` 请求：Bearer `session_token`，`tx_hash`、`block_hash`、`signed_extrinsic_hex`、`action`(`subscribe`/`cancel`/`change`)，订阅或换档时另带 `membership_level`；响应：finalized 链上平台订阅镜像。owner 只从 session 派生，Worker 必须复算交易哈希、严格解码签名者与调用参数、确认完整 extrinsic 位于指定 finalized 主链区块，并读取同一区块订阅状态；禁止采信请求自报价格、状态或期限。该镜像路由只用 Bearer，不生成第二次账户或设备签名。
  - `GET /v1/square/creator/plan` 与 `GET /v1/square/creator/plan/{creator_account}`：返回创作者展示资料和链上 `tier_id` 引用；名称、说明、权益文案不进入链上，扣款价格不由 D1 返回为真源。
  - `POST /v1/square/creator/plan`：请求 Bearer `session_token`、`tx_hash`、`block_hash`、`signed_extrinsic_hex`、`tiers[]`；保存一次 `set_creator_plans` 签名交易证明和同一区块 `CreatorPlans` 对应的展示名称。请求中的价格只用于逐字段核对已签 call 与 finalized storage，不能覆盖链上扣款真源；创作者账户只从 session 派生，且 Worker 必须在同一区块确认该账户拥有有效平台订阅。完全相同的 HTTP 重试幂等，不产生第二次签名。
  - `POST /v1/square/creator/subscription/confirm` 请求：Bearer `session_token`，`tx_hash`、`block_hash`、`signed_extrinsic_hex`、`action`(`subscribe`/`cancel`/`change`)、`creator_account`，订阅或换档时另带 `tier_id`、`billing_period`；响应：finalized 链上创作者订阅镜像。subscriber 只从 session 派生，Worker 必须核对完整签名交易和同一区块 `Subscriptions[(subscriber, Creator(creator_account))]`。
  - 平台和创作者订阅、取消、换套餐以及创作者修改付款套餐均由 CitizenApp 直接提交 P-TX-014 链交易；Worker 不提供支付 checkout、预付、webhook、取消扣款或续费接口。
  - `POST /v1/square/uploads/prepare` 请求：Bearer `session_token`，`post_category`、`content_format`(`normal`/`article`)、`title_length`、`text_length`、`media_items[]`、`manifest_hash`；响应：`upload_id`、`post_id`、`storage_receipt_id`、`expires_at`、`estimated_bytes`、`manifest_object_key`、`manifest_upload_url`、`media_items[]`。Worker 按有效会员强制校验内容权益和统一资源硬上限，并用 D1 单条条件写原子预留活动上传数、订阅周期图片数和视频秒数；图片只返回同域 Worker 上传地址，视频统一签发绑定 `Upload-Length` 与 `maxDurationSeconds` 的 Stream TUS 地址。
  - `media_items[]` 响应字段：`media_kind`、`content_type`、`byte_size`、`provider`(`cloudflare_images`/`cloudflare_stream`)、`provider_asset_id`、`upload_method`(`worker`/`tus`)、`resource_key`、`asset_state`、`upload_url`。
  - `PUT /v1/square/uploads/manifest?upload_id=...`：Bearer + P-256 设备请求签名；Worker 有界读取最多 256KiB，复核真实 sha256 等于 prepare 的 `manifest_hash` 后写 R2。
  - `PUT /v1/square/uploads/media?upload_id=...&media_index=...`：仅图片；Bearer + P-256 设备请求签名；Worker 校验实际字节、MIME、文件头、尺寸和申报大小后，以服务端 Token 写 Cloudflare Images。视频调用本接口一律拒绝。
  - `POST /v1/square/uploads/complete` 请求：Bearer `session_token`，`upload_id`、`manifest_hash`、`content_hash`；响应：`upload_id`、`post_id`、`content_hash`、`storage_receipt_id`、`storage_state`(`completed`/`processing`)。Worker 读取 R2 manifest 并复核 manifest hash、owner、`post_category`、`content_format`、标题 / 正文长度、媒体数量和 `square_media_assets` 一致性；返回的 `storage_receipt_id` 必须等于 prepare 阶段预生成的回执；视频上传已接收但 Stream 尚未 ready 时返回 `processing`。
  - `POST /v1/square/uploads/stream/webhook` 请求：Cloudflare Stream webhook 原始 JSON + `Webhook-Signature`；响应：`action`、`provider_asset_id`、`asset_state`。本接口不需要 Bearer，必须用 `STREAM_HOOK_SECRET` 校验签名。
  - `storage_until` 当前不由上传完成接口返回；CitizenApp 发布交易使用 `GET /v1/square/membership` 的 finalized 镜像字段 `membership.paid_until` 作为链上 `storage_until`，且 Worker 在准备上传前已经用未陈旧链时钟执行平台订阅门禁。
  - `POST /v1/square/posts/confirm` 请求：Bearer `session_token`，`post_id`、`block_hash`、可选 `tx_hash`；响应：`post`
  - `DELETE /v1/square/posts/{post_id}` 请求：Bearer `session_token`；响应：`post_id`、`post_state = deleted`、`cleanup{deleted_media_assets,deleted_r2_objects}`。仅作者本人可调用；Worker 删除 Cloudflare Images / Stream provider asset、R2 manifest、D1 媒体索引、上传任务和帖子行，不保留软删残行；链上 `SquarePosts`、发布事件和 0.1 元发布费记录不改写。
  - `GET /v1/square/feed/recommended` 请求：可选 Bearer `session_token`、`limit`；响应：`posts[]`
  - `GET /v1/square/feed/following` 请求：可选 Bearer `session_token`、`limit`；响应：`posts[]`
  - `GET /v1/square/feed/campaign` 请求：可选 Bearer `session_token`、`limit`；响应：`posts[]`
  - `POST /v1/square/follows` 请求：Bearer `session_token`、`followed_account`
  - `DELETE /v1/square/follows/{followed_account}` 请求：Bearer `session_token`
  - `POST /v1/square/signals` 请求：Bearer `session_token`、`post_id`、`signal_type`
  - `GET /v1/square/users/{owner_account}` 请求：可选 Bearer `session_token`；响应：`profile`（`owner_account`、`display_name`、`bio`、`avatar_object_key`、`banner_object_key`、`cid_number`、`is_certified`、`counts{following,followers,posts}`、`is_following`、`updated_at`）。公开可读；带 session 时 `is_following` 反映登录者视角；认证以链上已确认发布携带的 `cid_number` 为真源。
  - `GET /v1/square/users/{owner_account}/posts` 请求：可选 `category`（all/normal/campaign）、`content_format`（all/normal/article）、`limit`、`cursor`；响应：`posts[]`、`next_cursor`（按 `created_at` keyset 游标）。帖子 Tab 传 `content_format=normal` 排除文章，文章 Tab 传 `content_format=article`。
  - `GET /v1/square/users/{owner_account}/follows` 请求：`type`（following/followers）、`limit`、`cursor`；响应：`accounts[{owner_account,created_at}]`、`next_cursor`
  - `GET /v1/square/contacts` 请求：Bearer `session_token` + P-256 设备请求证明，可选 `limit`、`cursor`；响应：`items[{contact_id,ciphertext,nonce,mac,updated_at}]`、`next_cursor`。`owner_account` 只从 session 派生，Worker 不接收联系人账户或联系人名称明文。
  - `PUT /v1/square/contacts/{contact_id}` 请求：Bearer `session_token` + P-256 设备请求证明，`ciphertext`、`nonce`、`mac`、`updated_at`；响应：`contact_id`、`updated_at`、`applied`。`contact_id` 固定为端侧通讯录索引密钥对联系人 `address` 的 HMAC-SHA256 hex，Worker 只校验形状和大小，不持有索引密钥且不能反推出联系人。
  - `DELETE /v1/square/contacts/{contact_id}` 请求：Bearer `session_token` + P-256 设备请求证明；响应：`contact_id`、`deleted`；只能删除当前 session 所属钱包的对应密文记录。
  - `PUT /v1/square/profile` 请求：Bearer `session_token`，可选 `display_name`(≤40)、`bio`(≤160)、`avatar_object_key`、`avatar_content_hash`、`banner_object_key`、`banner_content_hash`（头像/背景 key 只能分别为本人固定 `profile/{owner}/avatar`、`profile/{owner}/banner`）；响应：与 GET `users/{owner_account}` 同构的完整 `profile`。
  - `POST /v1/square/profile/assets/prepare` 请求：Bearer `session_token`，`kind`(`avatar`/`banner`)、`content_type`、`byte_size`、`sha256`；头像最多 512KiB/1024×1024，背景最多 1536KiB/1920×720；响应本人 `object_key`、`content_hash` 与同域 Worker `upload_url`。
  - `PUT /v1/square/profile/assets?object_key=...&byte_size=...&sha256=...`：Bearer + P-256 设备请求签名；Worker 校验真实字节、MIME、文件头、尺寸和 sha256 后覆盖固定 R2 对象键。
  - `GET /v1/square/media/{object_key}`：必须有钱包 Bearer session；只允许本人资料命名空间中的固定头像/背景键且校验已存大小。该只读图片请求不要求 P-256 签名，以支持 `Image.network` 携带 session header 渲染。
- 用户公开资料 R2 契约：`profile/{sanitize(owner_account)}/profile.json`（schema `citizenapp.square.profile.v1`：`owner_account`、`display_name`、`bio`、`avatar_object_key`、`avatar_content_hash`、`banner_object_key`、`banner_content_hash`、`updated_at`）。计数与认证不入 profile.json，响应时由 D1/链上派生。头像/背景固定对象键分别为 `profile/{owner_account}/avatar` 与 `profile/{owner_account}/banner`。
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
  - `profile/{owner_account}/{avatar|banner}`
- R2 manifest 字段（阶段 5 App 端实际生成的规范化内容清单）：
  - `schema`: 固定为 `citizenapp.square.post.v1`
  - `owner_account`
  - `post_category`
  - `content_format`（可选，`normal`/`article`；**仅文章写入**，普通帖不带 → 默认 normal，保持旧 manifest 形状与哈希）
  - `title`（可选，文章标题 10–50 字；普通帖不带）
  - `text`（动态正文 ≤300 字；文章正文按会员计划校验，自由 20000 字，民主 / 投票 / 竞选 30000 字）
  - `media_items[]`（动态最多 9 张图 + 1 个视频；文章 `[0]`=首图，`[1..]`=正文图，自由正文图最多 50 张，民主 / 投票 / 竞选最多 100 张）
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
  - `chain_clock`: 单行 `clock_id=1`、`chain_timestamp`、`finalized_block_number`、`finalized_block_hash`、`observed_at`；只允许更高 finalized 区块推进，旧证明不能刷新观测时间。
  - `square_memberships`: 以钱包 `owner_account` 为主键；`membership_level`、`pending_membership_level`、`started_at`、`last_charged_at`、`last_charged_price_fen`、`paid_until`、`subscription_status`、`finalized_block_number`、`finalized_block_hash`、`verified_at`、`entitlement_lapsed_at`、`last_tx_hash`。
  - `square_creator_tiers`: 复合主键 `(creator_account,tier_id)`；`name`、`tier_order`、三个公历周期的 finalized 价格镜像、`finalized_block_number`、`finalized_block_hash`、`verified_at`、`last_tx_hash`。名称是展示资料，价格镜像只用于证明与展示审计，链上 `CreatorPlans` 始终是扣款真源。
  - `square_creator_subscriptions`: 复合主键 `(subscriber_account,creator_account)`；当前及待生效 `tier_id`/`billing_period`、链上时间、最近扣款价格、状态、finalized 锚点、`verified_at`、`last_tx_hash`。
  - `chain_transaction_confirmations`: 以 `tx_hash` 为主键，保存首次绑定的钱包、区块、extrinsic 序号、动作、规范化 `request_hash`、链时间和确认时间；相同请求幂等，任何改绑拒绝。
  - `square_uploads`: `upload_id`、`post_id`、`owner_account`、`post_category`、`manifest_hash`、`content_hash`、`storage_receipt_id`、`estimated_bytes`、`object_keys_json`、`status`、`created_at`、`completed_at`
  - `square_media_assets`: `upload_id`、`post_id`、`owner_account`、`media_index`、`media_kind`、`provider`、`provider_asset_id`、`upload_method`、`resource_key`、`content_type`、`byte_size`、`asset_state`、`declared_duration_seconds`、`duration_seconds`、`width`、`height`、`error_code`、`created_at`、`updated_at`、`ready_at`、`archive_state`、`archived_at`、`r2_archive_key`
  - `square_posts`: `post_id`、`owner_account`、`cid_number`、`post_category`、`content_format`、`title`、`text`、`content_hash`、`storage_receipt_id`、`chain_block`、`created_at`、`post_state`
  - `square_follows`: `owner_account`、`followed_account`、`created_at`
  - `square_user_signals`: `owner_account`、`post_id`、`signal_type`、`weight`、`created_at`
  - `resource_reservations`: `reservation_id`、`owner_account`、`resource_key`、`period_start`、`period_end`、`byte_size`、`image_count`、`video_seconds`、`expires_at`、`reservation_state`、`created_at`、`used_at`
  - `resource_usage`: `owner_account`、`resource_key`、`period_start`、`period_end`、`byte_size`、`image_count`、`video_seconds`、`updated_at`
  - `resource_totals`: `resource_key`、`byte_size`、`object_count`、`video_seconds`、`updated_at`
- 统一资源限制真源：`citizenapp/cloudflare/src/limits/catalog.ts`。代码硬上限不可由环境变量放宽；所有路由必须先匹配白名单和正文上限，再进入风控/D1；所有 R2、KV、Images、Stream 和推送写入必须持有统一限制校验结果。
  - 资料：头像 512KiB/1024×1024/1 个；背景 1536KiB/1920×720/1 个；profile JSON 16KiB。
  - 广场：manifest 256KiB；标清图 1MiB/最长边 1600；高清图与首图 3MiB/最长边 2560。
  - 视频（三档，ADR-036）：自由会员 40MiB/480p/60 秒；民主会员 1536MiB/1080p/30 分钟；薪火会员 8GiB/1080p/3 小时（资源键 `square_video_spark`）。Stream webhook 必须按实际时长和分辨率二次校验，超限立即删 provider asset 并标记错误。
  - Chat：设备请求 16KiB/账户最多 8 台；KeyPackage 128KiB/设备最多 20 个/最长 7 天；密文请求 256KiB；信令 64KiB；唤醒载荷 1KiB。
  - 外部入口：Stream webhook 64KiB；已签名 extrinsic 64KiB、外层 JSON 132KiB；链 RPC 响应 4MiB。会员没有外部支付 webhook。
- Worker 环境变量：
  - `SQUARE_API_URL`：CitizenApp 编译期 define，用于显式覆盖广场、聊天和链启动清单 Worker API 根地址；默认直连 production Worker，本地调试可显式传 `http://127.0.0.1:8787`。
  - `CHAIN_URL`：Access 保护的链 RPC HTTPS 地址，只允许作为 Cloudflare 远端 Secret，不写入仓库和 CitizenApp。
  - `CHAIN_ID`、`CHAIN_SECRET`：Worker 调用 Access 应用的服务令牌，必须与 URL 成套配置为远端 Secret；当前代码只允许内部固定方法 `state_getStorage`、`author_submitExtrinsic`、`chain_getHeader`、`chain_getBlock`、`chain_getBlockHash`，用于 finalized storage、签名交易中继和 finalized 交易包含证明，不提供通用代理。
  - `CF_ACCOUNT_ID`、`R2_ACCESS_ID`、`R2_SECRET_KEY`、`R2_BUCKET`：只供 Worker 内部把退订视频从 R2 冷归档回灌 Stream；不得签发用户上传 URL。
  - `CF_API_TOKEN`：Worker 校验图片后写 Images、签发 Stream TUS 和管理媒体；必须使用 Cloudflare Secret，不得下发 CitizenApp。
  - `IMAGES_URL`：Cloudflare Images delivery 地址前缀，不含 asset id 和 variant。
  - `STREAM_URL`：Cloudflare Stream 播放地址前缀。
  - `STREAM_HOOK_SECRET`：Stream webhook 签名 secret；必须使用 Cloudflare secret。
  - `MEMBERSHIP_RECONCILE_ENABLED`、`CREATOR_RECONCILE_ENABLED`：平台和创作者镜像对账默认开关；运行期 KV 开关可以收紧或恢复默认，但不能改变链上真源。
  - `MEMBERSHIP_RECONCILE_BATCH`：单轮镜像对账上限，只控制 Worker 读取量，不改变权益规则。
  - `VITE_API_URL`：官网构建时可选 Worker API 根地址；未设置时使用 production Worker 默认地址。
- CitizenApp 本地缓存字段：
  - `SquareDraft`: `owner_account`、`post_category`、`text`、`media_drafts[]`、`draft_state`、`updated_at_millis`、`last_error`、可选 `upload_id/post_id/content_hash/storage_receipt_id/storage_until/tx_hash/block_hash_hex`；当前落地复用 `AppKvEntity`，不新增 Isar schema。
  - `SquareUploadTask`: 当前不落独立 Isar schema；发布中的上传状态由 `SquareDraft` 和 Worker `square_uploads.status` 表达。
  - `SquarePostCache`: `post_id`、`owner_account`、`cid_number`、`post_category`、`content_hash`、`storage_receipt_id`、`manifest_url`、`cover_url`、`cached_at`
  - `SquareFeedCursor`: `owner_account`、`feed_kind`、`cursor`、`updated_at`
  - `SquareUserSignalCache`: `owner_account`、`post_id`、`signal_type`、`created_at`、`synced`
  - `ContactCache`: 按 `owner_account` 隔离保存解密后的 `address`、`contact_name`、`created_at`、`updated_at`，复用 `AppKvEntity`，不是公开资料真源。
  - `ContactPendingOps`: 按 `owner_account` 隔离保存待同步的添加、改名、删除操作；云端成功后立即移除，不把普通旧缓存当作待上传操作。
- 通讯录云端密文契约：
  - schema 固定为 `citizenapp.contacts.v1`；明文记录只存在 CitizenApp 端，字段固定为 `owner_account`、`address`、`contact_name`、`created_at`、`updated_at`。
  - `encryption_key` 与 `index_key` 必须由同一热钱包 seed 经 HKDF-SHA256 域隔离派生；seed 和派生密钥不得上传 Cloudflare。
  - 单条记录使用 AES-256-GCM；AAD 必须绑定 schema、`owner_account` 与 `contact_id`，防止跨账户或跨联系人替换。
  - D1 `square_contacts` 只允许保存 `owner_account`、`contact_id`、`ciphertext`、`nonce`、`mac`、`updated_at`；禁止保存联系人 `address`、`contact_name`、公开昵称或关系明文。
- 编码：HTTP JSON 字段统一 snake_case；R2 manifest 为 UTF-8 JSON；hash 字段为 sha256 hex；Worker 阶段 3 已落地字段的时间统一使用毫秒时间戳。
- CitizenApp 阶段 5/6 实现真源：
  - `citizenapp/lib/8964/services/square_api_client.dart`
  - `citizenapp/lib/8964/services/square_upload_service.dart`
  - `citizenapp/lib/8964/services/square_publish_service.dart`
  - `citizenapp/lib/8964/pages/square_compose_page.dart`
  - `citizenapp/lib/8964/pages/square_home_page.dart`
  - `citizenapp/lib/my/user/user.dart`
- Worker 阶段 6 实现真源：
  - `citizenapp/cloudflare/src/membership/citizen_coin.ts`
  - `citizenapp/cloudflare/src/membership/creator.ts`
  - `citizenapp/cloudflare/src/membership/reconcile.ts`
  - `citizenapp/cloudflare/src/membership/service.ts`
  - `citizenapp/cloudflare/src/uploads/service.ts`
  - `citizenapp/cloudflare/src/media/cloudflare_assets.ts`
  - `citizenapp/cloudflare/src/chain/rpc.ts`
  - `citizenapp/cloudflare/src/chain/square_event.ts`
  - `citizenapp/cloudflare/src/posts/confirm.ts`
  - `citizenapp/cloudflare/src/posts/repository.ts`
- 会员支付只允许在 CitizenApp 内完成；CitizenWeb 不承载会员订阅或钱包签名入口。
- 签名/验签规则：
  - Worker session 必须由已登记的 P-256 设备子钥对 `signing_payload` 签名获得；设备子钥归属仍由钱包主钥绑定证明建立。会话不得读取链上账户、余额或存在性存款。
  - Worker 只能把钱包签名证明用于登录和上传授权，不得托管钱包私钥。
  - 通讯录密文的 AES/HMAC 密钥只在 CitizenApp 端由热钱包 seed 域隔离派生；Worker 不得接收、生成、托管或恢复通讯录密钥，也不得记录联系人明文。
  - manifest 与图片必须经同域 Worker 有界读取并验证 P-256 设备签名；视频 TUS 地址必须绑定 `owner_account`、`upload_id`、精确字节和最长时长。
  - CitizenApp 必须先用 finalized 余额确认钱包至少保留 `1.21 元`（ED 1.11 元 + 发布费 0.1 元），余额不足不得进入 Worker prepare 或媒体上传。
  - CitizenApp 必须在链上扣费交易入块后才上传 manifest 与主媒体；链上未入块不得占用 R2 / Images / Stream 存储，只能保存本地草稿。
  - 不存在用户 R2 写入授权、Images 客户端直传或开发上传代理；本地、staging、production 共用同一条 Worker/Images/Stream 目标流程。
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
  - Worker 有界图片上传、Stream TUS 上传与 Stream webhook 签名测试
  - CitizenApp 广场 API adapter 测试
  - App 真机或模拟器广场浏览与发布流程验收

### P-API-CITIZENAPP-003：CitizenApp Chat 瞬时转发

- 状态：当前（2026-07-11 已彻底删除云端消息与附件存储，staging 已部署）
- 类型：接口契约 / 瞬时密文与 WebRTC 信令转发契约
- 唯一真源：
  - 方案任务卡：`memory/08-tasks/open/20260711-chat-square-step1.md`
  - Chat 技术文档：`memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`
  - 落地后实现真源：`citizenapp/cloudflare/src/chat/` 与 `citizenapp/lib/chat/transport/`
- 生产者：
  - `citizenapp/lib/chat/`：OpenMLS 加密、KeyPackage、本机发送队列、WebRTC 附件。
  - `citizenapp/cloudflare/`：设备登记、一次性 KeyPackage、WebSocket 瞬时密文/信令转发和无内容唤醒。
- 消费者：
  - `citizenapp/lib/chat/`
  - Cloudflare D1 database `citizenapp-square-db`
  - Cloudflare Durable Objects / WebSocket，按 `owner_account` 路由到 `ChatRealtimeObject`。
- HTTP API 字段：
  - `POST /v1/chat/devices/register`：`device_id`、`device_public_key_hex`、`push_provider`、`push_token`、`binding_signature`、`expires_at`、`nonce`；账户只取 session。
  - `POST /v1/chat/keypackages` 请求：Bearer `session_token`，`owner_account`、`device_id`、`device_public_key_hex`、`key_package_id`、`key_package`、`cipher_suite`、`created_at`、`expires_at`
  - `GET /v1/chat/keypackages/{owner_account}` 请求：Bearer `session_token`，`limit`
  - `POST /v1/chat/keypackages/consume` 请求：Bearer `session_token`，`owner_account`、`key_package_id`、`requester_account`
  - `POST /v1/chat/envelopes`：`envelope_id`、`sender_device_id`、`recipient_account`、`recipient_device_id`、`envelope`；仅在当前请求内转发。
  - `POST /v1/chat/signals`：`sender_device_id`、`recipient_account`、`recipient_device_id`、`signal`；仅转发 SDP/ICE/设备就绪信令。
  - `GET /v1/chat/ws`：Bearer session + `x-chat-device`；收到 `gmb_chat_envelope_v2` 时客户端立即解密，收到 `gmb_chat_signal_v1` 时交给 WebRTC。
  - Durable Object binding：`CHAT_REALTIME`，class `ChatRealtimeObject`，对象名称固定为 `owner_account`。
- D1 表字段：
  - `chat_devices`: `owner_account`、`device_id`、`device_public_key_hex`、`push_provider`、`push_token`、`expires_at`、`created_at`
  - `chat_device_binding_nonces`: `owner_account`、`nonce_hash`、`expires_at`、`created_at`
  - `chat_keypackages`: `owner_account`、`device_id`、`key_package_id`、`key_package`、`cipher_suite`、`created_at`、`expires_at`
- R2：Chat 禁止使用 R2；消息、会话和附件没有云端对象键。
- 编码：
  - HTTP JSON 字段统一 snake_case。
  - `envelope` 载荷承载 `GMB_CHAT_V1 / ChatEnvelope` Protobuf bytes 的 base64url 表示。
  - `key_package` 承载 OpenMLS KeyPackage bytes 的 base64url 表示。
  - 附件经 WebRTC DTLS DataChannel 在设备间传输并只落两端设备。
  - WebRTC 只配置公开 `stun:stun.cloudflare.com:3478` 发现直连候选，不配置中继 URL 或中继凭证；直连失败时附件继续留在发送设备等待重试。
- 签名/验签规则：
  - Worker session 必须由已登记的硬件 P-256 设备子钥静默签名获得。
  - 设备绑定必须由同一硬件 P-256 设备子钥对 `signing_message(OP_SIGN_CHAT_DEVICE_BIND=0x1A, SCALE(owner_account, device_id, device_public_key_hex, expires_at, nonce))` 签名；Worker 只用 session owner 查询 `square_device_subkeys.p256_pubkey` 验签。
  - Chat 设备绑定、KeyPackage、瞬时收发和实时连接禁止读取钱包 seed；钱包主私钥不得参与 OpenMLS 消息加密。
  - Cloudflare Worker 不得生成、保存或恢复 OpenMLS 私钥。
  - Chat 路由不得保存消息、会话、联系人明文、附件字节或附件引用。P-API-CITIZENAPP-002 的端侧加密通讯录密文属于独立 USER 同步接口，Chat Worker 不得读取、复用或解密。
  - Cloudflare Worker 不得签发或保存附件中继凭证，不得承担附件流量中继。
- 禁止事项：
  - 禁止保存私聊或群聊明文。
  - 禁止把私密聊天写入广场公开评论表。
  - 禁止把 Cloudflare 瞬时转发描述成公民链节点、区块链节点或全节点聊天能力。
  - 禁止要求用户安装或开启区块链软件后才能使用互联网聊天。
- 必跑测试：
  - Worker chat API 单元测试。
  - 设备绑定验签测试。
  - KeyPackage 发布/拉取/消费测试。
  - 瞬时密文投递和接收设备不可达测试。
  - WebRTC 瞬时信令、仅 STUN 直连和设备附件帧测试。
  - CitizenApp `ChatCloudTransport` 测试。
  - CitizenApp 附件文件选择和设备私有缓存测试。
  - OpenMLS 1:1 和群聊密文 round-trip 测试。

### P-API-CITIZENAPP-004：CitizenApp Chain Bootstrap Manifest

- 状态：当前（2026-07-10 已升级并全量发布 bootstrap v2；staging 版本 `ff19bc46-dc17-4f77-a53f-aed2739142a0`、production 版本 `00d836aa-9c43-4561-ba33-8730d780c1a0`；Cloudflare 只治理链身份、bootnodes 和服务发现，CitizenApp checkpoint 只来自签名安装包；已签名交易受控广播 path 保持不变）
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
  - 部署环境变量 `CHAIN_*`
- 消费者：
  - CitizenApp 轻节点连接状态机
  - CitizenApp 广场和聊天服务发现
- HTTP API 字段：
  - `GET /v1/chain/bootstrap` 响应顶层字段：`ok`、`schema`、`generated_at`、`cache_ttl_seconds`、`chain`、`light_client`、`p2p`、`services`、`security`、`degradation`
  - `schema`: 固定 `citizenapp.chain.bootstrap.v2`；App 不接受旧版 schema
  - `chain`: `chain_id`、`chain_name`、`chain_type`、`protocol_id`、`genesis_hash`、`state_root`、`ss58_format`、`token_symbol`、`token_decimals`
  - `light_client`: `mode`、`truth_source`、`api_is_truth`、`bundled_assets_required`
  - `light_client.mode`: 固定 `smoldot`
  - `light_client.truth_source`: 固定 `p2p_finalized_storage`
  - `light_client.api_is_truth`: 固定 `false`
  - `light_client` 禁止包含任何 checkpoint、远端轻同步资产 URL 或摘要字段；信任锚只来自签名安装包
  - `p2p`: `bootnodes`、`bootnodes_source`、`min_peer_count_hint`
  - `services`: `square_base_url`、`chat_base_url`、`media_base_url`、`signed_extrinsic_relay`
  - `services.signed_extrinsic_relay`: `enabled`、`path`；默认 `enabled=false/path=null`，仅当 Worker 显式配置 `RELAY_ENABLED=1` 且服务节点 RPC 已配置时返回 `enabled=true/path=/v1/chain/extrinsics/relay`
  - `security`: `exposes_rpc_url`、`rpc_proxy`、`exposes_private_key_material`、`validator_rpc_public`，全部固定 `false`
  - `degradation`: `p2p_unavailable`、`chain_success_source`
- Worker 环境变量：
  - `CHAIN_BOOTNODES`: 公开 bootnode multiaddr 列表，允许换行、逗号或分号分隔；不是密钥。
  - `BOOT_TTL_SECONDS`: 启动清单 HTTP 缓存秒数。
  - `CHAIN_GENESIS_HASH`: 当前链 genesis hash。
  - `CHAIN_STATE_ROOT`: 当前轻形态 chainspec genesis `stateRootHash`。
  - `RELAY_ENABLED`: 已签名交易受控广播开关，默认 `0`。
  - `RELAY_MAX_BYTES`: relay 接受的 signed extrinsic 最大字节数。
  - `RELAY_PER_MINUTE`: relay 每分钟按请求 IP hash 限流数量。
  - `CHAIN_URL`: Access 保护的私有链 RPC HTTPS 地址，只放远端 Secret。
  - `CHAIN_ID`、`CHAIN_SECRET`: Access 服务令牌，必须成套放入远端 Secret；缺失任一项时 relay 固定关闭。
- 编码：HTTP JSON，字段统一 snake_case；时间统一毫秒时间戳；hash 字段为 hex；`bootnodes` 元素为 Substrate multiaddr 字符串。
- 签名/验签规则：本接口不携带用户签名，不接受交易载荷；只声明受控广播 path 是否可用，广播协议见 `P-API-CITIZENAPP-005`。
- 禁止兼容：不兼容 API-only 链连接方案；不得把本接口演化成通用 JSON-RPC fallback。
- 禁止事项：
  - 禁止响应中返回 `CHAIN_URL`、两项 `CHAIN_ID / CHAIN_SECRET`、Validator RPC、Archive RPC 或任何私密 RPC 完整 URL。
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
- 远端验收：staging 与 production 必须分别验证 schema v2、`light_client` 精确四字段、无 checkpoint/RPC URL、`/v1/chain/rpc` 为 404；生产包验收不得残留 staging base URL Dart define。

### P-API-CITIZENAPP-005：CitizenApp Signed Extrinsic Relay

- 状态：当前（2026-07-08 第 4 步已落地 Worker 受控广播、D1 审计表和 App submit-only 兜底）
- 类型：接口契约
- 唯一真源：
  - Worker：`citizenapp/cloudflare/src/chain/extrinsic_relay.ts`
  - D1 基线：`citizenapp/cloudflare/migrations/0001_square_core.sql`（交易广播审计结构已合并）
  - App：`citizenapp/lib/rpc/signed_extrinsic_relay_api.dart`、`citizenapp/lib/rpc/chain_rpc.dart`
- 详细文档：
  - `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
  - `memory/08-tasks/open/20260708-citizenapp-chain-edge-architecture.md`
- 生产者：
  - CitizenApp 本地完成签名后的 submit-only 兜底逻辑。
  - Cloudflare Worker `POST /v1/chain/extrinsics/relay`。
- 消费者：
  - CitizenChain 服务节点的 `author_submitExtrinsic`。
  - D1 表 `chain_extrinsic_relays`。
- HTTP API 字段：
  - 请求：`signed_extrinsic_hex`，完整 signed extrinsic hex，必须以 `0x` 开头。
  - 响应：`ok`、`schema=citizenapp.chain.extrinsic_relay.v1`、`relay_id`、`relay_status=broadcast`、`deduplicated`、`tx_hash`、`accepted_at`、`chain_success_source=finalized_runtime_storage_or_events`。
- 编码：HTTP JSON，字段统一 snake_case；`tx_hash` 为 32 字节 hex；Worker 不保存原始 extrinsic body，只保存 `extrinsic_sha256`。
- 签名/验签规则：App 在本地完成交易签名；Worker 不接触私钥、不生成签名、不修改交易载荷，只把完整 signed extrinsic 交给服务节点 RPC。
- 禁止兼容：不得演化成通用 JSON-RPC proxy；不得新增兼容旧 RPC URL 下发字段。
- 禁止事项：
  - 禁止请求体携带 `private_key`、`mnemonic`、`seed`、`secret`、`keystore`、`password`、`recovery_phrase` 等密钥材料。
  - 禁止响应中返回 `CHAIN_URL`、两项 `CHAIN_ID / CHAIN_SECRET`、Validator RPC、Archive RPC 或任何私密 RPC 完整 URL。
  - 禁止把 relay 返回 `tx_hash` 当成链上成功；业务成功必须继续以 finalized runtime storage 或事件为准。
  - 禁止 App 在交易本身已被判定为 invalid / bad proof / stale / future / payment 类错误时再走 relay 兜底。
- 必跑测试：
  - `npm --prefix citizenapp/cloudflare run typecheck`
  - `npm --prefix citizenapp/cloudflare test -- chain_bootstrap.test.ts chain_extrinsic_relay.test.ts`
  - `flutter analyze lib/rpc/chain_bootstrap_api.dart lib/rpc/chain_rpc.dart lib/rpc/signed_extrinsic_relay_api.dart test/rpc/chain_bootstrap_api_test.dart test/rpc/signed_extrinsic_relay_api_test.dart`
  - `flutter test test/rpc`
  - `npm --prefix citizenapp/cloudflare run migrate:local`

### P-CHAT-002：CitizenApp Nearby Chat Transport

- 状态：草案（2026-07-05 方案冻结；待后续阶段实现）
- 类型：近场传输契约
- 唯一真源：
  - Chat 技术文档：`memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`
  - 落地后实现真源：`citizenapp/android/chat/`、`citizenapp/ios/chat/`、`citizenapp/lib/chat/transport/`
- 生产者：`citizenapp/lib/chat/transport/ChatNearbyTransport`
- 消费者：`citizenapp/lib/chat/`
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
  - `envelope` 承载 `GMB_CHAT_V1 / ChatEnvelope` Protobuf bytes。
  - 近场 transport 不改变 OpenMLS 会话、不改变 `ChatEnvelope`。
- 签名/验签规则：
  - 近场初次通信必须显示安全码或二维码校验入口。
  - 钱包地址只作为聊天身份；OpenMLS 设备密钥负责端到端加密。
- 禁止事项：
  - 禁止近场依赖 Cloudflare、链 RPC 或区块链节点通信节点。
  - 禁止近场传输明文私聊/群聊内容。
  - 禁止为近场另建一套消息格式。
- 必跑测试：
  - `ChatNearbyTransport` envelope 去重测试。
  - Android 真机近场收发测试。
  - iOS 真机近场收发测试。
  - Android / iOS 跨平台近场 smoke。

### P-TX-013：Square.publish_square_post

- 状态：草案（阶段 4 runtime 发布索引已落地；阶段 5 CitizenApp 交易编码和提交入块闭环已落地；阶段 6 Worker 链上事件确认和正式 feed 入库已落地）
- 类型：交易载荷格式 / storage 契约
- 唯一真源：
  - 方案任务卡：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
  - 落地后实现真源：`citizenchain/runtime/misc/square-post/`
- 详细文档：
  - `memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
  - `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/8964/chain/`
  - `citizenapp/lib/rpc/`
- 消费者：
  - `citizenchain/runtime/misc/square-post/`
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
  - pallet index：`34`。
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
  - runtime 将 `SquarePost` 路由为 `FeeRoute::Onchain { transaction_amount: 0, payer: signer }`，对每条发布交易按 `ONCHAIN_MIN_FEE` 扣 0.1 元发布费。
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
  - runtime `SquarePost` 路由为签名者付款的零金额 `Onchain` 测试；实际最低链上费用 0.1 元和 8:1:1 分账复用现有 `OnchainFeeRouter` 测试。
  - CitizenApp 发布交易编码测试：阶段 5 已由 `citizenapp/test/8964/square_chain_service_test.dart` 覆盖。
  - Worker 链上事件解码与发布确认测试：阶段 6 已由 `citizenapp/cloudflare/test/chain_confirm.test.ts` 覆盖。

### P-TX-014：SquarePost 公民币订阅交易

- 状态：当前目标契约（实现按 `20260716-citizen-coin-subscription` 分步骤落地）
- 类型：交易载荷格式
- 唯一真源：
  - `memory/01-architecture/gmb/subscription-part1-tech.md`
  - `citizenchain/runtime/misc/square-post/`
  - `citizenchain/runtime/misc/square-post/tests/fixtures/subscription_scale_vectors.json`
- 生产者：
  - `citizenapp/lib/rpc/subscription_rpc.dart`
  - `citizenchain/onchina/src/platform/`
- 消费者：
  - `citizenchain/runtime/misc/square-post/`
  - `citizenapp/cloudflare/src/chain/subscription.ts`
  - `citizenwallet`（仅负责平台调价载荷确认、一次签名和响应二维码展示）
- pallet：`SquarePost`
- pallet index：`34 / 0x22`
- call index：
  - `0`：`publish_post`，见 P-TX-013。
  - `1`：`subscribe(issuer, plan, expected_price_fen)`。
  - `2`：`cancel(issuer)`。
  - `3`：`set_creator_plans(tiers)`。
  - `4`：`change_subscription_plan(issuer, new_plan, expected_price_fen)`。
  - `5`：`propose_set_platform_price(actor_cid_number, membership_level, new_price_fen)`。
- 类型标签：
  - `IssuerKey::Platform = 0x00`。
  - `IssuerKey::Creator(AccountId32) = 0x01 || account_id[32]`。
  - `MembershipLevel::Freedom/Democracy/Spark = 0x00/0x01/0x02`。
  - `BillingPeriod::Monthly/Quarterly/Yearly = 0x00/0x01/0x02`。
  - `SubscriptionPlan::Platform{membership_level} = 0x00 || membership_level`。
  - `SubscriptionPlan::Creator{tier_id,billing_period} = 0x01 || Compact(tier_id_len) || tier_id || billing_period`。
  - `SubscriptionStatus::Active/Cancelled/Terminated = 0x00/0x01/0x02`。
- call data 顺序：
  - 平台订阅：`22 01 || 00 || 00 || membership_level || expected_price_fen:u128LE`。
  - 创作者订阅：`22 01 || 01 || creator_account[32] || 01 || Compact(tier_id_len) || tier_id || billing_period || expected_price_fen:u128LE`。
  - 平台取消：`22 02 || 00`。
  - 创作者取消：`22 02 || 01 || creator_account[32]`。
  - 创作者覆盖套餐：`22 03 || Compact(tier_count) || CreatorTier...`；每个 CreatorTier 为 `Compact(tier_id_len) || tier_id || Compact(period_count) || (billing_period || price_fen:u128LE)...`。
  - 换套餐：`22 04 || issuer || new_plan || expected_price_fen:u128LE`。
  - 平台调价：`22 05 || actor_cid_number:SCALE Vec<u8> || membership_level || new_price_fen:u128LE`。
- 价格规则：
  - `expected_price_fen` 只保护首次扣款或换套餐签名到入块期间的价格；不得写入订阅状态作为续费价格真源。
  - 平台每次扣款读取当前 `PlatformPrice`。
  - 创作者每次扣款读取当前 `CreatorPlans`。
- 签名规则：
  - 订阅者只对 `subscribe/cancel/change_subscription_plan` 签名；创作者只对自己的 `set_creator_plans` 管理调用签名。
  - `subscribe` 建立持续自动扣款授权，直到订阅者签名取消；续费和周期推进由 runtime 内部执行，不存在外部 call 或再次签名。
  - `propose_set_platform_price` 由 OnChina 生成 QR_V1/k=1 请求二维码，CitizenWallet 只签名一次并显示响应二维码，OnChina 回扫响应后通过统一提交入口上链；业务 pallet 只创建统一内部投票提案。
- 禁止兼容：
  - 不兼容旧 `SubscriptionPlan::Level/CreatorPrice/Terms` tag。
  - 不兼容 `BillingKeeper/charge_due`。
  - 不兼容 Cloudflare/App 带价续费、外部周期确认、设备签名或旧 call 墓碑。
- 必跑测试：
  - runtime 金标 SCALE 向量和 call index 测试。
  - CitizenApp 编码与 runtime fixture 逐字节一致测试。
  - Cloudflare storage 解码金标测试。
  - CitizenWallet 平台调价动作两态识别测试。

### P-STORAGE-006：SquarePost 订阅 storage

- 状态：当前目标契约（实现按 `20260716-citizen-coin-subscription` 分步骤落地）
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/misc/square-post/`
- 详细文档：`memory/01-architecture/gmb/subscription-part1-tech.md`
- storage：
  - `Subscriptions<(AccountId, IssuerKey)> -> SubscriptionState`，key hasher 为 `Blake2_128Concat`。
  - `PlatformPrice<MembershipLevel> -> u128`，key hasher 为 `Twox64Concat`。
  - `PlatformCidNumber -> Option<CidNumber>`。
  - `CreatorPlans<AccountId> -> BoundedVec<CreatorTier>`，key hasher 为 `Blake2_128Concat`。
  - `RenewalSchedule<(due_at_be, AccountId, IssuerKey)> -> ()`，第一个 key 使用大端时间戳保持到期顺序。
  - `RenewalIndex<(AccountId, IssuerKey)> -> due_at`，保证每个订阅只有一个当前调度项。
  - `MigrationBlocked -> bool`，发现旧订阅数据时阻止启用新协议。
- `SubscriptionState` SCALE 字段顺序：
  1. `plan: SubscriptionPlan`。
  2. `pending_plan: Option<SubscriptionPlan>`。
  3. `started_at: u64LE`。
  4. `last_charged_at: u64LE`。
  5. `last_charged_price_fen: u128LE`。
  6. `paid_until: u64LE`。
  7. `subscription_status: SubscriptionStatus`。
- 状态规则：
  - `Active` 必须有对应 `RenewalSchedule` 和 `RenewalIndex`；`current_timestamp < paid_until` 时权益有效。
  - Cancelled 不再续费，但不得缩短 `paid_until`。
  - Terminated 由余额不足、真实转账失败或套餐失效产生，不再调度且不重试。
  - pending plan 只在下一次成功续费后替换当前 plan。
- 生产者：`square-post` subscribe、cancel、change、set plans、runtime 自动续费和平台调价终态回调。
- 消费者：CitizenApp finalized 链读、Cloudflare finalized 镜像和对账器。
- 禁止事项：
  - 禁止恢复 `BillingKeeper` storage。
  - 禁止 D1 保存第二份扣款价格真源。
  - 禁止在 CitizenApp 或 Cloudflare 计算并提交订阅公历到期时间。
  - 禁止使用区块高度、固定天数或固定毫秒推导 `paid_until`。
  - 禁止 App、Cloudflare、keeper 或任意外部账户提交续费；runtime 只处理有界到期索引。
- 必跑测试：runtime storage key/value 金标、Cloudflare 解码、CitizenApp 解码、TryRuntime pre/post 不变量。

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
  - **OnChina 控制台链写动作码（`b.d`=裸 SCALE call data，CitizenWallet 解码核对并只签名一次、显示响应二维码，OnChina 回扫响应后通过统一入口提交）**：链交易统一 `a=(pallet<<8)|call`(禁止扁平小整数,会撞非链动作码 1..9)。机构创建=公权 `0x1e05`(PublicManage 30/call 5)/私权 `0x1f05`(PrivateManage 31/call 5,见 P-TX-001);本机构治理=公权 `0x1e08`/私权 `0x1f08`，注册局直接登记管理员=公权 `0x1e09`/私权 `0x1f09`(见 P-TX-012);公民投票身份注册=`0x0a00`(CitizenIdentity 10/call 0,见 P-TX-011);公民参选身份上链=`0x0a01`(CitizenIdentity 10/call 1,见 P-TX-011);个人多签管理员集合变更为 `0x1d00`(PersonalAdmins 29/call 0,见 P-TX-007);平台调价提案=`0x2205`(SquarePost 34/call 5,见 P-TX-014);CREG 市注册局无独立链动作码；非链文本治理 `a=3 = ACTION_ONCHINA_ADMIN / QR_ACTION_ONCHINA_ADMIN`(onchina_admin_governance JSON)。动作码由 `onchina/src/core/institution_call.rs::chain_action_code(pallet,call)` 与 call data 同源派生,非链常量在 `core/qr/mod.rs`,runtime 注释真源在 `primitives::sign`,均与 `qr-action-registry.md` 同步；动作码 `8` 已取消登记，Chat 设备绑定不得进入 QR。
  - Substrate 交易 payload 长度 >256B 时必须签 `blake2_256(payload)`
- 禁止兼容：开发期严格模式，不做别名兼容
- 禁止事项：
  - 禁止恢复 `display` / `summary` / `fields`
  - 禁止未登记的 `a` 进入生产
  - 禁止把内部哈希、nonce、原始公钥 hex 当作普通用户确认字段展示
- 必跑测试：`citizenwallet/test/signer/payload_decoder_test.dart`、QR sign request 测试

### P-QR-003：QR_V1 / k=5 chat_node_pairing

- 状态：已删除（2026-07-05 聊天方案改为 Cloudflare 互联网聊天 + 近场聊天；区块链节点通信节点聊天方式不再作为正式路线）
- 类型：扫码协议内固定码
- 唯一真源：无当前代码真源；旧实现文件已删除
- 详细文档：
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`
  - `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`
- 生产者：无；桌面节点不再生成 Chat 配对二维码。
- 消费者：无；CitizenApp 扫到 `k=5` 按未知类型拒绝。
- 字段：
  - 无当前字段；旧 `b.node_peer_id`、`b.node_multiaddr`、`b.endpoint_kind` 已删除。
- 编码：无当前编码；`QR_V1/k=5` 不再是合法扫码流向。
- 签名/验签规则：正式聊天不再扫描区块链软件通信节点二维码。
- 禁止兼容：不得恢复旧联系人码、旧 Chat 联系人 bundle、旧 `communication` 模式字段或通信节点配对流程。
- 禁止事项：
  - 禁止用本二维码添加联系人。
  - 禁止把本二维码作为交易、转账、治理或 CID 身份码处理。
  - 禁止恢复通信节点配对、桌面通信节点二维码、节点 Chat 消息服务或已删除的节点聊天协议。
- 删除验收：已删除 `citizenapp/lib/qr/bodies/chat_node_pairing_body.dart`、`citizenapp/lib/chat/chat_node_settings_page.dart`、桌面通信节点二维码生成和相关测试残留；`test/qr/qr_router_test.dart` 覆盖 `k=5` 拒绝。

### P-CRED-003：CitizenIdentity VotingIdentityPayload

- 状态：当前
- 类型：凭证载荷 / 交易载荷内层结构
- 唯一真源：`citizenchain/runtime/misc/citizen-identity/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- 消费者：
  - `citizenapp/lib/signer/citizen_identity_sign_service.dart`
  - `citizenwallet/lib/signer/payload_decoder.dart`
  - `citizenwallet/lib/signer/qr_signer.dart`
  - `citizenchain/runtime/misc/citizen-identity`
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
  - action registry 的统一中文动作名为“公民签名确认”；投票/参选只作为载荷详情展示，不得拆成第二套动作标题。
  - `QR_V1` 非链动作 `a=2 citizen_identity` 的签名字节为 `blake2_256(GMB || 0x10 || payload_bytes)`。
  - runtime 通过 `primitives::sign::OP_SIGN_CITIZEN_IDENTITY` 验证目标公民钱包签名。
  - `citizen_age_years` 必须大于等于 16;OnChina 和 runtime 都必须校验。
  - 同一上链 operation 固定只允许管理员 Passkey 一次、目标公民钱包签名一次、管理员最终链交易签名一次。`prepare` 创建短期操作，`complete` 只能原子消费对应公民回执，不得再次生成安全 grant 或消费 Passkey。
  - 公民钱包绑定和上链投影必须等最终链交易确认后写入。
- 禁止兼容：不兼容旧 `citizen-identity-v1|...` 文本载荷,不保留旧签原文规则。
- 禁止事项：
  - 禁止本地新增公民阶段要求钱包账户。
  - 禁止未满 16 周岁公民推送链上身份。
  - 禁止二维码携带展示摘要或字段别名。
  - 禁止 `prepare/complete` 分阶段重复认证同一角色，禁止为最终链签再叠加管理员 grant 钱包签名。
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/qr_signer_test.dart`
  - `cargo test --manifest-path citizenchain/Cargo.toml -p citizen-identity`

### P-TX-011：CitizenIdentity.register_voting_identity / upgrade_to_candidate_identity

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/misc/citizen-identity/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- 消费者：
  - `citizenchain/runtime/misc/citizen-identity`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `registrar_account`
  2. `VotingIdentityPayload` 或 `CandidateIdentityPayload`
  3. `citizen_signature`
- 编码：
  - SCALE call data
  - `0x0a00 register_voting_identity` 仅携带投票身份字段。
  - `0x0a01 upgrade_to_candidate_identity` 携带投票身份字段 + `birth_province_code / birth_city_code / birth_town_code / citizen_full_name / citizen_sex / birth_date`；其中 `birth_date` 为 `u32` 小端 `YYYYMMDD`，位于性别字段之后且为必填字段；该交易同时写入投票身份和参选身份。
  - pallet index：`10`
  - call index：`0`
  - 前两个字节固定为 `[0x0a, 0x00]`
  - 动作码：`a=0x0a00`
- 签名/验签规则：
  - 外层链交易由当前注册局管理员的 CitizenWallet 只签名一次并显示响应二维码；OnChina 回扫后经统一提交入口上链。
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
  - `propose_enact_law`: `[pallet=25, call=0] + tier + scope_code + houses + proposer_body + executive + legislature + vote_type + title + title_en + chapters + effective_at`
  - `propose_amend_law`: `[pallet=25, call=1] + law_id + proposer_body + executive + legislature + vote_type + title + title_en + chapters + effective_at`
  - `propose_repeal_law`: `[pallet=25, call=2] + law_id + proposer_body + executive + legislature + vote_type`
- 编码：
  - 裸 SCALE call data
  - `tier`/`vote_type` 为单字节枚举序号
  - `scope_code` 为 `u32`
  - `law_id` 为 `u64`
  - `houses` / `proposer_body` / `executive` / `legislature` 使用 `(InstitutionCode[4], AccountId32)`
  - `chapters` 为 `章 > 节 > 条 > 款` 的 SCALE 结构
  - `effective_at` 为 `u64` 毫秒时间戳，不是块号
  - 动作码：`0x1900` / `0x1901` / `0x1902`
- 签名/验签规则：
  - 外层链交易由当前立法/提案机构管理员的 CitizenWallet 只签名一次并显示响应二维码；OnChina 回扫后经统一提交入口上链。
  - 业务投票、计票、签署和守卫流程统一归投票引擎与 legislation-vote，不得由 OnChina 或客户端复刻。
  - `houses / proposer_body / executive / legislature` 虽作为客户端载荷传入，runtime 必须按 `tier + scope_code + vote_type` 复核法定机构码、机构账户、行政区与院序；客户端传值不是授权真源。
- 禁止兼容：不兼容旧区块高度生效载荷，不保留旧字段顺序。
- 禁止事项：
  - 禁止前端显示或让用户填写旧区块高度生效字段。
  - 禁止未登记动作码进入冷钱包 decoder。
- 必跑测试：
  - `cargo test -p onchina --manifest-path citizenchain/Cargo.toml law`
  - `cargo test -p legislation-yuan --manifest-path citizenchain/Cargo.toml`

### P-TX-013：ElectionVote 投票交易载荷

- 状态：当前
- 类型：交易载荷格式 / runtime 调用边界
- 唯一真源：`citizenchain/runtime/votingengine/election-vote/src/lib.rs`
- 详细文档：`memory/05-modules/citizenchain/runtime/election-campaign/ELECTION_CAMPAIGN_TECHNICAL.md`
- 生产者：合法选举活动的投票客户端
- 消费者：`citizenchain/runtime/votingengine/election-vote`
- 字段：
  - `cast_popular_vote`: `[pallet=29, call=2] + proposal_id + candidate`
  - `cast_mutual_vote`: `[pallet=29, call=3] + proposal_id + candidate`
- 编码：裸 SCALE call data；`proposal_id` 为 `u64`，`candidate` 为 `AccountId32`。
- 签名/验签规则：外层链交易由候选/选民快照中有效选民账户签名；选民资格、防重复和计票归 election-vote。
- 禁止兼容：底层 `create_popular_election` / `create_mutual_election` 外部调用物理删除，不保留旧创建载荷或过渡入口。
- 禁止事项：
  - 禁止客户端直接向 election-vote 传入组织机构、目标机构、岗位、席位、候选人或选民快照来创建选举。
  - 禁止 election-vote 直接写入 entity 岗位、任职或 admins；选举结果必须返回 election-campaign 业务层复核。
- 必跑测试：
  - `cargo test -p election-vote --manifest-path citizenchain/Cargo.toml`
  - `cargo test -p citizenchain --manifest-path citizenchain/Cargo.toml runtime_call_filter`

### P-CHAT-001：GMB_CHAT_V1

- 状态：当前（统一消息/加密格式；互联网只做瞬时转发，附件只走设备间通道）
- 类型：接口契约 / 编码协议 / 端到端加密消息外层
- 唯一真源：`citizenapp/chat/proto/chat_envelope.proto`
- Dart 生成物：`citizenapp/lib/chat/proto/chat_envelope.pb.dart`、`citizenapp/lib/chat/proto/chat_envelope.pbenum.dart`、`citizenapp/lib/chat/proto/chat_envelope.pbjson.dart`
- 详细文档：
  - `memory/04-decisions/ADR-020-citizenapp-p2p-chat.md`
  - `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/chat/`：OpenMLS 加密、会话状态机、消息队列。
  - `citizenapp/cloudflare/src/chat/`：Cloudflare 瞬时转发接口，不生成明文。
- 消费者：
  - `citizenapp/lib/chat/`
  - `citizenapp/cloudflare/src/chat/`
- 字段：
  - `ChatEnvelope.protocol_version`
  - `ChatEnvelope.envelope_id`
  - `ChatEnvelope.conversation_id`
  - `ChatEnvelope.sender_account`
  - `ChatEnvelope.recipient_account`
  - `ChatEnvelope.sender_device_id`
  - `ChatEnvelope.mls_wire_message`
  - `ChatEnvelope.encrypted_metadata`
  - `ChatEnvelope.created_at_millis`
  - `ChatEnvelope.ttl_millis`
  - `ChatEnvelope.mls_message_kind`
  - `ChatEnvelope.ratchet_tree`
  - `MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_UNSPECIFIED`
  - `MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_WELCOME`
  - `MlsWireMessageKind.MLS_WIRE_MESSAGE_KIND_APPLICATION`
  - `ChatRoute.protocol_version`
  - `ChatRoute.peer_account`
  - `ChatRoute.route_display_name`
  - `ChatRoute.device_id`
  - `ChatRoute.device_public_key_hex`
  - `ChatRoute.safety_number`
  - `ChatRoute.nearby_peer_hint`
  - `ChatRoute.created_at_millis`
  - `ChatRoute.expires_at_millis`
  - `ChatKeyPackage.protocol_version`
  - `ChatKeyPackage.owner_account`
  - `ChatKeyPackage.device_id`
  - `ChatKeyPackage.device_public_key_hex`
  - `ChatKeyPackage.key_package_id`
  - `ChatKeyPackage.key_package`
  - `ChatKeyPackage.cipher_suite`
  - `ChatKeyPackage.created_at_millis`
  - `ChatKeyPackage.expires_at_millis`
  - `PublishChatKeyPackageRequest.owner_account`
  - `PublishChatKeyPackageRequest.device_id`
  - `PublishChatKeyPackageRequest.device_public_key_hex`
  - `PublishChatKeyPackageRequest.key_package_id`
  - `PublishChatKeyPackageRequest.key_package`
  - `PublishChatKeyPackageRequest.cipher_suite`
  - `PublishChatKeyPackageRequest.created_at_millis`
  - `PublishChatKeyPackageRequest.expires_at_millis`
  - `FetchChatKeyPackagesRequest.owner_account`
  - `FetchChatKeyPackagesRequest.requester_account`
  - `FetchChatKeyPackagesRequest.limit`
  - `ConsumeChatKeyPackageRequest.owner_account`
  - `ConsumeChatKeyPackageRequest.key_package_id`
  - `ConsumeChatKeyPackageRequest.requester_account`
- 验收接口：
  - 互联网聊天只走 `P-API-CITIZENAPP-003：CitizenApp Chat 瞬时转发`。
  - 近场聊天只走 `P-CHAT-002：CitizenApp Nearby Chat Transport`。
  - 区块链节点不承担聊天投递、密钥池或设备配对。
- 编码：外层 Protobuf；OpenMLS 标准 wire bytes 放入 `mls_wire_message`；链内 SCALE 不作为 Chat 主协议。
- 当前实现状态：Dart Protobuf、OpenMLS Rust FFI、Isar 本地消息库、Cloudflare 瞬时密文转发、WebRTC 附件和无内容推送后台收发均已落地；`ChatRuntime.ensureReady(ownerAccount)` 对同一账户/设备执行 single-flight，登录与设备登记只使用硬件 P-256 设备子钥，钱包 seed 和 CitizenWallet 均不进入聊天运行态。
- 签名/验签规则：
  - `ChatRoute` 是 Chat 模块内部路由缓存，不是第二套通讯录，不得替代“我的通讯录”联系人详情。
  - 公民端发消息必须读取用户资料中的聊天账户；未设置聊天账户不得发送。
  - 创建热钱包时由钱包主私钥一次性绑定硬件 P-256 设备子钥；聊天运行态不得读取钱包 seed。
  - Chat 设备绑定载荷固定为 `owner_account, device_id, device_public_key_hex, expires_at_millis, nonce` 的 SCALE bytes。
  - 签名字节固定为 `signing_message(OP_SIGN_CHAT_DEVICE_BIND=0x1A, payload)`；Worker 必须从 session 派生 owner，并用 `square_device_subkeys.p256_pubkey` 验签。
  - 客户端不得提交 owner 授权真源；CitizenWallet 不参与 Chat 设备绑定。
  - KeyPackage 由 Chat 设备密钥管理，必须具备 TTL、一次性领取即硬删除、防重放和注销清理。
  - 首次 MLS 会话发送会产生 Welcome + application 两条 wire message；Welcome 必须通过 `ChatEnvelope.ratchet_tree` 伴随传递 ratchet tree bytes。
  - Worker 必须校验 session sender、登记设备和 `ChatEnvelope.recipient_account`。
  - 附件字节和文件元数据不得进入 Worker、D1、KV、DO Storage 或 R2。
  - 近场 transport 只传输同一个 `ChatEnvelope`，不得另建明文近场消息格式。
- 存储边界：
  - CitizenApp 本地保存明文消息、OpenMLS provider storage、发送队列和路由缓存。
  - 删除聊天记录只删除当前设备本地会话、消息、发送队列、pending 入站记录和附件缓存，不删除联系人，不影响其他设备或对方设备。
  - Cloudflare D1 只保存设备登记、一次性 KeyPackage 和防重放哈希。
  - 发送失败的密文只保存在发送设备本机队列，Cloudflare 不提供远程补拉；推送后台窗口只建立瞬时连接、发送 `peer_ready` 和重试本机队列。
  - Android / iOS 近场 transport 不做长期服务端存储。
- 禁止事项：
  - 禁止把 CID 号码、实名信息、身份档案字段写入 Chat 协议。
  - 禁止把 Chat 路由缓存做成第二套通讯录。
  - 禁止复用钱包私钥作为 Chat 端到端加密密钥。
  - 禁止把私聊或群聊明文写入 Cloudflare、链、节点或广场公开表。
  - 禁止恢复区块链节点聊天或任何云端聊天内容存储。
- 必跑测试：`cargo test`（`citizenapp/rust`）、`flutter test --concurrency=1 test/chat`、Worker Chat API、Worker `/v1/chat/ws`、Protobuf round-trip、OpenMLS、P-256 重放、WebRTC 附件帧和远端无内容存储检查。
- 运行态 smoke：以两台真机验证前台直达、推送唤醒、本机队列恢复重试和 WebRTC 附件；任一发送设备长期离线时消息必须等待该设备恢复，不能上传云端代存。

### P-TX-001：PublicManage/PrivateManage.propose_create_{public,private}_institution

- 状态：当前(机构管理已拆分公权/私权两 pallet,取代旧 `OrganizationManage.propose_create_institution`)
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/entity/public-manage/src/lib.rs`(`propose_create_public_institution` call 5)
  - `citizenchain/runtime/entity/private-manage/src/lib.rs`(`propose_create_private_institution` call 5)
  - 两 call 参数形态完全相同（下列 11 字段），仅 pallet 前缀不同
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
  5. `admins`（`BoundedVec<InstitutionAdmin>`；每项字段顺序为 `admin_name + admin_account`，至少两人）
  6. `actor_cid_number`
- 编码：
  - SCALE call data
  - pallet index：公权机构=`30`(PublicManage),私权机构=`31`(PrivateManage);由 `institution_code` 经 `primitives::cid::code::is_private_legal_code` 派生(onchina `create_institution_pallet_index` 单源)
  - call index：`5`(两 pallet 同)
  - 前两个字节:公权=`[0x1e, 0x05]`(动作码 `0x1e05`)、私权=`[0x1f, 0x05]`(动作码 `0x1f05`)
- 签名/验签规则：
  - 创建机构不再有内层登记凭证；钱包只签最终链交易一次。
  - runtime 通过最终 extrinsic `origin + actor_cid_number` 查询对应注册局机构 admins，确认签名管理员属于该机构并通过注册局登记权限校验。
  - `admins` 的钱包账户必须唯一，姓名不能为空；钱包能解析公民姓名时使用公民姓名，否则使用“管理员”。姓名不参与授权。
  - runtime 从 CID 唯一派生机构码、全部强制协议账户和严格多数阈值；协议账户初始余额统一为零。
  - 名称分档：runtime 用 `primitives::cid::code::is_public_legal_code(institution_code)` 判定;公权/私权机构均必须带非空 `cid_full_name`+`cid_short_name` 并上链
  - 最终链交易签名覆盖机构名称、`admins` 完整 SCALE 载荷和 `town_code`；任一字段不得改包。
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

### P-TX-012：PublicManage/PrivateManage 机构治理与注册局登记管理员

- 状态：仅适用于机构维护/治理凭证；机构首次创建已退出本凭证
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/entity/public-manage/src/lib.rs`(`propose_institution_governance` call 8 / `register_institution_admins` call 9)
  - `citizenchain/runtime/entity/private-manage/src/lib.rs`(`propose_institution_governance` call 8 / `register_institution_admins` call 9)
  - `citizenchain/runtime/entity/entity-primitives/src/institution_governance.rs`(`InstitutionGovernanceAction`)
- 生产者：
  - `citizenchain/onchina/src/core/institution_call.rs`
- 消费者：
  - `citizenchain/runtime/entity/public-manage` / `citizenchain/runtime/entity/private-manage`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  - `propose_institution_governance(30/31.8)`：`cid_number`、`InstitutionGovernanceAction`、`register_nonce`、`signature`、`actor_cid_number`、`credential_signer_pubkey`、`scope_province_name`、`scope_city_name`
  - `register_institution_admins(30/31.9)`：`cid_number`、`admins(admin_name + admin_account)`、`register_nonce`、`signature`、`actor_cid_number`、`credential_signer_pubkey`、`scope_province_name`、`scope_city_name`
- 编码：
  - SCALE 裸 call data
  - pallet index：公权机构=`30`(PublicManage)，私权机构=`31`(PrivateManage)
  - call index：本机构治理=`8`，注册局直接登记管理员=`9`
  - 动作码：`0x1e08`、`0x1e09`、`0x1f08`、`0x1f09`
- 签名/验签规则：
  - 外层 `origin` 是持私钥的钱包管理员；机构本体和机构账户无私钥，不参与签名。
  - 本机构治理必须满足 `actor_cid_number == cid_number`，且 `origin` / `credential_signer_pubkey` 必须属于该 CID 的当前 `admins`。
  - 注册局直接登记管理员时，`actor_cid_number` 是注册局机构 CID，`origin` / `credential_signer_pubkey` 必须属于注册局机构 `admins`，并通过注册局登记权限校验。
  - 管理员集合变更完整替换 `admins`，普通机构至少两名管理员；姓名只展示，授权只看 `admin_account`。
  - `InstitutionGovernanceAction` 内的岗位任职来源只能是 `InstitutionGovernance`；普选、互选、任命结果必须由对应业务/投票引擎结果入口写入。
  - 交易费属于机构操作链上费 0.1 元，只从 `actor_cid_number` 的费用账户扣除；管理员钱包只签名，不允许回落扣管理员钱包。
- 禁止兼容：
  - 禁止恢复公权/私权机构管理员集合变更旧 call。
  - 禁止把岗位任职当管理员授权真源。
  - 禁止 OnChina 本地改库形成第二套管理员集合。
  - 禁止 CitizenWallet 解码后容忍尾字段或未知 `InstitutionGovernanceAction` 变体。
- 必跑测试：
  - `cargo check -p entity-primitives -p internal-vote -p public-manage -p private-manage -p citizenchain`
  - `cargo test -p citizenchain --lib`
  - `cargo test -p onchina core::institution_call`
  - `citizenwallet/test/signer/pallet_registry_test.dart`
  - `citizenwallet/test/signer/payload_decoder_test.dart`

### P-TX-010：AddressRegistry address payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/misc/address-registry/src/lib.rs`
  - `citizenchain/onchina/src/domains/address/chain_call.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/ADDRESS_REGISTRY_TECHNICAL.md`
  - `memory/05-modules/citizenchain/onchina/ADDRESS_TECHNICAL.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenchain/onchina/src/domains/address/chain_call.rs`
- 消费者：`citizenchain/runtime/misc/address-registry`
- 字段：
  - `set_catalog_version(33.0)`：`registrar_account`, `catalog_version`, `catalog_hash`
  - `set_address_name(33.1)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_name`
  - `remove_address_name(33.2)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`
  - `set_address(33.3)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail`
  - `remove_address(33.4)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail`
- 编码：
  - SCALE 裸 call data
  - pallet index：`33`
  - call index：`0..4`
  - 前两个字节：`[0x21, call_index]`
  - 动作码：`a=(33<<8)|call_index`,即 `0x2100..0x2104`
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
  - pallet index：`21`
  - call index：`1`
  - 前两个字节固定为 `[0x15, 0x01]`
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
- 唯一真源：`citizenchain/onchina/src/core/chain_runtime.rs`
- 详细文档：`memory/05-modules/citizenchain/node/offchain-transaction/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/core/chain_runtime.rs`
- 消费者：
  - `citizenchain/runtime/entity/public-manage` / `citizenchain/runtime/entity/private-manage`(链端维护/治理验签)
- 字段：
  - 外层业务字段：`cid_number`、`cid_full_name`、`cid_short_name`、法定代表人三字段、`account_names`、岗位与任职
  - 凭证字段：`credential.register_nonce`、`credential.actor_cid_number`、`credential.credential_signer_pubkey`、`credential.scope_province_name`、`credential.scope_city_name`、`credential.signature`
- 编码：
  - HTTP JSON 响应
  - runtime 验签 payload 按 OnChina 后端 `build_institution_registration_info_credential` 的 SCALE tuple 顺序
- 签名/验签规则：
  - OnChina 后端用签发机构管理员密钥签发。
  - 链端用 `actor_cid_number` 读取机构 admins pallet 的 `AdminAccounts[cid_number]`，确认 `credential_signer_pubkey` 属于该机构 `admins` 后验签。
  - `scope_province_name / scope_city_name` 只表示业务作用域，不表示签发人身份。
- 禁止兼容：不把 `subject_property / private_type / partnership_kind / parent_cid_number` 纳入链端注册凭证
- 禁止事项：
  - 禁止用普通机构详情接口替代 `/registration-info`
  - 禁止 CitizenApp 自己拼 `register_nonce / signature / actor_cid_number / credential_signer_pubkey / scope_*`
  - 禁止把机构首次创建重新接回本凭证；创建机构只允许最终链交易签名一次。
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
  - `citizenwallet` 公民钱包一次签名与响应二维码链路（不广播交易）
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
  - pallet index：`20`
  - call index：`0`
  - 前两个字节固定为 `[0x14, 0x00]`
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
  - pallet index：`21`
  - call index：`0`
  - 前两个字节固定为 `[0x15, 0x00]`
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
- 类型：投票引擎内部人口快照协议
- 唯一真源：`citizenchain/runtime/misc/citizen-identity`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
  - `memory/05-modules/citizenchain/runtime/misc/citizen-identity/CITIZEN_IDENTITY_TECHNICAL.md`
- 生产者：`citizen-identity` 内部 `CitizenIdentityReader`
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `citizenchain/runtime/votingengine/legislation-vote`
  - `citizenchain/runtime/votingengine/election-vote`
- 字段：
  1. `PopulationScope::Country`
  2. `PopulationScope::Province(province_code)`
  3. `PopulationScope::City(province_code, city_code)`
  4. `PopulationScope::Town(province_code, city_code, town_code)`
- 编码：只保存为 runtime 内部 `PopulationScope` 与 `snapshot_id`，不提供独立 SCALE
  extrinsic；CitizenIdentity call 5、JointVote call 2、LegislationVote call 0 永久留洞。
- 创建/授权规则：
  - 联合公投创建提案时内联创建全国快照。
  - 立法特别案只从已验证的 `actor_cid_number` 推导国家/省/市作用域后内联创建。
  - 快照创建、提案写入和绑定处于同一事务，任何失败全部回滚。
- 禁止兼容：开发期不兼容任何链下签发人口证明格式
- 禁止事项：
  - 禁止前端、OnChina 或冷钱包构造作用域、快照 ID 或人口分母。
  - 禁止恢复独立快照 action、payload、解码和两步准备流程。
  - 禁止跳过 runtime 链上人口读取。
- 必跑测试：
  - `citizenchain/runtime/src/tests/cases.rs` 中 population snapshot 相关测试
  - `citizenapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-TX-003：ResolutionIssuance.propose_issuance

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
  - 联合提案在同一建案事务内由 `joint-vote` 创建全国人口快照并绑定，不存在独立快照交易或载荷字段。
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
- 唯一真源：`citizenchain/runtime/transaction/multisig/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/transaction/multisig-transfer/MULTISIG_TRANSFER_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`citizenapp`
- 消费者：
  - `citizenchain/runtime/transaction/multisig`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- CI 同步：
  - `.github/workflows/citizenwallet-ci.yml` 必须从 `MultisigTransfer` / `multisig` 同步 `citizenwallet/lib/signer/pallet_registry.dart`
- 字段：
  - `propose_transfer(17.0)`：`org`、`account_id`、`beneficiary`、`amount`、`remark`
  - `propose_safety_fund_transfer(17.1)`：`beneficiary`、`amount`、`remark`
  - `propose_sweep_to_main(17.2)`：`account_id`、`amount`
- 编码：
  - SCALE call data
  - pallet index：`17`
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
  - `cargo test --manifest-path citizenchain/runtime/transaction/multisig/Cargo.toml`

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

### P-TX-007：PersonalAdmins.propose_admin_set_change

- 状态：当前（仅个人多签）
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/admins/personal-admins/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart`
- 消费者：
  - `citizenchain/runtime/admins/personal-admins`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `institution_code`
  2. `account_id`
  3. `admins`（`Vec<AccountId32>`）
  4. `new_threshold`
- 编码：
  - SCALE call data
  - pallet index：`29`（PersonalAdmins）
  - call index：`0`
  - 前两个字节：`[0x1d, 0x00]`
  - 布局：`institution_code:[u8;4] + account_id:[u8;32] + Compact<Vec<AccountId32>> + new_threshold:u32_le`
- 签名/验签规则：
  - `new_threshold` 是管理员更换通过后写入投票引擎的目标动态阈值。
  - 个人多签阈值必须满足 `threshold * 2 > admins_len && threshold <= admins_len`。
  - 公权/私权机构不使用本 call；其管理员钱包集合只能由 entity 岗位任职结果派生。
- 禁止兼容：不兼容缺少 `new_threshold` 的旧载荷。
- 禁止事项：
  - 禁止把本 call 用于任何公权或私权机构。
  - 禁止 CitizenWallet 公民钱包解码旧载荷或忽略尾部多余字节。
  - 禁止在 CitizenApp / CitizenWallet 内实现投票、计票或通过判定。
- 必跑测试：
  - `citizenapp/test/governance/admins-change/admins_change_codec_test.dart`
  - `citizenwallet/test/signer/payload_decoder_test.dart`

### P-FEE-001：五类费用路由与付款账户

- 状态：当前
- 类型：费用分类、付款关系与签名尾协议
- 唯一真源：`citizenchain/runtime/primitives/src/fee_policy.rs::FeeRoute<AccountId, Balance>`
- 唯一路由：`citizenchain/runtime/src/configs.rs::RuntimeFeeRouter`
- 收费执行器：
  - 链上交易费、实际投票费：`citizenchain/runtime/transaction/onchain/src/lib.rs::OnchainChargeAdapter`
  - 投票回调中的链上资金执行费：`citizenchain/runtime/transaction/onchain/src/lib.rs::OnchainExecutionFeeCharger`
  - 链下清算费：`citizenchain/runtime/transaction/offchain/`
- 生产者：runtime `RuntimeCall` 路由
- 消费者：runtime 收费执行器、node 事件 RPC、CitizenApp 签名构造、CitizenWallet 冷签校验
- 类型：
  - `Free`
  - `Onchain { transaction_amount, payer }`
  - `Offchain { fee_amount, payer: OffchainFeePayer::BatchItemPayers }`
  - `Vote { payer }`
  - `Reject`
- 规则：
  - 机构发起提案、机构资料操作和机构账户操作统一为 `Onchain`，由 `actor_cid_number` 下唯一 `InstitutionFee` 账户支付最低 0.1 元。
  - 机构具体账户交易必须同时验证 `institution_account` / `funding_account` 属于同一 actor CID。
  - 机构签名者必须属于 `AdminAccounts[actor_cid_number].admins`；管理员只负责签名，不是机构费用的回落付款人。
  - 实际 `cast_*` / 表决动作统一为 `Vote`，由投票签名者支付 1 元；发起提案不是投票。
  - Fullnode 不是机构，绑定或重绑奖励钱包由其自身签名者支付 0.1 元。
  - 链下清算批次允许多个付款公民；`BatchItemPayers` 明确表示每个 item 的 `payer` 从其 L2 存款支付该 item 的 `fee_amount`，收款方机构费用账户是收款账户，不得错误标成付款账户。
  - `TRANSACTION_TIP = 0`；非零 tip 在 runtime 入池/执行和 CitizenWallet 签名前双重拒绝。
  - `WeightToFee = 0`、`LengthToFee = 0`，不得引入五类之外的框架收费。
  - `OnchainFeeCharger` 只承接业务已核验的确切 `payer + transaction_amount`，不是第二套费用分类或付款路由；它必须复用 `fee_policy` 公式、统一分账与 `FeePaid` 事件。
  - 机构执行期手续费只从 actor CID 的费用账户支付，本金从明确的机构账户支付；个人多签执行期手续费由个人账户支付。扣费和本金变化必须原子执行，任何余额不足都失败，不得改扣管理员。
  - 任何路由、授权、CID/账户关系或费用账户解析失败都返回 `Reject`；不允许默认分类、可选付款人、付款人回落或双轨兼容。
- 签名尾：`era + Compact<nonce> + Compact<tip=0> + mode + additional_signed`；所有热签和冷签必须使用同一布局。
- 必跑测试：
  - `cargo test -p primitives -p onchain -p citizenchain -p chain-signing`
  - `cargo test -p node -p onchina`
  - CitizenApp / CitizenWallet `flutter test`、`flutter analyze`

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
  - 公权/私权机构 key：`cid_number`；value：`institution_code`、`admins: BoundedVec<AccountId32>`，不重复保存 CID、主账户或生命周期状态。
  - 个人多签 key：`personal_account`；value：独立 `AdminAccount`，字段为 `cid_number`（固定空）、`institution_code`、`kind`、`admins`、`creator`、`created_at`、`updated_at`、`status`。
  - 机构岗位、任期和任职来源不进入 AdminAccounts，统一读取 entity 的 `InstitutionRoles` 与 `InstitutionRoleAssignments`。
- 编码：
  - 机构 storage key：`twox128(PublicAdmins|PrivateAdmins) ++ twox128("AdminAccounts") ++ blake2_128_concat(cid_number)`。
  - 个人 storage key：`twox128(PersonalAdmins) ++ twox128("AdminAccounts") ++ blake2_128_concat(personal_account)`。
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自链上授权流程
- 禁止兼容：不兼容旧 `AdminsChange::Subjects / AdminsChange::Institutions` 当前路径
- 禁止事项：
  - 禁止恢复 `AdminsChange` 单 pallet 当前真源叙述
  - 禁止恢复机构 `AdminProfile`、机构 `creator/created_at/updated_at` 或岗位来源副本
  - 禁止只凭 `UNIN/SFGT/SFGP` 自动选择 `PrivateAdmins`
- 必跑测试：
  - `public-admins`、`private-admins`、`personal-admins` 单测
  - CitizenApp 多签发现相关测试

### P-STORAGE-002：PublicManage/PrivateManage.InstitutionAccounts

- 状态：当前(机构生命周期已拆 PublicManage(idx30)/PrivateManage(idx31),storage 名不变但前缀随 pallet 名变;取代旧 `OrganizationManage`)
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
- 同 pallet 的 `Institutions[cid_number] → InstitutionInfo`（2026-07-13 法定代表人三字段已收口到 entity）：
  - value 字段顺序固定为 `cid_full_name`、`cid_short_name`、`town_code`、`legal_representative_name: Option<_>`、`legal_representative_cid_number: Option<_>`、`legal_representative_account: Option<AccountId>`、`institution_code`、`created_at`（8 项，不保存重复生命周期状态）
  - 三个法定代表人字段必须同时有值或同时为空；首次注册创建与没有真实任免资料的创世机构全部为 `None`
  - 已删 `main_account`/`fee_account`/`admins`/`admins_len`/`threshold`/`creator`/`account_count`：主/费账户由派生且在 InstitutionAccounts;管理员真源 admins 模块;阈值真源 internal-vote
  - 消费方镜像须按机构码路由 PublicManage/PrivateManage 前缀；node、OnChina、CitizenApp 已按上述 8 字段 SCALE 顺序对齐
- 同 pallet 的岗位与任职 storage：
  - `InstitutionRoles[cid_number][role_code] → InstitutionRole`
  - `InstitutionRoleAssignments[cid_number][role_code] → BoundedVec<InstitutionAdminAssignment>`
  - `InstitutionRole` 字段顺序：`cid_number`、`role_code`、`role_name`、`term_required`、`role_status`
  - `InstitutionAdminAssignment` 字段顺序：`cid_number`、`admin_account`、`role_code`、`term_start:u32`、`term_end:u32`、`assignment_source`、`assignment_source_ref`、`assignment_status`
  - 普选/互选结果整体替换目标岗位的有效任职；任职账户必须属于同机构既有 admins，结果不得修改 admins storage。
- 编码：
  - double map storage key(前缀 = `twox_128(PublicManage|PrivateManage)` ++ `twox_128(InstitutionAccounts|Institutions)`)
  - value：SCALE
- 签名/验签规则：storage 本身不签名；call index 5 的机构创建最终链交易必须覆盖 `admins(admin_name + admin_account)` 完整 SCALE 载荷，任一姓名或账户被替换均验签失败
- 禁止兼容：不兼容旧 `Accounts` mirror、不兼容旧 `OrganizationManage` 前缀
- 禁止事项：
  - 禁止活跃代码继续读取已删的 `OrganizationManage::Institutions/InstitutionAccounts`
  - 禁止把机构账户当个人多签账户读取
- 必跑测试：
  - `cargo check -p node`(institution_read 前缀路由)
  - `cargo test --manifest-path citizenchain/node/Cargo.toml cid_lifecycle`（节点守卫镜像与三字段完整性）
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
  - 机构主体：`actor_cid_number` / `cid_number`
  - 机构具体资产操作账户：`institution_account` / `funding_account`
  - 个人多签主体：`personal_account`
  - `admins`
  - `admins_len`
  - `threshold`
  - `status`
- 编码：
  - 机构以 `cid_number` 作为 `AdminAccounts` 和 `ActiveInstitutionThresholds` key；主账户、费用账户和制度专属账户都只是该 CID 下的账户，不建立独立管理员集合。
  - 公权、私权、个人多签分别存放在 `PublicAdmins`、`PrivateAdmins`、`PersonalAdmins`。
  - 机构 `admins` 来自 entity 有效岗位任职；个人多签管理员集合继续由 `PersonalAdmins.propose_admin_set_change` 治理。
- 签名/验签规则：
  - 一人一票一笔交易；机构投票资格由创建提案时锁定的 CID 管理员快照决定，个人多签按 `personal_account` 快照决定。
  - 动态机构读取 `ActiveInstitutionThresholds[cid_number]`，个人多签读取 `ActivePersonalThresholds[personal_account]`；固定治理机构读取代码级固定阈值。
  - 机构任职结果只允许沿用既有阈值，不能创建或修改阈值制度。
- 禁止兼容：不保留机构 `AdminProfile`、机构 Pending 管理员变更或旧管理员集合 call
- 禁止事项：
  - 禁止省储行永久质押账户进入内部投票
  - 禁止给任一机构账户建立第二套管理员集合，禁止用主账户替代 CID 授权
  - 禁止业务模块绕过 entity 直接改机构 admins
  - 禁止把个人多签管理员模型混入机构岗位任职
- 必跑测试：
  - `cargo test -p public-admins -p private-admins -p personal-admins --lib`
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
- **pallet_index=35**（契约真源；当前 runtime 最高 idx=34，下一空号=35）
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
