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
- 链上通信：smoldot PoW 轻节点 + Rust 原生 typed capability（异步 FFI，不阻塞主线程）（`lib/rpc/` + `third_party/smoldot-dart/` + `rust/`）
- P2P IM 路线：信息 Tab + 钱包账户聊天身份 + 用户自己的私人通信全节点 + 近场无网点对点通信；当前已落地基础模型、信息 Tab、绑定 payload、私人节点传输骨架和 node 端 owner-only mailbox Spike；Android 近场模块规划为 `android/im/`，iOS 近场模块规划为 `ios/im/`
- 外部接口：HTTP API（由 OnChina 提供，用于电子护照状态、管理员目录、公权机构目录等非链上查询场景）
- 行政区字典：安装包内置 `assets/admin_divisions/`，由 `citizenchain/onchina/src/cid/china/china.sqlite` 直接生成；运行中只读本地包，不向 OnChina 联网更新行政区。
- 公权机构包：安装包内置 `assets/public_institutions/`，发布期从已通过 `check-gov --strict` 的 OnChina 真实 HTTP 接口导出；2026-06-19 重新创世版本 1 当前包包含 43 省、248643 条公民端完整公权机构。citizenapp 公民端不按 OnChina 管理端“公权机构 / 市公安局 / 教育机构”等后台功能 tab 分流或排除。

### 2.1 Android 打包和正式签名

- Android 包名固定为 `org.citizenapp`，由 `citizenapp/android/app/build.gradle.kts` 的 `namespace` 与 `applicationId` 共同约束。
- Android 版本号来自 `citizenapp/pubspec.yaml` 的 `version: x.y.z+n`：
  - `x.y.z` 对应 `versionName`
  - `n` 对应 `versionCode`
  - 每次正式发布新 APK 必须递增 `versionCode`，否则 Android 端无法按更新包安装。
- `release` 构建只允许使用固定 release keystore，不允许回退 debug 签名；缺少签名配置时直接失败。
- Android APK 只面向真实手机常用 ARM ABI：`arm64-v8a` 与 `armeabi-v7a`。`smoldot` native 库必须同时产出这两份 `libsmoldot.so`，并放入 `android/app/src/main/jniLibs/<abi>/`；当前不支持 x86 / x86_64 Android 设备，所有 APK 构建命令必须显式使用 `--target-platform android-arm,android-arm64`。
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
- 所有业务读写必须通过 `lib/isar/wallet_isar.dart` 的 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()` 排队执行，禁止业务模块直接调用 `WalletIsar.instance.db()` 再读写 collection。
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

### 2.6 P2P IM 技术架构

citizenapp 的 P2P IM 技术路线已确定为“信息 Tab 统一入口 + 钱包账户聊天身份 + 私人通信全节点 + 近场无网点对点通信”架构：

- 用户入口：公民端在“多签”Tab 与“交易”Tab 之间新增“信息”Tab；两种通信来源的消息都在“信息”Tab 集中显示，用户不选择通信模式。
- 聊天账户：citizenapp 钱包账户就是用户可见聊天账户，也是聊天窗口发送公民币时的收付款账户；钱包私钥只用于设备绑定证明和链上转账签名，不作为 IM 消息加密密钥。
- 远程通信：公民连接用户自己的私人通信全节点，自己的节点直连对方私人通信全节点投递密文消息。
- 通信全节点：放在 `citizenchain/node/src/im/`，只服务自己的手机和自己的密文收件箱；不互为中继，不做公共 Relay / DHT / rendezvous，不替别人存消息。
- 当前 Spike：公民端已新增钱包绑定 payload 和私人节点传输骨架；节点端已新增端点校验、设备绑定、密文信封、内存态 owner-only mailbox、`/gmb/im/1` request-response incoming handler 和显式端点直连投递命令。双节点运行态互投、OpenMLS、Protobuf 和 Isar schema 尚未接入。
- 节点可达性：支持 IPv4、IPv6、用户自有 `dns4` / `dnsaddr` 端点；不可达时消息留在发送队列重试，不借别人通信全节点中继。
- 近场通信：不设计独立“局域网模式”，不要求同一路由器；Android 优先 Nearby Connections，必要时回退 Wi-Fi Direct / Wi-Fi Aware / BLE；iOS 使用 Multipeer Connectivity；Android 与 iOS 跨平台近场通过 BLE GATT 做控制和短消息。
- 原生目录：Android 近场模块放在 `citizenapp/android/im/`，iOS 近场模块放在 `citizenapp/ios/im/`，两个 `im` 目录只承载 IM 近场通信功能。
- Flutter 层：`citizenapp/lib/im/` 承载统一消息层、信息 Tab 数据模型、加密身份、支付提示、消息存储、发送队列和自动传输路由，通过 Platform Channels 调用 Android/iOS 原生能力。
- 设置页：只在需要绑定或查看通信全节点时增加“通信全节点”设置项，不增加“通信模式选择”。
- 成熟组件优先：端到端加密主选 OpenMLS；外层协议使用 Protobuf；传输、附件分片和近场能力优先复用成熟库或系统框架，不自研底层通信协议和加密算法。
- 公共节点边界：公共归档节点、普通节点不承担 IM 信令、中继或消息存储；聊天内容、通信端点、设备公钥、PeerId、更新时间和撤销状态都只属于 IM 通信体系。

详细方案见：`memory/05-modules/citizenapp/im/IM_TECHNICAL.md`。

## 3. 当前目录结构

```text
citizenapp/
├── android/
│   └── im/                 ← Android IM 近场原生模块（Nearby Connections 或 Wi-Fi fallback / BLE）
├── ios/
│   └── im/                 ← iOS IM 近场原生模块（Multipeer Connectivity / BLE）
├── assets/
├── lib/
│   ├── main.dart
│   ├── Isar/
│   ├── im/                 ← 信息 Tab、IM 统一消息层、加密、支付提示、存储、传输抽象
│   ├── rpc/                ← 链上 RPC 公共模块
│   ├── ui/                 ← App 级 UI、底部 Tab 入口壳与通用组件
│   ├── onchain/            ← 普通链上转账 / 纯链上支付
│   ├── trade/              ← 本地交易记录共用能力（非功能入口）
│   ├── offchain/           ← 扫码支付 / 清算行能力
│   ├── citizen/institution/← 机构管理(链访问只读核心 + ADR-028 统一机构模型;创建/关闭收归 onchina)
│   ├── transaction/multisig-transfer/ ← 多签转账(公私个共用,交易域)
│   ├── transaction/personal-manage/   ← 个人多签管理(自助创建/关闭)
│   ├── admins-change/      ← 管理员更换一级业务模块
│   ├── 8964/               ← 底部广场 Tab：未来广场功能入口
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

- `公民`
- `广场`
- `信息`
- `交易`
- `我的`

### 4.2 公民 Tab

`lib/citizen/` 是底部“公民”Tab 的入口与公民展示域：

- `citizen/citizen_tab_page.dart`：公民 Tab 二级导航（提案 / 立法 / 选举 / 治理 / 公权）
- `citizen/all/`：公民 Tab 内“提案”页，展示治理/机构提案动态
- `8964/`：底部“广场”Tab 当前入口目录，后续广场功能统一放入该目录
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
lib/votingengine/citizen-vote/   ← 公民投票客户端预留目录
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
  - 国储会 1
  - 省储会 43
  - 省储行 43
- 省储会、省储行分类列表固定为一行两列展示，避免因不同 Android 机型逻辑宽度差异出现单列大卡或列数漂移；国储会单张卡片横跨整行显示到右侧边缘，但高度与省储会/省储行卡片一致
- 国储会保持直接展示；省储会、省储行默认折叠，标题行最右侧使用线性右箭头/下箭头展开或折叠对应 43 个机构
- 省储会分组标题图标使用 `assets/icons/government-line.svg`，省储行分组标题图标使用 `assets/icons/bank.svg`；机构卡片内部不显示全称/简称左侧图标，只显示机构简称和右箭头
- 省储会、省储行卡片支持同分组内长按拖拽排序；排序只保存为本机 UI 偏好，不写链、不跨设备同步，也不再按管理员机构自动顶前
- 治理机构详情页固定数据立即显示：机构全称/简称、类型、身份 ID、制度账户、内部门槛等均来自本地静态注册表或制度常量，不等待链上 RPC
- 管理员列表、当前用户管理员身份、主账户余额和机构提案列表属于动态数据，进入详情页后分区后台读取；任何单项读取失败只能影响对应区域，不得让详情页整页转圈
- 管理员列表展示**链上实名资料**(A2 `AdminProfile`):姓名/职务/任期/来源/身份CID/账户;端侧由 `lib/citizen/shared/admin_profile.dart` 按机构码路由 Public/Private/Personal Admins 解码，由 `lib/citizen/shared/admin_profile_card.dart` 固定渲染顶部“序号/激活状态”、第 1 行“姓名:/职务:”、第 2 行“任期:/来源:”、第 3 行“身份CID:”、第 4 行“账户:”、第 5 行“余额:”。字段值为空时只留空值区域，不隐藏标签、不显示本地姓名兜底；余额通过 `ChainRpc.fetchFinalizedBalances` 批量读取 finalized `System.Account.free`，0 余额正常显示，查询失败才留空。个人多签 kind=2 仅账户无资料。机构(公权+私权)创建/关闭已收归 onchina,citizenapp 不再发起,管理员展示为**只读**(治理档可冷钱包激活)
- 更多制度账户余额只在用户展开账户区时按需读取；下拉刷新会强制刷新管理员、主余额、提案和已展开的更多账户余额
- 治理机构详情页和公民-提案的提案列表使用 `AppKvEntity` 持久化展示读库：提案摘要、全局提案 ID 索引、机构提案 ID 索引先从本机 Isar 读取；链上只负责按 TTL、下拉刷新、返回刷新和新区块节流检查增量校验
- 提案列表本地缓存只用于展示，不得作为投票资格、是否已投票、执行状态提交前校验的最终真相；这些关键判断仍必须实时读取 runtime storage
- 提案详情页使用 `ProposalDetailLocalStore` 持久化详情快照：转账提案、多签管理提案、Runtime 升级提案进入详情页时先读本机快照；`Voting` 状态按短 TTL 后台刷新链上状态/计票/投票记录，终态提案默认只展示本地快照，手动刷新才重读链
- 管理员主体使用 `AdminAccountService` 的两层缓存：30 秒内存缓存负责同页面/相邻页面去重，`AppKvEntity` 持久化快照负责 App 重启后首屏显示；管理员变更、激活、手动刷新时清对应 subject，投票提交前仍链上复核
- 管理员投票记录读取必须批量化：内部投票统一通过 `InternalVoteQueryService.fetchAdminVotesBatch()`，联合投票统一通过 `RuntimeUpgradeService.fetchJointAdminVotesBatch()`，禁止详情页按 43 个管理员逐条 `fetchStorage`
- 余额展示使用 `AccountBalanceSnapshotStore` 持久化短 TTL 快照：机构主账户、安全基金、手续费账户、个人/机构多签关闭页可先显示本地余额；转账/投票/创建/关闭提交前余额检查仍必须实时读链
- 提案列表、管理员投票、转账提案和 Runtime 升级主要路径已接入；公民投票提交仍待后续补齐

### 4.3 广场 Tab

- 底部第 2 个按钮文案为“广场”，进入 `lib/8964/square_tab_page.dart`。
- `lib/8964/` 是未来广场功能的唯一代码目录；公民 Tab 内旧“广场”子 tab 已改为“提案”，代码迁移到 `lib/citizen/all/`。
- 当前广场页只保留稳定入口壳，不承载个人多签、机构账户或提案列表逻辑。

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
- 机构(公权/私权)注册、创建、关闭已收归 OnChina 注册局控制台 + 冷钱包；CitizenApp 交易页不提供机构多签注册或展示入口。
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
- 电子护照档案状态：`myid.*`（含档案状态、钱包地址、身份 CID、护照号、护照有效期、投票状态）
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
| 38 | 伊犁省 | `prcyls.crcfrcn.com` |
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

- `archive_status` 只表示本机电子护照档案状态：`unset / pending / registered`。
- `wallet_address` 表示电子护照使用的钱包 SS58 地址；`wallet_pubkey` 只在系统内部用于验签和查询，不在普通 UI 展示。
- `passport_no` 在 citizenapp 展示为“护照号”；缺失时显示“未生成”。
- `cid_number` 在 citizenapp 展示为“身份 CID”；缺失时显示“未生成”。
- `passport_valid_from / passport_valid_until` 展示当前电子护照有效期。
- `identity_status` 只表示身份 CID 状态；`NORMAL` 显示“状态：正常”，其他值显示“状态：异常”。
- “投票账户地址”展示当前钱包 SS58 地址。
- 选择钱包但后端尚未查到公民档案时，电子护照页底部操作固定为左侧“更换钱包”、右侧“扫码签名”。
- 扫码签名必须使用 `MyIdSignPage` 扫描链上中国平台的鉴权签名码，并生成 `sign_response`。
- 注册局完成公民档案登记后，citizenapp 通过状态接口同步结果；citizenapp 不创建公民档案记录。

### 7.1 区块链能力矩阵（转账 / 提案 / 投票）

| 能力 | 链上入口 | 手机端模块 | 签名域 | 当前状态 |
| --- | --- | --- | --- | --- |
| 转账 | `Balances::transfer_keep_alive` extrinsic（直连 RPC 节点） | `lib/onchain` | `onchain_tx` | 已实现（本机签名主链路） |
| 提案 | 业务治理 pallet `propose_*` | `lib/proposal` + 各业务模块目录 | `onchain_tx`（交易签名）+ CID 快照签名字段 | Runtime 升级已接入，多签转账在 `lib/multisig-transfer` |
| 投票 | 业务治理内部投票 / 投票引擎 `joint_vote` / `citizen_vote` | `lib/proposal` + 各业务模块目录 | `onchain_tx`（交易签名）+ CID 投票凭证签名字段 | 管理员内部/联合投票已接入，公民投票待补齐 |

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
- `MyIdService` 电子护照档案状态仍在 `SharedPreferences`（`myid.*`）
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
