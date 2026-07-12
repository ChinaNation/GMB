# CITIZENAPP 技术总文档（当前实现态）

## 1. 项目定位

`citizenapp` 当前为单仓 Flutter 客户端项目（iOS/Android），不再内置独立后端目录。

边界说明：

- 区块链 Runtime/共识逻辑不在本仓库实现（由 `citizenchain` 提供）
- CID 与链交互由外部服务系统承载
- `citizenapp` 负责端上钱包、登录签名、纯链上支付入口、绑定签名与状态展示

## 2. 当前技术栈

- App：Flutter + Dart
- 手机机密存储：`flutter_secure_storage`（Keychain/Keystore）
- 手机业务存储：Isar
- 链上通信：smoldot PoW 轻节点 + Rust 原生 typed capability（异步 FFI，不阻塞主线程）（`lib/rpc/` + `smoldotdart/` + `smoldotpow/`）
- P2P Chat 路线：聊天 Tab + 钱包账户聊天身份 + Cloudflare 瞬时密文/信令转发 + WebRTC 设备附件 + 近场无网点对点通信；消息、会话和附件只保存在设备，区块链节点不参与聊天
- 外部接口：Cloudflare Worker 与 OnChina 投影 API 只承接聊天、广场、启动清单、公开目录和非链上查询场景；链上状态真源仍是 smoldot 轻节点读取的 finalized runtime storage。电子护照状态直接读取链上 `CitizenIdentity::VotingIdentityByAccount`，不再走 OnChina 本地状态接口。
- 行政区字典：安装包内置 `assets/admin_divisions/`，由 `citizenchain/onchina/src/cid/china/china.sqlite` 直接生成；运行中只读本地包，不向 OnChina 联网更新行政区。
- 公权机构包：安装包内置 `assets/public_institutions/`，发布期从已完成链上 `PublicManage` 投影的 OnChina 真实 HTTP 接口导出；2026-07-04 起源码创世快照口径只包含国家/省/市公权机构 49,593 条，镇级和后续新增机构通过链投影增量进入本地缓存。上一轮 49,581 冻结锚点已因补齐 12 个国家级机构而失效；当前 GitHub `CitizenChain WASM` #99 已正式 bake 到 `genesis_hash=0xb57c61a97f2b1fd7fa78756060a0c3e9a0ed6b1048bb8424b034a8f5f99a9971`、`state_root=0x6a380e96686b152d1eaff8aafc526c23da43058cac2b98be8e98ea1f9e5eff63`，CitizenApp 轻形态 `chainspec.json` 已同步为该 `stateRootHash`，当前端上快照 `public_institution_root=fae09caa31e07cf03953b1a774be72e2614735dce2859a4e2f91fee248955492`。citizenapp 公民端不按 OnChina 管理端“公权机构 / 市公安局 / 教育机构”等后台功能 tab 分流或排除。

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
- 内部投票成功真源是 runtime `InternalVote::InternalVotesByAccount(proposal_id, admin)`。
- 联合投票成功真源是 runtime `JointVote::JointVotesByAdmin(proposal_id, institution, admin)`。
- 本地 `PendingVoteStore` 只表示“已提交但还没有从 runtime 投票 storage 读到结果”，不能覆盖链上状态。
- 交易池 watch 的 `timeout / finalityTimeout / retracted / future / error` 不直接清除 pending；只有链上已记录投票，或 nonce 已消耗但链上仍无投票记录，才更新 pending 状态。
- pending 投票确认窗口为 20 分钟；超过窗口仍无 runtime 投票记录且 nonce 未推进，视为本地提交未进入链，清除 pending 并允许用户重新投票，避免“投票中”无限转圈。
- 投票按钮的 `submitting` 只覆盖签名和提交阶段；拿到 txHash 后必须立即结束按钮转圈，链上确认由后台刷新和 pending 状态机处理。

### 2.6 P2P Chat 技术架构

citizenapp 的 P2P Chat 技术路线已确定为“聊天 Tab 统一入口 + 钱包账户聊天身份 + Cloudflare 瞬时转发 + WebRTC 设备附件 + 近场无网点对点通信”架构：

- 用户入口：公民端在“多签”Tab 与“交易”Tab 之间提供“聊天”Tab；互联网聊天和近场聊天的消息都在“聊天”Tab 集中显示，用户不选择底层通信模式。
- 聊天账户：CitizenApp 钱包账户就是用户可见聊天账户，也是聊天窗口内发起既有转账功能时的收付款账户；创建钱包时由钱包主私钥一次性绑定 P-256 设备子钥，此后聊天设备绑定和会话登录只使用 P-256 设备子钥，钱包 seed 不进入聊天运行态。
- 互联网聊天：Worker 校验钱包 session 和登记设备，Durable Object 只在当前请求中转发 OpenMLS `ChatEnvelope`；未送达密文只留发送设备本机队列，无内容推送在系统允许的后台窗口自动连接两端并触发本机重试。
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
- 启动清单：Cloudflare Worker 已提供 `GET /v1/chain/bootstrap`，返回推荐 bootnodes、链参数、lightSyncState/checkpoint 摘要、聊天/广场入口和受控广播占位，用于缩短首次连接时间和明确降级原因；启动清单不是链上状态真源。App 初始化轻节点时会先尝试读取该清单，校验 `chain_id/protocol_id/stateRoot/SS58` 与本地 `chainspec.json` 一致后，才把推荐 bootnodes 注入内存版 chainspec；清单不可用或不匹配时继续使用本地 assets。
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
- 管理员列表展示**链上实名资料**(A2 `AdminProfile`):姓名/职务/任期/来源/身份CID/账户;端侧由 `lib/citizen/shared/admin_profile.dart` 按机构码路由 Public/Private/Personal Admins 解码，由 `lib/citizen/shared/admin_profile_card.dart` 固定渲染顶部“序号/激活状态”、第 1 行“姓名:/职务:”、第 2 行“任期:/来源:”、第 3 行“身份CID:”、第 4 行“账户:”、第 5 行“余额:”。字段值为空时只留空值区域，不隐藏标签、不显示本地姓名兜底；余额通过 `ChainRpc.fetchFinalizedBalances` 批量读取 finalized `System.Account.free`，0 余额正常显示，查询失败才留空。个人多签 kind=2 仅账户无资料。机构(公权+私权)创建/关闭已收归 onchina,citizenapp 不再发起,管理员展示为**只读**(治理档可冷钱包激活)
- 更多制度账户余额只在用户展开账户区时按需读取；下拉刷新会强制刷新管理员、主余额、提案和已展开的更多账户余额
- 治理机构详情页使用机构提案索引和摘要缓存；公民-提案使用当前年提案缓存按可见范围过滤，默认机构码为 `NRC/NLG/NSN/NRP/NED/NJD/NSP/PRS`，再叠加当前钱包订阅公权机构的主账户命中提案。提案摘要可写入 `AppKvEntity` 复用，但公民-提案不得读取或保存全局治理索引。
- 提案列表本地缓存只用于展示，不得作为投票资格、是否已投票、执行状态提交前校验的最终真相；这些关键判断仍必须实时读取 runtime storage
- 提案详情页使用 `ProposalDetailLocalStore` 持久化详情快照：转账提案、多签管理提案、Runtime 升级提案进入详情页时先读本机快照；`Voting` 状态按短 TTL 后台刷新链上状态/计票/投票记录，终态提案默认只展示本地快照，手动刷新才重读链
- 管理员主体使用 `AdminAccountService` 的两层缓存：30 秒内存缓存负责同页面/相邻页面去重，`AppKvEntity` 持久化快照负责 App 重启后首屏显示；管理员变更、激活、手动刷新时清对应 subject，投票提交前仍链上复核
- 管理员投票记录读取必须批量化：内部投票统一通过 `InternalVoteQueryService.fetchAdminVotesBatch()`，联合投票统一通过 `RuntimeUpgradeService.fetchJointAdminVotesBatch()`，禁止详情页按 43 个管理员逐条 `fetchStorage`
- 余额展示使用 `AccountBalanceSnapshotStore` 持久化短 TTL 快照：机构主账户、安全基金、手续费账户、个人/机构多签关闭页可先显示本地余额；转账/投票/创建/关闭提交前余额检查仍必须实时读链
- 提案列表、管理员投票、转账提案、立法投票展示和 Runtime 升级主要路径已接入；联合公投提交仍待后续补齐

### 4.3 广场 Tab

- 2026-07-05 起底部第 1 个按钮为“广场”，App 启动默认进入广场推荐页；“公民”Tab 右移到第 2 个按钮。
- 广场入口仍为 `lib/8964/square_tab_page.dart`，该入口直接挂载 `lib/8964/pages/square_home_page.dart`。
- `lib/8964/` 是广场功能的唯一代码目录；公民 Tab 内旧“广场”子 tab 已改为“提案”，代码迁移到 `lib/citizen/all/`。
- 当前代码已提供推荐、关注、竞选三分类前端壳、发布页和详情页；目标状态为用户图文/视频动态广场，不承载个人多签、机构账户或提案列表逻辑。
- 广场用户身份统一使用钱包账户 `owner_account`；会员身份、关注关系、推荐信号、发布草稿和上传任务都绑定 `owner_account`。
- 会员体系为四档，**会员档 `membership_level` 与身份档 `required_identity_level` 解耦**（`plans.ts`）：访客身份含自由会员 `freedom`、民主会员 `democracy` 两档（民主权益对齐投票公民，身份仍为 `visitor`）；投票公民会员 `voting`；竞选公民会员 `candidate`。**订阅资格精确匹配**（`identityEligibleForPlan`：`required_identity_level === identity_level`，**禁止降档/越级**）：访客身份只能订 `freedom` / `democracy`，投票身份只能订 `voting`，竞选身份只能订 `candidate`。**发帖额度按所购会员套餐**（`membershipPlan(level).quota`，民主即得投票额度；竞选专属发帖仍以 `membershipLevel==='candidate'` 闸门）。官网 `citizenweb/src/pages/Membership.tsx` 与 App `lib/my/membership/membership_page.dart` 均按「一张访客卡内自由/民主左右切换（默认自由）」呈现，非本人身份档卡订阅置灰。Stripe subscription webhook 写入 Cloudflare Worker / D1；App 按当前状态显示“订阅会员 / 取消订阅 / 续订会员”，操作统一打开官网，不在 App 内嵌支付。`resolveMembershipEntitlement` 以链上 storage 为资格真源。
- 会员套餐的业务计价唯一使用美元：自由会员 `freedom` `$2.99/month`、民主会员 `democracy` `$9.99/month`、投票公民会员 `voting` `$9.99/month`、竞选公民会员 `candidate` `$99.99/month`。价格到 Stripe Price 的映射统一使用 `FREEDOM_PRICE_ID` / `DEMOCRACY_PRICE_ID` / `VOTING_PRICE_ID` / `CANDIDATE_PRICE_ID`。`plans[]` 暴露 `price_currency = usd`、`price_usd_cents`、`price_usd_monthly`。Stripe Checkout 可按 Stripe 能力让用户使用本地法币或 USDC 完成支付，但本地币种仅属于支付呈现 / 换汇结果，不改变会员套餐计价；USDT 不作为目标支付方式。
- 认证用户是链上已绑定 `cid_number` 的钱包账户；非认证用户是未绑定 `cid_number` 的钱包账户。普通动态 / 普通文章四档会员都可发布但额度不同；竞选动态 / 竞选文章只有竞选公民会员可发布。当前 runtime 的 `campaign` 链上发布仍按 `VotingIdentityByAccount` 拦截，竞选公民会员资格由 Cloudflare Worker 作为 App 业务服务端强制校验；若未来要求链上也强制 Candidate 身份，必须按 runtime 二次确认规则单独修改。
- 广场默认分类为推荐；用户可切换关注、竞选，后续可按产品需要增加最新分类。推荐流初期只做可解释规则，不做黑盒模型。
- 广场媒体内容不存链上，不改造 CitizenChain 全节点存储媒体；`manifest.json` 存 Cloudflare R2，图片/首图走 Cloudflare Images Direct Creator Upload，视频走 Cloudflare Stream Direct Creator Upload / tus，经 Cloudflare delivery / Stream playback URL 访问。
- CitizenChain 只负责发布交易入块、每条发布按最低链上费用扣 0.1 元、竞选发布权限校验、发布索引和事件；阶段 4 已落地 `citizenchain/runtime/otherpallet/square-post/`，pallet index `36`、call index `0`，链上只记录 `post_id`、`owner_account`、可空 `cid_number`、`post_category`、`content_hash`、`storage_receipt_id`、`storage_until`、`created_block`。
- Cloudflare Worker 负责钱包签名登录、官网 Stripe Checkout 创建、官网 Stripe webhook、会员和链上身份资格校验、Images/Stream 一次性上传授权、manifest R2 上传授权、上传回执、Stream webhook、链上发布事件确认、帖子删除清理、推荐/关注/竞选 feed、关注关系、点赞/隐藏/不感兴趣/举报等推荐信号。`GET /v1/square/membership` 返回 `plans[]`、链上身份资格、可订阅等级、支付状态和最终权益状态；官网先调用 `POST /v1/square/membership/subscribe/challenge`，钱包签名后再调用 `POST /v1/square/membership/subscribe` 创建 Stripe subscription Checkout；取消订阅使用 `/cancel/challenge` 与 `/cancel`，已取消待到期的会员通过重新订阅恢复续订。`POST /v1/square/membership/webhook` 使用 `Stripe-Signature` 与 `STRIPE_HOOK_SECRET` 校验订阅回调，并在写入 `square_memberships` 前强制校验 Stripe subscription item 的 Price 为 USD 且金额匹配四档会员。广场主媒体存入 `square_media_assets`，图片 provider 为 `cloudflare_images`，视频 provider 为 `cloudflare_stream`。`POST /v1/square/uploads/prepare` 必须携带 `content_format`、`title_length`、`text_length`，Worker 按当前有效会员计划先拒绝超额声明；`complete` 再读取 R2 manifest、校验 manifest hash、owner、分类、正文、标题、媒体数量和 `square_media_assets` 一致性，防止绕过 App 伪造声明。`DELETE /v1/square/posts/{post_id}` 只允许作者本人调用，并硬删除 Images/Stream provider 资产、R2 manifest、D1 媒体索引、上传任务和帖子行；链上 `SquarePosts` 不改写。
- App 端发布闭环当前口径：`lib/8964/services/square_api_client.dart` 负责 Worker 登录、会员、上传接口、`content_format/title_length/text_length` 声明和帖子删除接口；`lib/my/user/user.dart` 在「我的」Tab 钱包和通讯录之间提供会员入口，展示当前订阅状态与四档权益，并按状态提供打开官网的订阅、取消、续订命令；`lib/8964/services/square_upload_service.dart` 负责生成内容 manifest、在 prepare 阶段取得 `post_id/storage_receipt_id` 和 Images/Stream 上传 URL，并按 Worker 返回的当前会员计划做交互前置校验，最终仍以 Worker 强校验为准；`lib/8964/chain/square_chain_service.dart` 负责编码并提交 `SquarePost.publish_square_post`；`lib/8964/services/square_publish_service.dart` 串联“余额校验、链上扣费入块、媒体上传、Worker 确认 feed”。动态/文章详情页的“修改”视为新发布：新 `post_id/content_hash/storage_receipt_id` 入块并确认成功后，再调用 Worker 删除旧帖 Cloudflare 数据；新发布失败时不触碰旧帖。
- 阶段 6 已在 Worker 侧接入链上发布确认：`citizenapp/cloudflare/src/chain/rpc.ts` 通过 `CHAIN_URL` 与两项 `CHAIN_ID / CHAIN_SECRET` Secret 访问 Access 保护的 HTTPS 上游并读取指定区块 `System.Events`，`src/chain/square_event.ts` 解码 `SquarePostPublished`，`src/posts/confirm.ts` 在事件字段、上传记录和 R2 manifest 全部一致后写入 `square_posts.post_state = published`。Worker 链调用器只允许内部固定的 `state_getStorage` 和 `author_submitExtrinsic`，不接收 App 指定的 method 或 RPC URL。
- 阶段 6 已在 App 端改为正式 feed 口径：`SquarePublishService` 链上入块后调用 Worker `POST /v1/square/posts/confirm`，`SquareHomePage` 默认和分类切换均通过 `SquareApiClient.fetchFeed()` 拉取 Worker 推荐、关注、竞选 feed。
- App 发布页已支持图片/视频选择、热钱包本机签名和冷钱包 QR 签名；动态页限制 300 字、最多 9 张图片和 1 个视频；文章页限制标题 10-50 字、正文 UI 上限 30000 字、正文图 UI 上限 100 张，并支持普通文章 / 竞选文章选择。竞选内容在 App 端先按链上 `CitizenIdentity::VotingIdentityByAccount` 查询结果做基础拦截，竞选公民会员资格由 Worker 再按当前会员状态强制校验。
- 阶段 5 的 R2 manifest 是 App 先生成的规范化内容清单，字段包含 `schema`、`owner_account`、`post_category`、可选 `content_format`、可选 `title`、`text`、`media_items[].file_name/content_type/byte_size/sha256`；`content_format` 默认 `normal`，文章写 `article`，链上仍只写 `post_category`。`post_id`、`storage_receipt_id` 和 manifest R2 object key 由 Worker/D1 的 `square_uploads` 记录维护，Images/Stream asset id、provider、状态和播放地址由 `square_media_assets` 维护，不要求 App 在 prepare 前伪造。
- `storage_until` 由 App 读取 Worker 会员状态中的 `membership.expires_at` 后写入链上发布交易；Worker prepare 响应返回预生成 `storage_receipt_id`，complete 响应返回同一个回执，不托管钱包资金，也不签链上交易。
- Worker 工程位于 `citizenapp/cloudflare/`，阶段 3 已提供 `auth/`、`membership/`、`uploads/`、`storage/`、`posts/`、`feeds/`、`moderation/` 模块、D1 迁移、Wrangler 配置和 Vitest 测试；阶段 8 已补齐 staging/production 绑定模板和运维脚本，阶段 9/11 已完成 Cloudflare staging 部署与远端 smoke。
- 广场 manifest 的 R2 object key 固定使用 `square/{owner_account}/posts/{post_id}/manifest.json`；App 不持有 R2 API key，生产上传 manifest 必须先通过 Worker 获取 R2 S3 短期预签名 PUT URL。广场图片/视频不再生成 R2 media object key；App 只拿 Images/Stream 一次性上传 URL。
- `PUT /v1/square/uploads/dev-put` 只是本地 Miniflare/R2 验收代理，只有 `DEV_UPLOAD_PROXY=1` 时可用，生产环境不得开启。
- Worker 侧 D1 只保存动态元数据、会员订阅状态、登录挑战、关注、上传状态、媒体 provider asset 状态和推荐信号；KV 只做 session/feed 短缓存，不作为长期内容存储。`square_memberships` 当前字段包含会员等级、Stripe customer/subscription/price、订阅状态、周期结束时间、链上身份等级快照和最后校验时间；`square_media_assets` 包含 Images/Stream asset id、上传方式、ready/error 状态、播放 URL、缩略图、时长、宽高；本地法币展示金额、换汇结果、USDC 钱包流水等支付审计信息留在 Stripe 后台，不进入会员权益表；Stripe secret key、Stripe webhook secret、Cloudflare API token、Stream webhook secret、R2 secret、链 RPC 完整 URL 一律只放部署环境。
- Worker 本地验证命令固定为 `npm --prefix citizenapp/cloudflare run typecheck`、`npm --prefix citizenapp/cloudflare test`、`npm --prefix citizenapp/cloudflare run migrate:local`、`npm --prefix citizenapp/cloudflare run dev:local -- --port 8787`。
- Worker 远端命令固定为 `npm --prefix citizenapp/cloudflare run migrate:staging`、`npm --prefix citizenapp/cloudflare run deploy:staging`、`npm --prefix citizenapp/cloudflare run migrate:production`、`npm --prefix citizenapp/cloudflare run deploy:production`；`wrangler.toml` 已登记 staging/production 的实际 D1、KV、R2 和公共媒体交付地址，密钥只通过 Wrangler Secret 配置。
- `GET /v1/chain/bootstrap` 属于 Cloudflare 边缘启动清单接口，返回 `citizenapp.chain.bootstrap.v2`：`chain`、`light_client`、`p2p`、`services`、`security`、`degradation`。该接口只治理链身份、公开 bootnodes 和服务发现，不返回 checkpoint、轻同步资产 URL/摘要或 RPC URL，不代理 JSON-RPC，不接触私钥；`signed_extrinsic_relay.enabled` 仅在 Worker 显式配置 `RELAY_ENABLED=1` 且服务节点 RPC 已配置时为 `true`，path 固定 `/v1/chain/extrinsics/relay`。
- `POST /v1/chain/extrinsics/relay` 是已签名交易受控广播兜底接口，只接受 `signed_extrinsic_hex`，只调用服务节点 `author_submitExtrinsic`，拒绝私钥/助记词/seed/keystore 等密钥字段，并用 D1 表 `chain_extrinsic_relays` 记录 `extrinsic_sha256`、`tx_hash`、`request_ip_hash`、状态和错误码用于审计、限流和去重；该表不保存原始 extrinsic body 或 RPC URL。
- App 端接入文件为 `citizenapp/lib/rpc/chain_bootstrap_api.dart`、`citizenapp/lib/rpc/smoldot_client.dart`、`citizenapp/lib/rpc/signed_extrinsic_relay_api.dart` 和 `citizenapp/lib/rpc/chain_rpc.dart`。`ChainBootstrapApi` 只接受 HTTPS 或本地调试 HTTP，拒绝 `api_is_truth=true`、`rpc_proxy=true`、任何 RPC URL 字段，并且只接受固定 signed extrinsic relay path；`SmoldotClientManager` 只把清单中的 bootnodes 当作 P2P 启动加速信息；`ChainRpc.submitExtrinsic` 仅在轻节点提交失败且错误像链路故障时走 relay 兜底，交易本身已显示 invalid/bad proof/stale/future/payment 类错误时不兜底。
- 阶段 9 已完成 Cloudflare staging 部署：Worker URL 为 `https://citizenapp-square-api-staging.stews87-fawn.workers.dev`，R2 bucket 为 `citizenapp-square-media-staging`，D1 为 `citizenapp-square-db-staging` / `4ba85b05-657a-46ac-ab19-8bbd84fe850a`，KV 为 `staging-FEED_CACHE` / `91133becebc24f27bf10a00cb001f27e`。staging 已完成远端 D1 迁移、`/health` smoke、未登录拒绝和 `dev-put` 禁用验证。
- staging 曾用已废弃的单一链 RPC Secret 完成阶段 12/13 历史验收；2026-07-10 起当前代码不再读取该旧配置。当前已建立 `CitizenChain RPC` Access 应用、Service Auth 策略和 `nrcgch-rpc.crcfrcn.com -> 127.0.0.1:9944` Tunnel 路由，并在 staging Worker 配置 `CHAIN_URL`、`CHAIN_ID`、`CHAIN_SECRET` 三项 Secret；废弃的 `SQUARE_CHAIN_RPC_URL` 已删除。Oracle 服务器 connector 尚未在线，因此带令牌请求当前在通过 Access 后返回 tunnel unavailable，不能把 Cloudflare 控制面配置当作完整私有链路验收。R2 阶段 11、链确认负向 smoke 和 runtime metadata 阻塞结论仍保留为历史事实。
- Cloudflare production Worker URL 为 `https://citizenapp-square-api.stews87-fawn.workers.dev`，R2 bucket 为 `citizenapp-square-media`，D1 为 `citizenapp-square-db-production` / `0c5a0924-83ef-4347-bacc-b3f6f36da460`，KV 为 `citizenapp-square-production-FEED_CACHE` / `b72bbbcb36d240acb317fdaf79ce46f4`。2026-07-11 已配置 R2、链 RPC、Stripe、Images、Stream 全部短名 Secret，启用 Cloudflare Images/Stream Starter 套餐和唯一生产 Stream webhook；`IMAGES_URL` / `STREAM_URL` 已指向真实账户交付域名。production 已通过真实 R2 manifest、Images、Stream 上传授权签发、Stream 签名 webhook、health/bootstrap、会员/聊天未登录拒绝和开发上传入口关闭验收；临时媒体、会员、上传和会话数据均已清理。App 已内置该 URL 为唯一默认并删除本机兜底。
- production 每日 UTC 03:00 扫描退订满 90 天的视频，`ARCHIVE_ENABLED=1` 时按“Stream 导出到 R2 Infrequent Access、确认落成后删除 Stream”的顺序冷归档；重新订阅后从 R2 回灌 Stream。staging 与本地保持关闭。2026-07-11 已用 production 远端绑定触发空扫描，返回 200 且无错误。
- App 端广场/聊天 Worker 地址默认即线上生产 `SquareApiConfig.prodBaseUrl = https://citizenapp-square-api.stews87-fawn.workers.dev`（Chat 瞬时转发与广场共用同一 Worker）；`SQUARE_API_URL` 编译期 define 仅作显式覆盖（HTTPS，或本机 `http://127.0.0.1` 联调）。真机 debug/release 不传 define 时默认直连 Cloudflare。
- 广场链上确认本地 E2E 不使用冻结 `--dev` chainspec；冻结 dev spec 可能仍带旧 WASM，metadata 中没有 `SquarePost`。需要先用当前源码 WASM 构建节点，再基于 `citizenchain-fresh` 生成临时 chainspec 并给测试钱包补余额。
- 广场本地 E2E 链节点必须至少两个节点通过 WSS peer 互连。当前节点挖矿逻辑在 `sync_service.is_offline()` 时不会出块，单节点即使有 pending extrinsic 也不会打包；第二节点 bootnode 地址必须使用 `/wss/p2p/{peer_id}`，两端 `system_health.peers > 0` 后才能验证 `publish_square_post` 入块。
- 阶段 7 已完成本地真实 E2E：钱包登录 Worker、D1 会员校验、R2 manifest 上传、本地媒体上传代理、上传完成、`SquarePost.publishSquarePost` 入块、Worker 按区块事件确认并写入 `square_posts`、推荐 feed 返回已发布动态。该验收只使用本地 Miniflare/D1/R2 和临时 chainspec，不写入 Cloudflare 密钥或链 RPC 私密地址。
- `citizenapp/cloudflare/.gitignore` 必须忽略 `.dev.vars`、`.wrangler/`、`node_modules/`、`coverage/` 和 `dist/`；Cloudflare token、R2 access key、R2 secret key 不得写入仓库。
- App 本地 Isar 只缓存草稿、上传任务、feed 快照、浏览状态和推荐信号同步状态；本地缓存不得作为发布权限、认证状态或链上发布成功的最终真相。
- 发布流程当前状态：App 先用 finalized 余额校验至少 `1.21 元`（ED 1.11 元 + 发布费 0.1 元）→ 钱包签名登录 Worker → Worker 校验会员、`post_category` 与 `content_format` 额度并在 prepare 阶段生成 `post_id/storage_receipt_id`、manifest 上传 URL 和 Images/Stream 上传 URL → App 提交链上 `publish_square_post` → runtime 按最低链上费用扣 0.1 元并校验竞选权限 → App 等待入块 → 入块后 App 上传 manifest 到 R2、上传图片/视频到 Images/Stream 并调用 complete → Worker 读取 R2 manifest 复核真实内容额度和媒体资产一致性 → App 调用 Worker 确认接口 → Worker 用链上事件、上传记录、R2 manifest 和 `square_media_assets` 交叉校验后写入正式 feed → App 刷新 Worker feed。
- 链上未入块、余额不足或后台发布流程失败时，App 用 `AppKvEntity` 保存当前钱包的广场发布草稿；草稿只用于用户继续编辑或再次发起发布，不是链上发布成功或 feed 可见的真源。
- Worker 链 RPC 只允许由远端 Secret `CHAIN_URL` 提供 Access 保护的 HTTPS 地址，并由 `CHAIN_ID`、`CHAIN_SECRET` 提供服务令牌；三项必须成套存在，缺失任一项时 relay 对 App 保持关闭。R2 S3 预签名所需变量同样只允许放在 Cloudflare 远端变量或 Secret 中；仓库、CitizenApp、R2 manifest、bootstrap 响应和 relay 响应都不得保存链 RPC 私密地址或 Cloudflare 凭证。
- 统一协议登记见 `memory/07-ai/unified-protocols.md` 的 `P-API-CITIZENAPP-002`、`P-API-CITIZENAPP-004`、`P-API-CITIZENAPP-005` 和 `P-TX-013`；阶段任务卡见 `memory/08-tasks/open/20260705-citizenapp-square-r2-worker.md` 与 `memory/08-tasks/open/20260708-citizenapp-chain-edge-architecture.md`。

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
- 清算行目录中 CID 搜索结果仍来自公开 API，但链上 `ClearingBankNodes[cid_number]` endpoint 使用 `AppKvEntity` 本地 24 小时快照；用户 `UserBank[user]` 绑定状态使用 3 分钟短快照，绑定/解绑路径必须清理或强制刷新

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
- 电子护照不再写 `myid.*` 本地档案状态；`MyIdService` 只读本机钱包列表和链上 `CitizenIdentity::VotingIdentityByAccount`。
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

- 冻结 chainspec 固定登记 44 个权威 bootnode，第 1 个是国储会节点，其余节点后续逐步部署。
- App 安装包内置 chainspec 和 light sync state；Cloudflare bootstrap 可补充同一网络的 bootnodes，但不能替换本地信任锚。
- 已部署 bootnode 对公网只开放 `30333/TCP` WSS/libp2p。RPC `9944` 只监听服务器回环地址，不写入 App、不写入 bootstrap，也不作为 App 的节点列表。
- Cloudflare 不运行 CitizenChain 节点；首期 Worker 只通过独立 Tunnel 访问国储会节点的本机 RPC，不同时连接全部 44 个节点。

### 6.3 连接与降级策略

- smoldot 根据 chainspec bootnodes 发现 P2P peers，并在可用 peers 间同步；不再维护或随机轮询 44 个 HTTP RPC 地址。
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

## 7. 外部 API 对接（当前）

App 通过 `ApiClient` 访问非链上外部服务，当前已使用接口：

- `GET /api/v1/health`
- `GET /api/v1/admins/catalog`

电子护照页面字段约定：

- `identity_wallet_account` 展示为“投票账户”，来源是命中 `VotingIdentityByAccount` 的本机钱包地址。
- `identity_cid_number` 展示为“身份 CID 号”，来源是链上 `VotingIdentity.cid_number`。
- `identity_status` 由链上 `citizen_status` 和护照有效期窗口派生：正常、未生效、已过期、已吊销、未上链、异常、读取失败。
- `passport_valid_from / passport_valid_until` 展示链上护照有效期。
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

## 10. CID 连接路径

citizenapp 访问 CID 只保留两条路径，路径由 `CITIZENAPP_ONCHINA_ENV` 选择：

```bash
# 生产路径：不传该参数时默认也是 prod
flutter build apk --release \
  --dart-define=CITIZENAPP_ONCHINA_ENV=prod

# 本地开发路径：必须使用脚本建立 USB 转发
cd /Users/rhett/GMB/citizenapp
./scripts/citizenapp-run.sh
```

- `prod`：固定访问 `https://cid.crcfrcn.com`。
- `dev_usb`：固定访问 `http://127.0.0.1:8899`，由 `adb reverse tcp:8899 tcp:8899` 转发到本电脑运行的 OnChina 后端。
- `./scripts/citizenapp-run.sh` 只注入 `--dart-define=CITIZENAPP_ONCHINA_ENV=dev_usb`，并且必须成功建立 `adb reverse tcp:8899 tcp:8899`。
- 禁止保留或新增任意 CID API URL 注入、旧端口默认值、手机直连电脑局域网地址、生产失败回退开发、开发失败回退生产等第三路径。

## 11. 关联模块文档

- RPC 模块：`lib/rpc/RPC_TECHNICAL.md`
- 二维码模块：`lib/qr/QR_TECHNICAL.md`
- 签名模块：`lib/signer/SIGNER_TECHNICAL.md`
- 公民治理模块：`memory/05-modules/citizenapp/governance/GOVERNANCE_TECHNICAL.md`
- 用户模块：`lib/user/USER_TECHNICAL.md`
- 钱包模块：`lib/wallet/WALLET_TECHNICAL.md`
- 链上支付模块：`memory/05-modules/citizenapp/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`
