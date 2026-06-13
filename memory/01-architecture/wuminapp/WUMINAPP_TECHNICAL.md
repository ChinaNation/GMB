# WUMINAPP 技术总文档（当前实现态）

## 1. 项目定位

`wuminapp` 当前为单仓 Flutter 客户端项目（iOS/Android），不再内置独立后端目录。

边界说明：

- 区块链 Runtime/共识逻辑不在本仓库实现（由 `citizenchain` 提供）
- SFID 与链交互由外部服务系统承载
- `wuminapp` 负责端上钱包、登录签名、纯链上支付入口、绑定签名与状态展示

## 2. 当前技术栈

- App：Flutter + Dart
- 手机机密存储：`flutter_secure_storage`（Keychain/Keystore）
- 手机业务存储：Isar
- 链上通信：smoldot PoW 轻节点 + Rust 原生 typed capability（异步 FFI，不阻塞主线程）（`lib/rpc/` + `third_party/smoldot-dart/` + `rust/`）
- P2P IM 预定路线：统一消息层 + 用户自己的通信全节点 + 近场无网点对点通信；Android 近场模块规划为 `android/im/`，iOS 近场模块规划为 `ios/im/`
- 外部接口：HTTP API（由 SFID/网关系统提供，仅用于 SFID 绑定、管理员目录等非链上查询场景）

### 2.1 Android 打包和正式签名

- Android 包名固定为 `org.chinanation.citizen`，由 `wuminapp/android/app/build.gradle.kts` 的 `namespace` 与 `applicationId` 共同约束。
- Android 版本号来自 `wuminapp/pubspec.yaml` 的 `version: x.y.z+n`：
  - `x.y.z` 对应 `versionName`
  - `n` 对应 `versionCode`
  - 每次正式发布新 APK 必须递增 `versionCode`，否则 Android 端无法按更新包安装。
- `release` 构建只允许使用固定 release keystore，不允许回退 debug 签名；缺少签名配置时直接失败。
- Android APK 只面向真实手机常用 ARM ABI：`arm64-v8a` 与 `armeabi-v7a`。`smoldot` native 库必须同时产出这两份 `libsmoldot.so`，并放入 `android/app/src/main/jniLibs/<abi>/`；当前不支持 x86 / x86_64 Android 设备，所有 APK 构建命令必须显式使用 `--target-platform android-arm,android-arm64`。
- 本地正式签名配置只允许写在被 `.gitignore` 忽略的 `wuminapp/android/key.properties`：

```properties
storeFile=app/wuminapp-release.jks
storePassword=...
keyAlias=...
keyPassword=...
```

- GitHub Actions 的 `wuminapp-ci.yml` 分两种模式：
  - push / PR：只构建 Debug APK 做工程检查，不读取正式签名密钥。
  - 手动 `Run workflow`：通过 GitHub Secrets 注入 release keystore，构建正式签名 `公民.apk` artifact。
- 手动发布需要配置的 GitHub Secrets：
  - `WUMINAPP_RELEASE_KEYSTORE_BASE64`：release keystore 文件的 base64 内容
  - `WUMINAPP_ANDROID_STORE_PASSWORD`：keystore 密码
  - `WUMINAPP_ANDROID_KEY_ALIAS`：签名 key alias
  - `WUMINAPP_ANDROID_KEY_PASSWORD`：签名 key 密码
  - `WUMINAPP_RELEASE_KEYSTORE_SHA256`：可选，用于校验 keystore 文件 SHA-256
- Android 更新成立的前提是：包名不变、release keystore 不变、`versionCode` 递增。

### 2.2 Android 应用更新

Android 更新只认手动发布的 GitHub Release，不认 push / PR 检查产物。

手动 `Run workflow` 发布时，`wuminapp-ci.yml` 会在正式签名 `公民.apk` 后生成并上传：

- `公民.apk`
- `wuminapp-android-update.json`

更新清单协议：

```json
{
  "app": "wuminapp",
  "platform": "android",
  "package_name": "org.chinanation.citizen",
  "version_name": "0.1.1",
  "version_code": 2,
  "apk_asset": "公民.apk",
  "apk_sha256": "...",
  "published_at": "2026-05-17T00:00:00Z",
  "notes": "wuminapp Android 0.1.1+2"
}
```

关键规则：

- 移动端 Release 不标记为 GitHub `Latest`，避免影响桌面端 Tauri updater 的 `releases/latest/download/citizenchain-latest.json`。
- App 启动进入主界面后异步检查 GitHub Release 列表，寻找包含 `wuminapp-android-update.json` 的 Release。
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
- `android/app/src/main/kotlin/org/chinanation/citizen/MainActivity.kt`：提供 `org.chinanation.citizen/update` MethodChannel，读取包版本并拉起系统安装器。
- `android/app/src/main/res/xml/update_file_paths.xml`：FileProvider 只暴露 App cache 中的更新 APK。

### 2.3 首次启动权限策略

wuminapp 不采用“首启强制弹出全部权限”的模式，按平台权限模型分层处理：

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

- wuminapp 的业务数据统一落在本机 Isar/MDBX 中，低端 Android 机可能同时触发余额刷新、钱包交易流水同步、多签账户发现、钱包创建/导入等写入路径。
- 所有业务读写必须通过 `lib/isar/wallet_isar.dart` 的 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()` 排队执行，禁止业务模块直接调用 `WalletIsar.instance.db()` 再读写 collection。
- 统一队列对 `MdbxError (11): Try again`、`active transaction` 等短暂 busy 错误做小间隔重试；业务页面不再自行实现 MDBX 重试。
- `isar.writeTxn()` 只允许保留在数据库打开和启动迁移阶段，因为此时统一写队列本身还依赖数据库实例初始化。
- 钱包 settings 行缺失时，普通读取路径可通过统一写队列创建；已经在写事务内部的路径必须调用 `*_InTxn` 方法，禁止嵌套 `writeTxn`。
- 钱包交易流水同步属于低优先级后台任务；监听新区块和 finalized 区块事件时必须复用 `WalletIsar` 队列，遇到本地库繁忙直接让路，不能和钱包页、治理页抢 MDBX 写锁。
- `交易` Tab 是默认首屏，页面自身只允许延迟刷新本地交易流水；不得在启动阶段发起 nonce 轮询确认 RPC、输出旧确认错误或未处理 Isar 异常。
- UI 错误提示必须区分“本地钱包数据库繁忙”和“轻节点/链上读取失败”，不能把 Isar/MDBX 错误包装成区块链网络错误。

### 2.5 投票状态真源

- wuminapp 投票页面不得把 `author_submitExtrinsic` 返回的 txHash、交易池 watch 状态或本地 nonce 推进当成投票成功。
- 内部投票成功真源是 runtime `InternalVote::InternalVotesByAccount(proposal_id, admin)`。
- 联合投票成功真源是 runtime `JointVote::JointVotesByAdmin(proposal_id, institution, admin)`。
- 本地 `PendingVoteStore` 只表示“已提交但还没有从 runtime 投票 storage 读到结果”，不能覆盖链上状态。
- 交易池 watch 的 `timeout / finalityTimeout / retracted / future / error` 不直接清除 pending；只有链上已记录投票，或 nonce 已消耗但链上仍无投票记录，才更新 pending 状态。
- pending 投票确认窗口为 20 分钟；超过窗口仍无 runtime 投票记录且 nonce 未推进，视为本地提交未进入链，清除 pending 并允许用户重新投票，避免“投票中”无限转圈。
- 投票按钮的 `submitting` 只覆盖签名和提交阶段；拿到 txHash 后必须立即结束按钮转圈，链上确认由后台刷新和 pending 状态机处理。

### 2.6 P2P IM 技术架构

wuminapp 的 P2P IM 技术路线已确定为“信息 Tab 统一入口 + 远程通信全节点 + 近场无网点对点通信”架构：

- 用户入口：公民端在“多签”Tab 与“交易”Tab 之间新增“信息”Tab；两种通信来源的消息都在“信息”Tab 集中显示，用户不选择通信模式。
- 远程通信：公民连接用户自己的通信全节点，通信全节点只保存端到端加密后的 IM 消息。
- 通信全节点：后续放在 `citizenchain/node/src/im/`，只复用桌面节点软件已经集成的 libp2p 网络能力，承接全天候在线收件箱、密文投递和设备绑定；IM 不依赖钱包、治理、交易或身份实名模块。
- 近场通信：不设计独立“局域网模式”，不要求同一路由器；Android 优先复用成熟 Nearby Connections，必要时回退 BLE + Wi-Fi Direct；iOS 使用 Multipeer Connectivity。
- 原生目录：Android 近场模块放在 `wuminapp/android/im/`，iOS 近场模块放在 `wuminapp/ios/im/`，两个 `im` 目录只承载 IM 近场通信功能。
- Flutter 层：`wuminapp/lib/im/` 承载统一消息层、信息 Tab 数据模型、加密身份、消息存储、发送队列和自动传输路由，通过 Platform Channels 调用 Android/iOS 原生能力。
- 设置页：只在需要绑定或查看通信全节点时增加“通信全节点”设置项，不增加“通信模式选择”。
- 成熟组件优先：协议、加密、传输、附件分片和近场能力优先复用成熟库或系统框架，不自研底层通信协议和加密算法。
- 公共节点边界：公共归档节点、普通节点不承担 IM 信令、中继或消息存储；聊天内容、通信端点、设备公钥、PeerId、更新时间和撤销状态都只属于 IM 通信体系。

详细方案见：`memory/05-modules/wuminapp/im/IM_TECHNICAL.md`。

## 3. 当前目录结构

```text
wuminapp/
├── android/
│   └── im/                 ← 预定：Android IM 近场原生模块（Nearby Connections 或 Wi-Fi Direct）
├── ios/
│   └── im/                 ← 预定：iOS IM 近场原生模块（Multipeer Connectivity）
├── assets/
├── lib/
│   ├── main.dart
│   ├── Isar/
│   ├── im/                 ← 预定：信息 Tab、IM 统一消息层、加密、存储、传输抽象
│   ├── rpc/                ← 链上 RPC 公共模块
│   ├── ui/                 ← App 级 UI、底部 Tab 入口壳与通用组件
│   ├── onchain/            ← 普通链上转账 / 纯链上支付
│   ├── trade/              ← 本地交易记录共用能力（非功能入口）
│   ├── offchain/           ← 扫码支付 / 清算行能力
│   ├── organization-manage/← 机构多签管理
│   ├── personal-manage/    ← 个人多签管理
│   ├── admins-change/      ← 管理员更换一级业务模块
│   ├── citizen/            ← 公民 Tab：公权 / 广场 / 治理
│   ├── qr/                 ← 二维码统一模块（登录/收款/用户码）
│   ├── signer/
│   ├── user/
│   └── wallet/
└── test/
```

说明：

- 原 `mobile/` 内容已上移到项目根目录
- 原 `backend/` 已移除
- 正式产品文档统一收口到 `memory/01-architecture/wuminapp/`

## 4. App 当前实现

### 4.1 主导航

底部 4 Tab：

- `公民`
- `多签`
- `交易`
- `我的`

### 4.2 公民 Tab

`lib/citizen/` 是底部“公民”Tab 的入口与公民展示域：

- `citizen/citizen_tab_page.dart`：公民 Tab 二级导航（公权 / 广场 / 治理）
- `citizen/vote/`：广场页，展示全局治理提案列表；不再保留公民宪法引言背景水印
- `citizen/public/`：公共页占位
- `governance/`：治理域，与链端 `runtime/governance/` 对齐，包含 4 个 pallet 子模块（`admins-change/` / `organization-manage/` / `personal-manage/` / `runtime-upgrade/`）+ 治理列表入口 `governance_list_page.dart` + 治理提案聚合页 `governance_proposals_page.dart` + 跨子模块多签管理详情 `duoqian_manage_detail_page.dart`

跨 pallet 共用能力分布：

```text
lib/governance/shared/proposal/  ← 提案通用模型/上下文/查询/限额/缓存
lib/governance/shared/duoqian_create_amount_rules.dart ← 多签创建金额预校验规则
lib/votingengine/internal-vote/  ← 内部投票客户端(投票服务/查询/待确认存储/投票UI)
lib/votingengine/joint-vote/     ← 联合投票客户端预留目录
lib/votingengine/citizen-vote/   ← 公民投票客户端预留目录
lib/transaction/duoqian-transfer/ ← 多签转账业务(创建/详情/投票/列表适配/缓存统一收口),不进 governance/ 子模块
```

管理员更换、机构多签注册/关闭等业务都归 `lib/governance/<对应子模块>/`,治理提案聚合页只保留入口跳转或调用。

### 4.2.1 机构页

- 机构分类卡片已内置：
  - 国储会 1
  - 省储会 43
  - 省储行 43
- 省储会、省储行分类列表固定为一行两列展示，避免因不同 Android 机型逻辑宽度差异出现单列大卡或列数漂移；国储会单张卡片横跨整行显示到右侧边缘，但高度与省储会/省储行卡片一致
- 国储会保持直接展示；省储会、省储行默认折叠，标题行最右侧使用线性右箭头/下箭头展开或折叠对应 43 个机构
- 省储会分组标题图标使用 `assets/icons/government-line.svg`，省储行分组标题图标使用 `assets/icons/bank.svg`；机构卡片内部不显示名称左侧图标，只显示机构名称和右箭头
- 省储会、省储行卡片支持同分组内长按拖拽排序；排序只保存为本机 UI 偏好，不写链、不跨设备同步，也不再按管理员机构自动顶前
- 治理机构详情页固定数据立即显示：机构名称、类型、身份 ID、制度账户地址、内部门槛等均来自本地静态注册表或制度常量，不等待链上 RPC
- 管理员列表、当前用户管理员身份、主账户余额和机构提案列表属于动态数据，进入详情页后分区后台读取；任何单项读取失败只能影响对应区域，不得让详情页整页转圈
- 更多制度账户余额只在用户展开账户区时按需读取；下拉刷新会强制刷新管理员、主余额、提案和已展开的更多账户余额
- 治理机构详情页和公民-广场的提案列表使用 `AppKvEntity` 持久化展示读库：提案摘要、全局提案 ID 索引、机构提案 ID 索引先从本机 Isar 读取；链上只负责按 TTL、下拉刷新、返回刷新和新区块节流检查增量校验
- 提案列表本地缓存只用于展示，不得作为投票资格、是否已投票、执行状态提交前校验的最终真相；这些关键判断仍必须实时读取 runtime storage
- 提案详情页使用 `ProposalDetailLocalStore` 持久化详情快照：转账提案、多签管理提案、Runtime 升级提案进入详情页时先读本机快照；`Voting` 状态按短 TTL 后台刷新链上状态/计票/投票记录，终态提案默认只展示本地快照，手动刷新才重读链
- 管理员主体使用 `AdminAccountService` 的两层缓存：30 秒内存缓存负责同页面/相邻页面去重，`AppKvEntity` 持久化快照负责 App 重启后首屏显示；管理员变更、激活、手动刷新时清对应 subject，投票提交前仍链上复核
- 管理员投票记录读取必须批量化：内部投票统一通过 `InternalVoteQueryService.fetchAdminVotesBatch()`，联合投票统一通过 `RuntimeUpgradeService.fetchJointAdminVotesBatch()`，禁止详情页按 43 个管理员逐条 `fetchStorage`
- 余额展示使用 `AccountBalanceSnapshotStore` 持久化短 TTL 快照：机构主账户、安全基金、手续费账户、个人/机构多签关闭页可先显示本地余额；转账/投票/创建/关闭提交前余额检查仍必须实时读链
- 提案列表、管理员投票、转账提案和 Runtime 升级主要路径已接入；公民投票提交仍待后续补齐

### 4.3 多签 Tab

- 底部第 2 个按钮文案为“多签”，直接进入 `lib/governance/duoqian_account_list_page.dart`
- 多签列表在用户第一次点击 `多签` Tab 时构建，避免应用启动时提前触发本地多签账户发现
- 多签 Tab 顶部标题为“多签”，右上角 `+` 统一提供：
  - 新增个人多签 → `lib/governance/personal-manage/personal_duoqian_create_page.dart`
  - 新增机构多签 → `lib/governance/organization-manage/institution_duoqian_create_page.dart`
- 页面主体为个人 + 机构多签统一账户列表，按本机发现/缓存时间倒序展示
- 多签列表首屏只读取本机 Isar，不等待链上状态查询或全量 discovery；链上状态刷新在后台执行。
- 多签本地状态复用 `AppKvEntity`：`stringValue` 保存 `active / pending / closed`，
  `intValue` 保存最近一次成功链上状态同步时间。
- 多签详情页首屏同样只读取本机持久化数据：个人多签读取 `PersonalDuoqianEntity`
  / `PersonalDuoqianLocalState` / `personal_duoqian_detail:*`，机构多签读取
  `DuoqianInstitutionEntity` / `InstitutionDuoqianLocalState` /
  `institution_duoqian_detail:*`；不得为了显示名称、地址、状态、管理员数量、阈值或余额快照而阻塞等待链上 RPC。
- 多签详情页的状态刷新时间与余额刷新时间必须分离：`lastChainRefreshAtMillis`
  只代表账户状态/管理员/阈值新鲜，`lastBalanceRefreshAtMillis` 才代表 Active 余额新鲜；
  本地余额为空或余额时间过期时，即使状态 TTL 未过，也要只刷新余额。
- 详情页不展示“同步中”类 UI；TTL 到期、下拉刷新、转账/投票/关闭返回时才精准刷新当前账户，链上失败只保留本机已储存数据，不把页面打成加载失败。
- Active 多签账户 60 分钟内不自动重复查询链上状态；Pending / Closed 账户
  10 分钟内不自动重复查询链上状态；用户下拉刷新才忽略 TTL 强制刷新。
- 自动 discovery 只在首次进入多签 Tab 或本机钱包 pubkey 列表变化时触发；不做每日自动扫描，也不提供单独“扫描我的多签”按钮。
- 下拉刷新会强制刷新已知账户状态，并执行个人/机构多签全量 discovery。
- 创建、关闭、投票、删除返回列表时只刷新相关多签账户或本地记录，不触发全量扫描。
- 个人/机构多签链上状态刷新必须使用 `ChainRpc.fetchStorageBatchChunked()` 分阶段批量读取 storage，
  禁止列表页逐个账户循环调用详情查询。
- 新增个人/机构多签前，App 先按 runtime 口径校验发起钱包 free 余额覆盖
  `初始资金 + max(初始资金 * 0.1%, 0.10 元) + 1.11 元 ED`；余额不足时
  不进入签名和提交流程
- 多签创建类交易不能把 txHash 当成功；必须等待入块，并在同一区块确认
  `PersonalDuoqianProposed` 或 `InstitutionCreateProposed` 事件后，才允许写本地
  多签/提案记录
- 如果交易已入块但没有成功事件，App 必须先解析 `System.ExtrinsicFailed` 并显示真实
  `PersonalManage / AdminsChange / OrganizationManage` 模块错误；不得把“未找到成功事件”
  当作最终失败原因
- 原“消息”占位页、通讯录按钮和搜索框已删除；通讯录仍归 `lib/my/user/` 用户域维护

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
- 交易页在链上支付表单上方保留/插入独立入口：
  - 扫码支付 → `lib/transaction/offchain-transaction/services/offchain_scan_flow.dart`
- `lib/transaction/onchain-transaction/` 只处理普通链上转账 / 纯链上支付
- 扫码支付、多签、普通链上支付均为独立功能域；多签入口不再通过交易页分流
- 清算行目录中 SFID 搜索结果仍来自公开 API，但链上 `ClearingBankNodes[sfid_number]` endpoint 使用 `AppKvEntity` 本地 24 小时快照；用户 `UserBank[user]` 绑定状态使用 3 分钟短快照，绑定/解绑路径必须清理或强制刷新

### 4.5 钱包与签名

钱包能力收口在 `lib/wallet/`：

- `core/`：钱包生命周期、Isar、机密 key 规范、生物识别守卫
- `capabilities/`：登录签名编排、API（SFID 绑定/管理员目录）、证明态
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
- 登录码：挑战解析、系统签名验证、防重放、回执生成
- 收款码：生成与解析，预填转账表单
- 用户码：通讯录交换，兼容旧版格式

关键口径：

- 登录协议：`WUMIN_QR_V1`
- 签名协议：`WUMIN_QR_V1`
- 用户协议：`WUMIN_QR_V1`（联系人 purpose=contact / 收款 purpose=transfer）
- 登录协议与链上转账/投票签名协议完全分离；前者只用于 `sfid/cpms` 扫码登录，后者只用于链上交易 `payload` 签名
- 系统身份：通过 `sys_pubkey`/`sys_sig` 密码学验证二维码确由系统私钥签发（不再使用 `aud` 白名单）
- 登录签名串：

```text
WUMIN_QR_V1|system|challenge|expires_at
```

详细技术文档见：`lib/qr/QR_TECHNICAL.md`

### 4.7 双签名模式（技术方案）

- 模式 A：本机签名
  - 私钥/助记词仅保存在手机 secure storage
  - 交易和登录均由 `WalletManager` 调起本机 sr25519 签名
- 模式 B：扫码签名
  - 手机不保存私钥，仅保存钱包地址/公钥
  - 手机生成待签名请求二维码，外部设备签名后返回签名回执二维码
  - 协议由 `QrSigner` 统一编解码与校验（`WUMIN_QR_V1`）
  - 在线手机使用 `QrSignSessionPage`，离线设备使用 `QrOfflineSignPage`
  - 登录挑战先经过 `LoginSystemSignatureVerifier` 校验 `sys_pubkey + sys_sig`

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
- `AdminRoleCacheEntity`
- `ObservedAccountEntity`
- `LoginReplayEntity`
- `AppKvEntity`

`LocalTxEntity` 只记录本机钱包进入 wuminapp 后的余额变化流水，不补扫导入前历史；`WalletTxSyncCursorEntity` 只记录 finalized 补同步游标，newHeads 只用于把当前区块内命中的流水先标记为 `inBlock`，不得把 best/latest block 当成 finalized。

当前 schema 版本：`wallet.data.schema.version = 3`。v3 会清空旧 `LocalTxEntity` 和 `WalletTxSyncCursorEntity`，从当前本机时刻重新记录交易流水。

### 5.3 偏好层（SharedPreferences）

仍有少量非机密配置使用（按模块逐步收口）：

- 登录防重放记录：`login.used_challenges`
- 电子护照绑定状态：`sfid.bind.*`（含绑定状态、投票账户、身份ID号码、身份ID状态）
- 用户资料：
  - `user.profile.nickname`
  - `user.profile.avatar_path`

## 6. 链上通信架构

### 6.1 通信模式

App 直连区块链引导节点的 RPC 端口，不经过中间网关服务：

```text
公民  --JSON-RPC-->  引导节点 :9944（44 个节点，自动选择）
```

每个引导节点同时承担两个角色：

- P2P 端口（30333）：服务于全节点网络同步
- RPC 端口（9944）：服务于公民 查询与交易

### 6.2 RPC 节点列表

App 内置 44 个引导节点的 RPC 地址，域名与 `citizenchain/node/src/chain_spec.rs` 中的 P2P 引导节点一致：

| 序号 | 机构 | 域名 |
| --- | --- | --- |
| 1 | 国储会 | `nrcgch.crcfrcn.com` |
| 2 | 中枢省 | `prczss.crcfrcn.com` |
| 3 | 岭南省 | `prclns.crcfrcn.com` |
| 4 | 广东省 | `prcgds.crcfrcn.com` |
| 5 | 广西省 | `prcgxs.crcfrcn.com` |
| 6 | 福建省 | `prcfjs.crcfrcn.com` |
| 7 | 海南省 | `prchns.crcfrcn.com` |
| 8 | 云南省 | `prcyns.crcfrcn.com` |
| 9 | 贵州省 | `prcgzs.crcfrcn.com` |
| 10 | 湖南省 | `prchus.crcfrcn.com` |
| 11 | 江西省 | `prcjxs.crcfrcn.com` |
| 12 | 浙江省 | `prczjs.crcfrcn.com` |
| 13 | 江苏省 | `prcjss.crcfrcn.com` |
| 14 | 山东省 | `prcsds.crcfrcn.com` |
| 15 | 山西省 | `prcsxs.crcfrcn.com` |
| 16 | 河南省 | `prches.crcfrcn.com` |
| 17 | 河北省 | `prchbs.crcfrcn.com` |
| 18 | 湖北省 | `prchis.crcfrcn.com` |
| 19 | 陕西省 | `prcsis.crcfrcn.com` |
| 20 | 重庆省 | `prccqs.crcfrcn.com` |
| 21 | 四川省 | `prcscs.crcfrcn.com` |
| 22 | 甘肃省 | `prcgss.crcfrcn.com` |
| 23 | 北平省 | `prcbps.crcfrcn.com` |
| 24 | 海滨省 | `prchas.crcfrcn.com` |
| 25 | 松江省 | `prcsjs.crcfrcn.com` |
| 26 | 龙江省 | `prcljs.crcfrcn.com` |
| 27 | 吉林省 | `prcjls.crcfrcn.com` |
| 28 | 辽宁省 | `prclis.crcfrcn.com` |
| 29 | 宁夏省 | `prcnxs.crcfrcn.com` |
| 30 | 青海省 | `prcqhs.crcfrcn.com` |
| 31 | 安徽省 | `prcahs.crcfrcn.com` |
| 32 | 台湾省 | `prctws.crcfrcn.com` |
| 33 | 西藏省 | `prcxzs.crcfrcn.com` |
| 34 | 新疆省 | `prcxjs.crcfrcn.com` |
| 35 | 西康省 | `prcxks.crcfrcn.com` |
| 36 | 阿里省 | `prcals.crcfrcn.com` |
| 37 | 葱岭省 | `prccls.crcfrcn.com` |
| 38 | 天山省 | `prctss.crcfrcn.com` |
| 39 | 河西省 | `prchxs.crcfrcn.com` |
| 40 | 昆仑省 | `prckls.crcfrcn.com` |
| 41 | 河套省 | `prchts.crcfrcn.com` |
| 42 | 热河省 | `prcrhs.crcfrcn.com` |
| 43 | 兴安省 | `prcxas.crcfrcn.com` |
| 44 | 合江省 | `prchjs.crcfrcn.com` |

RPC 地址格式：`http://<域名>:9944`

### 6.3 节点选择策略

- 启动时随机打乱节点列表
- 依次尝试连接，使用第一个可达的节点
- 当前节点连接失败时自动切换到下一个
- 44 个节点全部不可达时抛出异常

### 6.4 RPC 公共模块（`lib/rpc/`）

`lib/rpc/` 是链上通信的唯一收口模块，所有业务模块共享：

```text
lib/rpc/
├── chain_rpc.dart   ← 节点列表、连接管理、底层 JSON-RPC 方法
├── onchain.dart     ← onchain 模块 RPC 功能（extrinsic 构造、转账、状态查询）
└── rpc.dart         ← barrel export
     ↑          ↑          ↑
  wallet/    onchain/   citizen/
 （余额查询）（转账）  （提案/投票）
```

详细技术文档见：`lib/rpc/RPC_TECHNICAL.md`

### 6.5 链上能力

| 能力 | RPC 方法 | 模块 | 状态 |
| --- | --- | --- | --- |
| 余额查询 | `state_getStorage`（`System.Account`） | `wallet` | 已实现 |
| 钱包交易流水 | newHeads/finalizedHeads + `System.Events` | `wallet` + `transaction/shared` | 已实现：本机开始跟踪后先 `inBlock`，finalized 后 `已确认` |
| 转账 | 直连节点构造/提交 extrinsic | `onchain` | 已实现 |
| 提案 | 直连节点提交治理 extrinsic | `citizen/proposal` | 已部分实现 |
| 投票 | 直连节点提交投票 extrinsic | `citizen/proposal` | 管理员投票已实现，公民投票待补齐 |

## 7. 外部 API 对接（当前）

App 通过 `ApiClient` 访问非链上外部服务，当前已使用接口：

- `GET /api/v1/health`
- `GET /api/v1/admins/catalog`
- `GET /api/v1/app/myid/status?wallet_address=<walletAddress>`

电子护照页面字段约定：

- `bind_status` 只表示绑定状态：`unset / pending / bound`。
- `wallet_address / wallet_pubkey` 表示电子护照使用的钱包。
- `sfid_number` 在 wuminapp 展示为“身份ID”；缺失时显示“未绑定”。
- `identity_status` 只表示身份ID状态；`NORMAL` 显示“状态：正常”，其他值显示“状态：异常”。
- “投票账户”展示当前绑定/待绑定的钱包地址，不再使用“绑定账户”文案。
- 选择钱包但未完成绑定时，电子护照页底部操作固定为左侧“更换钱包”、右侧“扫码签名”。
- 扫码签名必须使用 `MyIdSignPage` 扫描身份ID系统的鉴权签名码，并生成 `sign_response`。
- SFID 完成绑定后，wuminapp 通过状态接口同步结果；wuminapp 不创建电子护照绑定记录。

### 7.1 区块链能力矩阵（转账 / 提案 / 投票）

| 能力 | 链上入口 | 手机端模块 | 签名域 | 当前状态 |
| --- | --- | --- | --- | --- |
| 转账 | `Balances::transfer_keep_alive` extrinsic（直连 RPC 节点） | `lib/onchain` | `onchain_tx` | 已实现（本机签名主链路） |
| 提案 | 业务治理 pallet `propose_*` | `lib/proposal` + 各业务模块目录 | `onchain_tx`（交易签名）+ SFID 快照签名字段 | Runtime 升级已接入，多签转账在 `lib/duoqian-transfer` |
| 投票 | 业务治理内部投票 / 投票引擎 `joint_vote` / `citizen_vote` | `lib/proposal` + 各业务模块目录 | `onchain_tx`（交易签名）+ SFID 投票凭证签名字段 | 管理员内部/联合投票已接入，公民投票待补齐 |

### 7.2 区块链字段与格式标准（总则）

- 链上账户：runtime 内部是 `AccountId32` / `[u8; 32]`，extrinsic call data 写入原始 32 字节。
- App 内部账户标识：`mainAddress` / `duoqianAddress` / `pubkeyHex` 等字段统一使用 64 位 hex，不带 `0x`。
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

- `memory/05-modules/wuminapp/onchain/ONCHAIN_TECHNICAL.md`（普通链上转账）
- `memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`（公民治理 / 提案 / 投票）

## 8. 安全基线（当前）

- 私钥/助记词不落 Isar 与远端服务
- 助记词读取强制生物识别/设备密码验证（存储层统一守卫，不可关闭）
- 设备无生物识别也无密码时自动跳过验证
- 登录系统身份通过密码学签名验证（`sys_pubkey`/`sys_sig`）
- 绑定请求与交易状态依赖外部服务返回

## 9. 已知限制

- 登录防重放当前仍在 `SharedPreferences`，尚未切到 Isar 的 `LoginReplayEntity`
- `SfidBindingService` 状态仍在 `SharedPreferences`（`sfid.bind.*`）
- 链下交易模块仍为占位
- 扫码签名当前已完成协议层实现，业务 UI 仍以本机签名为主
- 提案/投票尚未实现直连 RPC

## 10. SFID 连接路径

wuminapp 访问 SFID 只保留两条路径，路径由 `WUMINAPP_SFID_ENV` 选择：

```bash
# 生产路径：不传该参数时默认也是 prod
flutter build apk --release \
  --dart-define=WUMINAPP_SFID_ENV=prod

# 本地开发路径：必须使用脚本建立 USB 转发
cd /Users/rhett/GMB/wuminapp
./scripts/wuminapp-run.sh
```

- `prod`：固定访问 `https://sfid.crcfrcn.com`。
- `dev_usb`：固定访问 `http://127.0.0.1:8899`，由 `adb reverse tcp:8899 tcp:8899` 转发到本电脑运行的 SFID 后端。
- `./scripts/wuminapp-run.sh` 只注入 `--dart-define=WUMINAPP_SFID_ENV=dev_usb`，并且必须成功建立 `adb reverse tcp:8899 tcp:8899`。
- 禁止保留或新增任意 SFID API URL 注入、旧端口默认值、手机直连电脑局域网地址、生产失败回退开发、开发失败回退生产等第三路径。

## 11. 关联模块文档

- RPC 模块：`lib/rpc/RPC_TECHNICAL.md`
- 二维码模块：`lib/qr/QR_TECHNICAL.md`
- 签名模块：`lib/signer/SIGNER_TECHNICAL.md`
- 公民治理模块：`memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`
- 用户模块：`lib/user/USER_TECHNICAL.md`
- 钱包模块：`lib/wallet/WALLET_TECHNICAL.md`
- 链上支付模块：`memory/05-modules/wuminapp/onchain/ONCHAIN_TECHNICAL.md`
