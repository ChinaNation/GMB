# WUMINAPP 技术总文档（当前实现态）

## 1. 项目定位

`wuminapp` 当前为单仓 Flutter 客户端项目（iOS/Android），不再内置独立后端目录。

边界说明：

- 区块链 Runtime/共识逻辑不在本仓库实现（由 `citizenchain` 提供）
- SFID 与链交互由外部服务系统承载
- `wuminapp` 负责端上钱包、登录签名、纯链上支付入口、绑定指令发起、状态展示

## 2. 当前技术栈

- App：Flutter + Dart
- 手机机密存储：`flutter_secure_storage`（Keychain/Keystore）
- 手机业务存储：Isar
- 链上通信：smoldot PoW 轻节点 + Rust 原生 typed capability（异步 FFI，不阻塞主线程）（`lib/rpc/` + `third_party/smoldot-dart/` + `rust/`）
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
- 设置页“关于”区域显示真实本机版本；有更新时在版本号前显示“更新”按钮。
- 用户点击“更新”后，App 下载 `公民.apk` 到本机 cache，校验 SHA-256 后才拉起 Android 系统安装器。
- Android 系统负责最终覆盖安装校验；包名或 release keystore 不一致时，系统拒绝安装。

代码边界：

- `lib/update/`：更新清单解析、GitHub Release 检查、APK 下载、SHA-256 校验和更新状态管理。
- `lib/main.dart`：主界面启动后触发异步检查。
- `lib/my/user/user.dart`：设置页“关于”区域展示当前版本和更新按钮。
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

- wuminapp 的业务数据统一落在本机 Isar/MDBX 中，低端 Android 机可能同时触发余额刷新、pending 交易对账、多签账户发现、钱包创建/导入等写入路径。
- 所有业务读写必须通过 `lib/isar/wallet_isar.dart` 的 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()` 排队执行，禁止业务模块直接调用 `WalletIsar.instance.db()` 再读写 collection。
- 统一队列对 `MdbxError (11): Try again`、`active transaction` 等短暂 busy 错误做小间隔重试；业务页面不再自行实现 MDBX 重试。
- `isar.writeTxn()` 只允许保留在数据库打开和启动迁移阶段，因为此时统一写队列本身还依赖数据库实例初始化。
- 钱包 settings 行缺失时，普通读取路径可通过统一写队列创建；已经在写事务内部的路径必须调用 `*_InTxn` 方法，禁止嵌套 `writeTxn`。
- 后台 pending 交易对账只在应用锁通过后启动；若前台钱包/治理页面正在读写本地库，本轮后台对账直接让路到下一次周期。
- `交易` Tab 是默认首屏，页面自身的启动对账也必须延后执行，并在本地库 busy 时静默跳过本轮；不得在启动阶段输出 `对账触发失败` 或未处理 Isar 异常。
- UI 错误提示必须区分“本地钱包数据库繁忙”和“轻节点/链上读取失败”，不能把 Isar/MDBX 错误包装成区块链网络错误。

## 3. 当前目录结构

```text
wuminapp/
├── android/
├── ios/
├── assets/
├── lib/
│   ├── main.dart
│   ├── Isar/
│   ├── rpc/                ← 链上 RPC 公共模块
│   ├── ui/                 ← App 级 UI、底部 Tab 入口壳与通用组件
│   ├── onchain/            ← 普通链上转账 / 纯链上支付
│   ├── trade/              ← 本地交易记录与 pending 对账共用能力（非功能入口）
│   ├── offchain/           ← 扫码支付 / 清算行能力
│   ├── organization-manage/← 机构多签管理
│   ├── personal-manage/    ← 个人多签管理
│   ├── admins-change/      ← 管理员更换一级业务模块
│   ├── citizen/            ← 公民 Tab：投票 / 治理 / 机构 / 提案
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

- `citizen/citizen_tab_page.dart`：公民 Tab 二级导航（投票 / 治理 / 公共）
- `citizen/vote/`：投票页，当前保留原公民宪法引言占位，后续扩展公民投票聚合能力
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
- 机构分类列表固定为一行两列展示，避免因不同 Android 机型逻辑宽度差异出现单列大卡或列数漂移
- 提案列表、管理员投票、转账提案和 Runtime 升级主要路径已接入；公民投票提交仍待后续补齐

### 4.3 多签 Tab

- 底部第 2 个按钮文案为“多签”，直接进入 `lib/governance/duoqian_account_list_page.dart`
- 多签列表在用户第一次点击 `多签` Tab 时构建，避免应用启动时提前触发本地多签账户发现
- 多签 Tab 顶部标题为“多签”，右上角 `+` 统一提供：
  - 新增个人多签 → `lib/governance/personal-manage/personal_duoqian_create_page.dart`
  - 新增机构多签 → `lib/governance/organization-manage/institution_duoqian_create_page.dart`
- 页面主体为个人 + 机构多签统一账户列表，按本机发现/缓存时间倒序展示
- 新增个人/机构多签前，App 先按 runtime 口径校验发起钱包 free 余额覆盖
  `初始资金 + max(初始资金 * 0.1%, 0.10 元) + 1.11 元 ED`；余额不足时
  不进入签名和提交流程
- 多签创建类交易不能把 txHash 当成功；必须等待入块，并在同一区块确认
  `PersonalDuoqianProposed` 或 `InstitutionCreateProposed` 事件后，才允许写本地
  多签/提案记录
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
- 交易页在链上支付表单上方保留/插入独立入口：
  - 扫码支付 → `lib/transaction/offchain-transaction/services/offchain_scan_flow.dart`
- `lib/transaction/onchain-transaction/` 只处理普通链上转账 / 纯链上支付
- 扫码支付、多签、普通链上支付均为独立功能域；多签入口不再通过交易页分流

### 4.5 钱包与签名

钱包能力收口在 `lib/wallet/`：

- `core/`：钱包生命周期、Isar、机密 key 规范、生物识别守卫
- `capabilities/`：登录签名编排、API（SFID 绑定/管理员目录）、证明态
- `pages/`：钱包页面
- `widgets/`：钱包专用组件

余额查询由 `lib/rpc/` 模块直连链上节点完成，不经过外部网关。

签名能力收口在 `lib/signer/`：

- `local_signer.dart`：手机本机签名（助记词在手机）
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
- `TxRecordEntity`
- `AdminRoleCacheEntity`
- `ObservedAccountEntity`
- `LoginReplayEntity`
- `AppKvEntity`

当前 schema 版本：`wallet.data.schema.version = 5`。

### 5.3 偏好层（SharedPreferences）

仍有少量非机密配置使用（按模块逐步收口）：

- 登录防重放记录：`login.used_challenges`
- SFID 绑定状态：`sfid.bind.*`
- 用户资料：
  - `user.profile.nickname`
  - `user.profile.avatar_path`

## 6. 链上通信架构

### 6.1 通信模式

App 直连区块链引导节点的 RPC 端口，不经过中间网关服务：

```text
手机 App  --JSON-RPC-->  引导节点 :9944（44 个节点，自动选择）
```

每个引导节点同时承担两个角色：

- P2P 端口（30333）：服务于全节点网络同步
- RPC 端口（9944）：服务于手机 App 查询与交易

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
| 转账 | 直连节点构造/提交 extrinsic | `onchain` | 已实现 |
| 提案 | 直连节点提交治理 extrinsic | `citizen/proposal` | 已部分实现 |
| 投票 | 直连节点提交投票 extrinsic | `citizen/proposal` | 管理员投票已实现，公民投票待补齐 |

## 7. 外部 API 对接（当前）

App 通过 `ApiClient` 访问非链上外部服务，当前已使用接口：

- `GET /api/v1/health`
- `POST /api/v1/chain/bind/request`
- `GET /api/v1/admins/catalog`

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

## 10. 本地开发

```bash
cd /Users/rhett/GMB/wuminapp
flutter pub get
flutter run \
  --dart-define=WUMINAPP_RPC_URL=http://127.0.0.1:9944 \
  --dart-define=WUMINAPP_API_BASE_URL=http://<sfid服务地址>:8899
```

- `WUMINAPP_RPC_URL`：覆盖默认 RPC 节点地址，本地开发时指向本机节点。不设置时 App 自动从 44 个引导节点中选择。
- `WUMINAPP_API_BASE_URL`：指向 `sfid` 的 HTTP API 基地址，例如 `http://147.224.14.117:8899`。
- 真机调试时 `WUMINAPP_RPC_URL` / `WUMINAPP_API_BASE_URL` 都必须使用手机可达地址，不可用 `127.0.0.1`。
- 手机访问 RPC 不一定需要公网互联网；如果使用局域网地址（如 `10.x.x.x`），手机只需要和节点处于同一可达网络（同一 Wi-Fi / 热点 / VPN / USB 网络共享）即可。如果使用公网域名节点，则需要普通互联网连接。

## 11. 关联模块文档

- RPC 模块：`lib/rpc/RPC_TECHNICAL.md`
- 二维码模块：`lib/qr/QR_TECHNICAL.md`
- 签名模块：`lib/signer/SIGNER_TECHNICAL.md`
- 公民治理模块：`memory/05-modules/wuminapp/governance/GOVERNANCE_TECHNICAL.md`
- 用户模块：`lib/user/USER_TECHNICAL.md`
- 钱包模块：`lib/wallet/WALLET_TECHNICAL.md`
- 链上支付模块：`memory/05-modules/wuminapp/onchain/ONCHAIN_TECHNICAL.md`
