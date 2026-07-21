# CITIZENAPP 技术总文档（当前实现态）

## 1. 项目定位

`citizenapp` 当前为单仓 Flutter 客户端项目（iOS/Android），不再内置独立后端目录。

边界说明：

- 区块链 Runtime/共识逻辑不在本仓库实现（由 `citizenchain` 提供）
- 公民、机构和清算行身份直接读取链数据
- `citizenapp` 负责端上钱包、登录签名、纯链上支付入口、绑定签名与状态展示
- Cloudflare Session 只证明已登记 P-256 设备子钥控制当前钱包，不证明链上账户存在、余额充足或具备链上业务资格；各业务动作必须独立读取其资格真源

## 2. 当前技术栈

- App：Flutter + Dart
- 手机机密存储：`flutter_secure_storage`（Keychain/Keystore）
- 手机业务存储：Isar
- 链上通信：smoldot PoW 轻节点 + Rust 原生 typed capability（异步 FFI，不阻塞主线程）（`lib/rpc/` + `smoldotdart/` + `smoldotpow/`）
- P2P Chat 路线：聊天 Tab + 钱包账户聊天身份 + Cloudflare 瞬时密文/信令转发 + WebRTC 设备附件 + 近场无网点对点通信；消息、会话和附件只保存在设备，区块链节点不参与聊天
- 外部接口：Cloudflare Worker 承接聊天控制面、广场、会员、支付、媒体资源和端到端加密通讯录密文。通讯录账户与私人名称明文只在设备；公民、机构、管理员与清算行身份统一由 smoldot 读取 finalized runtime storage。
- 行政区字典：安装包内置 `assets/admin_divisions/`，由 `citizenchain/onchina/src/cid/china/china.sqlite` 直接生成；运行中只读本地包，不向 OnChina 联网更新行政区。
- 公权机构包：安装包内置 `assets/public_institutions/`。生成器在同一个 finalized 块分页读取 `PublicManage::Institutions` 与 `PublicManage::InstitutionAccounts`，生成 43 省、49,593 条机构的本地查询索引。manifest 保存块号、块哈希、创世哈希、状态根、分片哈希和机构根，Isar 只缓存该链快照。绑定、付款和权限判断必须精确读取当前 finalized 链状态。

### 2.1 Android 打包和正式签名

- Android 包名固定为 `org.citizenapp`，由 `citizenapp/android/app/build.gradle.kts` 的 `namespace` 与 `applicationId` 共同约束。
- Android 版本号来自 `citizenapp/pubspec.yaml` 的 `version: x.y.z+n`：
  - `x.y.z` 对应 `versionName`
  - `n` 对应 `versionCode`
  - 每次正式发布新 APK 必须递增 `versionCode`，否则 Android 端无法按更新包安装。
- `release` 构建只允许使用固定 release keystore，不允许回退 debug 签名；缺少签名配置时直接失败。
- Android APK 唯一支持 64 位 ARM `arm64-v8a`。`smoldot` native 库只允许写入 `android/app/src/main/jniLibs/arm64-v8a/libsmoldot.so`；不得恢复任何其他 Android ABI，所有 APK 构建命令必须显式使用 `--target-platform android-arm64`，Gradle packaging 必须排除依赖插件夹带的非 ARM64 native 库。发布验收必须检查 APK 的 `lib/` 目录，而不能只检查项目的 `jniLibs/` 源目录。
- 本地正式签名配置只允许写在被 `.gitignore` 忽略的 `citizenapp/android/key.properties`：

```properties
storeFile=app/citizenapp-release.jks
storePassword=...
keyAlias=...
keyPassword=...
```

- GitHub Actions 的 `citizenapp-ci.yml` 分两种模式：
  - push / PR：只构建 Debug APK 做工程检查，不读取正式签名密钥，不发布 GitHub Release，不清理旧 artifact。
  - 手动 `Run workflow`：通过 `GMB_APP_KEY` 注入统一移动端 release keystore，构建正式签名 `公民.apk` artifact，并发布 Android 更新 Release。
- 手动发布只需要配置一个 GitHub Secret：`GMB_APP_KEY`。内容至少包含：
  - `keystore=<base64后的jks>`
  - `password=<keystore密码>`
- `GMB_APP_KEY` 同时用于公民和公民钱包；默认 Android key alias 为 `upload`，如现有 keystore 使用其他别名，可在同一个 secret 内增加 `alias=<key别名>`；key password 默认复用同一个 `password`，不再拆成多个 GitHub secret。
- Android 更新成立的前提是：包名不变、release keystore 不变、`versionCode` 递增。

### 2.2 Android 应用更新

Android 更新只认手动发布的 GitHub Release，不认 push / PR 检查产物。

手动 `Run workflow` 发布时，`citizenapp-ci.yml` 会在正式签名 `公民.apk` 后生成并上传：

- `公民.apk`
- `citizenapp-android-update.json`

更新清单协议：

```json
{
  "app": "citizenapp",
  "platform": "android",
  "package_name": "org.citizenapp",
  "version_name": "1.0.0",
  "version_code": 1,
  "apk_asset": "公民.apk",
  "apk_sha256": "...",
  "published_at": "2026-05-17T00:00:00Z",
  "notes": "citizenapp Android 1.0.0+1"
}
```

关键规则：

- 移动端 Release 不标记为 GitHub `Latest`，避免影响桌面端 Tauri updater 的 `releases/latest/download/citizenchain-latest.json`。
- App 启动进入主界面后异步检查 GitHub Release 列表，寻找包含 `citizenapp-android-update.json` 的 Release。
- 只有清单中的 `app/platform/package_name` 与当前 App 匹配，且 `version_code` 大于本机 `versionCode`，才认为有更新。
- 有更新时，底部“我的”tab 与“我的-设置”入口显示红点；红点只读取 `AppUpdateController.state.hasUpdate`，不另建已读/未读状态。
- 设置页“关于”区域显示真实本机版本；有更新时在版本号前显示“更新”按钮。
- 用户点击“更新”后，App 下载 `公民.apk` 到本机 cache，校验 SHA-256 后才拉起 Android 系统安装器。
- Android 系统负责最终覆盖安装校验；包名或 release keystore 不一致时，系统拒绝安装。

代码边界：

- `lib/update/`：更新清单解析、GitHub Release 检查、APK 下载、SHA-256 校验和更新状态管理。
- `lib/update/update_badge.dart`：更新红点组件，只表达“仍有可安装更新”。
- `lib/main.dart`：主界面启动后触发异步检查。
- `lib/my/user/user.dart`：我的页“设置”入口红点、设置页“关于”区域当前版本和更新按钮。
- `android/app/src/main/kotlin/org/chinanation/citizen/MainActivity.kt`：提供 `org.citizenapp/update` MethodChannel，读取包版本并拉起系统安装器。
- `android/app/src/main/res/xml/update_file_paths.xml`：FileProvider 只暴露 App cache 中的更新 APK。

### 2.3 首次启动权限策略

citizenapp 不采用“首启强制弹出全部权限”的模式，按平台权限模型分层处理：

- 网络权限：Android release 主 manifest 必须声明 `android.permission.INTERNET`；这是普通权限，系统安装时自动授予，不会出现运行时弹窗。iOS 普通外网访问同样不需要运行时授权。
- 通知权限：App 首次进入主界面前展示一次权限说明页，用户点“开启通知并继续”后才申请通知权限。Android 13+ 通过 `POST_NOTIFICATIONS` 运行时权限申请；iOS 通过 `UNUserNotificationCenter.requestAuthorization` 申请。用户拒绝后不阻塞进入 App。
- 相机与相册权限：不在首启强制索取。扫码页、相册识别、头像选择、保存二维码等功能在用户触发时由 `mobile_scanner`、`image_picker`、`saver_gallery` 等功能路径按需申请。
- 安装 APK 权限：仅在用户点击更新并拉起安装器时处理；Android 8+ 如未授权“允许安装未知应用”，跳转系统设置，不能绕过系统确认。

代码边界：

- `lib/security/app_permission_bootstrap.dart`：记录首启权限说明是否已展示，并通过原生 MethodChannel 发起通知权限申请。
- `lib/security/app_permission_gate.dart`：应用锁通过后展示一次权限说明页；拒绝通知权限不影响进入主界面。
- `lib/main.dart`：在 `_AppLockGate` 进入主界面前接入 `AppPermissionGate`。
- `android/app/src/main/AndroidManifest.xml`：声明 release 网络权限与 Android 13+ 通知权限。
- `android/app/src/main/kotlin/org/chinanation/citizen/MainActivity.kt`：提供 Android 通知权限申请通道。
- `ios/Runner/AppDelegate.swift`：提供 iOS 通知权限申请通道。

### 2.4 端上 Isar 读写串行化

- citizenapp 的业务数据统一落在本机 Isar/MDBX 中，低端 Android 机可能同时触发余额刷新、钱包交易流水同步、多签账户发现、钱包创建/导入等写入路径。
- 所有业务读写必须通过 `lib/isar/app_isar.dart` 的 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()` 排队执行，禁止业务模块直接调用 `WalletIsar.instance.db()` 再读写 collection。
- 统一队列对 `MdbxError (11): Try again`、`active transaction` 等短暂 busy 错误做小间隔重试；业务页面不再自行实现 MDBX 重试。
- `isar.writeTxn()` 只允许保留在数据库打开和启动迁移阶段，因为此时统一写队列本身还依赖数据库实例初始化。
- 钱包 settings 行缺失时，普通读取路径可通过统一写队列创建；已经在写事务内部的路径必须调用 `*_InTxn` 方法，禁止嵌套 `writeTxn`。
- 钱包交易流水同步属于低优先级后台任务；监听新区块和 finalized 区块事件时必须复用 `WalletIsar` 队列，遇到本地库繁忙直接让路，不能和钱包页、治理页抢 MDBX 写锁。
- `交易` Tab 是默认首屏，页面自身只允许延迟刷新本地交易流水；不得在启动阶段发起 nonce 轮询确认 RPC、输出旧确认错误或未处理 Isar 异常。
- UI 错误提示必须区分“本地钱包数据库繁忙”和“轻节点/链上读取失败”，不能把 Isar/MDBX 错误包装成区块链网络错误。

### 2.5 投票状态真源

- citizenapp 投票页面不得把 `author_submitExtrinsic` 返回的 txHash、交易池 watch 状态或本地 nonce 推进当成投票成功。
- 内部投票成功真源是 runtime `InternalVote::InternalVotesByTicket(proposal_id, ticket)`；机构票据为 CID + 岗位码 + 钱包，个人多签票据为钱包。
- 联合投票成功真源是 runtime `JointVote::JointVotesByTicket(proposal_id, institution_ticket)`。
- 投票服务必须等待交易入块，并按本次提交的完整票据回读对应 runtime storage；只有读到相同票据的 true/false 后才向页面返回成功。
- 页面不保存或读取账户级 pending 投票。同一钱包兼任多个岗位时，每个 `CID + 岗位码 + 钱包` 的提交、回读和可投状态必须互相独立。
- 交易池 watch 的 `timeout / finalityTimeout / retracted / future / error` 只能作为本次提交失败信息，不能伪造链上票或阻塞该钱包的其他岗位票据。

### 2.6 P2P Chat 技术架构

citizenapp 的 P2P Chat 技术路线已确定为“聊天 Tab 统一入口 + 钱包账户聊天身份 + Cloudflare 瞬时转发 + WebRTC 设备附件 + 近场无网点对点通信”架构：

- 用户入口：公民端在“多签”Tab 与“交易”Tab 之间提供“聊天”Tab；互联网聊天和近场聊天的消息都在“聊天”Tab 集中显示，用户不选择底层通信模式。
- 聊天账户：CitizenApp 钱包账户就是用户可见聊天账户，也是聊天窗口内发起既有转账功能时的收付款账户；创建钱包时由钱包主私钥一次性绑定 P-256 设备子钥，此后聊天设备绑定和会话登录只使用 P-256 设备子钥，钱包 seed 不进入聊天运行态。
- 互联网聊天：Worker 校验钱包 session 和登记设备，Durable Object 只在当前请求中转发 OpenMLS `ChatEnvelope`；未送达密文只留发送设备本机队列，无内容推送在系统允许的后台窗口自动连接两端并触发本机重试。
- 用户主页：广场作者、关注/粉丝、聊天对方与通讯录联系人统一进入 `UserProfilePage`。公开资料继续使用 R2 profile JSON + 资料媒体 + D1/链派生信号；通讯录不复制公开资料。
- 通讯录：联系人明文按默认热钱包隔离保存在 Isar；Cloudflare D1 只保存由热钱包 seed 域隔离密钥生成的 AES-256-GCM 单联系人密文和 HMAC `contact_id`，用于同一钱包换设备恢复。Worker 不持有密钥，也不能读取联系人账户或私人联系人名称。
- 附件：Worker 只转发 SDP/ICE，附件经 WebRTC DTLS DataChannel 设备间传输；Chat 禁止使用 R2。
- 近场通信：不设计独立“局域网模式”，不要求同一路由器；Android 优先 Nearby Connections，必要时回退 Wi-Fi Direct / Wi-Fi Aware / BLE；iOS 使用 Multipeer Connectivity；Android 与 iOS 跨平台近场通过 BLE GATT 做发现和短消息控制。
- 删除边界：区块链节点聊天、桌面通信节点设置、手机节点配对和云端聊天内容存储均不得恢复。
- 原生目录：Android / iOS 近场模块落地时分别使用 `citizenapp/android/chat/`、`citizenapp/ios/chat/`；当前不保留空目录、占位文件或旧目录。
- Flutter 层：`citizenapp/lib/chat/` 承载统一消息层、聊天 Tab 数据模型、加密身份、支付提示、消息存储、发送队列和自动传输路由，通过 Cloudflare transport 或 Platform Channels 调用近场能力。
- 成熟组件优先：端到端加密主选 OpenMLS；外层协议使用 Protobuf；传输、附件分片和近场能力优先复用成熟库或系统框架，不自研底层通信协议和加密算法。
- 公共节点边界：公共归档节点、普通节点和桌面区块链软件不承担 Chat 信令、中继或消息存储；聊天设备公钥、KeyPackage、密文投递状态和撤销状态都属于 Chat 通信体系。

详细方案见：`memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`。

### 2.7 链连接与边缘服务架构

citizenapp 的链连接目标不是 API-only，而是“端上轻节点 + Cloudflare 启动加速 + 受控服务端兜底”的组合：

- 正常状态：App 内置 smoldot 轻节点连接 CitizenChain P2P 网络，best/finalized 能推进；余额、身份、提案、投票、交易成功等关键判断全部以 finalized 链状态为准。
- 降级状态：P2P 暂时不可用或 peers 长时间为 0；聊天、广场、公开目录和本地缓存继续可用，链上关键状态只显示最近 finalized 快照或“等待链同步”，不得用 Worker/API 查询结果替代链上真源。
- 离线状态：设备网络不可用；只展示本地缓存和可离线准备的签名内容，不承诺链上最新状态。
- 启动清单：Cloudflare Worker 已提供 `GET /v1/chain/bootstrap`，返回链身份、推荐 bootnodes、聊天/广场入口和受控广播状态，不返回远端 checkpoint、轻同步资产或 RPC URL；启动清单不是链上状态真源。App 初始化轻节点时会先尝试读取该清单，校验 `chain_id/protocol_id/stateRoot/SS58` 与本地 `chainspec.json` 一致后，才把推荐 bootnodes 注入内存版 chainspec；清单不可用或不匹配时继续使用本地 assets。
- 生命周期：`SmoldotClientManager.ensureStarted()` 是轻节点唯一启动闸口，合并并发初始化并允许失败重试；`dispose()` 异步等待原生 chain/client 释放，生命周期代际切换后旧初始化、旧同步和旧重试不得覆盖新状态。`main.dart` 不再全局预热轻节点；状态进度读取只等待初始化，finalized 读取、交易提交和链事件订阅统一等待同步完成。
- 非链页面边界：广场浏览和“我的”头像身份徽章只读取按钱包账户隔离的 `visitor/voting/candidate` 本地展示快照，不得因此启动 smoldot；快照不是授权、发布或身份真源。广场发布、电子护照详情、交易和治理等主动链流程仍读取 finalized 链状态。轻节点已由其他主动流程进入 operational 后，常驻页面只监听状态变化刷新一次快照，不轮询。
- 交易提交：App 本地完成签名。P2P 可用时优先通过轻节点提交；P2P 不可用但网络可用时，可把已签名 extrinsic 交给受控 API 广播到 RPC service node。API 只转发完整签名交易，不接触私钥、不改载荷、不保存原始 extrinsic body、不把广播成功显示成链上成功；最终成功仍必须来自 finalized runtime storage 或区块事件。
- 广场与聊天：广场媒体/feed 走 Worker/D1/R2/KV；Chat 只使用 Worker/DO 瞬时转发和 D1 最小设备数据，区块链节点不承担聊天中继或媒体存储。

该口径由 `memory/04-decisions/ADR-032-citizenapp-chain-edge-architecture.md` 固化；任何恢复 API-only 链读写、Matrix 聊天路线或区块链节点聊天的方案都必须先改 ADR 并重新确认。

## 3. 当前目录结构

```text
citizenapp/
├── android/
│   └── im/                 ← Android Chat 近场原生模块（Nearby Connections 或 Wi-Fi fallback / BLE）
├── ios/
│   └── im/                 ← iOS Chat 近场原生模块（Multipeer Connectivity / BLE）
├── assets/
├── lib/
│   ├── main.dart
│   ├── Isar/
│   ├── im/                 ← 聊天 Tab、Chat 统一消息层、加密、支付提示、存储、传输抽象
│   ├── rpc/                ← 链上 RPC 公共模块
│   ├── ui/                 ← App 级 UI、底部 Tab 入口壳与通用组件
│   ├── onchain/            ← 普通链上转账 / 纯链上支付
│   ├── trade/              ← 本地交易记录共用能力（非功能入口）
│   ├── offchain/           ← 扫码支付 / 清算行能力
│   ├── citizen/institution/← 机构管理(链访问只读核心 + ADR-028 统一机构模型;创建/关闭收归 onchina)
│   ├── transaction/multisig-transfer/ ← 多签转账(公私个共用,交易域)
│   ├── transaction/personal-manage/   ← 个人多签管理(自助创建/关闭)
│   ├── admins-change/      ← 管理员更换一级业务模块
│   ├── 8964/               ← 底部广场 Tab：图文/视频动态、推荐、关注、竞选分类
│   ├── citizen/            ← 公民 Tab：提案 / 立法 / 选举 / 治理 / 公权
│   ├── qr/                 ← 二维码统一模块（登录/收款/用户码）
│   ├── signer/
│   ├── user/
│   └── wallet/
└── test/
```

说明：

- 原 `mobile/` 内容已上移到项目根目录
- 原 `backend/` 已移除
- 正式产品文档统一收口到 `memory/01-architecture/citizenapp/`

## 4. App 当前实现

### 4.1 主导航

底部 5 Tab：

- `广场`
- `公民`
- `信息`
- `交易`
- `我的`

### 4.2 公民 Tab

`lib/citizen/` 是底部“公民”Tab 的入口与公民展示域：

- `citizen/citizen_tab_page.dart`：公民 Tab 二级导航（提案 / 立法 / 选举 / 治理 / 公权）
- `citizen/all/`：公民 Tab 内“提案”页，展示默认公共机构 + 当前钱包订阅公权机构的统一提案流
- `8964/`：底部“广场”Tab 目录，承载图文/视频动态、推荐、关注、竞选分类和发布入口
- `citizen/public/`：公权机构地理浏览与关注组
- `citizen/governance/`：治理 tab 壳和 NRC/PRC/PRB 浏览入口
- `citizen/proposal/`：统一发起提案入口与各提案页面
- `citizen/institution/`：机构身份/账户/管理员**只读**链访问核心 + ADR-028 统一机构模型(机构创建/关闭已收归 onchina 控制台 + 冷钱包)
- `transaction/multisig-transfer/`：机构(公权/私权)+ 个人**共用**多签转账交易
- `transaction/personal-manage/`：个人多签创建、关闭、发现与链上交互(公民自助)

跨 pallet 共用能力分布：

```text
lib/citizen/shared/proposal/  ← 提案通用模型/上下文/查询/限额/缓存
lib/citizen/proposal/proposal_registry.dart ← ProposalSubject → ProposalCapability 能力表
lib/votingengine/internal-vote/  ← 内部投票客户端(投票服务/查询/待确认存储/投票UI)
lib/votingengine/joint-vote/     ← 联合投票客户端预留目录
lib/votingengine/legislation-vote/ ← 立法投票客户端(法律案表决与立法公投进度)
lib/transaction/multisig-transfer/ ← 多签转账业务(创建/详情/投票/列表适配/缓存统一收口,交易域)
```

管理员更换归 `lib/citizen/proposal/admins-change/`;机构管理(身份/账户/管理员只读)归 `lib/citizen/institution/`(机构创建/关闭收归 onchina);多签转账归 `lib/transaction/multisig-transfer/`;个人多签归 `lib/transaction/personal-manage/`。统一发起提案入口只负责主体能力判断和页面路由,不复制业务提交流程。

### 4.2.1 公民宪法阅读页

- 公民宪法入口固定读取链上 `law_id=0`；运行态唯一真源是 `LegislationYuan.Laws`、`LegislationYuan.LawVersions`、`LegislationYuan.LawVersionLabels` 与 `ConstitutionImmutableManifest` 的 finalized storage 原始 SCALE 数据。
- `citizen/legislation/data/legislation_api.dart` 只把本机 `AppKvEntity` 用作展示快照：首次无快照时页面可等待链上读取；已有快照后再次进入先显示本地正文，再后台按链上有效版本、待生效版本、正文 `contentHash`、版本标签和不可修改条款 manifest 核对更新。
- 本地快照不得参与投票资格、法律有效性或链上状态判断；缓存内容与链上原始 SCALE 完全一致时不重复改写本机记录。
- `citizen/legislation/law_reader_page.dart` 使用链上正文自带的章、节、条标题，不在 UI 前端额外拼接“第 x 章 / 第 x 节 / 第 x 条”；只有链上标题为空时才使用兜底标题。
- 公民宪法默认不展开章和节；用户只能通过标题行右侧展开按钮展开或收起。节级结构与章级结构一致，默认收起。
- 长正文滚动时章标题使用 pinned header 固定在 AppBar 下方，便于用户在阅读过程中直接收起当前章；正文内容继续独立滚动。
- 款正文直接显示链上 `Clause.text/textEn`，不得在 UI 层额外拼接“第 x 款 / Paragraph x”；宪法真源中的款正文已自带“第一款 / Paragraph 1”等正文编号。
- 不可修改条款徽章中文固定显示“不可修改条款”，英文固定显示“Immutable Clause”。
- 版本名只读链上 `LawVersionLabels[(law_id, version)]`；公民宪法 `law_id=0/version=1` 有标签时中文显示“创世版”、英文显示“Genesis Edition”，没有标签的版本继续显示 `vN`。
- 英文模式下，标题、顶部 `Constitution · Genesis Edition/vN`、生效时间、版本史、投票类型和“Immutable Clause”徽章统一显示英文；中文模式显示对应中文。

### 4.2.2 机构页

- 机构分类卡片已内置：
  - 国家储委会 1
  - 省储委会 43
  - 省储行 43
- 省储委会、省储行分类列表固定为一行两列展示，避免因不同 Android 机型逻辑宽度差异出现单列大卡或列数漂移；国家储委会单张卡片横跨整行显示到右侧边缘，但高度与省储委会/省储行卡片一致
- 国家储委会保持直接展示；省储委会、省储行默认折叠，标题行最右侧使用线性右箭头/下箭头展开或折叠对应 43 个机构
- 省储委会分组标题图标使用 `assets/icons/government-line.svg`，省储行分组标题图标使用 `assets/icons/bank.svg`；机构卡片内部不显示全称/简称左侧图标，只显示机构简称和右箭头
- 省储委会、省储行卡片支持同分组内长按拖拽排序；排序只保存为本机 UI 偏好，不写链、不跨设备同步，也不再按管理员机构自动顶前
- 治理机构详情页固定数据立即显示：机构全称/简称、类型、身份 ID、制度账户、内部门槛等均来自本地静态注册表或制度常量，不等待链上 RPC
- 管理员列表、当前用户管理员身份、主账户余额和机构提案列表属于动态数据，进入详情页后分区后台读取；任何单项读取失败只能影响对应区域，不得让详情页整页转圈
- 管理员模型按目标分流：公权机构为 `PublicAdmin { admin_account, cid_number, family_name, given_name }`，私权机构和个人多签为 `Admin { admin_account, family_name, given_name }`。机构管理员与 entity 岗位任职做左连接，无岗位管理员仍保留但不具备业务权限；个人多签展示及管理员集合变更完全独立，不复用机构岗位模型。
- 更多制度账户余额只在用户展开账户区时按需读取；下拉刷新会强制刷新管理员、主余额、提案和已展开的更多账户余额
- 治理机构详情页使用机构提案索引和摘要缓存；公民-提案使用当前年提案缓存按可见范围过滤，默认机构码为 `NRC/NLG/NSN/NRP/NED/NJD/NSP/PRS`，再叠加当前钱包订阅公权机构的主账户命中提案。提案摘要可写入 `AppKvEntity` 复用，但公民-提案不得读取或保存全局治理索引。
- 提案列表本地缓存只用于展示，不得作为投票资格、是否已投票、执行状态提交前校验的最终真相；这些关键判断仍必须实时读取 runtime storage
- 提案详情页使用 `ProposalDetailLocalStore` 持久化详情快照：转账提案、多签管理提案、Runtime 升级提案进入详情页时先读本机快照；`Voting` 状态按短 TTL 后台刷新链上状态/计票/投票记录，终态提案默认只展示本地快照，手动刷新才重读链
- 管理员主体使用 `AdminAccountService` 的两层缓存：30 秒内存缓存负责同页面/相邻页面去重，`AppKvEntity` 持久化快照负责 App 重启后首屏显示；管理员变更、激活、手动刷新时清对应 subject，投票提交前仍链上复核
- 票据投票记录读取必须批量化：机构内部投票统一通过 `InternalVoteQueryService.fetchTicketVotesBatch()`，联合投票统一通过 `RuntimeUpgradeService.fetchJointTicketVotesBatch()`；个人多签才允许使用账户票查询，禁止详情页逐个选民调用 `fetchStorage`
- 余额展示使用 `AccountBalanceSnapshotStore` 持久化短 TTL 快照：机构主账户、安全基金、手续费账户、个人/机构多签关闭页可先显示本地余额；转账/投票/创建/关闭提交前余额检查仍必须实时读链
- 提案列表、管理员投票、转账提案、立法投票展示和 Runtime 升级主要路径已接入；联合公投提交仍待后续补齐

### 4.3 广场 Tab

- 2026-07-05 起底部第 1 个按钮为“广场”，App 启动默认进入广场推荐页；“公民”Tab 右移到第 2 个按钮。
- 广场入口仍为 `lib/8964/square_tab_page.dart`，该入口直接挂载 `lib/8964/pages/square_home_page.dart`。
- `lib/8964/` 是广场功能的唯一代码目录；公民 Tab 内旧“广场”子 tab 已改为“提案”，代码迁移到 `lib/citizen/all/`。
- 当前代码已提供推荐、关注、竞选三分类前端壳、发布页和详情页；目标状态为用户图文/视频动态广场，不承载个人多签、机构账户或提案列表逻辑。
- 广场用户身份统一使用钱包账户 `owner_account`；会员身份、关注关系、推荐信号、发布草稿和上传任务都绑定 `owner_account`。
- 会员体系为三档（ADR-036，**会员与身份彻底解耦**）：自由会员 `freedom`、民主会员 `democracy`、薪火会员 `spark`。`membership_level` 是纯付费订阅轴，任意身份可订阅任意会员档；发帖分类权限按链上身份，Cloudflare 用量额度按平台会员档，两个权限轴互不替代。平台价格唯一真源为 finalized `SquarePost::PlatformPrice`，付款统一使用链上公民币；CitizenApp 保留三张会员卡，在 App 内完成订阅、取消和换档的一次热钱包签名，不打开外部支付页面。权益口径为未陈旧 finalized 链时钟下仍未到 `paid_until` 的 `Active` 或 `Cancelled`；`Terminated`、过期、缺失或陈旧镜像全部拒绝。
- 认证用户是链上已绑定 `cid_number` 的钱包账户；非认证用户是未绑定 `cid_number` 的钱包账户。身份认证与会员档位彼此独立（ADR-036）。普通动态 / 普通文章三档会员都可发布但额度不同；竞选动态 / 竞选文章按竞选身份（`candidate`）校验，与会员档无关。当前 runtime 的 `campaign` 链上发布仍按 `VotingIdentityByAccount` 拦截（voting+）；App 业务侧（compose / SquarePublishService / Worker `prepareUpload`·`confirm`）按更严的竞选身份 `candidate` 校验；若未来要求链上也强制 Candidate 身份，必须按 runtime 二次确认规则单独修改。
- 广场默认分类为推荐；用户可切换关注、竞选，后续可按产品需要增加最新分类。推荐流初期只做可解释规则，不做黑盒模型。
- 广场媒体内容不存链上，不改造 CitizenChain 全节点存储媒体；`manifest.json` 存 Cloudflare R2，图片/首图经 Worker 有界校验后由服务端写 Cloudflare Images，视频全部使用绑定精确字节和最长时长的 Cloudflare Stream TUS，经签名 Images delivery / Stream playback URL 访问。
- CitizenChain 负责发布交易入块、统一链上交易收费、竞选发布权限校验、发布索引和事件；`SquarePost` pallet index 为 `34`、发布 call index 为 `0`。同一 pallet 的订阅 call、状态、价格、扣款和自动续费契约见 P-TX-014/P-STORAGE-006。
- Cloudflare Worker 负责设备子钥钱包登录、finalized 订阅镜像与权益门禁、链上身份资格校验、加密通讯录密文 CRUD、统一资源限制、D1 原子额度预留、R2/Images/Stream 写入、上传回执、Stream webhook 实际时长/分辨率复核、链上发布事件确认、帖子删除和 feed。登录 Session 不读取 `System.Account` 或余额；链身份/余额只在需要它的业务入口校验。`citizenapp/cloudflare/src/limits/catalog.ts` 是所有请求体、文件、账户数量、周期用量和出站载荷的唯一硬上限；环境变量只能收紧。`POST /v1/square/uploads/prepare` 在调用媒体提供商前先用未陈旧链时钟校验平台订阅，再原子预留活动上传数、订阅周期图片数和视频秒数；`complete` 核销一次，删除帖子只回收实际存储总量而不返还周期上传额度。
- App 端发布闭环当前口径：`lib/8964/services/square_api_client.dart` 负责 Worker 登录、会员和上传；manifest、profile 与图片 PUT 都对原始字节生成 P-256 请求签名，视频只向 Stream TUS 地址发送字节。`lib/8964/services/square_upload_service.dart` 生成 manifest、取得 `post_id/storage_receipt_id` 与 `worker/tus` 上传计划；最终额度和真实文件校验只以 Worker 为准。修改内容仍视为新发布，新帖确认成功后再硬删除旧帖 Cloudflare 数据。
- Worker 链上游由 `citizenapp/cloudflare/src/chain/rpc.ts` 通过 `CHAIN_URL` 与两项 `CHAIN_ID / CHAIN_SECRET` Secret 访问 Access 保护的 HTTPS 服务。内部方法白名单只包含 finalized storage、签名交易广播、区块头/区块体/规范区块哈希读取所需方法，不接收 App 指定的 method 或 RPC URL。订阅镜像必须复核完整已签名 extrinsic 的 finalized 区块包含关系和同一区块 storage；发布确认继续交叉校验链上事件、上传记录和 R2 manifest。
- 阶段 6 已在 App 端改为正式 feed 口径：`SquarePublishService` 链上入块后调用 Worker `POST /v1/square/posts/confirm`，`SquareHomePage` 默认和分类切换均通过 `SquareApiClient.fetchFeed()` 拉取 Worker 推荐、关注、竞选 feed。
- App 发布页已支持图片/视频选择、热钱包本机签名和冷钱包 QR 签名；动态页限制 300 字、最多 9 张图片和 1 个视频；文章页限制标题 10-50 字、正文 UI 上限 30000 字、正文图 UI 上限 100 张，并支持普通文章 / 竞选文章选择。竞选内容在 App 端先按链上 `CitizenIdentity::VotingIdentityByAccount` 查询结果做基础拦截，竞选公民会员资格由 Worker 再按当前会员状态强制校验。
- 阶段 5 的 R2 manifest 是 App 先生成的规范化内容清单，字段包含 `schema`、`owner_account`、`post_category`、可选 `content_format`、可选 `title`、`text`、`media_items[].file_name/content_type/byte_size/sha256`；`content_format` 默认 `normal`，文章写 `article`，链上仍只写 `post_category`。`post_id`、`storage_receipt_id` 和 manifest R2 object key 由 Worker/D1 的 `square_uploads` 记录维护，Images/Stream asset id、provider、状态和播放地址由 `square_media_assets` 维护，不要求 App 在 prepare 前伪造。
- `storage_until` 由 App 读取 Worker 的 finalized 会员镜像字段 `membership.paid_until` 后写入链上发布交易；Worker prepare 已先执行同一未陈旧订阅门禁，响应返回预生成 `storage_receipt_id`，complete 响应返回同一个回执。Worker 不托管钱包资金、不签链上交易，也不计算订阅日期。
- Worker 工程位于 `citizenapp/cloudflare/`，功能目录包括 `auth/`、`membership/`、`contacts/`、`limits/`、`uploads/`、`storage/`、`posts/`、`feeds/`、`chat/` 和 `moderation/`。所有外部方法/path 先匹配路由白名单和正文上限，未知路由不得进入 D1。
- 广场 manifest 的 R2 object key 固定使用 `square/{owner_account}/posts/{post_id}/manifest.json`；头像、背景和 manifest 只通过同域 Worker 上传。图片不生成 R2 主媒体 key，由 Worker 写 Images；视频只拿 Stream TUS 地址。不存在 R2 PUT 预签名、Images 客户端直传或本地开发代理分支。
- Worker D1 使用 `chain_clock`、`square_memberships`、`square_creator_tiers`、`square_creator_subscriptions` 和 `chain_transaction_confirmations` 保存可重建 finalized 镜像与最小交易证明，并使用 `resource_reservations`、`resource_usage`、`resource_totals` 强制资源额度；`square_media_assets.resource_key` 指向统一限制表。KV session、session index 和身份缓存写入前同样校验实际 JSON 字节。Cloudflare API token、Stream webhook secret、R2 冷归档只读签名密钥和链 RPC 地址只放部署环境。
- Worker 本地验证命令固定为 `npm --prefix citizenapp/cloudflare run typecheck`、`npm --prefix citizenapp/cloudflare test`、`npm --prefix citizenapp/cloudflare run db:local`、`npm --prefix citizenapp/cloudflare run dev:local -- --port 8787`。
- Worker 远端命令固定为 `npm --prefix citizenapp/cloudflare run db:staging`、`npm --prefix citizenapp/cloudflare run deploy:staging`、`npm --prefix citizenapp/cloudflare run db:production`、`npm --prefix citizenapp/cloudflare run deploy:production`；`wrangler.toml` 已登记 staging/production 的实际 D1、KV、R2 和公共媒体交付地址，密钥只通过 Wrangler Secret 配置。
- `GET /v1/chain/bootstrap` 属于 Cloudflare 边缘启动清单接口，返回 `citizenapp.chain.bootstrap.v2`：`chain`、`light_client`、`p2p`、`services`、`security`、`degradation`。该接口只治理链身份、公开 bootnodes 和服务发现，不返回 checkpoint、轻同步资产 URL/摘要或 RPC URL，不代理 JSON-RPC，不接触私钥；`signed_extrinsic_relay.enabled` 仅在 Worker 显式配置 `RELAY_ENABLED=1` 且服务节点 RPC 已配置时为 `true`，path 固定 `/v1/chain/extrinsics/relay`。
- `POST /v1/chain/extrinsics/relay` 是已签名交易受控广播兜底接口，只接受 `signed_extrinsic_hex`，只调用服务节点 `author_submitExtrinsic`，拒绝私钥/助记词/seed/keystore 等密钥字段，并用 D1 表 `chain_extrinsic_relays` 记录 `extrinsic_sha256`、`tx_hash`、`request_ip_hash`、状态和错误码用于审计、限流和去重；该表不保存原始 extrinsic body 或 RPC URL。
- App 端接入文件为 `citizenapp/lib/rpc/chain_bootstrap_api.dart`、`citizenapp/lib/rpc/smoldot_client.dart`、`citizenapp/lib/rpc/signed_extrinsic_relay_api.dart` 和 `citizenapp/lib/rpc/chain_rpc.dart`。`ChainBootstrapApi` 只接受 HTTPS 或本地调试 HTTP，拒绝 `api_is_truth=true`、`rpc_proxy=true`、任何 RPC URL 字段，并且只接受固定 signed extrinsic relay path；`SmoldotClientManager` 只把清单中的 bootnodes 当作 P2P 启动加速信息；`ChainRpc.submitExtrinsic` 仅在轻节点提交失败且错误像链路故障时走 relay 兜底，交易本身已显示 invalid/bad proof/stale/future/payment 类错误时不兜底。
- Cloudflare staging API 唯一入口为 `https://www.crcfrcn.com/api-staging`，由 `CitizenApp API Staging` Access 应用保护；R2 bucket、D1 与 KV 继续使用现有 staging 绑定。第 4 步只在本地重建并验收目标基线，没有写远端 D1 或部署 Worker；远端 schema 与活动版本必须在后续获得单独部署授权后重新读取确认，文档不得把历史版本号冒充当前状态。
- Cloudflare 链上游统一为 `chain.crcfrcn.com` 的 Access 保护路径，经健康的 `nrcgch-rpc` Tunnel 到达国储会服务器 `127.0.0.1:18080` 网关，再由网关按固定 JSON-RPC 方法转发本机节点。staging/production Worker 使用同一个 `CitizenChain` Service Auth 凭据组，Secret 值只保存在 Cloudflare。Cloudflare Access 只保留 `CitizenApp API Staging` 与 `chain` 两个应用，只保留对应两个可重用策略和一个链服务令牌。
- Cloudflare production API 唯一入口为 `https://www.crcfrcn.com/api`，并继续使用现有 production R2、D1 与 KV 绑定。会员没有外部支付 webhook；Stream 账户 webhook 固定为 `/api/v1/square/uploads/stream/webhook`。Images/Stream 只签发短期私有交付地址，Feed 不保存长期播放 URL。第 4 步未触碰 production，活动版本和数据库基线必须在获准部署前只读复核。
- Cloudflare DNS 严格保留 8 条：`www`、`chain`、`nrcgch`、`prchbs`、`prches`、`prcsds`、`prcsxs`、`prczss`。production 与 staging API 都复用 `www` 的路径路由，不创建额外 API DNS。
- production 每日 UTC 03:00 扫描退订满 90 天的视频，`ARCHIVE_ENABLED=1` 时按“Stream 导出到 R2 Infrequent Access、确认落成后删除 Stream”的顺序冷归档；重新订阅后从 R2 回灌 Stream。staging 与本地保持关闭。2026-07-11 已用 production 远端绑定触发空扫描，返回 200 且无错误。
- App 端广场/聊天 API 默认即线上 production `SquareApiConfig.prodBaseUrl = https://www.crcfrcn.com/api`（Chat 瞬时转发与广场共用同一 Worker）；`SQUARE_API_URL` 编译期 define 仅作显式本地联调覆盖。官网同源使用 `/api`，原生 App 无 Origin 时使用 P-256 设备请求证明。
- Cloudflare WAF 与 Worker 双层限流继续保护 `/api/*` 与 `/api-staging/*`；Stream webhook 按独立签名入口处理，Worker 再按认证接口、上传、Chat、读取、写入分别使用 IP 哈希或钱包账户做精确限流。具体远端规则在部署步骤只读复核后才能作为当前事实记录。
- Chat 的 staging/production 使用独立 Google FCM 服务账号密钥；附件 WebRTC 只配置公开 Cloudflare STUN 发现直连候选，不配置中继服务、接口、密钥或 D1 表。两套 Worker 的旧中继 Secret 和 Cloudflare 控制台中两个 Realtime 应用均已永久删除。APNs 凭证暂不配置，仅影响 iOS 后台推送，不阻塞 Android FCM、前台 Chat、广场或会员。
- 广场媒体盈利保护由 Worker 强制执行（三档，ADR-036）：自由会员每月 300 图/30 分钟视频/1 个活动上传，民主会员每月 1500 图/180 分钟/2 个活动上传，薪火会员每月 5000 图/1800 分钟/3 个活动上传。媒体估算成本达到有效订阅毛收入预算的 85% 时暂停新视频，达到 100% 时暂停全部新媒体；文字浏览、账户和 Chat 不受媒体熔断影响。
- 广场链上确认本地 E2E 不使用冻结 `--dev` chainspec；冻结 dev spec 可能仍带旧 WASM，metadata 中没有 `SquarePost`。需要先用当前源码 WASM 构建节点，再基于 `citizenchain-fresh` 生成临时 chainspec 并给测试钱包补余额。
- 广场本地 E2E 链节点必须至少两个节点通过 WSS peer 互连。当前节点挖矿逻辑在 `sync_service.is_offline()` 时不会出块，单节点即使有 pending extrinsic 也不会打包；第二节点 bootnode 地址必须使用 `/wss/p2p/{peer_id}`，两端 `system_health.peers > 0` 后才能验证 `publish_square_post` 入块。
- 本地真实 E2E 必须使用与生产相同的 Worker 上传接口和 D1 目标基线；禁止恢复仅本地存在的上传代理。Images/Stream provider 调用使用测试 Token 或 mock，HTTP 路由、签名、有界读取、R2 与 D1 必须真实运行。
- `citizenapp/cloudflare/.gitignore` 必须忽略 `.dev.vars`、`.wrangler/`、`node_modules/`、`coverage/` 和 `dist/`；Cloudflare token、R2 access key、R2 secret key 不得写入仓库。
- CitizenApp 聊天、广场、会员和媒体共用同一套 Worker 源码，但 Cloudflare 远端严格分为 staging 与 production 两套 Worker、D1、KV、R2、路由和 Secret。唯一人工发布与运行态测试入口是根 `citizenconsole/` 本地控制台，其不含密钥的源码由 Git 追踪；生产部署逐次通过 Touch ID，Secret 只保存在 macOS Keychain 或 GitHub Secrets。会员部署验收必须使用真实 signed extrinsic、finalized 链状态、真实 Worker/D1/HTTP 和门禁结果，不再包含外部支付 Sandbox、支付 webhook 或链下订阅授权。
- `citizenapp/cloudflare/migrations/0001_square_core.sql` 是清空数据库后的唯一重建基线，不是可重复执行的增量迁移；当前后续迁移仅保留通讯录和充值业务文件。订阅与创作者旧增量迁移已删除，不允许恢复。双环境部署脚本禁止自动重放基线或未审核迁移；检测到未来新增迁移文件时必须停止发布并单独审查。
- App 本地 Isar 只缓存草稿、上传任务、feed 快照、浏览状态和推荐信号同步状态；本地缓存不得作为发布权限、认证状态或链上发布成功的最终真相。
- 发布流程当前状态：App 校验 finalized 余额 → 钱包签名登录 Worker → Worker 校验会员和内容并原子预留额度、生成 `post_id/storage_receipt_id` 与 `worker/tus` 上传计划 → App 提交链上发布并等待入块 → manifest 与图片以原始字节签名上传 Worker，视频上传 Stream TUS → Worker 按真实文件和 provider 结果核销额度 → complete/confirm 交叉校验链上事件、R2 manifest 与媒体索引后写正式 feed。任一步失败都不得绕过资源限制或回退旧上传路径。
- 链上未入块、余额不足或后台发布流程失败时，App 用 `AppKvEntity` 保存当前钱包的广场发布草稿；草稿只用于用户继续编辑或再次发起发布，不是链上发布成功或 feed 可见的真源。
- Worker 链 RPC 只允许由远端 Secret `CHAIN_URL` 提供 Access 保护的 HTTPS 地址，并由 `CHAIN_ID`、`CHAIN_SECRET` 提供服务令牌；三项必须成套存在，缺失任一项时 relay 对 App 保持关闭。R2 S3 预签名所需变量同样只允许放在 Cloudflare 远端变量或 Secret 中；仓库、CitizenApp、R2 manifest、bootstrap 响应和 relay 响应都不得保存链 RPC 私密地址或 Cloudflare 凭证。
- 统一协议登记见 `memory/07-ai/unified-protocols.md` 的 `P-API-CITIZENAPP-002`、`P-API-CITIZENAPP-004`、`P-API-CITIZENAPP-005`、`P-TX-013`、`P-TX-014` 与 `P-STORAGE-006`；订阅任务卡为 `memory/08-tasks/open/20260716-citizen-coin-subscription.md`。

### 4.4 交易 Tab

- 底部第 3 个按钮文案仍为"交易"，不因目录拆分改名
- 当前 `交易` Tab 进入 `lib/transaction/transaction_tab_page.dart`
- `transaction_tab_page.dart` 复用 `lib/transaction/onchain-transaction/onchain_payment_page.dart` 中的 `OnchainPaymentPanel`，交易页直接展示原链上支付表单
- 交易页顶部保持原结构：
  - 左上角：我的通讯录
  - 中间标题：交易
  - 右上角：选择交易钱包
- `ChainProgressBanner` 保留在交易页内容顶部
  - Flutter widget test 环境下保留提示条结构，但不读取轻节点状态、不启动轮询定时器；真机/debug/release 环境继续正常展示 peer / best / finalized / syncing
- 交易页在链上支付表单上方保留/插入一行双入口：
  - 扫码支付 → `lib/transaction/offchain-transaction/services/offchain_scan_flow.dart`
  - 多签账户 → `lib/transaction/personal-manage/personal_account_list_page.dart`
- `lib/transaction/onchain-transaction/` 只处理普通链上转账 / 纯链上支付
- 扫码支付入口不显示右箭头，仍保持现有扫码支付流程。
- “多签账户”只展示个人多签账户列表；右上角 `+` 直接进入 `lib/transaction/personal-manage/personal_account_create_page.dart`。
- 个人多签列表首屏只读取本机 `PersonalAccountEntity` 和 `PersonalAccountLocalState`，后台只扫描 `PersonalAdmins.AdminAccounts`；不读取、发现、同步或展示任何机构账户。
- 机构(公权/私权)注册、创建、关闭已收归 OnChina 注册局工作台 + 冷钱包；CitizenApp 交易页不提供机构多签注册或展示入口。
- 清算行目录枚举 finalized `ClearingBankNodes`，用户绑定读取 `UserBank`，关键操作不使用长 TTL 权限缓存；机构名称从 finalized 链快照的 Isar 索引补充，主/费账户按统一链原语派生

### 4.5 钱包与签名

钱包能力收口在 `lib/wallet/`：

- `core/`：钱包生命周期、Isar、机密 key 规范、生物识别守卫
- `capabilities/`：登录签名编排、API（电子护照状态/管理员目录）、证明态
- `pages/`：钱包页面
- `widgets/`：钱包专用组件

余额查询由 `lib/rpc/` 模块直连链上节点完成，不经过外部网关。

签名能力收口在 `lib/signer/`：

- `local_signer.dart`：公民本机签名（助记词在手机）
- `qr_signer.dart`：扫码签名协议（私钥在外部设备）

签名算法：`sr25519`。

调用点：

- 登录扫码签名前
- 链上交易签名前

签名前守卫：`WalletManager._readMnemonic()` 内置生物识别/设备密码验证，所有读取助记词的路径自动触发。

### 4.6 二维码模块

二维码模块在 `lib/qr/`，统一管理所有扫码能力：

- 协议定义与路由分发（`QrRouter`）
- 签名请求/响应：生成外部签名请求、扫描签名响应并验签
- 收款码：生成与解析，预填转账表单
- 用户码：通讯录交换，只保留 `QR_V1` 当前格式

关键口径：

- 唯一协议：`QR_V1`
- CitizenApp 不承担 OnChina 管理员扫码登录职责；登录签名请求由 OnChina 页面生成,CitizenWallet 公民钱包签名。
- 链上转账/投票签名使用 `k=1` 请求和 `k=2` 响应；业务动作由 `b.a` 区分。
- 用户协议使用 `k=3 user_contact` 和 `k=4 user_transfer`。

详细技术文档见：`lib/qr/QR_TECHNICAL.md`

### 4.7 双签名模式（技术方案）

- 模式 A：本机签名
  - 私钥/助记词仅保存在手机 secure storage
  - 交易和登录均由 `WalletManager` 调起本机 sr25519 签名
- 模式 B：扫码签名
  - 手机不保存私钥，仅保存钱包地址/公钥
  - 手机生成待签名请求二维码，外部设备签名后返回签名响应二维码
  - 协议由 `QrSigner` 统一编解码与校验（`QR_V1`）
  - 在线手机使用 `QrSignSessionPage`，离线设备使用 `QrOfflineSignPage`
  - 登录签名请求先经过 `LoginSystemSignatureVerifier` 校验 `sys_pubkey + sys_sig`

## 5. 手机端三层存储（当前）

### 5.1 机密层（Secure Storage）

仅存高敏感数据：

- `wallet.secret.<wallet_id>.mnemonic.v1`
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 5.2 业务层（Isar）

钱包域核心集合：

- `WalletProfileEntity`
- `WalletSettingsEntity`
- `LocalTxEntity`
- `WalletTxSyncCursorEntity`
- `AdminGroupCacheEntity`
- `ObservedAccountEntity`
- `LoginReplayEntity`
- `AppKvEntity`

`LocalTxEntity` 只记录本机钱包进入 citizenapp 后的余额变化流水，不补扫导入前历史；`WalletTxSyncCursorEntity` 只记录 finalized 补同步游标，newHeads 只用于把当前区块内命中的流水先标记为 `inBlock`，不得把 best/latest block 当成 finalized。

当前 schema 版本：`wallet.data.schema.version = 3`。v3 会清空旧 `LocalTxEntity` 和 `WalletTxSyncCursorEntity`，从当前本机时刻重新记录交易流水。

### 5.3 偏好层（SharedPreferences）

仍有少量非机密配置使用（按模块逐步收口）：

- 登录防重放记录：`login.used_challenges`
- 电子护照不再写 `myid.*` 本地档案状态；`MyIdService` 只读默认热钱包和链上 `CitizenIdentity::VotingIdentityByAccount`、`CandidateIdentityByAccount`。
- 用户资料：
  - `user.profile.nickname`
  - `user.profile.avatar_path`

## 6. 链上通信架构

### 6.1 通信模式

CitizenApp 的链上真源是内置 `smoldot` 轻节点连接 CitizenChain P2P 网络，不直连任何公网 RPC：

```text
CitizenApp --smoldot/WSS--> 权威 bootnode :30333 --> CitizenChain P2P
CitizenApp --HTTPS--------> Cloudflare Worker --> 聊天、广场、启动清单
已签名交易 --仅链路故障兜底--> Worker --> Access + Tunnel --> 国储会 127.0.0.1:9944
```

余额、身份、提案、投票结果等链上状态由轻节点通过区块头、最终性和 storage proof 验证。Cloudflare bootstrap 只提供经过本地 chain id、protocol id、genesis state root 约束的启动元数据和 bootnodes，不是链上状态真源，也不返回 RPC URL。交易始终在手机本地签名；Worker 只允许在 P2P 提交出现链路故障时广播已经签名的 extrinsic，不接触私钥。

### 6.2 Bootnode 来源与端口

- 当前 chainspec 只登记 6 个已部署 bootnode：`nrcgch`、`prczss`、`prchbs`、`prches`、`prcsds`、`prcsxs`；未部署节点不得写入 App 安装包或 Cloudflare 启动清单。
- App 安装包内置 chainspec 和 light sync state；Cloudflare bootstrap 可补充同一网络的 bootnodes，但不能替换本地信任锚。
- 已部署 bootnode 对公网只开放 `30333/TCP` WSS/libp2p。RPC `9944` 只监听服务器回环地址，不写入 App、不写入 bootstrap，也不作为 App 的节点列表。
- Cloudflare 不运行 CitizenChain 节点；Worker 只通过独立 Tunnel 访问受控链入口，App 通过 6 个已部署 bootnode 进入 P2P 网络。

### 6.3 连接与降级策略

- smoldot 根据 chainspec bootnodes 发现 P2P peers，并在可用 peers 间同步；不维护 HTTP RPC 轮询列表。
- P2P 暂时不可用时，聊天和广场继续使用 Cloudflare 服务；链余额、身份、提案和投票等页面明确进入链状态降级，禁止把 API 缓存冒充 finalized 链状态。
- 已签名交易优先由轻节点提交。只有错误被判定为连接故障、bootstrap 明确启用固定 relay path 且 Worker 服务端开关已启用时，才调用受控广播兜底；invalid、bad proof、stale、future、payment 等链语义错误不得改走 relay。
- Worker 链上游首期只有国储会节点一个；后续需要容灾时只增加少量不同地区的独立 Tunnel 上游，不连接全部权威 bootnode。

### 6.4 链通信公共模块（`lib/rpc/`）

`lib/rpc/` 是链上通信唯一收口模块，所有业务模块共享：

```text
lib/rpc/
├── smoldot_client.dart               ← 轻节点生命周期、P2P 同步、proof 读取与提交
├── chain_bootstrap_api.dart           ← HTTPS 启动清单及安全边界校验
├── signed_extrinsic_relay_api.dart    ← 已签名交易受控广播兜底
├── chain_rpc.dart                     ← 业务统一链调用入口
└── rpc.dart                           ← barrel export
```

详细技术文档见：`memory/05-modules/citizenapp/rpc/RPC_TECHNICAL.md`。

### 6.5 链上能力

| 能力 | 主链路 | 模块 | 状态 |
| --- | --- | --- | --- |
| 余额与链上 storage | smoldot finalized storage proof | `wallet` + 业务模块 | 已实现 |
| 交易流水 | smoldot 同步块与 `System.Events` | `wallet` + `transaction/shared` | 已实现：本机开始跟踪后先 `inBlock`，finalized 后“已确认” |
| 转账 | 本机构造签名，smoldot 提交；连接故障时可受控 relay | `onchain` | 已实现 |
| 提案 | 本机构造签名，通过统一链调用入口提交 | `citizen/proposal` | 已部分实现 |
| 投票 | 本机构造签名，通过统一链调用入口提交 | `citizen/proposal` | 管理员投票已实现，联合公投提交待补齐 |

## 7. 身份与机构数据（当前）

公民身份读取两张身份表，机构和管理员读取对应 pallet finalized storage，清算行读取 `ClearingBankNodes` 与 `UserBank`。

电子护照页面字段约定：

- `identity_wallet_account` 展示为“投票账户”，来源是命中 `VotingIdentityByAccount` 的本机钱包地址。
- `identity_cid_number` 展示为“身份 CID 号”，来源是链上 `VotingIdentity.cid_number`。
- `identity_status` 由链上 `citizen_status` 和护照有效期窗口派生：正常、未生效、已过期、已吊销、未上链、异常、读取失败。
- `passport_valid_from / passport_valid_until` 展示链上护照有效期。
- 页面固定渲染匿名访客、投票身份、竞选身份三张卡；当前身份卡首位且唯一标记“当前身份”。匿名访客卡只显示“没有公民身份信息”。
- 非当前公民卡只渲染字段名称，不渲染真实值、空值占位或示例值；当前竞选身份卡可展示投票身份与竞选身份字段，非当前投票卡不得复制这些值。
- 投票身份卡字段为投票账户、公民身份 CID 号、居住选区、身份状态、投票身份有效期；竞选身份卡另含公民姓名、性别、出生日期、出生地。
- finalized storage 读取或 SCALE 解析失败时，三卡继续存在，但均不标记当前身份、不展示真实值；错误态不能伪装成匿名访客。
- 电子护照页不得展示护照号、本机登记状态、钱包二维码、选择钱包、更换钱包或扫码签名按钮。
- 注册局完成身份上链后，citizenapp 只通过链上 storage 读取结果；OnChina 本地库不得作为电子护照真源。

### 7.1 区块链能力矩阵（转账 / 提案 / 投票）

| 能力 | 链上入口 | 手机端模块 | 签名域 | 当前状态 |
| --- | --- | --- | --- | --- |
| 转账 | `OnchainTransaction::transfer_with_remark` extrinsic（直连 RPC 节点，备注最多 99 UTF-8 字节） | `lib/onchain` | `onchain_tx` | 已实现（本机签名主链路） |
| 提案 | 业务治理 pallet `propose_*` | `lib/proposal` + 各业务模块目录 | `onchain_tx`（交易签名）+ CID 快照签名字段 | Runtime 升级已接入，多签转账在 `lib/multisig-transfer` |
| 投票 | 业务治理内部投票 / 投票引擎 `joint_vote` / `legislation-vote` / `election-vote` | `lib/proposal` + 各业务模块目录 | `onchain_tx`（交易签名）+ CID 投票凭证签名字段 | 管理员内部/联合投票已接入，联合公投提交待补齐 |

### 7.2 区块链字段与格式标准（总则）

- 链上账户：runtime 内部是 `AccountId32` / `[u8; 32]`，extrinsic call data 写入原始 32 字节。
- App 内部账户标识：`mainAccount` / `account` / `pubkeyHex` 等字段统一使用 64 位 hex，不带 `0x`。
- 用户展示和输入：地址统一显示、扫码和输入为 SS58 字符串（当前链 `ss58 = 2027`）。
- 通讯录联系人地址归属用户展示 / 输入边界，必须保存 SS58；不得把通讯录地址当成内部 `AccountId` hex 转换。
- RPC/JSON 边界：仅在具体接口要求时临时添加 `0x`，不得把内部 hex 当 SS58 传给 `decodeAddress`。
- 机构 ID：链上 `[u8; 48]`，App 统一使用 `0x` + 96 hex 表达。
- 签名算法：统一 `sr25519`。
- `nonce/signature`：治理场景均使用字节向量（运行时上限当前为 64 字节）。
- 提案状态：`voting/passed/rejected`（内部执行失败状态由业务 pallet 事件单独体现）。
- 投票引擎外部禁用项：
  - `create_joint_proposal`（外部调用禁止）
  - `internal_vote`（外部调用禁止）
  - 必须通过业务治理 pallet 发起。

详细字段与流程见：

- `memory/05-modules/citizenapp/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`（普通链上转账）
- `memory/05-modules/citizenapp/governance/GOVERNANCE_TECHNICAL.md`（公民治理 / 提案 / 投票）

## 8. 安全基线（当前）

- 私钥/助记词不落 Isar 与远端服务
- 助记词读取强制生物识别/设备密码验证（存储层统一守卫，不可关闭）
- 设备无生物识别也无密码时自动跳过验证
- 登录系统身份通过密码学签名验证（`sys_pubkey`/`sys_sig`）
- 绑定请求与交易状态依赖外部服务返回

## 9. 已知限制

- 登录防重放当前仍在 `SharedPreferences`，尚未切到 Isar 的 `LoginReplayEntity`
- `MyIdService` 电子护照状态已切为链上只读，不再使用 `SharedPreferences myid.*`
- 链下交易模块仍为占位
- 扫码签名当前已完成协议层实现，业务 UI 仍以本机签名为主
- 提案/投票尚未实现直连 RPC

## 10. 链数据读取路径

- 公民身份只读取 `CitizenIdentity::VotingIdentityByAccount` 与 `CandidateIdentityByAccount`。
- 机构身份和账户读取 `PublicManage` / `PrivateManage` finalized storage；目录首屏读取链快照与 Isar 派生索引。
- 清算行资格和端点读取 `OffchainTransaction::ClearingBankNodes`，用户绑定读取 `UserBank`。
- 管理员标签扫描 `PublicAdmins`、`PrivateAdmins`、`PersonalAdmins` 的 `AdminAccounts`。
- CitizenApp 身份类数据只保留链读取，不设第二条查询路径。

## 11. 关联模块文档

- RPC 模块：`lib/rpc/RPC_TECHNICAL.md`
- 二维码模块：`lib/qr/QR_TECHNICAL.md`
- 签名模块：`lib/signer/SIGNER_TECHNICAL.md`
- 公民治理模块：`memory/05-modules/citizenapp/governance/GOVERNANCE_TECHNICAL.md`
- 用户模块：`lib/user/USER_TECHNICAL.md`
- 钱包模块：`lib/wallet/WALLET_TECHNICAL.md`
- 链上支付模块：`memory/05-modules/citizenapp/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`
