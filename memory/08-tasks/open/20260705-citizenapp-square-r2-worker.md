任务需求：
- 按已确认架构设计并分步骤实现公民 App 广场功能。
- 广场用于发布图文、视频等动态，默认显示推荐内容，用户可切换关注、竞选等分类。
- 公民 App 用户唯一身份键为钱包账户 `owner_account`。
- 认证用户是已在注册局绑定 `cid_number` 并上链的钱包账户；非认证用户是未绑定 `cid_number` 的钱包账户。
- 认证用户和非认证用户都能发布普通动态、浏览广场、关注用户、使用同一套会员体系。
- 认证用户发布内容携带 `cid_number`；非认证用户发布内容不携带 `cid_number`。
- 认证用户能发布竞选分类内容；非认证用户不能发布竞选分类内容。
- 每条发布动态固定扣钱包 1 元发布费；发布费在公民链内处理，不再作为本任务争议点。
- 动态内容不存链上，不改造公民链全节点做内容存储。
- 广场内容存储主方案固定为 Cloudflare R2 + Cloudflare Worker + CDN；用户只在公民 App 内付会员费，不要求用户到外部存储网络分别付费。
- 开发流程必须分步骤执行；每一步执行前先输出完整技术方案，用户确认后再执行。

所属模块：
- citizenapp
- citizenchain/runtime
- cloudflare-square-worker
- memory

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/requirement-analysis-template.md
- memory/07-ai/thread-model.md
- memory/07-ai/unified-protocols.md
- memory/07-ai/unified-naming.md
- memory/07-ai/module-checklists/citizenapp.md
- memory/07-ai/module-definition-of-done/citizenapp.md
- memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md

新增任务卡确认记录：
- 用户已在当前任务明确允许创建本任务卡。
- 新增路径：`memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md`
- 用途：记录公民 App 广场的总技术方案、目录架构、分阶段实现计划、验收标准和风险边界。
- 原因：这是正式开发任务，必须先有 `memory/08-tasks/` 下任务卡。
- 是否会被 Git 跟踪：是。

核心边界：
- 不存在“公民链储存节点”这一设计，本任务不得引入该概念。
- 动态图片、视频、正文附件、封面、manifest 等内容不得存链上。
- 不得改造现有 CitizenChain 全节点承担广场媒体内容存储。
- 公民链只负责发布事实、扣 1 元发布费、竞选发布权限校验、发布索引和必要事件。
- Cloudflare R2 是本任务唯一内容存储主方案；不得在本任务内同时设计多套内容存储方案。
- R2 API key 不得放入 CitizenApp；上传必须通过 Worker 签发短期上传授权或代理上传。
- 用户会员身份绑定钱包账户，不绑定设备号、手机号或 Cloudflare 账户。
- 认证与非认证不是两套用户体系；唯一权限差异是竞选分类发布权限。
- 竞选分类权限必须以链上 `owner_account <-> cid_number` 绑定关系为真源，不能信任 App 或 Worker 自报。
- 涉及 `citizenchain/runtime/` 的任何修改，执行前必须单独列完整路径、预计改动内容和原因，并取得用户二次确认。
- 聊天功能只保留两种正式方式：Cloudflare 互联网聊天和手机近场聊天。
- 区块链节点通信节点聊天方式已删除，不再作为正式聊天链路继续扩展。
- 私聊和群聊必须使用 OpenMLS 端到端加密；Cloudflare 只能作为临时密文投递队列，未送达前短期保存密文、加密附件和必要投递元数据，接收端落本机并 ack 后必须删除 Cloudflare 临时副本。
- 广场点赞、公开评论、关注、举报、隐藏、不感兴趣属于公开互动，允许 Worker / D1 按明文业务数据处理。

目标总架构：

```text
CitizenApp
  ├─ 钱包账户 = 用户唯一身份键
  ├─ 认证状态 = 钱包账户是否绑定 cid_number
  ├─ 广场浏览 / 发布 / 关注 / 推荐 / 竞选分类
  ↓

Cloudflare Worker
  ├─ 钱包签名登录校验
  ├─ 会员校验、容量校验
  ├─ R2 上传授权
  ├─ 推荐流 / 关注流 / 竞选流 API
  ├─ 举报、隐藏、不感兴趣、点赞等行为记录
  ↓

Cloudflare R2
  ├─ 图片
  ├─ 视频
  ├─ 封面
  ├─ manifest.json
  ↓

Cloudflare CDN
  └─ 快速访问媒体内容

CitizenChain
  ├─ 发布交易入块
  ├─ 扣每条 1 元发布费
  ├─ 校验 campaign 必须是已绑定 cid_number 的认证用户
  └─ 只记录 post_id / owner_account / cid_number / post_category / content_hash / storage_receipt_id

CitizenApp Chat
  ├─ 互联网聊天：Cloudflare Worker + D1 临时密文 mailbox + R2 临时加密附件 + Durable Objects/WebSocket
  ├─ 近场聊天：蓝牙 / Wi-Fi 手机直连
  ├─ 统一身份：钱包地址 owner_account
  ├─ 统一加密：OpenMLS
  └─ 统一消息格式：GMB_IM_V1 / ImEnvelope
```

目标目录架构：

```text
GMB/
├── citizenapp/
│   ├── lib/
│   │   ├── 8964/                         # 广场唯一代码目录，现有架构文档已指定这里
│   │   │   ├── square_tab_page.dart       # 现有入口，改成真实广场首页
│   │   │   ├── models/                    # 动态、作者、媒体、会员、上传任务模型
│   │   │   ├── pages/                     # 推荐、关注、竞选、发布、详情、会员页
│   │   │   ├── widgets/                   # 动态卡片、媒体宫格、分类切换、作者标识
│   │   │   ├── services/                  # Worker API、上传、发布编排、推荐信号
│   │   │   ├── chain/                     # 广场发布交易、CID 绑定状态查询
│   │   │   └── storage/                   # 本地草稿、浏览缓存、上传状态
│   │   ├── isar/                          # 本地缓存 schema 和生成文件
│   │   └── rpc/                           # 复用现有链 RPC/交易提交能力
│   │
│   └── cloudflare/
│       (worker 文件已扁平化，直接位于 cloudflare/ 下，原 square_worker/ 层已移除；见卡 20260705-cloudflare-worker-flatten-dir)
│           ├── src/
│           │   ├── index.ts               # Worker HTTP 入口
│           │   ├── auth/                  # 钱包签名登录、session 校验
│           │   ├── membership/            # 会员等级、容量、到期时间
│           │   ├── uploads/               # R2 上传准备、完成、回执
│           │   ├── posts/                 # 动态发布索引、详情、删除/隐藏状态
│           │   ├── feeds/                 # 推荐、关注、竞选、最新流
│           │   ├── storage/               # R2 object key、manifest、hash 校验
│           │   ├── chain/                 # 同步链上发布事件、校验发布入块
│           │   ├── chat/                  # 私聊/群聊临时密文 mailbox、KeyPackage、ack 删除
│           │   ├── realtime/              # Durable Objects / WebSocket 新密文和评论推送
│           │   └── moderation/            # 举报、屏蔽、不感兴趣、降权
│           ├── migrations/                # D1 表结构迁移
│           ├── package.json               # Worker 工程依赖
│           ├── tsconfig.json              # TypeScript 配置
│           └── wrangler.toml              # Cloudflare 资源绑定名，不写 secret
│
├── citizenchain/
│   └── runtime/                           # 发布索引、1 元发布费、竞选权限校验
│       ├── otherpallet/
│       │   └── square-post/               # 广场发布 call、event、storage
│       └── src/
│           ├── lib.rs                     # 挂载 SquarePost pallet
│           ├── configs/mod.rs             # 复用/配置 1 元发布费和 8:1:1 分账
│           └── tests/                     # 发布费、竞选权限、字段校验测试
│
└── memory/
    ├── 01-architecture/citizenapp/        # 广场架构文档
    ├── 07-ai/unified-protocols.md         # 广场协议字段登记
    └── 08-tasks/open/                     # 本任务卡与执行记录
```

Cloudflare 目标资源：
- R2 bucket：`citizenapp-square-media`
- D1 database：`citizenapp-square-db`
- KV namespace：`citizenapp-square-feed-cache`
- Worker：`citizenapp-square-api`
- CDN/media domain：待部署阶段确认。

D1/KV 边界：
- R2 存媒体内容和 manifest。
- D1 存动态元数据、会员、关注关系、推荐信号、上传状态。
- KV 只做推荐流缓存和短期 session/cache，不作为长期内容存储。

核心身份模型：

```text
owner_account = 钱包账户
owner_account 已绑定 cid_number -> 认证用户
owner_account 未绑定 cid_number -> 非认证用户

post_category = normal:
  所有钱包账户都可发布

post_category = campaign:
  必须 owner_account 已绑定 cid_number
```

核心会员模型：

```text
membership.owner_account
membership.membership_level
membership.storage_quota_bytes
membership.storage_used_bytes
membership.expires_at
```

核心发布流程：

```text
1. CitizenApp 先用 finalized 余额校验钱包至少保留 ED + 1 元发布费。
2. CitizenApp 用钱包账户签名登录 Worker。
3. Worker 检查会员是否有效、容量是否足够、上传类型是否合法。
4. Worker 在 uploads/prepare 阶段生成 post_id、R2 object key 和 storage_receipt_id，但不写 R2 对象。
5. CitizenApp 提交链上 publish_square_post 交易，携带 post_id、content_hash、storage_receipt_id、storage_until。
6. CitizenChain runtime 扣 1 元发布费并按 8:1:1 分账。
7. CitizenChain runtime 校验竞选分类权限。
8. 交易入块后发出 SquarePostPublished 事件。
9. CitizenApp 只在扣费交易入块后上传图片/视频/封面/manifest 到 R2，并调用 uploads/complete。
10. Worker 校验上传对象和 content_hash，保留 prepare 阶段固定的 storage_receipt_id。
11. CitizenApp 调用 Worker 链上确认接口；Worker 交叉校验链上事件、D1 上传记录和 R2 manifest 后加入推荐、关注、竞选索引。
12. CitizenApp 从 Worker 拉取 feed，通过 CDN/R2 访问媒体。
```

R2 对象路径目标：

```text
square/{owner_account}/posts/{post_id}/manifest.json
square/{owner_account}/posts/{post_id}/cover.webp
square/{owner_account}/posts/{post_id}/media_001.webp
square/{owner_account}/posts/{post_id}/media_002.webp
square/{owner_account}/posts/{post_id}/video_001.mp4
```

链上发布字段目标：

```text
post_id
owner_account
cid_number              # 可空；认证用户由链上绑定关系派生，不由 App 伪造
post_category           # normal / campaign
content_hash
storage_receipt_id
storage_until
created_block
```

Worker 数据表目标：

```text
square_memberships
  owner_account
  membership_level
  storage_quota_bytes
  storage_used_bytes
  expires_at

square_uploads
  upload_id
  owner_account
  object_key
  content_hash
  storage_receipt_id
  status

square_posts
  post_id
  owner_account
  cid_number
  post_category
  text
  content_hash
  storage_receipt_id
  chain_block
  created_at

square_follows
  owner_account
  followed_account

square_user_signals
  owner_account
  post_id
  signal_type
  weight
  created_at

chat_devices
  owner_account
  device_id
  device_public_key_hex
  binding_signature
  expires_at
  revoked_at

chat_keypackages
  owner_account
  device_id
  key_package_id
  key_package
  cipher_suite
  created_at
  expires_at
  consumed_at

chat_conversations
  conversation_id
  conversation_type
  created_by_account
  created_at

chat_members
  conversation_id
  member_account
  member_device_id
  role
  joined_at
  removed_at

chat_envelopes
  envelope_id
  conversation_id
  sender_account
  sender_device_id
  recipient_account
  recipient_device_id
  mls_message_kind
  encrypted_payload
  attachment_manifest_key
  created_at
  expires_at
```

推荐流初始算法：

```text
score =
  新鲜度
+ 关注加权
+ 用户浏览、点赞、停留、点开详情等正反馈
+ 同地区或同 CID 相关性
- 用户隐藏、不感兴趣、快速划过等负反馈
- 被举报降权
```

推荐流初期只做可控规则，不做黑盒模型。

预计修改目录：
- `memory/08-tasks/open/`：记录本任务方案、执行记录、验收结果；文档。
- `memory/01-architecture/citizenapp/`：后续写入广场架构文档，记录存储、会员、发布和推荐边界；文档。
- `memory/07-ai/unified-protocols.md`：后续登记广场 Worker API、链上发布载荷、字段命名；文档。
- `citizenapp/lib/8964/`：实现广场首页、推荐流、关注流、竞选流、发布页、详情页、会员状态、上传状态；代码和残留清理。
- `citizenapp/lib/isar/`：增加本地草稿、上传任务、浏览缓存、推荐信号缓存；代码和生成文件。
- `citizenapp/lib/rpc/`：复用现有链 RPC 与交易提交管线，增加广场发布交易调用边界；代码。
- `citizenapp/cloudflare/`：新增 Cloudflare Worker 工程，承载登录校验、会员、R2 上传、feed API、D1/KV；代码和配置。
- `citizenchain/runtime/`：后续新增广场发布索引、1 元发布费、竞选权限校验；代码、测试、生成物，执行前必须二次确认。
- `citizenapp/lib/im/`：统一私聊/群聊运行态，保留 OpenMLS，删除区块链节点聊天链路，接入 Cloudflare 和近场 transport；代码和残留清理。
- `citizenapp/android/im/`、`citizenapp/ios/im/`：实现无互联网近场聊天；原生代码。
- 旧 `citizenchain/node/src/im/`、旧 `citizenchain/node/frontend/settings/communication-node/`：已删除区块链节点聊天实现和设置入口；代码和残留清理，不涉及 runtime。

分阶段执行计划：

## 阶段 0：任务卡与技术方案固化

目标：
- 创建本任务卡。
- 固化总架构、目录架构、边界、分阶段计划。

执行边界：
- 只创建本任务卡。
- 不改代码。
- 不创建 Worker 工程。
- 不修改 runtime。

验收标准：
- 本任务卡存在。
- 本任务卡包含目标架构、目录架构、阶段计划、预计修改目录和风险边界。

## 阶段 1：协议与数据结构方案

目标：
- 输出并确认 Worker API、R2 object key、D1 表、App 本地缓存字段、链上发布字段。
- 登记统一协议和字段命名。

执行前必须确认：
- 是否允许修改 `memory/07-ai/unified-protocols.md`。
- 是否允许新增或修改 `memory/01-architecture/citizenapp/` 下广场架构文档。
- 是否允许修改现有 CitizenApp 技术文档。

预计修改：
- `memory/07-ai/unified-protocols.md`
- `memory/01-architecture/citizenapp/`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`

验收标准：
- 协议字段不出现同义多名。
- 链上字段和 Worker 字段边界清晰。
- 不出现“内容上链”“公民链储存节点”“改造全节点存媒体”等错误口径。

阶段 1 执行记录：
- 已更新 `memory/07-ai/unified-protocols.md`，新增 `P-API-CITIZENAPP-002：CitizenApp Square Worker / R2 契约`。
- 已更新 `memory/07-ai/unified-protocols.md`，新增 `P-TX-013：Square.publish_square_post` 草案。
- 已更新 `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`，把广场 Tab 从入口壳口径扩展为目标架构口径。
- 已明确 Worker API、R2 object key、R2 manifest、D1 表、CitizenApp 本地缓存字段和链上发布字段。
- 阶段 1 未新增文件，未改代码，未修改 `citizenchain/runtime/`。

## 阶段 2：CitizenApp 广场前端壳

目标：
- 把现有 `citizenapp/lib/8964/square_tab_page.dart` 从“暂未开放”改为真实广场入口壳。
- 做推荐、关注、竞选、发布、详情的页面骨架。
- 发布页能根据认证状态限制竞选分类入口。
- 按用户追加确认，把底部“广场”和“公民”Tab 对调：广场移到最左，公民移到右侧，App 启动默认进入广场推荐页。

执行前必须确认：
- 新增文件清单。
- 页面结构和底部广场入口交互。
- 是否先接 mock 数据，还是直接等 Worker API。

预计修改：
- `citizenapp/lib/8964/`
- `citizenapp/lib/main.dart` 如入口 import 需要调整
- `memory/01-architecture/citizenapp/`

验收标准：
- App 底部广场 tab 进入真实广场页面。
- 默认选中推荐。
- 可切换关注、竞选。
- 非认证用户看得到竞选内容入口，但发布竞选内容时被明确拦截。
- 页面无重叠、无白屏。
- 完成真实运行态页面验收。

阶段 2 执行记录：
- 已新增 `citizenapp/lib/8964/models/square_models.dart`，定义广场 feed、动态分类、媒体、作者、动态模型。
- 已新增 `citizenapp/lib/8964/services/square_identity_state.dart`，阶段 2 复用当前钱包地址作为 `owner_account`，`cid_number` 链上查询留到后续阶段。
- 已新增 `citizenapp/lib/8964/pages/square_home_page.dart`，广场默认进入推荐页，并支持推荐、关注、竞选切换。
- 已新增 `citizenapp/lib/8964/pages/square_compose_page.dart`，发布页支持普通/竞选分类选择；当前未接链上 CID 时按未认证禁用竞选。
- 已新增 `citizenapp/lib/8964/pages/square_post_detail_page.dart`，提供动态详情前端壳。
- 已新增 `citizenapp/lib/8964/widgets/` 下的 feed tab、动态卡片、媒体宫格和空状态组件。
- 已修改 `citizenapp/lib/8964/square_tab_page.dart`，移除“暂未开放”，挂载真实广场首页壳。
- 已修改 `citizenapp/lib/main.dart`，底部导航顺序改为 `广场 / 公民 / 信息 / 交易 / 我的`，默认 `_currentIndex = 0`。
- 已新增 `citizenapp/test/8964/square_home_page_test.dart` 覆盖默认推荐页、分类切换和未认证钱包发布竞选禁用。
- 已更新 `citizenapp/test/widget_test.dart` 的启动冒烟预期，要求首屏出现广场推荐空状态。
- 已更新 `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md` 的导航顺序和广场当前实现口径。
- 阶段 2 未修改 `citizenchain/runtime/`，未创建 Cloudflare Worker，未改 Isar schema，未实现真实上传或链上发布交易。

阶段 2 验收结果：
- `dart format lib/8964 test/8964 test/widget_test.dart lib/main.dart`：通过。
- `flutter analyze lib/8964 lib/main.dart test/8964 test/widget_test.dart`：通过。
- `flutter test test/8964/square_home_page_test.dart test/widget_test.dart`：通过。
- `flutter analyze` 全量：未通过，唯一问题为既有未触碰文件 `citizenapp/lib/transaction/onchain-transaction/onchain_payment_service.dart:43:13 prefer_const_constructors`。
- `flutter run -d 3C071JEKB09000 --debug`：Debug APK 构建、安装并启动到 Pixel 8a，`org.citizenapp.MainActivity` 可见；但设备处于系统锁屏/通知层，ADB 无法解锁，截屏因 App 截屏保护为黑屏，UIAutomator 只能读到锁屏内容，未能完成广场页面人工验收。
- `flutter run -d macos --debug`：未配置 macOS desktop project，不能用于运行态验收。
- `flutter run -d chrome --debug`：项目不支持 web，且 `dart:ffi` / Isar / smoldot 依赖导致编译失败，不能用于运行态验收。
- 用户已在 2026-07-05 回复“复验通过了”，阶段 2 真机页面验收完成：启动首屏为广场推荐页、底部顺序为 `广场 / 公民 / 信息 / 交易 / 我的`、发布页未认证钱包禁用竞选发布入口。

## 阶段 3：Cloudflare Worker + R2 上传服务

目标：
- 新增 Cloudflare Worker 工程。
- 实现钱包签名登录、会员校验、容量校验、R2 上传准备、上传完成、manifest/hash 校验。

执行前必须确认：
- 新增目录 `citizenapp/cloudflare/`。
- Worker 技术栈、依赖和 wrangler 配置。
- D1/KV/R2 资源绑定名。
- 本地开发和测试方式。

预计修改：
- `citizenapp/cloudflare/`
- `memory/01-architecture/citizenapp/`
- `memory/07-ai/unified-protocols.md`

验收标准：
- 本地 Worker 可启动。
- 钱包签名登录接口可验签。
- 无有效会员或容量不足时不能上传。
- 上传成功后得到 `storage_receipt_id`。
- R2 secret 不进入仓库。

阶段 3 执行记录：
- 已新增 `citizenapp/cloudflare/` Cloudflare Worker 工程。
- 已新增 `package.json`、`package-lock.json`、`tsconfig.json`、`vitest.config.ts`、`wrangler.toml` 和 `.gitignore`；`.gitignore` 已忽略 `.dev.vars`、`.wrangler/`、`node_modules/`、`coverage/`、`dist/`、`*.tsbuildinfo`。
- 已新增 `migrations/0001_square_core.sql`，创建 `square_login_challenges`、`square_memberships`、`square_uploads`、`square_posts`、`square_follows`、`square_user_signals`。
- 已新增 `src/auth/`，实现钱包登录挑战、签名验签、session 写入 KV。验签直接导入 `@polkadot/util-crypto/signature/verify`，避免 Worker 顶层触发 wasm 初始化。
- 已新增 `src/membership/`，实现会员读取、有效期检查、容量检查和上传完成后的容量累计。
- 已新增 `src/storage/`，实现 R2 object key 规划、生产 R2 S3 预签名 PUT URL、本地开发上传代理 URL。
- 已新增 `src/uploads/`，实现上传准备、媒体类型/大小校验、R2 对象存在性校验、`storage_receipt_id` 生成。
- 已新增 `src/posts/`、`src/feeds/`、`src/moderation/`，实现推荐/关注/竞选 feed 查询、关注/取消关注、点赞/隐藏/不感兴趣/举报等推荐信号记录。
- 已新增 `test/` 下 Worker 单元测试，覆盖 R2 object key、登录 payload 和上传校验。
- 阶段 3 未部署 Cloudflare 远端资源，未创建真实 R2 bucket/D1/KV，未写入任何 Cloudflare token 或 R2 secret。
- 阶段 3 未修改 `citizenchain/runtime/`，未实现链上 `publish_square_post`，未实现链上事件同步，未接入 CitizenApp 真实上传 UI。

阶段 3 验收结果：
- `npm install`：通过，生成 `package-lock.json`；`node_modules/` 被 `.gitignore` 忽略。
- `npm run typecheck`：通过。
- `npm test`：通过，3 个测试文件、4 个测试用例全部通过。
- `npm run migrate:local`：通过，Wrangler 本地 D1 成功执行 `0001_square_core.sql`。
- `npm run dev -- --port 8787`：通过，本地 Worker 启动于 `http://localhost:8787`。
- `curl http://127.0.0.1:8787/health`：通过，返回 `{"ok":true,"service":"citizenapp-square-api","storage_backend":"cloudflare-r2","content_on_chain":false}`。
- 本地临时 sr25519 钱包完成 `challenge -> signature -> session`：通过，Worker 返回 `session_token`。
- `npx wrangler dev --local --port 8788 --var SQUARE_DEV_UPLOAD_PROXY:1`：通过，本地开发上传代理启动。
- 本地 D1 插入临时会员记录后，完成 `uploads/prepare -> dev-put manifest -> dev-put media -> uploads/complete`：通过，返回 `storage_state=completed` 和 `storage_receipt_id`。
- `git status --short --untracked-files=all citizenapp/cloudflare`：确认未跟踪候选只包含源码、配置、迁移、测试和 `package-lock.json`，不包含 `.wrangler/` 与 `node_modules/`。

## 阶段 4：CitizenChain 发布索引

目标：
- 新增链上发布交易。
- 每条发布扣 1 元。
- 按既定 8:1:1 规则分账。
- 竞选分类必须校验 `owner_account` 已绑定 `cid_number`。
- 只记录发布索引和哈希，不存动态内容。

执行前必须确认：
- `citizenchain/runtime/` 二次确认。
- 完整 runtime 修改路径。
- 发布费复用现有费用配置还是新增专用常量。
- `cid_number` 绑定关系读取路径。

预计修改：
- `citizenchain/runtime/otherpallet/square-post/`
- `citizenchain/Cargo.toml`
- `citizenchain/runtime/Cargo.toml`
- `citizenchain/runtime/src/lib.rs`
- `citizenchain/runtime/src/configs/mod.rs`
- `citizenchain/runtime/src/tests/`
- 可能涉及 runtime primitives 或 metadata 生成物，执行前另列清单。

验收标准：
- 普通用户可发布 normal。
- 未认证用户不能发布 campaign。
- 认证用户可发布 campaign。
- 每次发布扣 1 元。
- 分账符合 8:1:1。
- 链上事件足够 Worker 同步 feed。
- 不出现任何媒体内容上链。

阶段 4 执行记录：
- 用户已明确回复“确认执行阶段 4”，满足 `citizenchain/runtime/` 二次确认硬规则。
- 已新增 `citizenchain/runtime/otherpallet/square-post/`，作为广场动态链上发布索引 pallet。
- 已新增 `SquarePostCategory::Normal/Campaign`、`SquarePost` 索引结构、`SquarePosts`、`PublishedPostCountByAccount` 和 `SquarePostPublished` 事件。
- 已实现 `publish_square_post(post_id, post_category, content_hash, storage_receipt_id, storage_until)`；`owner_account` 由 signed origin 派生，`cid_number` 由 runtime 从 `CitizenIdentity::VotingIdentityByAccount` 派生。
- 已实现规则：普通动态所有钱包账户可发布；竞选动态必须有 `citizen_status = Normal` 的链上公民身份。
- 已在 `citizenchain/runtime/src/lib.rs` 挂载 `SquarePost`，pallet index 为 `36`。
- 已在 `RuntimeFeeKindClassifier` 把 `RuntimeCall::SquarePost(_)` 归类为 `VoteFlat`，复用现有 `VOTE_FLAT_FEE = 100 分 = 1 元` 和现有 `OnchainFeeRouter` 的 80/10/10 分账，不新增第二套收费或分账逻辑。
- 阶段 4 未把图片、视频、正文附件、封面或 manifest 写入 runtime storage。
- 阶段 4 未修改 runtime primitives，未生成 metadata，未改 CitizenApp 交易编码，未实现 Worker 链上事件同步。

阶段 4 验收结果：
- `cargo fmt --manifest-path citizenchain/Cargo.toml --all`：通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p square-post`：通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p square-post`：通过，6 个测试通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`：通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p citizenchain square_post`：通过，4 个 runtime 集成测试通过。
- 真实运行态验收：`CITIZENCHAIN_HEADLESS=1 citizenchain/target/debug/citizenchain --dev --tmp --rpc-port 9946 --port 30339` 成功启动本地无头 dev 节点，`chain_getBlockHash(0)` 返回 `0xb57c61a97f2b1fd7fa78756060a0c3e9a0ed6b1048bb8424b034a8f5f99a9971`。
- 运行态说明：第一次未设置 `CITIZENCHAIN_HEADLESS=1` 时 macOS 入口进入桌面壳，未开启 RPC；已按节点入口规则改用无头模式完成复验。

## 阶段 5：App 发布闭环

目标：
- CitizenApp 串起上传 R2、拿回执、提交链上发布、等待入块；入块后只返回本地动态，正式 feed 入库等 Worker 链上事件同步阶段完成。

执行前必须确认：
- App 与 Worker API 对接字段。
- 链上交易调用字段。
- 上传失败、链上失败、重复提交的处理策略。

预计修改：
- `citizenapp/lib/8964/services/`
- `citizenapp/lib/8964/chain/`
- `citizenapp/lib/rpc/`
- `citizenapp/lib/8964/pages/`
- `citizenapp/test/8964/`
- `memory/01-architecture/citizenapp/`
- `memory/07-ai/unified-protocols.md`
- `memory/08-tasks/open/`

验收标准：
- 发布普通图文动态完整跑通。
- 发布视频动态完整跑通。
- 非认证用户发布竞选动态被拦截。
- 认证用户发布竞选动态完整跑通。
- 链上交易未成功时，feed 不展示正式动态。

阶段 5 执行记录：
- 用户已明确回复“确认执行阶段 5”，并确认新增 App 文件路径。
- 已新增 `citizenapp/lib/8964/chain/square_chain_service.dart`，实现 `SquarePost.publish_square_post` call data 编码、提交入块、`CitizenIdentity::VotingIdentityByAccount` CID 查询和 Normal 身份解码。
- 已新增 `citizenapp/lib/8964/services/square_api_client.dart`，实现 Worker 钱包登录挑战/session、会员状态、上传准备、R2 PUT、上传完成接口适配。
- 已新增 `citizenapp/lib/8964/services/square_upload_service.dart`，实现 App 规范化内容 manifest、媒体 sha256、R2 manifest/媒体上传、`storage_receipt_id` 获取；`storage_until` 使用 Worker 会员 `expires_at`。
- 已新增 `citizenapp/lib/8964/services/square_publish_service.dart`，串联基础发布闭环；发布顺序已在阶段 10 改为“链上扣费入块后再上传 R2”。
- 已修改 `citizenapp/lib/8964/services/square_identity_state.dart`，认证状态改为读取链上 CID，查不到或状态非 Normal 时按未认证处理。
- 已修改 `citizenapp/lib/8964/pages/square_compose_page.dart`，接入图片/视频选择、热钱包本机签名、冷钱包 QR 签名、Worker 上传、链上发布和发布阶段状态展示。
- 已修改 `citizenapp/lib/8964/pages/square_home_page.dart`，发布页返回入块后的本地动态后插入当前页面列表；正式 feed 仍等待 Worker 链上事件同步。
- 已新增 `citizenapp/test/8964/square_chain_service_test.dart` 和 `citizenapp/test/8964/square_publish_service_test.dart`，覆盖交易编码、CID 解码、未认证竞选拦截和基础发布编排。
- 阶段 5 未修改 `citizenchain/runtime/`，未修改 Worker 服务端协议，未新增 Isar schema，未实现 Worker 链上事件同步。

阶段 5 验收结果：
- `dart format citizenapp/lib/8964/... citizenapp/test/8964/...`：通过。
- `flutter analyze lib/8964 test/8964`：通过。
- `flutter test test/8964/square_chain_service_test.dart test/8964/square_publish_service_test.dart test/8964/square_home_page_test.dart`：通过。
- 真实运行态验收：`flutter run -d 3C071JEKB09000 --debug` 成功构建、安装并启动到 Pixel 8a，Dart VM Service 正常暴露；验收后已用 `q` 退出运行进程。
- 运行态限制：本次未做真实发布端到端，因为需要有效会员 D1 数据、可达 Worker 地址、可同步的链节点、可支付 1 元的钱包余额和测试媒体；阶段 5 已完成 App 端闭环代码与真机启动验收，正式 feed 入库留给阶段 6。

## 阶段 6：链上确认与正式 feed

目标：
- Worker 读取指定区块 `System.Events`，确认 `SquarePostPublished` 事件与已完成上传记录一致。
- Worker 把通过链上事件确认的动态写入 `square_posts`，成为推荐、关注、竞选 feed 的正式内容。
- App 发布交易入块后调用 Worker 确认接口，返回 Worker/D1/R2 组合出的正式动态。
- App 广场首页默认从 Worker 推荐 feed 拉取内容，用户切换关注、竞选分类时读取对应 feed。

执行前必须确认：
- Worker 链上 RPC 地址由部署环境提供，不写入仓库。
- Worker 只能信任入块事件，不能信任 App 自报发布成功。
- 本阶段不修改 `citizenchain/runtime/`，只按阶段 4 已落地事件解码。

预计修改：
- `citizenapp/lib/8964/pages/`
- `citizenapp/lib/8964/widgets/`
- `citizenapp/lib/8964/services/`
- `citizenapp/cloudflare/src/chain/`
- `citizenapp/cloudflare/src/posts/`
- `citizenapp/cloudflare/src/feeds/`
- `citizenapp/test/8964/`
- `citizenapp/cloudflare/test/`
- `memory/01-architecture/citizenapp/`
- `memory/07-ai/unified-protocols.md`
- `memory/08-tasks/open/`

验收标准：
- Worker `POST /v1/square/posts/confirm` 只能在指定区块存在匹配 `SquarePostPublished` 事件时成功。
- Worker 写入 `square_posts.post_state = published` 后，推荐 feed 能返回正式动态。
- 关注 feed 未登录时返回空列表；登录后只展示已关注账户的正式动态。
- 竞选 feed 只展示 `post_category = campaign` 的正式动态。
- App 发布链上入块后调用 Worker 确认接口，不能只把本地草稿当作正式动态。
- App 默认进入推荐页并从 Worker feed 刷新，切换关注/竞选时读取对应 Worker API。
- Worker 和 App 测试覆盖事件解码、发布确认、feed 解析和页面刷新。

阶段 6 执行记录：
- 用户已明确回复“确认执行阶段 6”。
- 已新增 `citizenapp/cloudflare/src/chain/rpc.ts`，实现按 `SQUARE_CHAIN_RPC_URL` 调用链 RPC，读取指定区块 `System.Events`；RPC 地址属于环境配置，不写入仓库。
- 已新增 `citizenapp/cloudflare/src/chain/square_event.ts`，按阶段 4 固定的 pallet index `36`、event index `0` 解码 `SquarePostPublished` 事件，读取 `post_id`、`owner_account`、可空 `cid_number`、`post_category`、`content_hash`、`storage_receipt_id`、`storage_until`、`created_block`。
- 已新增 `citizenapp/cloudflare/src/posts/confirm.ts`，实现 `POST /v1/square/posts/confirm`：校验登录 session、上传记录归属、上传完成状态、链上事件字段、R2 manifest schema/owner/category，然后写入 `square_posts` 并返回正式 feed item。
- 已修改 `citizenapp/cloudflare/src/posts/repository.ts`，feed 只读取 `post_state = published` 的正式动态；推荐按发布时间倒序，关注按 D1 关注关系过滤，竞选只返回 `campaign`；feed item 会从 R2 manifest 和上传对象列表补齐 `media_items`。
- 已修改 `citizenapp/cloudflare/src/routes.ts` 和 `src/types.ts`，登记确认发布路由、链 RPC 环境变量和 feed 媒体字段。
- 已修改 `citizenapp/lib/8964/services/square_api_client.dart`，新增 `SquareFeedSource`、`SquarePublicationConfirmer`，实现 `fetchFeed()` 和 `confirmPublishedPost()`。
- 已修改 `citizenapp/lib/8964/services/square_publish_service.dart`，链上入块后调用 Worker 确认接口，返回 Worker 确认后的正式动态。
- 已修改 `citizenapp/lib/8964/services/square_upload_service.dart`，上传结果保留 Worker session，供发布确认接口复用。
- 已修改 `citizenapp/lib/8964/pages/square_home_page.dart`，广场首页默认读取 Worker 推荐 feed，切换关注/竞选时读取对应 feed，发布完成后先插入返回动态再刷新 Worker feed。
- 已新增 `citizenapp/cloudflare/test/chain_confirm.test.ts`，覆盖事件解码和发布确认入库。
- 已新增 `citizenapp/test/8964/square_feed_service_test.dart`，覆盖 App 解析 Worker feed 动态和媒体列表。
- 已更新 `citizenapp/test/8964/square_publish_service_test.dart` 与 `citizenapp/test/8964/square_home_page_test.dart`，适配 Worker 确认接口和 feed 注入。
- 阶段 6 未修改 `citizenchain/runtime/`，未部署 Cloudflare 远端资源，未写入任何 Cloudflare token、R2 access key、R2 secret key 或链 RPC 私密地址。

阶段 6 验收结果：
- `dart format citizenapp/lib/8964/models/square_models.dart citizenapp/lib/8964/pages/square_home_page.dart citizenapp/lib/8964/services/square_api_client.dart citizenapp/lib/8964/services/square_publish_service.dart citizenapp/lib/8964/services/square_upload_service.dart citizenapp/test/8964/square_publish_service_test.dart citizenapp/test/8964/square_home_page_test.dart citizenapp/test/8964/square_feed_service_test.dart`：通过。
- `npm --prefix citizenapp/cloudflare run typecheck`：通过。
- `npm --prefix citizenapp/cloudflare test`：通过，4 个测试文件、6 个测试用例全部通过。
- `flutter analyze lib/8964 test/8964`：通过。
- `flutter test test/8964/square_chain_service_test.dart test/8964/square_publish_service_test.dart test/8964/square_feed_service_test.dart test/8964/square_home_page_test.dart`：通过。
- `npm --prefix citizenapp/cloudflare run migrate:local`：通过，Wrangler 本地 D1 迁移成功。
- `npm --prefix citizenapp/cloudflare run dev -- --port 8787`：通过，本地 Worker 启动于 `http://localhost:8787`。
- `curl http://127.0.0.1:8787/health`：通过，返回 `ok=true`、`service=citizenapp-square-api`、`storage_backend=cloudflare-r2`、`content_on_chain=false`。
- `curl http://127.0.0.1:8787/v1/square/feed/recommended?limit=5`：通过，返回空推荐 feed。
- `curl http://127.0.0.1:8787/v1/square/feed/following?limit=5`：通过，未登录关注 feed 返回空列表。
- `curl http://127.0.0.1:8787/v1/square/feed/campaign?limit=5`：通过，返回空竞选 feed。
- 验收后已退出 Worker dev 进程；本阶段真实 HTTP 验收确认 feed API 可运行。完整“真实用户上传媒体、链上扣款入块、Worker 确认入库、App 真机刷新”端到端验收需要可用会员 D1 数据、可访问链 RPC、可支付测试钱包和测试媒体，归入阶段 7。

## 阶段 7：真实运行态验收与文档回写

目标：
- 完成 App、Worker、R2、本地链的真实运行态验收。
- 更新架构文档、协议文档、任务卡执行记录。
- 清理残留。

预计验收：
- Flutter analyze。
- Flutter test。
- Worker 单元测试。
- Worker 本地启动测试。
- R2 上传/读取真实或本地模拟测试。
- 本地链发布交易测试。
- App 真机或模拟器页面验收。
- `git diff --check`。

最终验收标准：
- 广场能浏览推荐、关注、竞选内容。
- 用户能发布图文和视频动态。
- 发布普通动态所有钱包账户可用。
- 发布竞选动态仅认证用户可用。
- 发布交易扣 1 元并入块。
- 媒体内容长期存 R2，不上链。
- App 不持有 R2 密钥。
- 会员按钱包账户管理容量和到期时间。
- 文档、协议、任务卡同步完成。
- 无旧口径残留。

阶段 7 最终执行记录：
- 用户已明确回复“确认执行阶段 7”。
- 已恢复 Worker 本地依赖并启动 `citizenapp-square-api` Worker，绑定本地 D1、R2、KV、本地上传代理和 `SQUARE_CHAIN_RPC_URL=http://127.0.0.1:9946`。
- 已完成 Worker/R2 真实前置链路：用 Alice 钱包账户完成 Worker 钱包签名登录；写入本地 D1 会员；调用 `uploads/prepare`；通过 `dev-put` 上传 manifest 和 image media 到本地 R2；调用 `uploads/complete` 获取 `storage_receipt_id`。
- 本次前置链路生成的测试动态：`post_id=sqp_85dd4370c0f94e9587dc47ae3e62b27d`，`content_hash=3185c925084efdf8e475ab6178785ea685e1fb2e5e19d39b874edc62974efcd9`，`storage_receipt_id=sqr_cac0c5ab132a80d01c7d693b44a8497ea1acf5ba1ff61b6eef096b8bc4075d48`。
- 已构建普通本地节点二进制并启动冻结 `--dev` 链；验收发现冻结 chainspec 仍使用旧 WASM，metadata 中只有 pallet index `0..35`，没有 `SquarePost`。
- 已进一步用 `WASM_BUILD_FROM_SOURCE=1 cargo build --manifest-path citizenchain/Cargo.toml -p node --bin citizenchain` 构建带当前源码 WASM 的节点二进制。
- 已启动 `--chain citizenchain-fresh` 本地链，metadata 已确认 `36:SquarePost` 存在，`api.tx.squarePost.publishSquarePost` 可见。
- `--chain citizenchain-fresh` 默认 genesis 不给 Alice/Bob 等开发账户余额，无法直接真实提交需要扣 1 元发布费的 `SquarePost.publish_square_post`。
- 已尝试用进程替换给 fresh spec 追加 Alice 测试余额，但 Substrate `--chain` 会 mmap spec 文件，不能读取 `/dev/fd/*`，报错 `Invalid input: Error mmaping spec file /dev/fd/12`。
- 用户已明确回复“允许临时 spec”，随后创建仓库外临时文件 `/tmp/citizenchain-square-e2e-spec.json`，用途为本地阶段 7 验收 chainspec，追加 Alice 测试余额；该文件不在仓库内，不会被 Git 跟踪，验收后删除。
- 临时 spec 生成自 `citizenchain-fresh`，并追加 Alice `w5GP3VQJbiMN5Vbg69e8xV1Kkgoroz5obJhpojqaBVNbwom9c` 余额 `1000000000`；链 metadata 验证 `hasSquarePost=true`、`publishSquarePost` 存在、Alice 余额可读。
- 单节点本地 fresh 链无法产块的原因已定位：`citizenchain/node/src/core/service.rs` 的 PoW `pool_ready()` 在 `sync_service.is_offline()` 或 major syncing 时返回 0；无 peer 的本地单节点即使交易进入 pending，也不会触发矿工打包。
- 已使用两节点本地互连完成验收：A 节点 `rpc-port=9946/port=30339`，B 节点 `rpc-port=9947/port=30340`，B 节点 bootnode 使用 A 节点 `/wss/p2p/{peer_id}` 地址；两节点 `system_health.peers=1` 后可正常出块。
- 已完成完整 E2E：Worker 钱包登录、会员校验、R2 manifest/image 上传、上传完成、链上 `SquarePost.publishSquarePost` 入块、Worker 读取链上事件确认、推荐 feed 返回同一条动态。
- 完整 E2E 发布证据：`post_id=sqp_4b63682f5c8346c5aa5de1fc83da83cb`，`content_hash=0bd97c8a95b72fe68a2d8425539ae389d0dc17b2f978503994371cccd74f3ed8`，`storage_receipt_id=sqr_027adbbe16e0aa086b75c529207442bbae218afb944b3d1dfad0f76fec781e3f`，`tx_hash=0xf9e946ec2949816002f006399a506d4710a1c36f6c71dcabf603bb9095239030`，`block_hash=0x2b3f7089006788995c36ed7ef93024b381e43100dcd0a38f989741520f808ab9`，`chain_block=1`。
- 本地链运行态观察：交易块导入后，矿工曾立即尝试下一轮空块 proposal，被 runtime `pow-difficulty` 的“空块不允许上链”保护阻止；该日志不影响本次广场发布 E2E 结论，但后续如要长期运行本地 fresh 链，需要单独优化空池 proposal 门控。

阶段 7 已通过验证：
- `npm --prefix citizenapp/cloudflare run migrate:local`：通过，本地 D1 无待执行迁移。
- Worker HTTP 真实运行态：`auth/challenge`、`auth/session`、`membership`、`uploads/prepare`、`uploads/dev-put`、`uploads/complete` 均通过。
- Worker + R2 + CitizenChain 完整真实 E2E：通过；`publishSquarePost` 入块后，`POST /v1/square/posts/confirm` 成功写入 `square_posts`，`GET /v1/square/feed/recommended?limit=5` 返回该 `post_id`，media item 数为 1。
- `npm --prefix citizenapp/cloudflare run typecheck`：通过。
- `npm --prefix citizenapp/cloudflare test`：通过，5 个测试文件、10 个测试用例全部通过。
- `flutter analyze lib/8964 test/8964`：通过。
- `flutter test test/8964/square_chain_service_test.dart test/8964/square_publish_service_test.dart test/8964/square_feed_service_test.dart test/8964/square_home_page_test.dart`：通过。
- `git diff --check`：通过。
- 阶段 7 当前未修改 `citizenchain/runtime/` 源码，未写入 Cloudflare token、R2 access key、R2 secret key 或链 RPC 私密地址。

## 阶段 IM-0：聊天架构冻结

目标：
- 按用户 2026-07-05 确认，公民 App 聊天只保留互联网聊天和近场聊天。
- 互联网聊天固定为 Cloudflare 密文 mailbox。
- 近场聊天固定为蓝牙 / Wi-Fi 手机直连。
- 统一身份为钱包地址，统一加密为 OpenMLS，统一消息格式为 `GMB_IM_V1 / ImEnvelope`。
- 区块链节点聊天方式已转为历史方案，并在阶段 IM-1 中完成代码删除。

执行范围：
- 只改任务卡、IM 技术文档、统一协议和统一命名登记。
- 不改代码。
- 不修改 `citizenchain/runtime/`。
- 不创建新的 Cloudflare Worker 代码。

执行记录：
- 已更新 `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`，正式聊天路线改为 `ImCloudflareTransport` + `ImNearbyTransport`。
- 已明确广场公开点赞、评论、关注、举报等为公开互动，允许 Worker / D1 明文处理。
- 已明确私聊和群聊必须 OpenMLS 端到端加密，Cloudflare 只临时保存未 ack / 未过期密文、加密附件和必要投递元数据。
- 已明确 `citizenchain/node/src/im/`、通信节点设置页、`im_node_pairing` 和节点聊天投递为历史删除对象；阶段 IM-1 已完成代码删除。

验收标准：
- 文档里正式聊天方式只剩 Cloudflare 互联网聊天和近场聊天。
- 不再把区块链节点通信节点描述为正式聊天方式。
- `git diff --check` 通过。

## 阶段 IM-1：删除区块链节点聊天链路并接入 Cloudflare transport 骨架

目标：
- 从当前代码中彻底删除旧区块链节点通信节点聊天方式。
- CitizenApp 只保留 Cloudflare 密文 mailbox 和近场两种 transport 枚举。
- 移除“设置通信节点”、桌面通信节点设置面板、`im_node_pairing` 和 `/gmb/im/1` 残留。
- 保持 OpenMLS、钱包聊天账户、信息 Tab、联系人详情消息入口和本地 Isar 消息库不被砍掉。

执行范围：
- `citizenapp/lib/im/`、`citizenapp/lib/im/transport/`、`citizenapp/lib/im/storage/`、`citizenapp/lib/isar/`：切换 transport 模型、路由缓存字段和运行态编排；涉及代码与生成文件。
- `citizenapp/im/proto/`：移除旧通信节点字段，保留 Cloudflare mailbox 和近场路由字段；涉及 Protobuf 真源和 Dart 生成物。
- `citizenapp/lib/qr/`、`citizenapp/test/qr/`：删除 `im_node_pairing` 解析，补 `k=5` 拒绝测试。
- `citizenapp/lib/my/user/user.dart`：删除“设置通信节点”设置入口。
- `citizenchain/node/src/`、`citizenchain/node/frontend/settings/`、`citizenchain/scripts/`：删除节点 IM mailbox、通信节点开关、桌面二维码、Tauri 命令和双节点 smoke。
- `memory/`、`website/src/whitepaper.md`：同步当前技术口径、协议登记和白皮书。
- 本阶段不修改 `citizenchain/runtime/`。

执行记录：
- 已新增 `citizenapp/lib/im/transport/im_cloudflare_transport.dart`，作为 Cloudflare 密文 mailbox 互联网聊天 transport 骨架；当前在 Worker chat API 未接入前返回明确 pending/failed 状态。
- 已将 `ImTransportType` 收敛为 `cloudflare` 和 `nearby`，删除 `privateNode`。
- 已重写 `ImRuntime` 的远程投递编排，删除通信节点配对配置读取、保存和校验逻辑。
- 已把 `ImRouteRecord` / `ImRouteCacheEntity` 从 `nodePeerId/nodeMultiaddr/node_endpoints` 改为 `cloudflareMailboxId/cloudflare_mailbox_id` 和 `nearbyPeerHint/nearby_peer_hint`。
- 已更新 `citizenapp/im/proto/im_envelope.proto` 并重新生成 Dart Protobuf 文件。
- 已删除 CitizenApp 端旧 `ImNodeSettingsPage`、`ImPrivateNodeTransport`、`ImNodePairingBody` 和对应测试。
- 已删除桌面节点端 `src/im/`、`src/settings/communication_node/`、通信节点前端设置面板、通信节点 Tauri 命令、`/gmb/im/1` 注册和 `im-two-node-smoke.sh`。
- 已更新 QR 协议，`k=5 im_node_pairing` 作为已删除流向拒绝解析。
- 已更新白皮书和 memory 技术文档，当前聊天路线只剩 Cloudflare 密文 mailbox 与近场通信。

验收结果：
- `flutter analyze --no-fatal-infos`：通过；全量 `flutter analyze` 仍只剩既有未触碰 info `citizenapp/lib/transaction/onchain-transaction/onchain_payment_service.dart:43:13 prefer_const_constructors`。
- `flutter test --concurrency=1 test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，25 个测试通过。
- `npm --prefix citizenchain/node/frontend run build`：通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p node`：通过。
- `flutter run -d 3C071JEKB09000 --debug`：Pixel 8a 真机构建、安装并进入运行态；VM Service 已启动，验证后已退出调试会话。
- 已扫描代码目录，未发现 `im_node_pairing`、`ImNodeSettingsPage`、`ImPrivateNodeTransport`、`CommunicationNode`、`/gmb/im/1`、`nodePeerId`、`nodeMultiaddr` 等旧实现残留。

## 阶段 IM-2：Cloudflare 密文 mailbox API 与 App transport HTTP 接入

目标：
- 在 Cloudflare Worker 中落地私聊/群聊密文 mailbox API。
- Worker 只保存设备绑定、KeyPackage、未 ack / 未过期的临时密文 envelope 和必要投递元数据。
- CitizenApp 的 `ImCloudflareTransport` 从骨架改为可调用真实 HTTP API。
- 不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

执行范围：
- `citizenapp/cloudflare/src/chat/`：新增 chat mailbox 模块；涉及代码。
- `citizenapp/cloudflare/migrations/0002_chat_mailbox.sql`：新增 D1 chat 表；涉及数据库迁移。
- `citizenapp/cloudflare/src/routes.ts`：注册 `/v1/chat/*` 路由；涉及代码。
- `citizenapp/cloudflare/test/chat.test.ts`：补 chat helper 与无效设备绑定签名测试；涉及测试。
- `citizenapp/lib/im/transport/im_cloudflare_transport.dart`：接入真实 Worker HTTP API；涉及代码。
- `citizenapp/test/im/im_cloudflare_transport_test.dart`：补 transport HTTP 行为测试；涉及测试。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`：同步当前协议与实现态；涉及文档。

执行记录：
- 已新增 `chat_devices`、`chat_keypackages`、`chat_envelopes` 三张 D1 表；不新增聊天明文表。
- 已实现 `POST /v1/chat/devices/register`，使用钱包 session，并按 `OP_SIGN_IM_WALLET_BINDING` 域重建 IM 设备绑定签名消息验签。
- 已实现 `POST /v1/chat/keypackages`、`GET /v1/chat/keypackages/{owner_account}`、`POST /v1/chat/keypackages/consume`。
- 已实现 `POST /v1/chat/envelopes`、`GET /v1/chat/envelopes/pending`、`POST /v1/chat/envelopes/ack`。
- 已把 `ImCloudflareTransport` 接到真实 HTTP API，支持设备登记、KeyPackage 发布/拉取/消费、密文投递、pending 拉取和 ack。
- 当时运行态边界：`ImRuntime` 尚未自动获取 Worker session，也尚未自动触发设备绑定签名；未配置 `mailboxBaseUrl/sessionToken` 时 transport 明确返回“Cloudflare 密文 mailbox 尚未配置”。该边界已在阶段 IM-3 解决。
- 本阶段未实现 WebSocket 新密文通知，未实现聊天附件 R2 加密上传，未做近场。

验收结果：
- `npx wrangler d1 migrations apply citizenapp-square-db --local`：通过，已在本地 D1 应用 `0001` 和 `0002_chat_mailbox` 迁移。
- `npm install`：通过，补齐 `cloudflare` Worker 本地 `node_modules`，用于 Worker 本地打包与运行态验收。
- `npm exec wrangler dev -- --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1`：通过，Worker 本地服务启动成功。
- `curl http://127.0.0.1:8789/health`：返回 200，`citizenapp-square-api` 正常。
- `curl http://127.0.0.1:8789/v1/chat/envelopes/pending`：返回 401 `missing_session`，证明 chat 路由已加载，未登录访问按预期拒绝。
- `curl -X POST http://127.0.0.1:8789/v1/chat/devices/register`：返回 401 `missing_session`，证明设备登记路由已加载，未登录访问按预期拒绝。
- `npm --prefix citizenapp/cloudflare run typecheck`：通过。
- `npm --prefix citizenapp/cloudflare test`：通过，5 个测试文件、10 个测试用例通过。
- `flutter analyze --no-fatal-infos lib/im/transport/im_cloudflare_transport.dart test/im/im_cloudflare_transport_test.dart`：通过。
- `flutter test --concurrency=1 test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，29 个测试通过。

## 阶段 IM-3：CitizenApp 自动 mailbox session、设备绑定与 KeyPackage 发布

目标：
- 用户只要在“我的 -> 用户资料 -> 设置通信账户”选择了默认聊天钱包，即可进入聊天窗口发送消息。
- 发送或同步前自动复用广场 Cloudflare Worker 钱包 session。
- 首次使用 IM 时自动生成/读取 IM 设备身份，自动用钱包签名登记设备绑定。
- 自动发布本设备 OpenMLS KeyPackage。
- 首次给对方发消息时，自动拉取并消费对方 KeyPackage，再投递 Welcome + application 密文 envelope。
- 聊天窗口打开后自动同步 pending 密文，不要求用户点击额外连接、绑定或扫码按钮。
- 不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

执行范围：
- `citizenapp/lib/8964/services/square_api_client.dart`：公开 Worker base URI，供 IM transport 复用同一个 Worker 登录态；涉及代码。
- `citizenapp/lib/im/im_runtime.dart`：新增自动 Worker session、设备绑定签名、设备登记、KeyPackage 发布、发送前 KeyPackage 拉取/消费；涉及代码和中文注释。
- `citizenapp/lib/im/im_chat_page.dart`：聊天窗口打开后自动触发 pending 同步；涉及代码。
- `citizenapp/test/im/im_envelope_session_test.dart`：补运行态自动 mailbox 准备测试；涉及测试。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`：同步 IM-3 当前实现态；涉及文档。

执行记录：
- 已在 `SquareApiClient` 暴露 `baseUri`，IM 运行态不新建第二套 Worker endpoint 配置。
- 已在 `ImRuntime` 发送和同步入口统一调用自动 mailbox 准备流程。
- 已复用广场 `ensureSession`，Worker session 仍由钱包账户签名登录获得。
- 已按 `OP_SIGN_IM_WALLET_BINDING` 对 IM 设备绑定 payload 生成 32 字节签名消息，并提交 `POST /v1/chat/devices/register`。
- 已缓存设备绑定有效期和 KeyPackage 发布有效期，避免每次发送都重复登记和发布。
- 已在首次 MLS 会话缺少对方 KeyPackage 时，自动拉取并消费对方 KeyPackage，再重试发送 Welcome + application envelope。
- 已在聊天窗口打开后自动执行一次 pending 同步；同步按钮保留为手动刷新，不是必要操作。
- 默认热钱包可自动签名；冷钱包通信账户保留签名回调注入边界，未接入聊天页扫码签名前会返回明确错误。

验收结果：
- `flutter analyze --no-fatal-infos lib/im/im_chat_page.dart lib/im/im_runtime.dart test/im/im_envelope_session_test.dart`：通过。
- `flutter test --concurrency=1 test/im/im_envelope_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_tab_page_test.dart test/im/im_chat_ui_adapter_test.dart`：通过，15 个测试通过。
- `npm --prefix citizenapp/cloudflare run typecheck && npm --prefix citizenapp/cloudflare test`：通过，5 个 Worker 测试文件、10 个测试用例通过。
- `flutter analyze --no-fatal-infos lib/im/im_runtime.dart lib/im/im_chat_page.dart lib/im/transport/im_cloudflare_transport.dart lib/8964/services/square_api_client.dart test/im/im_envelope_session_test.dart test/im/im_cloudflare_transport_test.dart`：通过。
- `flutter test --concurrency=1 test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，30 个测试通过。
- 旧通信节点残留扫描：未发现 `im_node_pairing`、`ImNodeSettingsPage`、`ImPrivateNodeTransport`、`CommunicationNode`、`/gmb/im/1`、`nodePeerId`、`nodeMultiaddr` 等旧实现残留。
- Worker 本地运行态 smoke：先在 `citizenapp/cloudflare` 执行 `npm install`，再执行 `./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1`；服务启动成功。
- `curl http://127.0.0.1:8789/health`：返回 200，`citizenapp-square-api` 正常。
- `curl http://127.0.0.1:8789/v1/chat/envelopes/pending`：返回 401 `missing_session`，证明 chat pending 路由已加载，未登录访问按预期拒绝。
- `curl -X POST http://127.0.0.1:8789/v1/chat/devices/register`：返回 401 `missing_session`，证明设备登记路由已加载，未登录访问按预期拒绝。
- 本地经验记录：`npm exec wrangler dev -- ...` 在当前 npm/路径状态下曾触发临时依赖解析问题；本仓库 smoke 以本地 `./node_modules/.bin/wrangler` 为准。

## 阶段 IM-4：前台自动收信轮询与信息页刷新

目标：
- 用户打开“信息”Tab 或聊天窗口后自动接收 Cloudflare mailbox pending 密文。
- 不要求用户点击同步、连接、绑定、扫码等额外操作。
- 信息 Tab 前台每 15 秒轻量轮询 pending，成功后刷新会话列表。
- 聊天窗口前台每 8 秒轮询 pending，成功后刷新当前会话。
- 弱网或 Worker 请求失败后退避到 30 秒，避免持续打请求。
- 页面销毁、离开页面或 App 退后台时停止轮询。
- 不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

执行范围：
- `citizenapp/lib/im/im_tab_page.dart`：信息 Tab 打开即同步，前台 15 秒轮询，失败 30 秒退避，生命周期停止；涉及代码和中文注释。
- `citizenapp/lib/im/im_chat_page.dart`：聊天页打开即同步，前台 8 秒轮询，失败 30 秒退避，生命周期停止；涉及代码和中文注释。
- `citizenapp/test/im/im_tab_page_test.dart`：补信息 Tab 自动轮询和聊天页自动轮询 widget 测试；涉及测试。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`：同步 IM-4 当前实现态；涉及文档。

执行记录：
- 已在信息 Tab 初始化时自动同步 pending；有通信账户且注入 IM 运行态时启动前台 15 秒轮询。
- 已在信息 Tab 轮询成功后刷新会话列表；失败不弹复杂错误，下一轮退避到 30 秒。
- 已在聊天窗口初始化时自动同步 pending；有同步链路时启动前台 8 秒轮询。
- 已在聊天窗口轮询成功后刷新消息列表；失败不打断用户输入，下一轮退避到 30 秒。
- 已接入 App 生命周期 observer，App 非前台时取消轮询，回到前台重新同步并恢复轮询。
- 已保留手动同步按钮作为刷新入口，但它不是必要操作。

验收结果：
- `flutter analyze --no-fatal-infos lib/im/im_tab_page.dart lib/im/im_chat_page.dart test/im/im_tab_page_test.dart`：通过。
- `flutter test --concurrency=1 test/im/im_tab_page_test.dart`：通过，4 个测试通过。
- `npm --prefix citizenapp/cloudflare run typecheck && npm --prefix citizenapp/cloudflare test`：通过，5 个 Worker 测试文件、10 个测试用例通过。
- `flutter analyze --no-fatal-infos lib/im/im_tab_page.dart lib/im/im_chat_page.dart lib/im/im_runtime.dart lib/im/transport/im_cloudflare_transport.dart lib/8964/services/square_api_client.dart test/im/im_tab_page_test.dart test/im/im_envelope_session_test.dart test/im/im_cloudflare_transport_test.dart`：通过。
- `flutter test --concurrency=1 test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，32 个测试通过。
- 旧通信节点残留扫描：未发现 `im_node_pairing`、`ImNodeSettingsPage`、`ImPrivateNodeTransport`、`CommunicationNode`、`/gmb/im/1`、`nodePeerId`、`nodeMultiaddr` 等旧实现残留。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功。
- `curl http://127.0.0.1:8789/health`：返回 200，`citizenapp-square-api` 正常。
- `curl http://127.0.0.1:8789/v1/chat/envelopes/pending`：返回 401 `missing_session`，证明 chat pending 路由已加载，未登录访问按预期拒绝。
- `curl -X POST http://127.0.0.1:8789/v1/chat/devices/register`：返回 401 `missing_session`，证明设备登记路由已加载，未登录访问按预期拒绝。

## 阶段 IM-5：互联网私聊端到端闭环验收

目标：
- 验证 A/B 两个聊天账户通过 Cloudflare mailbox 语义完成私聊闭环。
- 覆盖发送端生成 Welcome + application、mailbox 保存密文、接收端 pending 拉取、解密落库、ack 清空 mailbox。
- 验证接收端 Isar 只保存本机明文，mailbox 只承载密文 envelope bytes。
- 不恢复区块链节点聊天路线，不修改 `citizenchain/runtime/`。

执行范围：
- `citizenapp/test/im/im_envelope_session_test.dart`：补 mailbox pending 拉取、接收端落库、ack 清空的可运行状态机测试；涉及测试和中文注释。
- `citizenapp/test/im/im_mls_native_session_test.dart`：补 native OpenMLS mailbox 闭环用例；涉及测试和中文注释。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`、本任务卡：同步 IM-5 当前实现态；涉及文档。

执行记录：
- 已新增内存 mailbox 测试夹具，模拟 Cloudflare mailbox 的 pending / ack 语义。
- 已覆盖 Alice 端发送 Welcome + application 后，Bob 端通过 `fetchAndProcessPending` 拉取两条密文并 ack。
- 已验证 Bob 端本地 Isar 会话和消息记录写入 incoming 明文，Alice 本地库不共享给 Bob。
- 已新增 native OpenMLS mailbox 闭环用例，验证真实 OpenMLS 生成的密文 envelope 能经 mailbox 语义进入接收端解密落库路径。
- 本机纯 Dart 测试环境没有 host 版 `libsmoldot.dylib`，native OpenMLS 用例按现有 `smoldotNativeSkipReason()` 跳过；没有擅自编译生成 `/Users/rhett/GMB/citizenapp/rust/target/release/libsmoldot.dylib`。
- Worker 本地依赖缺失时执行 `npm install` 恢复 `node_modules/`，该目录已被 `citizenapp/cloudflare/.gitignore` 忽略；未修改 `package.json` 或 `package-lock.json`。
- 已清理 `citizenapp/lib/` 中遗留“通信全节点”注释口径，改为 Cloudflare mailbox / 近场 transport 边界。

验收结果：
- `flutter test --concurrency=1 test/im/im_envelope_session_test.dart test/im/im_mls_native_session_test.dart`：通过，5 个测试通过、2 个 native 测试因本机缺 host 版 `libsmoldot.dylib` 跳过。
- `npm run typecheck && npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、10 个测试用例通过。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功。
- `curl http://127.0.0.1:8789/health`：返回 200，`citizenapp-square-api` 正常。
- `curl http://127.0.0.1:8789/v1/chat/envelopes/pending?owner_account=alice-wallet&device_id=alice-phone`：返回 401 `missing_session`，证明 chat pending 路由已加载，未登录访问按预期拒绝。
- `curl -X POST http://127.0.0.1:8789/v1/chat/devices/register`：返回 401 `missing_session`，证明设备登记路由已加载，未登录访问按预期拒绝。
- 旧通信节点代码残留扫描：`citizenapp/lib`、`citizenapp/test`、`citizenchain/node/src`、`citizenchain/node/frontend/settings` 未发现 `通信全节点`、`通信节点功能`、`设置通信节点`、`ImNodeSettingsPage`、`ImPrivateNodeTransport`、`CommunicationNode`、`/gmb/im/1`、`nodePeerId`、`nodeMultiaddr`。
- 阶段 IM-5 未修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

## 阶段 IM-6：OpenMLS native host 真实执行验收

目标：
- 让 `test/im/im_mls_native_test.dart` 和 `test/im/im_mls_native_session_test.dart` 在 macOS 本机从 skip 变成真实执行。
- 验证 Rust OpenMLS C ABI、Dart FFI、KeyPackage、Welcome、application、持久化会话恢复、mailbox 拉取解密落库 ack 全链路。
- 修复本机 host `libsmoldot.dylib` 构建产物无法被 Dart FFI 加载的问题。
- 不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

执行范围：
- `citizenapp/scripts/build-smoldot-native.sh`：macOS 分支禁用 release strip，保证 host 调试库可被 Dart FFI/dyld 加载；涉及脚本和中文注释。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/05-modules/citizenapp/rpc/RPC_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`、本任务卡：同步 IM-6 当前实现态；涉及文档。

执行记录：
- 首次执行 `./scripts/build-smoldot-native.sh macos` 生成 `/Users/rhett/GMB/citizenapp/rust/target/release/libsmoldot.dylib`，但 Dart FFI 直接 `dlopen` 报 `mis-aligned LINKEDIT string pool`。
- 定位根因：`citizenapp/rust/Cargo.toml` release profile 的 `strip = true` 会让 macOS host dylib 触发 dyld LINKEDIT 对齐错误。
- 手动用 `CARGO_PROFILE_RELEASE_STRIP=false cargo build --release` 验证后，`dart /tmp/gmb_probe_dlopen.dart` 可直接加载 dylib。
- 已把 `CARGO_PROFILE_RELEASE_STRIP=false` 固定到 `scripts/build-smoldot-native.sh macos`，Android/iOS 打包库仍沿用 release profile。
- 已重新运行脚本入口，确认 host `libsmoldot.dylib` 正常生成且可加载。
- Worker 本地依赖缺失时再次执行 `npm install` 恢复 `node_modules/`；该目录已被 `.gitignore` 忽略，未进入 Git 状态。

验收结果：
- `./scripts/build-smoldot-native.sh macos`：通过，生成 host `libsmoldot.dylib`。
- `cargo test`（目录 `citizenapp/rust`）：通过，2 个 Rust OpenMLS 单元测试通过。
- `dart /tmp/gmb_probe_dlopen.dart`：通过，显式 dlopen host `libsmoldot.dylib` 成功。
- `flutter test --concurrency=1 test/im/im_mls_native_session_test.dart`：通过，2 个 native OpenMLS 会话测试真实执行通过，无 skip。
- `flutter test --concurrency=1 test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，37 个测试通过，无 native skip。
- `flutter analyze --no-fatal-infos lib/im/im_tab_page.dart lib/im/im_chat_page.dart lib/im/im_runtime.dart lib/im/transport/im_cloudflare_transport.dart lib/im/transport/im_transport.dart lib/isar/wallet_isar.dart test/im/im_tab_page_test.dart test/im/im_envelope_session_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart`：通过。
- `npm run typecheck && npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、10 个测试用例通过。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功；`/health` 返回 200；`/v1/chat/envelopes/pending` 未登录返回 401 `missing_session`。
- 阶段 IM-6 未修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

## 阶段 IM-7：加密附件发送与接收底座

目标：
- 实现互联网聊天的加密附件发送底座。
- R2 只保存 CitizenApp 本地加密后的 manifest 和附件分片。
- 附件内容密钥、nonce、mac、manifest hash 和 chunk hash 只放进 OpenMLS application 载荷，由 OpenMLS 端到端加密。
- Worker 只做附件上传准备、开发代理上传和上传完成确认，不保存附件密钥，不新增聊天附件 D1 表。
- 聊天 UI 先安全显示附件占位文案，不把 OpenMLS 控制 JSON 原样展示给用户。
- 不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

执行范围：
- `citizenapp/pubspec.yaml`、`citizenapp/pubspec.lock`：新增 `cryptography`，用于 App 本地 AES-GCM 附件加密；涉及依赖。
- `citizenapp/lib/im/im_message_flow.dart`：新增附件草稿、AES-GCM manifest/chunk 加密、上传编排和 OpenMLS 附件控制消息；涉及代码和中文注释。
- `citizenapp/lib/im/im_runtime.dart`：新增 `sendAttachment` 运行态入口；涉及代码。
- `citizenapp/lib/im/im_chat_ui_adapter.dart`：附件消息显示 `[附件] 文件名` 安全占位；涉及代码。
- `citizenapp/lib/im/transport/im_transport.dart`、`citizenapp/lib/im/transport/im_cloudflare_transport.dart`：新增附件 prepare/upload/complete transport 抽象和 Cloudflare HTTP 实现；涉及代码。
- `citizenapp/cloudflare/src/chat/service.ts`、`citizenapp/cloudflare/src/routes.ts`、`citizenapp/cloudflare/src/storage/presigned.ts`：新增聊天附件上传准备、开发代理上传、上传完成校验和路由；涉及代码。
- `citizenapp/test/im/im_envelope_session_test.dart`、`citizenapp/test/im/im_cloudflare_transport_test.dart`、`citizenapp/test/im/im_chat_ui_adapter_test.dart`、`citizenapp/cloudflare/test/chat.test.ts`：补附件加密、上传、transport 和展示测试；涉及测试。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`、本任务卡：同步 IM-7 当前实现态；涉及文档。

执行记录：
- 已新增 `ImAttachmentDraft` 和 `ImRuntime.sendAttachment`，发送入口复用自动 Worker session、IM 设备绑定、KeyPackage 拉取/消费和 mailbox 投递链路。
- 已使用 `AES-GCM-256` 在 CitizenApp 本地加密附件 manifest 和单个附件分片；IM-7 暂不做多分片续传。
- 已把附件内容密钥、manifest nonce/mac/hash、chunk object key、chunk nonce/mac/hash 写入 OpenMLS application 控制消息，再由 OpenMLS 生成 `mls_wire_message`。
- 已把 `ImEnvelope.attachment_manifest_hash` 设为加密 manifest 的 sha256 hex，并把 manifest/chunk R2 object key 放入 `chunk_refs`。
- 已新增 Worker `POST /v1/chat/attachments/prepare`、`PUT /v1/chat/attachments/dev-put`、`POST /v1/chat/attachments/complete`；complete 只检查 R2 对象存在。
- 已限定开发代理上传必须设置 `SQUARE_DEV_UPLOAD_PROXY=1`，生产路径应使用 R2 预签名 PUT。
- 已让聊天列表附件消息显示 `[附件] 文件名`，避免把控制 JSON 直接显示给用户。
- 阶段 IM-7 没有新增 D1 migration，没有新建聊天附件表。
- 阶段 IM-7 没有修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

验收结果：
- `dart format citizenapp/lib/im/im_message_flow.dart citizenapp/lib/im/im_runtime.dart citizenapp/lib/im/im_chat_ui_adapter.dart citizenapp/lib/im/transport/im_cloudflare_transport.dart citizenapp/lib/im/transport/im_transport.dart citizenapp/test/im/im_envelope_session_test.dart citizenapp/test/im/im_cloudflare_transport_test.dart citizenapp/test/im/im_chat_ui_adapter_test.dart`：通过。
- `npm run typecheck && npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、11 个测试用例通过。
- `flutter test --concurrency=1 test/im/im_envelope_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_chat_ui_adapter_test.dart`：通过，16 个测试通过。
- `flutter test --concurrency=1 test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，39 个测试通过。
- `flutter analyze --no-fatal-infos lib/im/im_message_flow.dart lib/im/im_runtime.dart lib/im/im_chat_ui_adapter.dart lib/im/transport/im_cloudflare_transport.dart lib/im/transport/im_transport.dart test/im/im_envelope_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_chat_ui_adapter_test.dart`：通过。
- `cargo test`（目录 `citizenapp/rust`）：通过，2 个 Rust OpenMLS 单元测试通过。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功；`/health` 返回 200；`/v1/chat/attachments/prepare` 未登录返回 401 `missing_session`。

## 阶段 IM-8：附件下载解密与聊天页文件选择

目标：
- 用户在聊天窗口点击附件按钮选择文件并发送。
- 接收方点击附件消息后下载密文 manifest/chunk，本机校验 sha256，再用 OpenMLS 控制消息中的 AES-GCM 参数解密。
- 解密后的附件保存到 CitizenApp 私有缓存目录，不进入 Cloudflare、链或节点。
- Worker 只签发附件下载 URL，必须校验当前钱包是该附件密文 envelope 的发送方或接收方。
- 不新增聊天附件 D1 表，不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

执行范围：
- `citizenapp/lib/im/im_chat_page.dart`：接入现成聊天 UI 的附件按钮、文件选择、附件消息点击下载和进度条；涉及代码和中文注释。
- `citizenapp/lib/im/im_message_flow.dart`：新增附件控制消息解析、manifest/chunk 下载校验、AES-GCM 解密和私有缓存保存；涉及代码。
- `citizenapp/lib/im/im_runtime.dart`：新增 `downloadAttachment` 运行态入口；涉及代码。
- `citizenapp/lib/im/im_chat_ui_adapter.dart`：把附件控制消息放入本机 UI metadata，显示层仍只展示 `[附件] 文件名`；涉及代码。
- `citizenapp/lib/im/im_tab_page.dart`、`citizenapp/lib/my/user/user.dart`：信息 Tab 和联系人详情两条聊天入口接入附件发送/下载运行态；涉及代码。
- `citizenapp/lib/im/transport/im_transport.dart`、`citizenapp/lib/im/transport/im_cloudflare_transport.dart`：新增附件下载计划和密文对象 GET；涉及代码。
- `citizenapp/cloudflare/src/chat/service.ts`、`citizenapp/cloudflare/src/routes.ts`、`citizenapp/cloudflare/src/storage/presigned.ts`：新增附件下载授权、开发代理下载和 R2 signed GET；涉及代码。
- `citizenapp/test/im/im_envelope_session_test.dart`、`citizenapp/test/im/im_cloudflare_transport_test.dart`、`citizenapp/test/im/im_chat_ui_adapter_test.dart`、`citizenapp/test/im/im_tab_page_test.dart`、`citizenapp/cloudflare/test/chat.test.ts`：补下载解密、transport、Worker 授权和 UI 测试；涉及测试。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`、本任务卡：同步 IM-8 当前实现态；涉及文档。

执行记录：
- 已复用 `flutter_chat_ui` 的 `onAttachmentTap`，聊天页附件按钮选择文件后调用 `ImRuntime.sendAttachment`。
- 已复用 `flutter_chat_ui` 的 `onMessageTap`，点击附件消息后调用 `ImRuntime.downloadAttachment`。
- 已新增 Worker `POST /v1/chat/attachments/download` 和 `GET /v1/chat/attachments/dev-get`；下载授权通过 `chat_envelopes.attachment_manifest_key` 确认当前钱包账户是发送方或接收方。
- 已新增 R2 signed GET 生成逻辑；开发环境 `dev-get` 仍要求 Bearer session。
- 已在 CitizenApp 本地下载密文 manifest/chunk 后校验 sha256，校验通过后用 `AES-GCM-256` 解密并保存到 App 私有文档目录 `im/attachments/`。
- 已保留 IM-8 单分片附件边界；多分片续传、缩略图、系统打开文件和下载状态持久化不在本阶段改 Isar schema。
- 阶段 IM-8 没有新增 D1 migration，没有新建聊天附件表。
- 阶段 IM-8 没有修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

验收结果：
- `flutter analyze --no-fatal-infos lib/im/im_message_flow.dart lib/im/im_runtime.dart lib/im/im_chat_ui_adapter.dart lib/im/im_chat_page.dart lib/im/im_tab_page.dart lib/im/transport/im_cloudflare_transport.dart lib/im/transport/im_transport.dart lib/my/user/user.dart test/im/im_envelope_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_chat_ui_adapter_test.dart test/im/im_tab_page_test.dart`：通过。
- `flutter test --concurrency=1 test/im/im_envelope_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_chat_ui_adapter_test.dart test/im/im_tab_page_test.dart`：通过，23 个测试通过。
- `flutter test --concurrency=1 test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，42 个测试通过。
- `npm run typecheck && npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、11 个测试用例通过。
- `cargo test`（目录 `citizenapp/rust`）：通过，2 个 Rust OpenMLS 单元测试通过。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功；`/health` 返回 200；`/v1/chat/attachments/download` 未登录返回 401 `missing_session`；`/v1/chat/attachments/dev-get` 未登录返回 401 `missing_session`。

## 阶段 IM-9：WebSocket 新密文通知与轮询兜底

目标：
- Worker 增加 `/v1/chat/ws`，让已登录且已登记的 IM 设备建立 WebSocket 实时通知通道。
- 发送方投递密文 envelope 后，Worker 只向接收钱包/设备推送“有新密文”的索引通知，不推送明文，也不推送密文正文。
- CitizenApp 信息 Tab 和聊天窗口优先使用 WebSocket 通知；连接不可用或断开时自动回到现有前台轮询。
- 不新增 D1 表，不改 Protobuf，不改 Isar schema，不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

预计修改目录：
- `citizenapp/cloudflare/src/chat/`：新增 Worker 实例内 WebSocket 连接表、连接认证、清理和新密文通知 fanout；涉及代码和中文注释边界。
- `citizenapp/cloudflare/src/routes.ts`：挂载 `/v1/chat/ws` 路由；涉及代码。
- `citizenapp/lib/im/transport/`：在 Cloudflare transport 中增加 WebSocket 客户端连接，保持 HTTP pending/ack 为真同步路径；涉及代码和中文注释。
- `citizenapp/lib/im/`：运行态、信息 Tab、聊天页接入实时通知优先和轮询兜底；涉及代码和中文注释。
- `citizenapp/lib/my/user/`：联系人详情消息入口补同一套实时同步入口；涉及代码。
- `citizenapp/test/im/`：补信息 Tab 与聊天页 WebSocket 通知优先、轮询兜底测试；涉及测试。
- `memory/05-modules/citizenapp/im/`、`memory/07-ai/`、本任务卡：同步 IM-9 当前实现态；涉及文档。

执行记录：
- 已新增 Worker `GET /v1/chat/ws`，要求 WebSocket upgrade、Bearer `session_token`、`owner_account` 与 session 匹配、`device_id` 是 active device。
- 已在 Worker 内维护 `owner_account + device_id` 连接表，收到 `POST /v1/chat/envelopes` 后推送 `gmb_im_new_envelope_v1` 通知。
- WebSocket 通知只包含 `envelope_id`、`conversation_id`、`recipient_account`、`recipient_device_id`、`mls_message_kind`、`created_at`；正式拉取、解密和 ack 仍走 pending/ack 旧流程。
- 已在 `ImCloudflareTransport` 增加 WebSocket 客户端连接；收到通知后交给上层触发 `syncPending`。
- 已在 `ImRuntime` 增加 `startRealtimeSync`，页面层无需处理 Worker token、设备登记或 KeyPackage 发布。
- 已在信息 Tab 和聊天窗口实现“实时连接成功则停轮询；实时不可用或断开则恢复轮询”；页面销毁或 App 退后台会关闭实时连接。
- 已在轮询成功后自动重试 WebSocket，避免一次弱网断线后长期停留在轮询模式。
- 当前 IM-9 是 Worker 单实例内连接表，能满足本地和单实例基础实时通知；Cloudflare 多实例、跨 isolate 的生产级 fanout 后续必须接 Durable Objects。
- 阶段 IM-9 没有新增文件，没有新增 D1 migration，没有修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

验收结果：
- `flutter analyze --no-fatal-infos lib/im/transport/im_cloudflare_transport.dart lib/im/im_runtime.dart lib/im/im_chat_page.dart lib/im/im_tab_page.dart lib/my/user/user.dart test/im/im_tab_page_test.dart`：通过。
- `flutter test --concurrency=1 test/im/im_tab_page_test.dart`：通过，8 个测试通过。
- `flutter test --concurrency=1 test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，44 个测试通过。
- `npm run typecheck && npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、11 个测试用例通过。
- `cargo test`（目录 `citizenapp/rust`）：通过，2 个 Rust OpenMLS 单元测试通过。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功；`/health` 返回 200；带 Upgrade 访问 `/v1/chat/ws` 未登录返回 401 `missing_session`；非 WebSocket 访问 `/v1/chat/ws` 返回 426 `websocket_required`。

## 阶段 IM-10：Durable Objects 生产级实时通知 fanout

目标：
- 把 IM-9 的 Worker 实例内 WebSocket 连接表升级为账户级 Durable Object，解决生产环境多 Worker 实例下通知可能找不到接收方 socket 的问题。
- `owner_account` 作为 DO 名称，同一个钱包聊天账户的所有在线设备连接都聚合到同一个 `ChatRealtimeObject`。
- Worker 写入 `chat_envelopes` 成功后，只调用接收钱包账户的 DO 发送 `gmb_im_new_envelope_v1` 新密文索引通知。
- 不新增 D1 表，不改 Protobuf，不改 App 端接口，不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

预计修改目录：
- `citizenapp/cloudflare/src/chat/`：新增 `realtime.ts`，实现 `ChatRealtimeObject`、WebSocket 接入、设备标签和通知 RPC；涉及代码和中文注释。
- `citizenapp/cloudflare/src/chat/service.ts`：移除 Worker 实例内 socket Map，改为鉴权后转发 WebSocket 到 DO、密文落库后调用 DO 通知；涉及代码。
- `citizenapp/cloudflare/src/index.ts`：导出 `ChatRealtimeObject` 供 wrangler 绑定；涉及代码。
- `citizenapp/cloudflare/src/types.ts`：增加 `CHAT_REALTIME` binding 类型；涉及代码。
- `citizenapp/cloudflare/wrangler.toml`：增加 Durable Object binding 和 migration；涉及配置，不写入任何 Cloudflare token 或 secret。
- `citizenapp/cloudflare/test/chat.test.ts`：补 `/v1/chat/ws` 转发到 DO、通知按接收钱包路由到 DO 的单测；涉及测试。
- `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`、本任务卡：同步 IM-10 当前实现态；涉及文档。

执行记录：
- 已新增 `ChatRealtimeObject`，一个钱包聊天账户对应一个 Durable Object；WebSocket attachment 保存 `owner_account`、`device_id`、`connected_at`。
- 已使用 `device:<device_id>` 作为 DO WebSocket tag；通知有 `recipient_device_id` 时只发指定设备，没有指定时发该账户所有在线设备。
- 已保留 Worker 层 session 校验、`owner_account` 匹配校验和 active device 校验；DO 不负责钱包登录态，不接触 D1 设备表。
- 已把 `POST /v1/chat/envelopes` 的通知从 Worker 实例内 Map 改为 `CHAT_REALTIME.getByName(recipient_account).notify(payload)`；通知失败不影响密文 envelope 已存储结果。
- 已在 `wrangler.toml` 添加 `CHAT_REALTIME` binding、`ChatRealtimeObject` migration，并同步 staging / production env binding。
- 阶段 IM-10 没有修改 CitizenApp 端代码，没有新增 D1 migration，没有修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

验收结果：
- `npm run typecheck && npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、13 个测试用例通过。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功，并识别 `env.CHAT_REALTIME Durable Object ChatRealtimeObject`；`/health` 返回 200；带 Upgrade 访问 `/v1/chat/ws` 未登录返回 401 `missing_session`；非 WebSocket 访问 `/v1/chat/ws` 返回 426 `websocket_required`。
- `flutter test --concurrency=1 test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，44 个测试通过。
- `cargo test`（目录 `citizenapp/rust`）：通过，2 个 Rust OpenMLS 单元测试通过。

## 阶段 IM-11：Cloudflare 临时 mailbox 最小化存储与本机删除底座

目标：
- Cloudflare 不作为聊天数据库，只作为互联网聊天的临时密文投递队列。
- 接收端必须先把消息和附件保存到 CitizenApp 本机，再 ack 删除 Cloudflare 临时副本。
- `ack` 成功后删除 `chat_envelopes` 行；若 envelope 关联加密附件，同时删除对应 R2 manifest/chunk 对象。
- 提交和拉取 mailbox 时顺手清理过期 envelope 及其 R2 加密附件对象。
- 发送端发送附件前先保存本机明文缓存，避免对方 ack 后发送端再依赖 Cloudflare 下载自己的附件。
- 增加本机会话删除底座：删除某个会话时清理本机 Isar 消息、会话、待发送队列、pending 入站记录和附件缓存目录。
- 不新增 D1 表，不改 Protobuf，不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

预计修改目录：
- `citizenapp/cloudflare/src/chat/`：把 mailbox ack 从标记状态改为删除临时 envelope，并删除对应 R2 加密附件对象；涉及代码和中文注释。
- `citizenapp/cloudflare/migrations/`：移除 `chat_envelopes.acked_at` 旧字段口径，新建部署按临时队列表结构初始化；涉及迁移定义清理。
- `citizenapp/cloudflare/test/`：补 ack 删除 envelope 和 R2 加密附件对象单测；涉及测试。
- `citizenapp/lib/im/`：附件发送/接收流程先落本机缓存再 ack，下载优先读本机缓存；涉及代码和中文注释。
- `citizenapp/lib/im/storage/`：增加本机会话删除能力；涉及代码和中文注释。
- `citizenapp/test/im/`：补附件先缓存再 ack、本机会话删除不误删其他会话测试；涉及测试。
- `memory/05-modules/citizenapp/im/`、`memory/07-ai/`、本任务卡：同步 IM-11 当前实现态；涉及文档。

执行记录：
- 已将 Worker `ackChatEnvelope` 改为删除 `chat_envelopes` 行，不再保留 `acked_at` 状态。
- 已新增过期清理：`submitChatEnvelope` 和 `fetchPendingChatEnvelopes` 入口会清理最多一批过期 envelope；附件 envelope 会按 manifest 所在目录删除 R2 加密对象。
- 已将 `chat_envelopes` 迁移定义和 pending 查询中的 `acked_at` 残留移除，当前 mailbox 表只表示未 ack / 未过期临时队列。
- 已让接收端 `fetchAndProcessPending` 只在消息成功解密落库后 ack；附件控制消息必须先缓存附件到本机，缺少缓存回调会直接失败而不会 ack。
- 已让附件下载优先读取本机私有缓存；发送端发送附件时也会先写入本机私有缓存。
- 已新增 `ImRuntime.deleteLocalConversation` 和 `ImIsarStore.deleteConversation`，本机删除会话时同步删除 Isar 记录和附件缓存目录。
- 阶段 IM-11 没有新增文件，没有新增 D1 migration，没有修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

验收结果：
- `flutter analyze --no-fatal-infos lib/im/im_message_flow.dart lib/im/im_runtime.dart lib/im/storage/im_isar_store.dart test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart`：通过，无问题。
- `flutter test --concurrency=1 test/im/im_isar_store_test.dart`：通过，2 个测试通过。
- `flutter test --concurrency=1 test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，45 个测试通过。
- `cargo test`（目录 `citizenapp/rust`）：通过，2 个 Rust OpenMLS 单元测试通过。
- `npm run typecheck && npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、15 个测试用例通过。
- Worker 本地运行态 smoke：`./node_modules/.bin/wrangler dev --local --port 8789 --var SQUARE_DEV_UPLOAD_PROXY:1` 启动成功，并识别 `CHAT_REALTIME` Durable Object、D1、R2、KV 绑定；`/health` 返回 200；带 Upgrade 访问 `/v1/chat/ws` 未登录返回 401 `missing_session`；非 WebSocket 访问 `/v1/chat/ws` 返回 426 `websocket_required`。

## 阶段 IM-12：聊天记录删除 UI 与本机彻底删除验收

目标：
- 用户可在信息 Tab 会话列表左滑删除某个会话的本机聊天记录。
- 用户可在聊天窗口右上角更多菜单删除当前会话的本机聊天记录。
- 删除前必须弹出简单二次确认：`删除聊天记录` / `确定删除这台设备上的聊天记录？`。
- 删除只影响当前设备本地会话、消息、待发送队列、pending 入站记录和附件缓存；不删除联系人，不影响对方设备或同一钱包的其他设备。
- 不通知 Cloudflare，不做云端删除，不修改 `citizenchain/runtime/`，不恢复区块链节点聊天路线。

预计修改目录：
- `citizenapp/lib/im/`：信息 Tab 左滑删除、聊天页更多菜单删除和删除后刷新/返回；涉及代码和中文注释。
- `citizenapp/lib/my/user/`：联系人详情进入聊天页时传入 runtime 本机会话删除回调；涉及代码。
- `citizenapp/test/im/`：补信息 Tab 左滑删除确认、聊天页菜单删除确认和返回上一页测试；涉及测试。
- `memory/05-modules/citizenapp/im/`、`memory/07-ai/`、本任务卡：同步 IM-12 当前实现态；涉及文档。

执行记录：
- 已在信息 Tab 会话列表为每个会话增加左滑删除入口，确认后调用 `ImRuntime.deleteLocalConversation`；没有 runtime 的测试/占位环境退回 `ImIsarStore.deleteConversation`。
- 已在聊天页 AppBar 右上角增加更多菜单，菜单项为 `删除聊天记录`，确认后停止当前同步/实时连接，删除本机记录，并在从列表进入时返回上一页。
- 已在联系人详情消息入口传入 `runtime.deleteLocalConversation(conversationId)`，保证从通讯录直接进入聊天页也能删除本机会话和附件缓存。
- 已补 `_FakeImStore.deleteConversation` 页面测试夹具，验证删除后只移除目标会话。
- 阶段 IM-12 没有新增文件，没有修改 Cloudflare Worker，没有修改 D1 migration，没有修改 `citizenchain/runtime/`，未恢复通信节点、节点 mailbox、`im_node_pairing` 或 `/gmb/im/1`。

验收结果：
- `flutter analyze --no-fatal-infos lib/im/im_tab_page.dart lib/im/im_chat_page.dart lib/my/user/user.dart test/im/im_tab_page_test.dart`：通过，无问题。
- `flutter test --concurrency=1 test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart`：通过，15 个测试通过。
- `flutter test --concurrency=1 test/im/im_mls_native_test.dart test/im/im_mls_native_session_test.dart test/im/im_cloudflare_transport_test.dart test/im/im_envelope_proto_test.dart test/im/im_envelope_session_test.dart test/im/im_route_cache_store_test.dart test/im/im_tab_page_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart test/qr/qr_router_test.dart`：通过，47 个测试通过。
- `cargo test`（目录 `citizenapp/rust`）：通过，2 个 Rust OpenMLS 单元测试通过。
- `git diff --check`：通过。
- 残留扫描：`citizenapp/lib/im`、`citizenapp/lib/my`、`citizenapp/test/im` 中未发现 `通信全节点`、`设置通信节点`、`im_node_pairing`、`/gmb/im/1`、`acked_at` 等旧聊天路线残留。

## 阶段 10：发布扣费入块后再上传 R2 与本地草稿保护

目标：
- App 发布动态时先校验钱包 finalized 余额至少为 `ED 1.11 元 + 发布费 1.00 元 = 2.11 元`。
- Worker `uploads/prepare` 只生成 `post_id`、R2 object key、短期上传授权和预生成 `storage_receipt_id`，不写 R2 对象。
- App 使用 prepare 阶段固定的 `post_id/content_hash/storage_receipt_id/storage_until` 提交链上发布交易。
- 只有交易入块后，App 才上传 R2 并调用 `uploads/complete`；链上未入块、余额不足或后台流程失败时保存本机草稿。
- 不修改 `citizenchain/runtime/`，发布费和 8:1:1 分账继续复用阶段 4 已确认的 runtime 实现。

预计修改目录：
- `citizenapp/cloudflare/src/uploads/`：调整上传准备和完成边界；涉及 Worker 代码和中文注释。
- `citizenapp/cloudflare/test/`：补预生成存储回执测试；涉及 Worker 测试。
- `citizenapp/lib/8964/services/`：拆分 prepare 与 upload，重排发布编排，增加余额守卫；涉及 App 代码和中文注释。
- `citizenapp/lib/8964/storage/`：新增广场发布草稿 KV 存储；涉及 App 代码，不新增 Isar schema。
- `citizenapp/lib/8964/pages/`：发布页恢复当前钱包的未完成草稿；涉及 App UI 代码。
- `citizenapp/test/8964/`：覆盖新发布顺序、余额不足、链上未入块草稿和草稿存储；涉及测试。
- `memory/01-architecture/citizenapp/`、`memory/07-ai/unified-protocols.md`、本任务卡：同步当前发布流程和 API 契约；涉及文档。

执行记录：
- Worker `prepareUpload` 已在准备阶段生成并返回 `storage_receipt_id`；`completeUpload` 不再重新生成回执，只校验 R2 对象存在、`content_hash == manifest_hash` 并把上传状态置为 `completed`。
- App `SquareUploadService` 已拆分为 `preparePostContent` 和 `uploadPreparedContent`；前者只计算 manifest/hash、检查会员和获取上传授权，后者只在链上入块后写 R2。
- App `SquarePublishService` 已按“余额校验 → prepare → 链上扣费入块 → R2 上传 → Worker 确认 feed”顺序编排；余额守卫使用 finalized 余额并要求至少 2.11 元。
- 已新增 `SquareDraftStore`，复用 `AppKvEntity` 保存当前钱包未完成发布草稿，不新增 Isar schema 或生成文件。
- 发布页打开后会按 `owner_account` 恢复上一条未完成草稿；发布成功后只尝试清理草稿，清理失败不影响已发布结果。
- 阶段 10 未修改 `citizenchain/runtime/`，未新增 Cloudflare secret，未要求 App 用户直接接触 Cloudflare/R2 账户。

验收记录：
- `npm run typecheck`（目录 `citizenapp/cloudflare`）：通过。
- `npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、15 个测试用例通过。
- `flutter analyze lib/8964 test/8964`（目录 `citizenapp`）：通过，无问题。
- `flutter test test/8964/square_publish_service_test.dart test/8964/square_draft_store_test.dart test/8964/square_chain_service_test.dart test/8964/square_feed_service_test.dart test/8964/square_home_page_test.dart`：通过，12 个测试通过。
- Worker 本地运行态 smoke：`npm run dev:local -- --port 8789` 启动成功；`GET /health` 返回 200；未登录 `POST /v1/square/uploads/prepare` 返回 401 `missing_session`。

## 阶段 11：staging 真实端到端发布前置验收与最小修复

目标：
- 在 Cloudflare staging 上复验 Worker、D1、KV、R2 上传授权和广场发布前置链路。
- 验证 `uploads/prepare` 在链上扣费入块前只生成 `post_id/storage_receipt_id` 和短期上传授权，不写 R2 对象，不把动态放入正式 feed。
- 明确 staging 当前是否具备链上确认 smoke 条件，不猜测链 RPC 地址。
- 不修改 `citizenchain/runtime/`，不回滚其它线程已完成的 Cloudflare Worker 扁平化迁移。

预计修改目录：
- `citizenapp/cloudflare/`：执行 staging 部署、D1/KV 远端 smoke、Worker typecheck/test 和残留清理；本阶段不改 Worker 业务代码。
- `citizenapp/lib/8964/pages/`：为广场首页补草稿仓库注入边界注释，避免页面测试触碰真实本机草稿存储；涉及代码和中文注释。
- `citizenapp/test/8964/`：同步 `SquareIdentityService` 当前默认钱包读取口径，补页面测试的默认钱包和空草稿仓库桩；涉及测试。
- `memory/01-architecture/citizenapp/`、`memory/07-ai/unified-protocols.md`、本任务卡：记录阶段 11 staging 真实验收结果、缺失配置和清理结果；涉及文档。

执行记录：
- Worker 目录已由其它任务扁平化到 `citizenapp/cloudflare/`；本阶段按新目录执行，没有恢复旧 `citizenapp/cloudflare/square_worker/`。
- `npx wrangler whoami` 已确认当前登录 Cloudflare 账户 `ChinaNation`，账户 ID 为 `f088b553d27d9c26b81f48f1924c3bbd`。
- 阶段 11 当时 staging 远端 secret 只有 `R2_ACCESS_KEY_ID`、`R2_ACCOUNT_ID`、`R2_SECRET_ACCESS_KEY`；尚未包含 `SQUARE_CHAIN_RPC_URL`，因此阶段 11 不能执行 Worker 链上确认 smoke。阶段 12 已补齐该 secret 并完成负向确认 smoke。
- staging 远端 D1 迁移检查显示 `citizenapp-square-db-staging` 没有待应用 migration。
- 已部署 staging Worker：`citizenapp-square-api-staging`，URL `https://citizenapp-square-api-staging.stews87-fawn.workers.dev`，版本 ID `f0be968a-0744-4543-8f1f-80318044d4e1`。
- 部署后 `GET /health` 返回 200，响应为 Cloudflare R2 后端且 `content_on_chain=false`。
- 未登录 `POST /v1/square/uploads/prepare` 返回 401 `missing_session`。
- 使用临时 KV session 和临时 D1 会员执行远端 `uploads/prepare`，返回 200 且包含 `storage_receipt_id`；D1 中对应 `square_uploads` 记录为 `status=prepared`、`content_hash=null`、`storage_receipt_id` 非空。
- 本次远端 smoke 没有执行 R2 PUT，没有写入 R2 对象；验收后已删除临时 KV session、临时会员和临时上传记录，D1 反查 `uploads_count=0`、`memberships_count=0`，KV 反查为 404。
- 已删除本轮 Wrangler 生成的 `citizenapp/cloudflare/.wrangler` 残留目录。
- 页面测试发现 `SquareIdentityService` 当前读取默认热钱包入口 `getDefaultWallet()`，已在测试桩中补齐该入口，并通过 `SquareHomePage.draftStore` 注入空草稿仓库，避免测试误触真实草稿存储。
- 阶段 11 没有修改 `citizenchain/runtime/`，没有新增 Cloudflare secret，没有 GitHub push/PR，没有保留 staging 临时数据。

验收记录：
- `npx wrangler secret list --env staging`（目录 `citizenapp/cloudflare`）：通过；阶段 11 当时尚未包含 `SQUARE_CHAIN_RPC_URL`，阶段 12 已补齐。
- `npx wrangler d1 migrations list citizenapp-square-db-staging --env staging --remote`（目录 `citizenapp/cloudflare`）：通过，无待执行 migration。
- `npm run deploy:staging`（目录 `citizenapp/cloudflare`）：通过，部署版本 ID `f0be968a-0744-4543-8f1f-80318044d4e1`。
- `curl GET /health`（staging Worker）：通过，HTTP 200。
- `curl POST /v1/square/uploads/prepare`（未登录）：通过，HTTP 401 `missing_session`。
- `curl POST /v1/square/uploads/prepare`（临时 session）：通过，HTTP 200，返回 `storage_receipt_id`。
- 远端 D1 验证：通过，临时上传记录为 `prepared` 且 `content_hash=null`；清理后临时上传和会员数量均为 0。
- `npm run typecheck`（目录 `citizenapp/cloudflare`）：通过。
- `npm test`（目录 `citizenapp/cloudflare`）：通过，5 个 Worker 测试文件、15 个测试用例通过。
- `flutter analyze lib/8964 test/8964`（目录 `citizenapp`）：通过，无问题。
- `flutter test test/8964/square_publish_service_test.dart test/8964/square_draft_store_test.dart test/8964/square_chain_service_test.dart test/8964/square_feed_service_test.dart test/8964/square_home_page_test.dart`（目录 `citizenapp`）：通过，12 个测试通过。

## 阶段 12：staging 链 RPC 确认 smoke 与阻塞确认

目标：
- 在 `SQUARE_CHAIN_RPC_URL` 已配置后，复验 staging Worker 能通过 Cloudflare Tunnel 访问国储会节点 RPC。
- 通过真实 staging KV/D1/R2/Worker 流程验证 `POST /v1/square/posts/confirm` 已进入链事件读取路径。
- 明确正向“发布交易入块 -> Worker 确认入库”smoke 是否具备测试钱包条件，不猜测链上余额或私钥。
- 清理本阶段产生的所有 staging 临时 KV、D1、R2 和本地 Wrangler 残留。

预计修改目录：
- `citizenapp/cloudflare/`：执行 staging 远端 smoke、R2 临时对象清理、Worker typecheck/test；本阶段不改 Worker 业务代码。
- `memory/01-architecture/citizenapp/`、`memory/07-ai/unified-protocols.md`、本任务卡：记录阶段 12 链 RPC 配置、负向确认 smoke、正向发布阻塞条件和清理结果；涉及文档。

执行记录：
- staging Worker secret 已包含 `SQUARE_CHAIN_RPC_URL`、`R2_ACCESS_KEY_ID`、`R2_ACCOUNT_ID`、`R2_SECRET_ACCESS_KEY`；链 RPC 完整 URL 只保存在 Cloudflare Secret，不写入仓库文档。
- Cloudflare Tunnel RPC host 已可达：`/health` 返回 200 `ok`；`system_properties` 返回 `ss58Format=2027`、`tokenDecimals=2`、`tokenSymbol=GMB`。
- 已复查 Worker 确认实现：`src/posts/confirm.ts` 先读取已完成上传，再通过 `fetchSystemEventsAtBlock()` 读取指定区块 `System.Events`，按 `post_id/owner_account/post_category/content_hash/storage_receipt_id` 匹配 `SquarePostPublished` 事件，最后校验 R2 manifest 并写入 `square_posts`。
- 已扫描当前链 `0..30` 区块，未发现 `SquarePostPublished` 事件；仓库公开测试助记词派生账户余额均为 0，当前仓库没有可签名且有余额的 staging 热钱包。
- 已使用临时 KV session、临时 D1 会员和真实 R2 预签名 PUT 执行 staging 负向链确认 smoke：`membership=200`、`uploads/prepare=200`、R2 `manifest PUT=200`、R2 `media PUT=200`、`uploads/complete=200`、`posts/confirm=409 square_event_not_found`。
- `square_event_not_found` 是本阶段预期结果：它证明 Worker 已经能访问链 RPC 并读取指定区块事件；由于临时上传没有对应真实链上发布交易，不能写入正式 feed。
- 正向 staging 发布确认 smoke 仍阻塞于测试条件：需要一个由当前执行方可签名、余额至少覆盖 ED + 1 元发布费、并可作为 CitizenApp 热钱包使用的 staging 钱包。不得使用公开助记词作为正式测试资金账户。
- 阶段 12 未修改 `citizenchain/runtime/`，未新增 Cloudflare secret，未写入 Cloudflare token/R2 key/RPC 完整地址，没有 GitHub push/PR。

验收记录：
- `curl` 国储会节点 Tunnel `/health`：通过，HTTP 200。
- `curl` 国储会节点 RPC `system_properties`：通过，返回 GMB 链属性。
- `npx wrangler secret list --env staging`（目录 `citizenapp/cloudflare`）：通过，确认 `SQUARE_CHAIN_RPC_URL` 已存在。
- staging 负向链确认 smoke：通过，临时内容完成 R2 上传和 D1 complete 后，Worker confirm 返回 HTTP 409 `square_event_not_found`。
- 远端清理验证：通过，临时 owner 在 `square_memberships`、`square_uploads`、`square_posts` 中计数均为 0；临时 KV session 读取为 404；临时 R2 `manifest.json` 和 `media_001.webp` 已删除。
- 本地残留清理：通过，已删除 `citizenapp/cloudflare/.wrangler` 本地缓存目录。

## 阶段 13：staging 正向发布确认 smoke（已尝试，阻塞）

目标：
- 使用用户提供的 staging 专用测试热钱包执行真实 `publish_square_post` 正向发布确认 smoke。
- 验证完整链路：Worker 登录、临时会员、`uploads/prepare`、链上扣 1 元发布费入块、R2 上传、`uploads/complete`、`posts/confirm`、推荐 feed 可见。
- 不把测试助记词写入仓库、文档、命令参数或日志。

预计修改目录：
- `citizenapp/cloudflare/`：执行 staging Worker/D1/R2/RPC smoke；本阶段未改 Worker 业务代码。
- `memory/01-architecture/citizenapp/`、`memory/07-ai/unified-protocols.md`、本任务卡：记录阶段 13 尝试结果、阻塞原因和清理结果；涉及文档。

执行记录：
- 已只读校验用户提供的测试助记词可派生目标地址 `w5EGb2HHsZz1wyQLA3YFmCbp1aC9TzSLoNwWFfd4qj3UpLjN2`，链上余额为 200 GMB，nonce 为 0，满足 ED + 1 元发布费。
- 首次正向 smoke 已完成 Worker 登录、临时会员、`uploads/prepare` 前置步骤，但在提交交易前读取链 RPC 时返回 Cloudflare 502 HTML，未生成 `tx_hash`，未提交链上交易，未扣费。
- 首次尝试失败后已清理临时 `square_uploads`、`square_follows`、`stage13_smoke` 会员、登录 challenge、KV session 和 R2 object key；D1 反查该 owner 临时上传数和临时会员数均为 0。
- 随后连续复测 `SQUARE_CHAIN_RPC_URL` 指向的 JSON-RPC 路径，`state_getRuntimeVersion` 稳定返回 `error code: 502`，WebSocket 也无法建立。
- `https://nrcgch-rpc.crcfrcn.com/health` 返回 HTTP 530：`The origin has been unregistered from Argo Tunnel`。
- 直连 `http://147.224.14.117:18080` 超时，说明当前本机无法绕过 Tunnel 访问该 RPC 服务。
- 因 Worker 链上确认依赖同一个 `SQUARE_CHAIN_RPC_URL`，当前继续提交链上发布交易会造成“链上已扣费但 Worker 无法确认入库”的不完整状态；本阶段已停止在提交交易前。
- Tunnel/DNS 恢复后已把 staging `SQUARE_CHAIN_RPC_URL` 切到一级域名 `nrcgch-rpc.crcfrcn.com`；公网 `/health`、`system_properties`、`state_getRuntimeVersion` 均恢复正常，staging Worker secret 已触发新版本部署。
- 恢复后重新执行正向 smoke：Worker 登录、临时会员、`uploads/prepare` 均通过；第一次用一次性 Node 编码提交，在 `author_submitExtrinsic` 的交易池校验阶段被 runtime 拒绝，返回 `TaggedTransactionQueue_validate_transaction` wasm trap，nonce 仍为 0、余额仍为 200 GMB。
- 为排除手工编码问题，已重新编译本地 `chain-signing` crate，并用仓库 Rust host 签名材料唯一真源构造 signed extrinsic；第二次提交仍在 `author_submitExtrinsic` 交易池校验阶段返回同一 `TaggedTransactionQueue_validate_transaction` wasm trap，说明失败不再是一次性 Node 编码问题。
- 只读检查远端 `state_getMetadata`：metadata 中不存在 `SquarePost`、`SquarePostPublished`、`publish_square_post`、`PublishedPostCountByAccount` 字符串；因此当前国储会节点运行的 runtime 不是包含广场 `SquarePost` pallet 的版本，无法接受 pallet index 36 的 `publish_square_post` 交易。
- 每次失败都发生在交易池校验阶段，交易没有进入交易池/区块；复验测试钱包 nonce 仍为 0，余额仍为 200 GMB，确认没有扣 1 元发布费。
- 每次失败后均已清理 staging 临时 D1 记录、R2 object key、KV session；最新复验该 owner 在 `square_uploads`、`square_posts`、`stage13_smoke` 会员和登录 challenge 中计数均为 0。本地一次性 Rust helper 已删除，`citizenapp/cloudflare/.wrangler` 未留下残留。
- 阶段 13 未修改 `citizenchain/runtime/` 源码，未新增 Cloudflare secret，未写入 Cloudflare token/R2 key/RPC 完整地址，没有 GitHub push/PR，没有链上扣费；仅刷新了本地 `target/` 下被 Git 忽略的 Rust 构建产物用于验证。

阻塞结论：
- 阻塞原因不是钱包、余额、Worker 登录、R2、Cloudflare Tunnel 或 Worker secret；阻塞点是远端国储会节点当前 runtime metadata 不包含 `SquarePost` pallet。
- 恢复条件：国储会节点必须运行包含 `runtime/otherpallet/square-post` 且 metadata 暴露 `SquarePost/publish_square_post/SquarePostPublished` 的 runtime。具体采用“测试链重启/清库重建”还是“正式 runtime upgrade”需要单独技术方案；任何涉及 `citizenchain/runtime/` 的源码修改或 runtime upgrade 流程，执行前必须按 runtime 二次确认硬规则单独确认。

当前执行状态：
- [x] 阶段 0：任务卡与技术方案固化
- [x] 阶段 1：协议与数据结构方案
- [x] 阶段 2：CitizenApp 广场前端壳
- [x] 阶段 3：Cloudflare Worker + R2 上传服务
- [x] 阶段 IM-0：聊天架构冻结
- [x] 阶段 IM-1：删除区块链节点聊天链路并接入 Cloudflare transport 骨架
- [x] 阶段 IM-2：Cloudflare 密文 mailbox API 与 App transport HTTP 接入
- [x] 阶段 IM-3：CitizenApp 自动 mailbox session、设备绑定与 KeyPackage 发布
- [x] 阶段 IM-4：前台自动收信轮询与信息页刷新
- [x] 阶段 IM-5：互联网私聊端到端闭环验收
- [x] 阶段 IM-6：OpenMLS native host 真实执行验收
- [x] 阶段 IM-7：加密附件发送与接收底座
- [x] 阶段 IM-8：附件下载解密与聊天页文件选择
- [x] 阶段 IM-9：WebSocket 新密文通知与轮询兜底
- [x] 阶段 IM-10：Durable Objects 生产级实时通知 fanout
- [x] 阶段 IM-11：Cloudflare 临时 mailbox 最小化存储与本机删除底座
- [x] 阶段 IM-12：聊天记录删除 UI 与本机彻底删除验收
- [x] 阶段 4：CitizenChain 发布索引
- [x] 阶段 5：App 发布闭环
- [x] 阶段 6：链上确认与正式 feed
- [x] 阶段 7：真实运行态验收与文档回写
- [x] 阶段 10：发布扣费入块后再上传 R2 与本地草稿保护
- [x] 阶段 11：staging 真实端到端发布前置验收与最小修复
- [x] 阶段 12：staging 链 RPC 确认 smoke 与阻塞确认
- [ ] 阶段 13：staging 正向发布确认 smoke（远端 runtime metadata 未包含 SquarePost，阻塞）
