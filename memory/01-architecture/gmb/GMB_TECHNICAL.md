# GMB 仓库技术文档（当前实现基线）

## 1. 文档目的

- 固化 `GMB` 仓库当前的四产品体系、技术边界、跨产品协作关系与维护规则。
- 作为仓库级技术总入口，帮助开发、联调、测试、发布时快速判断问题属于哪个产品、哪个模块、哪一类发布动作。
- 统一仓库文档、产品文档和模块文档职责，避免口径冲突。

## 2. 文档体系

仓库技术文档：

- `memory/01-architecture/gmb/GMB_TECHNICAL.md`

产品技术文档：

- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `citizenwallet` 产品级架构文档尚未单独建立，新增前必须按仓库新增文件规则单独确认。
- `citizenweb` 产品级架构文档尚未单独建立，新增前必须按仓库新增文件规则单独确认。

公民链内部能力文档：

- `memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`

模块技术文档：

- 位于 `memory/05-modules/` 中，按产品和模块目录归档，命名统一为 `*_TECHNICAL.md`。

## 3. 仓库总体定位

`GMB` 是一个多产品单仓库，围绕“公民链 + 公民 + 公民钱包 + 官方网站”构建完整数字主权系统。

仓库当前只保留四个产品：

- `citizenchain`：公民链，负责链上状态、共识、治理、发行、交易、节点运行、桌面节点软件和 OnChina 多机构工作台能力。
- `citizenapp`：公民，负责在线钱包、治理入口、交易入口、轻节点状态和用户端身份展示。
- `citizenwallet`：公民钱包，负责离线签名、扫码识别、冷钱包确认和签名结果生成。
- `citizenweb`：官方网站，负责 GMB 对外官网静态站点。

OnChina 不是独立产品；它是 `citizenchain/onchina/` 下的公民链内部能力。

### 3.1 跨产品账户标识目标

ADR-040 已冻结全仓统一账户模型，实施进度见 `memory/08-tasks/20260722-account-id-official-unify.md`：

- 链账户类型统一为 `AccountId`，单一账户字段统一为 `account_id`；多账户结构使用准确的 `<role>_account_id`。
- 公钥统一为 `public_key` / `signer_public_key` / `credential_signer_public_key`；SS58 展示字段统一为 `ss58_address`。
- `account_id` 和 32 字节公钥的文本形式统一为小写 `0x` 加 64 位十六进制。
- `account_id` 是跨产品授权、索引和持久化身份；`ss58_address` 是派生展示值；钱包是保存密钥并签名的软件，不是链账户类型。
- 当前代码仍存在的 wallet/admin/owner/pubkey/address 同义字段必须按任务卡分步删除，不得新增或保留兼容分支。

## 4. 产品矩阵与职责

### 4.1 CitizenChain

- 代码目录：`/Users/rhett/GMB/citizenchain`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- 内部 OnChina 文档：`/Users/rhett/GMB/memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`
- 技术栈：
  - 链节点与 Runtime：Rust + Substrate / Polkadot SDK
  - 桌面节点 UI：Rust + Tauri + React + TypeScript + Vite
  - OnChina：Rust + Axum + PostgreSQL + React + TypeScript + Vite
- 核心职责：
  - 链上状态机与共识
  - 治理、发行、交易、资格接入
  - 原生节点程序与桌面节点软件
  - OnChina 多机构工作台、注册局业务、行政区、机构登记、管理后台、公开查询和链侧凭证

### 4.2 CitizenApp

- 代码目录：`/Users/rhett/GMB/citizenapp`
- 产品文档：`/Users/rhett/GMB/memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- 技术栈：Flutter + Dart + Isar + smoldot
- 核心职责：
  - 在线钱包与端上签名
  - 治理和交易入口
  - 内置轻节点链上读取与交易提交
  - 公民身份状态展示
  - Cloudflare 边缘服务接入：Chat 瞬时密文/信令转发、广场媒体/feed、轻节点启动清单和受控签名交易转发

### 4.3 CitizenWallet

- 代码目录：`/Users/rhett/GMB/citizenwallet`
- 产品文档：尚未单独建立，新增前必须单独确认。
- 技术栈：Flutter + Dart
- 核心职责：
  - 离线钱包和冷签名
  - QR_V1 扫码识别
  - 中文确认页展示
  - 签名响应二维码生成

### 4.4 Website

- 代码目录：`/Users/rhett/GMB/citizenweb`
- 产品文档：尚未单独建立，新增前必须单独确认。
- 技术栈：React + TypeScript + Vite
- 核心职责：
  - 官方网站静态页面
  - 对外展示资料和发布说明入口

## 5. 跨产品主流程

### 5.1 公民绑定主流程

1. OnChina 注册局管理员录入公民电子护照身份。
2. CitizenApp 查询身份状态并提交账户绑定签名。
3. OnChina 验签后写入结构化绑定结果。
4. CitizenChain 通过链侧凭证或公开查询承接资格结果。
5. CitizenApp 读取绑定状态并展示用户身份能力。

### 5.2 链上投票主流程

1. CitizenChain 创建内部投票、联合投票、选举投票或立法投票提案。
2. OnChina 提供人口快照、投票资格校验和投票凭证签名。
3. CitizenApp 作为公民端入口提交投票签名或交易。
4. CitizenChain runtime 完成投票记账、状态流转与结果处理。

OnChina 不实现投票流程；投票流程统一归属投票引擎。

### 5.3 管理员扫码签名主流程

1. OnChina 生成 `QR_V1 / k=1 sign_request`。
2. CitizenWallet 扫码、解码、展示中文确认字段并签名。
3. CitizenWallet 生成 `QR_V1 / k=2 sign_response`。
4. OnChina 回收签名响应并完成验签或提交前校验。

CitizenApp 不承担管理员登录或冷钱包确认职责。

### 5.4 CitizenApp 链连接与边缘服务主流程

1. CitizenApp 启动后先读取本地 chain spec、lightSyncState 和缓存数据库，启动内置 smoldot 轻节点。
2. CitizenApp 通过 Cloudflare 边缘入口获取启动清单、推荐 bootnodes、服务健康状态和聊天/广场入口；这些信息只用于加速和服务发现，不是链上状态真源。
3. CitizenApp 轻节点连接 CitizenChain P2P 网络，并以 finalized 链状态作为余额、身份、提案、投票和交易成功的最终判断。
4. P2P 暂时不可用时，聊天和广场继续走 Cloudflare；链上关键状态进入降级展示，只显示最近 finalized 快照或等待同步状态。
5. 已签名交易可由受控 API 转发到服务节点 RPC，但 API 不接触私钥、不改交易、不把广播成功等同链上成功。

国储会核心节点不作为公民端公共 RPC 入口；生产网络必须拆分核心/权威节点、公开 bootnode、RPC service node、Archive/Indexer 等角色。

### 5.5 公民币订阅主流程

平台订阅与创作者订阅统一使用公民币，并入 `SquarePost` pallet index `34`；详细契约见 `memory/01-architecture/gmb/subscription-part1-tech.md` 和 `memory/07-ai/unified-protocols.md` 的 P-TX-014、P-STORAGE-006。

1. CitizenApp 从 finalized 链状态读取平台价格或创作者付款套餐。
2. CitizenChain 完成首扣，以当前区块唯一共识时间戳按 UTC 真实公历计算到期时间，并登记自动续费调度。
3. 到期后 runtime 无需用户再次签名，自动按最新链上价格从订阅钱包扣款并推进一个真实公历周期；不使用区块高度或固定天数表达期限。
4. 停链期间到期的周期在恢复出块后按到期顺序补扣；余额不足或套餐失效立即终止且不重试。
5. 平台款进入公民链基金会费用账户，创作者款全额进入创作者钱包。
6. CitizenApp 对订阅、取消、换套餐以及创作者覆盖设置自己套餐分别只签名一次并显示链上真实日期；Cloudflare 在交易 finalized 后只用 Bearer 会话与链读复核付款字段、保存镜像和创作者展示资料，镜像及失败重试不得再次生成账户签名或设备请求签名，也不计算日期、不提交扣款、不持有第二份价格真源。
7. OnChina 与 CitizenWallet 只承接公民链基金会平台调价的治理冷签流程，普通订阅保持 CitizenApp 热签。

成为创作者的唯一资格是当前拥有有效平台订阅。创作者改价对存量订阅的下一次真实扣款生效；当前已付周期不补差价。runtime 使用有界到期索引在区块结束阶段自动续费。现有链必须通过 StorageVersion 原地升级保留全部无关状态，禁止重新创世、替换 chainspec 或恢复旧订阅流程。

订阅、取消、换套餐和创作者设置套餐都是用户发起的非系统链上交易，统一收取链上交易费；业务转账金额为零不代表免交易费。runtime 内部自动续费不是外部交易，不另收用户交易费。

## 6. 共享协议与统一口径

### 6.1 QR_V1

- 扫码协议只有一个：`QR_V1`。
- 相关产品和能力：CitizenApp、CitizenWallet、CitizenChain node、OnChina。
- 二维码外层字段、动作码、签名原文、签名响应和中文展示字段必须以 `memory/07-ai/unified-protocols.md` 为唯一登记入口。

### 6.2 链地址与链参数

- 地址编码：`SS58 = 2027`
- 相关产品：CitizenChain、CitizenApp、CitizenWallet、OnChina。
- 地址显示、genesis hash、链 ID、Token 展示和交易 payload 必须跨端一致。

### 6.3 CID 链侧能力

CID 是身份号码和凭证能力，不是独立产品名。链侧能力由 CitizenChain 承接，OnChina 提供数据和凭证来源：

- 机构 CID 登记前置
- 公民身份上链确认
- 投票凭证
- 联合投票人口快照
- 注册局验签账户和管理员集合管理

## 7. 仓库目录结构

```text
GMB/
├── citizenchain/      # 公民链产品代码，含 runtime、node、OnChina
├── citizenapp/        # 公民代码
├── citizenwallet/     # 公民钱包代码
├── citizenweb/           # 官方网站代码
├── memory/            # AI 编程系统与正式文档真源
├── scripts/           # 非部署仓库工具，本机目录
├── citizenconsole/            # 本机私有部署控制台，整目录由 Git 忽略
├── docs/              # 展示资料与静态发布资料
└── .github/           # CI/CD、审查和安装包流水线
```

## 8. 发布边界

本机统一发布入口固定为 `citizenconsole/` 可视化控制台。控制台属于本机私有运维工具，整目录由 Git 忽略，不得纳入 Git、提交或推送 GitHub；`.runtime/`、日志、编译产物、本机状态和私密材料同样只留在本机。首页固定一行四张中文小卡片，顺序为“发币、公民、公民链、公民云”，每张卡片宽度为旧四等分宽度的二分之一并靠左排列。“公民”只是 CitizenConsole 内部 `mobile` 界面分组的展示名，内部分别保留 CitizenApp 与 CitizenWallet 的正式产品身份和动作脚本，不形成第三个移动产品。发币、公民链、公民云三张卡各自点击进入专属整页（非弹窗）：发币页管理稳定币充值发币订单与发币热钱包，公民链页管理节点 CI、发布与权威节点列表，公民云页管理两个生产部署动作与两条会员镜像对账线。三页同构——左侧「返回」、标题居中、返回只改 hash 绝不发 `GET /`（发出即作废发币解锁态、逼用户重刷指纹）；页面宽度统一为 `min(1440px, 100% - 40px)`，主区高度统一为 `calc(100vh - 132px)` 铺满视口。公民链的权威节点列表保留内部滚动，只是可视区随视口变高而不再固定 500px。只有「公民」卡仍点开弹窗显示可执行操作、Keychain/GitHub Secrets 状态和每项密钥的简短中文用途。全部弹窗统一为「无关闭按钮 + 点空白区域收回」（ESC 由 `<dialog>` 原生支持），判定按弹窗矩形精确计算——只比对点击目标是否为弹窗本身，会把「点在弹窗内边距上」误判成点了外面。需要清理的弹窗（开发升级的相机与倒计时、明文查看的密钥值）一律把清理挂在 `close` 事件上而非按钮上，确保从空白区域、ESC 或流程内按钮任一路径关闭都必然执行清理。已登记的本机 Keychain 配置只允许逐次 Touch ID 后在短时弹窗查看，关闭即清除；GitHub 远程 Secret 不可回读。CitizenWeb 不再建立独立控制台模块，官网唯一生产动作归入公民云。官网部署在执行前先做一次**连通性诊断**：用与 wrangler 相同的 node 运行时直连 Cloudflare API，DNS 与连接分别报告，并递归剥 cause、展开 `AggregateError` 打出真实 errno、系统调用与目标地址——undici 走 Happy Eyeballs 会把各地址的真实错误裹进 `cause.errors`，只看 `error.cause.code` 在最常见的 ECONNREFUSED 下只会得到一句无用的 `fetch failed`。**失败原因必须是判断出来的，不许一刀切**：网络层失败、令牌被拒、其它错误分别给不同结论，并报出死在哪一步；工具输出一律原样进日志，禁止 `>/dev/null` 吞掉真实报错（曾因 `wrangler whoami >/dev/null || echo 'CF_DEPLOY_TOKEN 验证失败'` 把一次 TCP 超时误报成令牌问题，连累两轮排查）。成功则只给三行结论：项目与分支、本次部署地址、正式域名与 HTTP 状态码；健康检查看状态码而非 curl 是否报错。

测试部署和 CI 无需密码；production、Release 和服务器部署每次执行前必须通过 macOS Touch ID，失败时不得启动目标命令。部署 Secret 只保存在 macOS Keychain 或 GitHub Secrets，`.ssh`、仓库及根目录不得保留部署私钥明文。移动端 `GMB_APP_KEY` 以 macOS Keychain 为唯一可查看生产真源，GitHub 同名 Secret 只是流水线投影；CitizenApp 或 CitizenWallet 正式 Release 必须在同一次 Touch ID 后先用本机值覆盖投影，同步失败则禁止启动 Release。两个移动端的 CI 与 Release 同时产出 iOS 与 Android 两端产物：iOS job 跑在 `macos-15`，与 Android job 并列互不阻塞，产出**未签名** `Runner.app`——本机只有 Apple Development 证书，分发证书 0 个、描述文件 0 个，团队 `7QJXLLBA6J` 是免费个人团队，开不出分发证书，因此可安装的 `.ipa` 在付费 Apple Developer Program 到位前不可能产出。iOS job 因此暂设 `continue-on-error`（一个发不出去的平台不该挡住 Android 真正能发的正式版），**一旦出现真签名步骤必须同时删掉该行**，由测试反向钉住。iOS 与 Android 共用同一个 `pubspec.yaml` 版本号，不产生第二套版本。GitHub `workflow_dispatch` 使用显式 `mode=ci/release` 隔离构建与发布；服务器部署由本地控制台独立执行，目标服务器直接下载 GitHub 最新成功 CI 产物，CI 模式不得创建 Release 或部署服务器。

`gh` 一律靠 Keychain 内的 `github:GH_TOKEN` 认证，`~/.config/gh/hosts.yml` 永久停用（它每次都触发 Keychain 弹窗）；SSH 私钥替代不了令牌——它只认证 git 协议，而 `workflow_dispatch`、运行状态查询和 `gh secret` 走的都是 REST API。写/删 GitHub Secret 时读取该令牌本身就强制一次 Touch ID，这一次读取即授权，不再叠加第二次 `authorize.production` 指纹。唯独状态轮询（`/api/status`，每几秒一次）永不注入令牌，否则等于每次轮询弹一次指纹；因此它在未认证或离线时必然查询失败，而失败只能上报为**未知**三态，绝不允许压成「未配置」——把一次远程查询失败伪装成业务否定值，会让 GitHub 上真实存在的签名私钥在面板上显示成空缺。密钥状态因此固定为三态：已配置（更换/删除）、未配置（配置）、未知（写入）。

CitizenConsole 的生产进程模型固定为：Apple `Apple Development` 签名并启用 Hardened Runtime
的原生应用作为根进程，创建不可重新连接的匿名 `AF_UNIX socketpair`，再启动签名包内的
Node、网页、动作脚本、充值代码和依赖。刻意不使用 `Developer ID Application` 与 Apple 公证：
二者要求付费会员资源，而本工具只在本机运行、只需本机可信，因此 `notarytool`、`stapler`、
`spctl` 全链路都不存在。Node 只能通过私有父子通道请求 Swift 固定操作目录，不能传入任意
Keychain 项或任意 Touch ID 文案；`get-task-allow=true`、资源被修改或缺少私有通道时均失败
关闭。生产应用不在每次启动时重新编译或自动签名，改代码后必须点首页的「编译」重新换包。

「编译」是全控制台唯一的全局串行点：它替换的是自己脚下那个签名包，因此有任何动作在运行
时一律拒绝启动；其余动作彼此完全独立、可任意并发。换包本身（四次 `mv` + 清理旧包）整段
屏蔽 `INT`/`TERM`，保证要么没换、要么换完——中间被打断会留下既无旧包也无新包的
`.runtime/`，下次启动校验失败关闭，控制台再也进不去。

充值发币页面仍保持一次 Touch ID 后持续解锁且不设置计时器，但发币私钥只驻原生根进程，
普通 UI Node 只持有非私密配置和原生会话状态。每笔发币由原生根进程通过匿名管道调用签名包
内的一次性隔离工作进程；点击锁定、离页、断连或根进程退出时原地清零私钥。控制台 HTTP
继续只监听回环地址，并同时执行 Host、会话 Cookie、Origin、Fetch Metadata、CSP 和安全
响应头校验；生产动作子进程使用环境白名单，不继承控制台完整进程环境。发币控制台页面主体
只保留订单列表，不显示“订单台账”标题；“刷新台账、拉取并发币、解锁/锁定、配置”固定在顶部
左侧同一操作行，红/绿解锁状态圆点固定在订单区域右上角，与顶部和右侧边框各间隔 16px。
“拉取并发币”未解锁时为灰色禁用态，只有**本页**持有已解锁会话才恢复自身配色并可点击：
判据是本页会话 id 而不只是“已解锁”，因为会话可能解锁在另一个页面，那时本页发不出正确的
`x-topup-session` 头，服务端一律 403。按钮初始即为禁用态（解锁状态要等首次配置请求回来），
结算结束后按当前会话重新判定而非无条件放开——途中会话可能已掉线。
“刷新台账”不需要解锁（台账只是本机账本读取，不碰私钥），保持常亮；但手动点击必须给出
带时刻与增量的反馈，空台账重绘后画面逐像素相同，固定文案等于静默。订单表按本地台账现有 `updated_at` 倒序生成从 1 开始的跨页连续展示序号，前端每页固定 20 条，右下角提供上一页和下一页。序号不写入本地台账、Cloudflare 或链上数据。配置列表只通过专用弹窗显示，关闭弹窗立即清理临时输入和短时查看值，
但不改变持续发币解锁会话。

公民云只保留“生产部署、官网部署”两个动作。日常控制台没有 D1、KV、R2 或 Queue
全量删除/重建入口；灾难恢复必须另建隔离流程、另行审查和授权，禁止与发布共享权限。
Worker“生产部署”一次 Touch ID 原子读取完整 Secret 和最小权限 `CF_DEPLOY_TOKEN`，但不读取
`CF_DATA_TOKEN`；本地依赖安装、零漏洞审计、类型检查和测试结束前，生产 Secret 封存在匿名管道且 npm
子进程无权读取 stdin。代码、配置和 Secret 原子上传为 0% 候选版本，通过正式域名 Version
Override 返回的准确 version id 后才一次切至 100%；切流后健康检查失败自动恢复旧版本 100%。
`wrangler.toml` 与 D1 schema 使用受审哈希门禁，普通发布检测到 Route、绑定、持久资源拓扑或
数据库结构变化时失败关闭。“官网部署”复用公民云生产 `CF_ACCOUNT_ID` 和 Pages 最小权限
`CF_DEPLOY_TOKEN`，一次 Touch ID 原子读取，只更新已经存在的 `citizenweb` Pages 项目并使用
`https://www.crcfrcn.com` 做真实健康检查。官网没有测试部署、本地预览、独立密钥或独立
控制台卡片；部署固定使用 `citizenweb/package-lock.json` 锁定的 Wrangler 版本，生产项目
存在性门禁只解析 `wrangler pages project list --json`。

“公民”详情页固定两行：第一行是公民的“编译iOS端 / 编译Android端 / 运行 CI / 正式 Release”，第二行是公民钱包的同四项操作。编译按端拆成两个入口，点哪个就编哪个——刻意不做“一个按钮 + 运行时探测设备”，因为探测总要在失败时选一个回落，而回落的那一端会被当成用户想编的那一端；目标平台由按钮传死，控制台层与编译脚本层各校验一次白名单，探测不到目标平台的设备就报错退出，绝不改编另一端。两端本地编译都**不产出留存产物**，`<product>/target/` 沉淀已整体删除，产物只在 GitHub。四列排不下原来的三分之二宽度，动作行改为占满整行，主标题与说明文字仍居中。公民云页的操作行从左到右为「生产部署、官网部署、平台对账、创作者对账」，「配置」靠右；后两个按钮本身就是对账熔断开关，状态直接写在按钮上，下方状态区只读地展示两条对账线各自的管辖范围与当前状态。两个开关**进页面时不自动读取**，初始为「点击读取」：读开关要用 Cloudflare 数据令牌，每读一次就强制一次 Touch ID，进页面就读等于只想看一眼部署按钮也被按指纹，刷新时停在 `#cloudflare` 更会连弹两次。第一次点击只读不写（不知道当前值就翻转等于闭眼改熔断开关），一次读取把两条线一起填好，之后点击才是真正的翻转；读或写失败时按钮退回可点的「点击重试」，不允许卡在禁用态。对账的真正执行者是 Worker Cron，控制台不提供手动触发入口——开关只用于熔断，关闭期间链上权益不受影响，仅镜像停止更新。公民链页的 5 个动作与「配置」固定同一行、按钮不带副标题；其 6 个公共密钥（`SSH_KEY`、`GH_TOKEN`、`GMB_TOP_KEY`、`GMB_TOP_PUBKEY`、`NRC_SIGNER_PUBLIC_KEY`、`CHAIN_GENESIS_HASH`）只在「配置」弹窗内维护，页面主体只留权威节点列表；44 个节点各自的三项密钥仍在各自条目内。密钥表的增删改查只有一处绑定实现，弹窗与模块详情共用——每一项都要过 Touch ID，复制第二套必然漂移。两个移动产品的 `GMB_APP_KEY`、`SSH_KEY` 和 `GH_TOKEN` 合并去重后各显示一次；每个动作仍携带精确 `productId` 并调用原 CitizenApp 或 CitizenWallet 脚本。全部动作彻底独立并发，一个功能一个日志标签，互不阻塞——曾经的「生产动作全局串行」已删除，它让任意一个卡住的远端任务锁死整个控制台。保留的限制只有两条：同一功能不允许同时开两个任务（44 个权威引导节点的部署各自是独立功能，靠 `nodeId` 参与去重键并在标签标题中带出节点名）；以及「编译」这一个全局串行点——它 `mv` 走的正是其它动作正在使用的签名包并删掉旧包，同一个动作前半段在旧包里跑、后半段解析到新包，混版执行在日志里看不出来，所以有任何动作在运行时一律拒绝编译。日志标签统一三段：**名称 · 状态 · ×**。名称 = 软件中文名 + 动作精简名（`公民iOS端`、`公民钱包Android端`、`公民链WASM CI`、`公民云生产部署`、`公民控制台编译`；44 个节点部署带节点名如 `公民链01 北京`），由服务端 `runTabName` 单源生成并随 `/api/run` 返回，前端不再自拼——前端拿不到节点名，两处必然漂移。动作精简名按 `mode` 取，不对按钮文案做字符串裁剪：按钮文案带着与状态重复的动词（「编译iOS端」「运行 CI」），裁剪规则会随文案改动悄悄失效。软件名一律取中文卡片名，不用 `CitizenWallet` 这类英文正式名。状态永远另占一栏、绝不拼进名称，否则同一任务在不同状态下名字长短跳变；编译任务运行中显示「编译中」、其余显示「运行中」，结束后一律由 `stateLabel` 给出「成功 / 失败 / 已停止」，不存在停在「编译中」的终态。**不提供热载**：编译的语义是把**能直接使用**的 App 装进设备，走 `flutter build` + 安装，而 iOS 必须 release 才能从桌面点开，release 关闭了 Dart VM Service、连 `flutter attach` 都连不上——热载在 iOS 上物理不可能，按两端一致铁律四端一并删除；触发端点、就绪检测和 `status` 事件已整体清除。日志首行仍用完整称谓（产品正式名 + 按钮原文案），与标签栏的精简名各司其职。日志标签任何状态都可关闭，且**关闭即终止**：× 是唯一的收场入口，点它就按进程组终止任务并摘掉标签，远端动作终止的是本机等待进程（如 `gh run watch`），GitHub 上已启动的 CI 不受影响。不设第二个「停止」按钮——曾经拆成「停止=真终止 / ×=只断开本页跟踪」两个按钮，结果编译标签只渲染「热载 + ×」漏了停止，而 `flutter run` 装机后常驻等待热载、永不自行退出，于是标签停不掉、关了刷新又被自动接管，还一直占着 anyRunActive 把控制台「编译」按钮挡死。编译标签的按钮顺序固定为「热载（就绪后）/ 编译中·失败（未就绪）→ ×」，热载是功能按钮、不是收场按钮。运行中的任务一律可停，每个动作都在独立进程组内，按进程组终止整棵进程树；远端动作停的是本机等待进程，GitHub 上已启动的 CI 不受影响。同一页面不会自动重新弹出已主动关闭的运行标签，重新打开控制台仍可接管尚未结束的任务。

- Runtime 升级：修改 `citizenchain/runtime/**` 或被 runtime 直接依赖且影响链上行为的 primitives。
- Native Node / 桌面安装包：修改 `citizenchain/node/**`、桌面前端、Tauri、打包或发布脚本。
- OnChina 服务发布：修改 `citizenchain/onchina/src/**`、`citizenchain/onchina/frontend/**`、数据库、权限、扫码或公开接口。
- Mobile App 发布：修改 `citizenapp/**` 或 `citizenwallet/**`。
- citizenweb 发布：修改 `citizenweb/**`。
- Chain Spec / Genesis 变更：修改 `citizenchain/node` chainspec 或 genesis preset。

## 9. 联调与变更控制

必须同步联调：

- QR_V1、签名和验签：CitizenWallet + 生成方（CitizenApp、CitizenChain node 或 OnChina）。
- 链地址、genesis hash、SS58、交易 payload：CitizenChain + CitizenApp + CitizenWallet + OnChina。
- 绑定、投票凭证、人口快照：CitizenChain + OnChina + CitizenApp。

必须同步更新文档：

- 产品边界变化：更新仓库文档和对应产品文档。
- 模块职责变化：更新产品文档和对应模块文档。
- 共享协议变化：更新 `memory/07-ai/unified-protocols.md`、相关产品文档和模块文档。

## 10. 维护要求

- 不得恢复 旧独立身份系统和旧离线实名系统作为产品目录、产品文档、CI 或部署入口。
- 不得把 OnChina 写成第五个产品。
- 任一产品新增或下线时，必须先更新本文件中的产品矩阵、目录结构与文档索引。
