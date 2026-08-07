# Wallet 模块技术文档（当前实现态）

## 1. 模块目标

`lib/wallet` 是钱包能力唯一收口模块，负责：

- 钱包创建/导入/删除/切换（热钱包 + 冷钱包）
- 本地账户 child mini-secret 的硬件金库读写；不持久化母种子或助记词
- 登录签名编排（签名执行由 `lib/signer` 负责）
- 转账/提案/投票所需钱包上下文输出（地址、公钥、算法、机构角色）
- finalized 余额查询（通过 `lib/rpc/` 直连链上节点）
- 管理员目录、观察账户、证明态等钱包周边能力

约束：钱包相关代码只应从 `wallet/...` 引用。

本地编译安装约定：`citizenapp/scripts/citizenapp-run.sh` 与 `citizenwallet/scripts/citizenwallet-run.sh`
都收一个必填的目标平台参数（`ios` 或 `android`），编译后把**可直接使用**的 App 安装到该平台设备上，
**不产出任何留存产物**——`<product>/target/` 沉淀已整体删除（曾各积压 181MB / 98MB）。
编译产物只在 GitHub：CI 出未签名产物，Release 出正式签名产物。

「编译」的语义是**把能用的软件装进设备**，与 CI / 正式 Release 是三条互不相干的通路，因此
两脚本一律 `flutter build` + 安装，**不再用 `flutter run`**（那只是把 App 挂在调试器上跑）：

- **iOS 必须 release**，安装走 `flutter install --release -d <flutter 设备 id>`。iOS 14+ 禁止
  Flutter debug 版脱离 flutter tooling / Xcode 启动，debug 版装进手机后从桌面点图标必然起不来
  （系统提示 `Cannot create a FlutterEngine instance in debug mode`，随后 signal 11）——表现
  就是「一点就闪退」。**不能用 `xcrun devicectl` 安装**：flutter 报的 iOS 设备 id 是硬件 UDID
  （`00008140-…`），devicectl 用的是另一套内部标识（`9B0DA677-…`），两者不通用。
- **Android 用 debug**：安卓 debug 版从桌面就能直接使用，且保留 `AppLog` 落盘诊断
  （release 下 `kReleaseMode` 使其为空操作），排障成本低得多；安装走
  `adb -s <id> install -r`（flutter 的安卓设备 id 即 adb serial）。

两端构建模式不同是 iOS 系统能力的客观差异，按「iOS/Android 两端必须一致」铁律在此显式登记：
**两端交付物都是可直接使用的 App**，这一条完全一致。要把 Android 也统一成 release，改 android
分支的 `--debug` 为 `--release` 并换 APK 路径即可（代价：丢失落盘诊断日志）。

目标平台不做自动探测：探测总要在失败时选一个回落，而回落的那一端会被当成用户想编的那一端
（旧实现的 `except: print('android')` 正是「以为编了 iOS、实际编的 Android」的来源）。
控制台「公民」详情页的「编译iOS端 / 编译Android端」两个按钮各自把平台传死。
探测不到目标平台的设备就报错退出，绝不改编另一端。

## 2. 目录结构

```text
lib/
├── Isar/
│   ├── wallet_isar.dart
│   └── wallet_isar.g.dart
├── rpc/
│   ├── chain_rpc.dart          ← 底层 RPC 通信（节点管理、JSON-RPC 方法）
│   ├── onchain.dart            ← onchain 模块 RPC 功能（转账、状态查询）
│   ├── rpc.dart
│   └── RPC_TECHNICAL.md
├── signer/
│   ├── local_signer.dart
│   ├── qr_signer.dart
│   └── SIGNER_TECHNICAL.md
└── wallet/
    ├── wallet.dart
    ├── core/
    │   ├── wallet_manager.dart         ← 钱包生命周期 + 账户 child 读取守卫
    │   ├── secure_seed_store.dart      ← 账户 child 硬件金库抽象
    │   ├── hardware_bound_seed_vault.dart
    │   └── wallet_secure_keys.dart
    ├── capabilities/
    │   ├── attestation_service.dart
    │   └── wallet_type_service.dart
    ├── pages/
    │   ├── account_detail_page.dart
    │   ├── wallet_page.dart
    │   └── transaction_history_page.dart
    └── widgets/
        ├── wallet_action_card.dart
        ├── wallet_identity_card.dart
        └── wallet_onchain_balance_card.dart
```

`wallet/` 目录只允许一层子目录；不得再出现 `ui/cards/` 这类二级业务目录。

## 3. 分层职责

### 3.1 `core`

- `wallet_manager.dart`
  - 钱包生命周期与地址派生
  - 热钱包：每个账户的 child 私钥写入硬件安全存储；不持久化母种子和助记词
  - 冷钱包：仅存公钥与地址到 Isar（不写 secure storage）
  - 账户私钥读取时由 `BiometricPrompt.CryptoObject` 强制强生物识别
  - 钱包元数据写入 Isar
- `lib/isar/app_isar.dart`
  - Isar 最终集合定义与数据库打开
  - 正式创世切换前已完成最终业务库重建；运行态不读取旧 schema、不执行旧格式 migration
  - 提供 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()` 作为全 App 业务读写唯一入口；余额刷新、交易流水同步、多签扫描、钱包导入等并发读写必须排队执行
  - `LocalTxEntity` 保存本机钱包进入 App 后的余额变化流水；`WalletTxSyncCursorEntity` 保存每个钱包的同步起点和最新同步高度
- `hardware_bound_seed_vault.dart`
  - 账户 child 私钥按 `account_id` 分密文保存；同钱包账户共享 Android Keystore KEK
  - 私钥明文只在生物识别成功后的解密与签名期间短暂进入内存
- `lib/security/secure_storage.dart`
  - CitizenApp 通用安全存储唯一入口为 `appSecureStorage`；业务代码不得各自创建
    `FlutterSecureStorage()`，避免 Android/iOS 选项漂移
  - Android 使用插件 10.x 默认 RSA-OAEP + AES-GCM，开启
    `migrateOnAlgorithmChange` 与 `migrateWithBackup`，算法升级时先备份再迁移
  - iOS 使用 `first_unlock_this_device` 且禁止同步，只允许本机首次解锁后读取，
    不随 iCloud 或换机迁移
  - 通用安全存储不得开启统一生物门禁：PIN 哈希、设备锁状态和短期令牌需要静默读取；
    seed 的生物认证只归 `HardwareBoundSeedVault` 原生硬件金库
  - 当前 iOS 仅完成 Dart Keychain 选项收口；Secure Enclave seed 金库、P-256
    设备子钥及真实 iPhone 验收归
    `20260728-citizenapp-ios-vault-secure-storage` 任务，未验收前不得宣称 iOS
    已具备 Android 对等硬件金库

### 3.2 `capabilities`

- `wallet_type_service.dart`
  - 扫描三张链上 `AdminAccounts`，生成管理员展示标签缓存
- `attestation_service.dart`
  - 证明 token（secure）+ 元信息（Isar）

电子护照归属 `lib/my/myid/`；钱包模块只提供钱包元数据、热钱包签名和唯一身份钱包标记。电子护照不再复用钱包页作为身份钱包选择器。

### 3.3 `pages`

- `wallet_page.dart`
  - 「我的钱包」只展示唯一热钱包的账户卡片及冷钱包卡片。热账户卡整卡进入账户详情，
    右侧固定为“扫码签名 + 竖三点”；竖三点菜单固定“重命名 / 账户详情 /
    删除钱包或删除账户”，账户0显示“删除钱包”，其余账户显示“删除账户”
  - 钱包列表只允许把链上唯一 `voting_account_id` 对应的钱包标为“身份钱包”，不得按多个钱包分别认证
  - 热钱包创建/导入（`CreateWalletPage` / `ImportWalletPage`）
  - 冷钱包创建/导入（`CreateColdWalletPage` / `ImportColdWalletPage`），导入冷钱包页标题右侧提供扫码图标，复用 `QrScanPage(raw)` 识别钱包二维码并只回填账户地址/公钥输入框
  - 余额显示与刷新（通过 `lib/rpc/ChainRpc.fetchFinalizedBalance()` / `fetchFinalizedBalances()` 直连节点）
  - 热账户详情页（`AccountDetailPage`）：用户二维码独立贴在账户卡右上角；完整 SS58 地址独占
    第二行，复制按钮靠齐内容右边界，不再用二维码预留位挤压地址宽度；
    充值列显示该 `account_id` 的 finalized total 链上余额，零钱包列显示该账户清算行余额，
    下方复用现有交易记录；AppBar 最右侧竖三点只提供“清算行 / 查看私钥”
  - 冷钱包详情页（`WalletDetailPage`）：余额卡片（含钱包名称）、二维码（含下载按钮）、地址、交易记录入口+最近记录
- `transaction_history_page.dart`
  - 交易记录列表页（`TransactionHistoryPage`）：按 `accountId` 过滤，显示业务类型、带正负号的余额变化、对方地址、时间、状态
  - 交易记录详情页（`TransactionDetailPage`）：显示余额变化、转账金额、手续费、发送方、接收方、对方地址、区块/事件定位、txHash、来源、状态与失败原因

### 3.4 `widgets`

- `wallet_identity_card.dart`
  - 钱包身份卡：钱包名、短地址、复制与二维码入口
- `wallet_action_card.dart`
  - 账户操作卡：充值、提现与零钱包；充值余额来自链上 finalized total，
    零钱包余额来自该 `account_id` 绑定的清算行，两者不得混用
- `wallet_onchain_balance_card.dart`
  - 链上余额卡：展示链上 finalized total 余额
- `wallet_identity_card.dart` / `pages/account_detail_page.dart`
  - 统一调用 `openAccountQrPage()`：身份账户生成固定 `k=3`，其它账户生成五分钟
    `k=4`；链上身份读取失败时从严拒绝

## 4. 关键流程

### 4.0 SS58 前缀单源（2026-07-30 创世前审计收敛）

GMB 链 SS58 前缀 **2027** 的唯一常量是：

```dart
// citizenapp/lib/citizen/shared/account_derivation.dart
const int kGmbSs58Prefix = 2027;   // 对齐链端 primitives::core_const::SS58_FORMAT
```

收敛前，前缀在 **24 个文件**里各自复制：钱包、rpc、qr、通讯录、投票身份、机构详情、
多签转账、个人多签、清算行目录、广场提案等各写一份 `_ss58Prefix = 2027` 或
`_ss58Format = 2027`，`personal_admin_list_page.dart` 甚至直接裸写 `2027`。
全部已改为 `import` 单源。

**此后禁止**在任何页面、service、rpc 层重新声明前缀常量或直接写字面量 `2027`；
链端一旦改 `SS58_FORMAT`，改一处即可全端同步，不必再全仓搜数字。

### 4.1 创建热钱包

1. 生成 `bip39` 助记词
2. 在内存中派生母 mini-secret：`mnemonic → entropy → substrate_bip39`，再硬派生账户0
   `//0` 的 child mini-secret
3. 用 `Keyring.sr25519.fromSeed(child_N)` 派生 SS58(2027) 地址与公钥（model B：`child_N`=助记词`//N` 硬派生，账户0=`//0`，无 bare 根）
4. 钱包元信息通过 `WalletIsar.instance.writeTxn()` 写入 Isar（`signMode: 'local'`）
5. 只把账户0 child mini-secret 经硬件 KEK 加密后的 blob 写入 Secure Storage；母种子清零
6. 创建流程立即复读 Isar 与 secure storage；校验失败必须回滚钱包记录和机密材料，不能展示助记词后留下空钱包列表
7. 助记词一次性展示给用户

### 4.2 导入热钱包

1. 校验助记词合法性
2. 在内存中派生母 mini-secret，再硬派生账户0 child、地址与公钥
3. 钱包元信息通过 `WalletIsar.instance.writeTxn()` 写入 Isar（`signMode: 'local'`），并在同一事务内分配 `walletIndex` 与更新当前激活钱包
4. 只保存账户0 child 的硬件加密 blob，母种子与助记词不落盘
5. 导入流程立即复读 Isar 与 secure storage；校验失败必须回滚钱包记录和机密材料
6. 设为当前激活钱包

### 4.3 创建冷钱包

1. 生成 `bip39` 助记词
2. 派生地址/公钥（同热钱包）
3. 仅写 Isar（`signMode: 'external'`），不写 secure storage
4. 助记词一次性展示，强警告用户自行保管

### 4.4 导入冷钱包

1. 只接受 SS58 展示地址，不接受 AccountId 或裸公钥代替地址
2. 页面可点击顶部扫码图标，调用摄像头识别当前钱包二维码
3. 扫码结果仅提取 `user_contact.body.ss58_address`、
   `user_transfer.body.ss58_address`、`gmb://account/<ss58_address>` 或裸 SS58 地址
4. 扫码后只回填输入框，不自动执行导入
5. 导入时通过 `Keyring().decodeAddress()` 解码 SS58，得到 32 字节后生成规范 AccountId
6. 仅写 Isar（`signMode: 'external'`），不写 secure storage

### 4.5 余额查询

1. 页面 `initState` 和下拉刷新触发 `_refreshBalancesFromChain()`
2. 页面先通过 `_loadWallets()` 读取一次本地钱包列表，再把同一份列表传给余额刷新，避免首屏加载和余额刷新并发读取 Isar
3. 一次收集所有本地钱包 AccountId，调用
   `ChainRpc.fetchFinalizedBalances(accountIds)` 批量读取 finalized 块上的
   `System.Account`
4. 轻节点先等待同步完成；若轻节点未初始化、同步失败或链路降级，直接向上抛出真实错误
5. 批量解码 SCALE 编码的 `AccountInfo.free` 余额（分），转换为元；钱包详情链上余额卡使用 `fetchFinalizedTotalBalance()` 显示 `free + reserved`
6. 若余额有变化，通过统一写队列更新 Isar 中的 `WalletProfileEntity.balance`
7. 刷新 UI 显示；若轻节点不可用，则页面显示统一提示，而不是把失败误判为 0 余额；若本地库短暂繁忙，只显示“本地钱包数据库繁忙”且保留已有列表
8. `ChainRpc.fetchBalance()` 是 best 视图余额，只能用于交易监听或诊断；钱包页面不得把 best 余额写入展示缓存

### 4.5.1 钱包交易流水同步

1. 钱包新建或导入到本机后，`ChainTxMonitor` 为该钱包建立 `WalletTxSyncCursorEntity`，finalized 补同步起点为当前 finalized 区块；不查询、不补录导入前历史。
2. 钱包页加载本地钱包后按 `accountId` 注册监听；监听 newHeads 时先把当前区块命中的 `OnchainTransaction::TransferWithRemark` 写成 `inBlock`，启动、重连和 finalized 后还会补扫 `finalized+1..best` 的未确认区块，避免错过 newHeads 的收款记录；监听 finalizedHeads 时按游标补同步并升级为 `finalized`。
3. 命中本机钱包的事件写入 `LocalTxEntity`：收入保存正数 `amountDeltaFen`，支出保存负数 `amountDeltaFen`；普通链上转账备注写入 `remark`；不再单独保存 `direction`。
4. 业务类型只写入 `type`，例如 `transfer / fee / reward / interest / issuance / burn / multisig_transfer`；列表方向由金额正负号推导。
5. 区块事件记录唯一键为 `accountId:blockHash:eventIndex`；本机提交后的 pending 记录唯一键为 `accountId:pending:txHash`，写入时按同钱包、同区块、同发送方、同接收方、同转账本金合并本机提交记录和重复区块事件，避免重复显示。
6. 删除非0账户时同步删除该 `account_id` 的账户行、硬件金库 child、`LocalTxEntity` 和
   `WalletTxSyncCursorEntity`；CID 通讯录与聊天密文归永久 CID，不得因删除非当前账户或
   换绑前账户而删除。
7. 删除账户0即删除整只热钱包：必须先显示危险提示，再用账户0对本机一次性随机挑战签名并
   本地验签；只有验签通过才能删除。删除时覆盖该钱包全部账户的账户行、硬件金库 child、
   交易记录、同步游标和本机 CID 隐私数据。再次导入同一链上账户从新的本机
   导入时刻重新记录。
8. 流水同步遇到本地 Isar/MDBX 繁忙时直接让路到下一轮，不和钱包列表、余额刷新、治理页面抢写锁。
9. 交易页 `签名交易` 下方的四个状态只统计当前交易钱包的转出记录；账户详情页和完整交易记录页才展示该账户全部收支流水。

### 4.5.2 硬件私钥生命周期

1. 同一热钱包的全部账户密文按 `account_id` 分开存储，但共享该 `walletIndex` 的严档
   Android Keystore KEK；读取任一 child 时由 `BiometricPrompt.CryptoObject` 每次认证。
2. 删除非0账户只删除该账户密文，绝不能删除共享 KEK；只有删除整钱包或创建失败回滚时
   才能删除共享 KEK。
3. 系统备份可能恢复密文但不会迁移 Android Keystore 私钥；缺失 KEK 必须映射为
   `SeedKeyInvalidated` 并向 UI 明确报告设备安全存储中的账户私钥不可用，不得泄露
   Kotlin 空值强转异常。
4. 查看私钥的唯一流程是“危险确认 → 设备安全存储读取 → 强生物识别 → 显示私钥”；
   查看页面不得要求输入助记词，也不得静默生成新 KEK 冒充恢复。
5. App 不保存助记词或母种子。KEK 缺失后，现有账户私钥密文无法在本机反解，必须
   fail-closed；任何重新导入属于独立的钱包生命周期操作，不能嵌入查看私钥流程。
6. 硬件金库生成的账户 child 密文 blob、PIN 哈希、设备锁设置与短期 attestation token
   通过 `appSecureStorage` 静默持久化；Chat 与通讯录用途子钥禁止持久化。新增安全存储
   调用不得绕开该单源。

### 4.5.3 账户卡片扫码签名

1. 热账户卡片扫码按钮复用现有 `QrScanPage`，只把页面标题设为“扫码签名”；蓝色对准框、
   提示小字、相册、手电筒和其它视觉参数不得由钱包页面改写。
2. 该入口只接受 `QR_V1 k=1` 签名请求，不把收款码分派到付款流程。
3. 卡片的 `Account.accountId`（业务字段 `account_id`）是强制签名账户；公民身份和广场动作请求中的
   `signer_public_key` 与该账户不一致时，必须在读取私钥前拒绝。
4. 三类签名服务最终统一调用 `WalletManager.signForAccountId()`，不得回退到账户0。

### 4.5.4 钱包卡片拖拽排序

1. `MyWalletPage` 使用 `ReorderableListView` 承载钱包卡片，长按拖拽触发 `_onReorder(oldIndex, newIndex)`。
2. `WalletManager.getWallets()` 返回 fixed-length list，UI 层不能直接对 `_wallets` 执行 `removeAt/insert`。
3. UI 层统一通过 `reorderWalletProfiles()` 先复制成可变列表，再按 Flutter `onReorder` 规则修正目标下标。
4. 页面先 `setState` 展示新顺序，再调用 `WalletManager.reorderWallets()` 把 walletIndex 顺序写入 Isar `sortOrder`。
5. `getWallets()` 查询时按 `sortOrder` 升序返回，相同值再用 `walletIndex` 兜底，保证重启后顺序稳定。

### 4.6 登录签名

CitizenApp 不承担 OnChina 管理员扫码登录职责。管理员登录由 OnChina 页面生成
`QR_V1 k=1,a=1` 登录签名请求,CitizenWallet 公民钱包扫码签名并返回 `k=2`
签名响应。CitizenApp 钱包模块不生成登录签名请求,也不解析登录签名响应。

### 4.7 链上支付签名（由 onchain 调用）

- **热钱包**：`WalletManager.signWithWallet()` 签名回调注入 `OnchainPaymentService`
  （账户 child 不出 WalletManager）；签名前必须重新派生本地公钥，并校验其转换得到的
  AccountId 与当前 `WalletProfile.accountId` 完全一致，不一致直接拒绝签名。
- **冷钱包**：构造 `QR_V1 k=1` 签名请求 → 导航到 `QrSignSessionPage` → 展示请求二维码
  → 用户用 CitizenWallet 离线设备扫码签名（离线端按 `a+d` 独立解码 payload）→
  扫描 `k=2` 签名响应二维码 → `QrSigner.parseResponse()` 校验
  `request_id + signer_public_key + signature` → 签名回调注入。

`OnchainPaymentService.submitTransfer()` 接受 `sign` 回调参数，由 UI 层根据 `signMode` 提供不同实现。

### 4.8 治理提案/投票签名（由 governance + signer 调用，规划）

1. 治理模块按业务类型组装提案/投票字段。
2. 钱包模块输出当前激活钱包上下文
   （`accountId / ss58Address / alg / ss58`）。
3. 根据 `signMode` 分流：
   - `local`：`WalletManager.signWithWallet()`（seed 不出类）。
   - `external`：调用 `QrSigner` 发起外部签名会话。
4. 回传签名结果给治理模块提交链上交易。
5. 选择了哪个管理员钱包，就必须由同一钱包完成签名：
   - 热钱包：`walletIndex` 对应 seed 派生公钥转换出的 AccountId 必须等于页面选中的
     `accountId`
   - 冷钱包：签名响应中的 `signerPublicKey` 转换出的 AccountId 必须等于页面选中的
     `accountId`
6. 联合公投和立法特别案不再存在独立人口快照签名；钱包只签业务提案，runtime
   使用同一笔提案的标准外层签名管理员和 `actor_cid_number` 完成授权，并在事务内
   创建快照。钱包不得恢复快照 action、call 常量或两步签名会话。

## 5. 存储设计（当前）

### 5.1 机密层（硬件金库 + flutter_secure_storage）

- `account_child_key_<account_id>`：账户 child mini-secret 的硬件 KEK 加密 blob；同一热钱包
  的账户共享 `walletIndex` KEK，但各 `account_id` 独立保存密文
- `wallet.session.<scope>.token`：短期会话 token
- PIN、设备锁和 attestation 所需最小状态

禁止保存助记词、母种子、Chat/通讯录用途子钥或预留的用户数据主钥。

### 5.2 业务层（Isar）

集合定义（`lib/isar/app_isar.dart`）：

- `WalletProfileEntity`
  - `walletIndex, walletName, walletIcon, balance, accountId, ss58Address, alg, ss58, createdAtMillis, source, signMode, sortOrder`
- `WalletSettingsEntity`
  - `activeWalletIndex, updatedAtMillis`
- `LocalTxEntity`
  - `recordKey, ss58Address, accountId, type, amountDeltaFen, transferAmountFen, feeFen, counterpartySs58Address, fromSs58Address, toSs58Address, remark, status, source, txHash, blockNumber, blockHash, eventIndex, extrinsicIndex, usedNonce, confirmedAtMillis, failureReason, createdAtMillis`
- `WalletTxSyncCursorEntity`
  - `ss58Address, accountId, trackingStartBlock, lastSyncedBlock, createdAtMillis, updatedAtMillis`
- `AdminGroupCacheEntity`
  - `accountId, adminGroupName, updatedAt`
- `LoginReplayEntity`
  - `requestId, expiresAt`
- `AppKvEntity`
  - `key, stringValue, intValue, boolValue`

写库约束：

- 钱包模块和其他业务模块不得直接调用 `WalletIsar.instance.db()` 后读写 collection，也不得直接调用 `isar.writeTxn()`；统一使用 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()`，避免 Android 真机上多个异步任务同时读写 MDBX 时出现 `MdbxError (11): Try again`。
- 钱包 settings 行的创建不得在已有写事务中再次开启写事务；事务内只能调用 `_getSettingsInTxn()` 这类明确带 `InTxn` 后缀的方法。
- 钱包创建/导入必须在返回 UI 前完成落库校验；任何一个落库或账户私钥写入步骤失败，
  都必须回滚同一 `walletIndex` 的 Isar 记录、已写入的 child 私钥密文和硬件 KEK；
  助记词只存在于创建/导入页面内存，不得持久化。
- `WalletManager.createWallet()` / `importWallet()` / `importColdWallet()` 的钱包元数据写入和当前钱包切换在同一事务内完成，避免钱包索引重复、激活钱包丢失或嵌套事务。
- 钱包页展示错误时，本地 Isar/MDBX 错误统一提示为本地钱包数据库繁忙，不再显示为轻节点或区块链连接失败。

### 5.3 其他 SharedPreferences

- 电子护照不再使用 `cid.bind.*` 或 `myid.*` 本地身份缓存；按默认热钱包读 finalized `CidByAccountId`，再闭环校验 `CidRegistry` Active、`AccountIdByCid` 反向绑定和 CID 主键身份。

### 5.4 钱包详情页布局 `WalletDetailPage`

页面元素（自上而下）：

1. 余额卡片：左上角钱包名称（可点击编辑），居中余额数字+元+GMB
2. 二维码：当前账户命中链上 CID 身份闭环时生成固定
   `QR_V1 kind=user_contact`；未注册账户或其它钱包子账户生成五分钟
   `kind=user_transfer` 临时收款码。链上身份读取失败时不生成二维码，不把钱包名
   伪装成公开昵称
3. 冷钱包离线签名入口由 CitizenWallet 承担；CitizenApp 钱包详情页不承载 `QrOfflineSignPage`
4. 地址+复制：地址居中两行显示，复制图标在右侧
5. 交易记录标题行：左侧"交易记录"，右侧箭头，点击进入完整交易记录列表
6. 最近交易记录：最多显示 5 条，显示与完整列表一致的状态标签，点击单条进入交易详情

### 5.5 交易记录数据来源

钱包详情页和交易记录页面直接复用 `LocalTxStore`（Isar `LocalTxEntity`），按 `accountId` 过滤。

- 本机提交普通转账成功后通过 `LocalTxStore.upsertLocalSubmitTransfer()` 写入 `source=local_submit / status=pending` 记录，用于立即反馈支出；如果区块事件已经先写入，则合并手续费、txHash、nonce 和备注，不新增第二条
- 交易池 included 回调先把本机提交记录升级为 `status=inBlock`；newHeads 命中收入或支出事件时写入 `source=chain_event / status=inBlock`；启动、重连和 finalized 后会补扫 finalized 之后的未确认区块，补齐错过实时订阅的 `inBlock` 流水
- finalized 区块事件监听命中后升级同一条区块事件记录为 `status=finalized`，并把匹配的本机提交记录合并为 finalized；该升级只能来自 finalized 高度，不能来自 best/latest 高度
- 钱包详情页展示最近 5 条，点击"交易记录"标题或右侧箭头进入完整列表，点击单条进入该笔交易详情
- `txHash` 只作为本机 pending 提交标识；单条链上流水的唯一定位以 `recordKey` 为准

## 6. 数据重建策略

正式创世切换前，CitizenApp 已按最终 Isar schema 完成一次空库重建，并创建唯一的
`WalletSettingsEntity(id=0)`。运行态不保存 `wallet.data.schema.version`，不执行旧
collection 清理 migration，也不读取旧字段。2026-07-26 正式创世后，当前正式热钱包
`Rhett` 及其应用数据不得再按开发期策略清空、重置、覆盖或通过卸载删除。
secure storage、Keychain/Keystore、助记词、seed、私钥和生物识别保护材料不属于
业务库，严禁随 Isar 删除。

## 7. 安全边界

- 助记词、母种子和账户 child 明文不写入 Isar/Postgres/日志
- **账户 child 不出 WalletManager**：身份账户签名统一走 `signForAccountId()`，账户0资金/
  治理签名走 `signWithWallet()`；账户 child 只在硬件解密、派生或签名期间短暂存在
- 通讯录本地与云端子钥都由 CID 当前链上绑定钱包账户的 child mini-secret 在真实数据
  解密确认缺钥或正式换绑作用域内派生，域固定为 `citizenapp.account-data/contacts-local` 与
  `citizenapp.account-data/contacts-cloud`。日常业务只从设备数据钥金库静默解封
  `ContactKeyMaterial`，不能接触当前账户 child，也不能用通讯录密钥签名或恢复钱包。
- 私有数据不另设稳定主钥、随机 CID 密钥、Worker 密钥或节点密钥。唯一派生输入是当前
  钱包账户 child mini-secret；HKDF salt 绑定 `genesis_hash + cid_number +
  binding_revision + account_id`，info 绑定用途和可选 context。用途密钥明文只在内存中
  短期使用；缺钥初始化后只保存由独立 Android Keystore AES-GCM / iOS Secure Enclave
  ECIES 硬件钥封装的密文 blob，不写入 Isar、D1、R2、Worker Secret 或日志。
- `activateAccountDataBinding` 只验证本机账户存在并单调写入公开 `AccountDataBinding`
  元数据，禁止读取 child。`readDataKeyForCurrentBinding/readDataKeysForBinding` 优先静默
  解封已有设备用途钥；只有真实数据访问确认缺钥或硬件封装失效时才鉴权一次生成并重试。
  同一账户在新设备导入后能重新派生相同密钥；换绑后的新账户直接接管 CID 并派生新的
  用途密钥，不能直接解密换绑前当前账户加密的历史私有数据。只有同次换绑取得当前账户
  签名时，客户端才对 Chat 与通讯录执行端内重加密；无签名换绑不读取此前账户、私钥、
  助记词、设备或缓存。
- 本地数据钥入口为 `ensureDeviceDataKeysForBinding`，只允许由真实数据访问确认缺钥或
  硬件封装失效时调用；它只派生、封装本地用途钥，绝不登记 P-256 子钥或调用 Turnstile。
  P-256 登记入口为 `registerDeviceSubkeyForBinding`，只允许由 Worker 明确返回
  `device_not_registered` 后调用；它绝不生成、覆盖或删除本地用途钥。两类入口各自使用
  独立静态 single-flight 和独立失败回滚，同一
  `(genesis_hash, cid_number, account_id)` 的并发调用各共享一次 Future；已有对应密钥时
  读取 child 次数为 0。CID finalized、页面门禁和后台预热不得调用。CID 换绑要求目标
  `account_id != 当前 account_id`；相同账户不得用新 `binding_revision` 伪装换绑，并在读取
  child、构造交易和数据交接之前拒绝。
- 删除钱包必须清除该钱包的账户 child 和设备子钥；只有被删账户拥有本机当前激活 CID
  绑定时，才清理对应的公开绑定元数据和内存用途子钥。删除本地钱包不得删除 CID 业务数据。
- 助记词不持久化，仅创建时一次性展示
- 冷钱包不在本机保存任何密钥材料
- 本机签名在本地完成，私钥材料不出端
- 账户 child 每次读取都由硬件金库触发强生物识别，取消、锁定、密钥失效或密文缺失均
  fail-closed；不存在预认证窗口、无认证读取或设备密码回退入口。
- 账户 child 只允许用于正式交易签名、明确钱包级鉴权、CID 注册/有效换绑交易签名、换绑
  数据交接、真实数据钥缺失/失效后的本地生成，以及 Worker 明确未登记后的 P-256 登记。
  广场、Chat、创作者、通讯录、会员/订阅的页面进入、已有钥的日常数据解密、后台推送预热
  和普通 Session/MLS 初始化一律禁止读取。
- Chat 与广场后台登录只使用不可导出的 P-256 设备子钥静默签名；Chat/MLS/附件/通讯录
  数据只使用设备硬件钥静默解封的用途钥，两条日常路径都不读取账户 child。
- 热钱包创建/导入入口先确认强生物识别可用；账户 child 读取后校验 64 位十六进制格式并
  在签名或派生完成后清零。
- 账户 child 密文键只允许由 `HardwareBoundSeedVault` 生成；Session 键只允许由
  `WalletSecureKeys` 生成，禁止业务代码散落硬编码
- walletIndex 分配与 profile 写入在同一 Isar 事务中完成（`_appendHotWalletAtomic` / `_appendColdWalletAtomic`），防止并发创建/导入时 index 冲突导致密钥覆盖；secure storage 写入在事务成功后执行

## 8. 主要接口（对外）

- `WalletManager`
  - `createWallet / importWallet / importColdWallet`
  - `deleteWallet / setActiveWallet`
  - `signForAccountId(accountId, payload)` — CID 当前账户 sr25519 签名
  - `signWithWallet(walletIndex, payload)` — 热钱包账户0 sr25519 签名
  - `walletIndexForAccountId(accountId)` — 定位当前账户实际所属热钱包，供设备子钥使用
  - `ensureDeviceDataKeysForBinding(binding)` — 真实数据缺钥时生成本地设备数据钥；与设备
    登记完全隔离
  - `registerDeviceSubkeyForBinding(binding)` — Worker 明确未登记时登记 P-256 子钥；与本地
    数据钥完全隔离
  - `ensureContactKeyMaterialForAccountId(accountId)` — 已有设备用途钥时静默返回通讯录
    加密钥和索引钥；真实缺钥时鉴权一次生成，页面进入不调用
- `ChainRpc`（`lib/rpc/chain_rpc.dart`）
  - `fetchFinalizedBalance` / `fetchFinalizedBalances` / `fetchFinalizedTotalBalance` — 直连节点查询 finalized 链上余额
- `ChainTxMonitor`（`lib/rpc/chain_tx_monitor.dart`）
  - 监听 newHeads/finalizedHeads 区块事件，补扫未 finalized 区块，按本机钱包游标增量写入并升级交易流水

## 9. 测试覆盖（当前）

- `test/wallet/wallet_manager_test.dart`
  - 热钱包创建/导入/删除/账户 child 硬件密文联动
  - 冷钱包导入/删除/无账户 child 存储
  - 当前 `account_id` 到所属 `walletIndex` 的精确定位
- `test/wallet/seed_derivation_test.dart`
  - 验证 `fromSeed` 与 `fromMnemonic` 产出一致公钥
- `test/wallet/attestation_service_test.dart`
  - attestation token 落 secure storage
  - attestation 元信息落 Isar
- `test/wallet/sign_service_test.dart`
  - 挑战解析、签名、防重放、钱包匹配
- `test/wallet/wallet_manager_reorder_test.dart`
  - `reorderWallets()` 写入 `sortOrder` 后，`getWallets()` 按新顺序返回
  - 尚未写入 `sortOrder` 的钱包按原 `walletIndex` 顺序初始化
- `test/wallet/pages/wallet_list_tile_test.dart`
  - 钱包卡片 UI 渲染契约
  - `reorderWalletProfiles()` 支持 fixed-length 钱包列表，且不改写原列表

## 10. 钱包模式规范

### 10.1 模式定义

- `signMode: 'local'`（热钱包 — 本机签名）
  - seed 保存在手机 secure storage
  - 转账、登录、提案、投票均可直接在手机签名
- `signMode: 'external'`（冷钱包 — 扫码签名）
  - 手机不保存私钥，仅保存钱包公开信息
  - 转账、登录、提案、投票均通过扫码请求外部设备签名

### 10.2 最小钱包上下文字段

| 字段 | 说明 |
| --- | --- |
| `ss58Address` | SS58 展示地址（当前链 `ss58 = 2027`） |
| `accountId` | 小写 `0x` + 64 位十六进制 AccountId |
| `alg` | 固定 `sr25519` |
| `ss58` | 地址格式版本（当前 2027） |
| `source` | `created/imported` |
| `signMode` | `local/external` |

### 10.3 Seed 派生链

```
mnemonic
  → entropy (bip39_mnemonic Mnemonic.fromSentence)
  → PBKDF2 (substrate_bip39 CryptoScheme.miniSecretFromEntropy)
  → 32 字节 mini-secret（master）
  → //N 硬 junction 派生 child_N（账户0=//0，无 bare 根）
  → Keyring.sr25519.fromSeed(child_N)
  → sr25519 keypair
```

说明：使用 Substrate 特定的 BIP39 派生（非标准 BIP32），与 `polkadart_keyring` 的 `fromMnemonic` 内部逻辑一致。

## 11. 治理字段联动要求

- 联合提案人口分母由 runtime 按 `PopulationScope` 从链上公民身份读取。
- 链上投票交易只提交账户签名、提案号和赞反意见。
- 钱包模块负责提供签名账户上下文，不负责生成投票资格或人口凭证。
- 钱包模块必须保证"登录签名"和"转账/治理签名"使用不同签名 payload。

## 12. 管理员链读取约束

- 钱包管理员展示标签直接扫描 `PublicAdmins`、`PrivateAdmins`、`PersonalAdmins` 的 `AdminAccounts`。
- `AdminGroupCacheEntity` 只保存链数据派生的展示标签，TTL 为 5 分钟，不参与任何管理员权限判断。
- 扫描出现分页或批量读取失败时拒绝覆盖完整缓存；链不可用时回退普通“手机钱包”标签。

## 13. PQC 抗量子签名升级(设计,待实现)

- **真源 = `memory/04-decisions/ADR-022-unified-pqc-crypto.md`**(取代旧 PQC 迁移方案);任务卡 `memory/08-tasks/open/20260618-pqc-card3-wallet-derivation-signing.md`。

热钱包随全系统从 sr25519 **在位升级**到 ML-DSA-65 签名,"四不变"(不换助记词/账户/地址/余额)。以 ADR-022 为准:

- **派生(model B //index,sr25519 不套 HKDF)**:每账户 `account_seed_N` = 该账户 child mini-secret(账户 N=助记词`//N`,账户0=`//0`,无 bare 根);sr25519 地址锚点 = `sr25519.fromSeed(account_seed_N)` **直接派生**(不经 HKDF);ML-DSA-65/ML-KEM-768 用 `HKDF-SHA512(account_seed_N, "GMB/account/ml-dsa-65" | ".../ml-kem-768")`。ML-DSA keygen/sign 走 Rust FFI(`gmb-pqc`),非 Dart。
- **签名/交易**:无感 bootstrap——未绑定账户首次交易构造 `bootstrap_pqc_dispatch`(sr25519 bootstrap 签名 + ML-DSA 交易签名,一次确认);后续走 `pqc_dispatch` general-tx(`signed_extrinsic_builder.dart:103/186`,**不扩 MultiSignature**)。
- **QR**:`sig_alg(sr25519|ml-dsa-65)` + `auth_mode(normal|pqc|bootstrap-pqc)` + `key_version` + `chunk_index/chunk_total` 分片(ML-DSA ~3.3KB,最坏体积按 bootstrap 实测)。
- **UI**:只展示一个账户/地址/余额,不暴露多公钥/绑定状态机/换账户。
- **安全**:`account_seed`/PQC 私钥不出本机;CID 不托管。

> 实现以本节 + ADR-022 为准,旧路线不再适用。

## 系统弹窗本地化(2026-08-06,双端对称)

口径:**默认中文;仅当手机系统语言是英文时显示英文;其余语言(日/法/德…)一律回落中文。**
公民与公民钱包、iOS 与 Android 四条线完全同规则。

iOS(两端各一份,缺一不可):

- 工程 `developmentRegion = zh-Hans` —— **这一条就是「其它语言回落中文」的开关**;
  `knownRegions` 含 `zh-Hans`,`Info.plist` 显式 `CFBundleLocalizations = ["zh-Hans","en"]`。
- `Runner/zh-Hans.lproj/InfoPlist.strings`(中文)与 `Runner/en.lproj/InfoPlist.strings`
  (英文)。两文件**键集必须完全一致**,改动成对进行。
- **`.lproj` 必须登记进 Xcode 工程**:pbxproj 不做通配,要有 `PBXFileReference` ×2 +
  `PBXVariantGroup` + `PBXBuildFile` + 挂进 Resources 构建阶段,否则文件躺在磁盘上
  却不进 `.app`,弹窗照旧英文。验证方式:`ls build/ios/iphoneos/Runner.app/*.lproj`
  必须出现 `zh-Hans.lproj` 与 `en.lproj`。
- 系统权限弹窗的**框架句式**(`…would like to send you notifications`)由 iOS 按 App
  声明的本地化语言生成,不是 App 文案;`NS*UsageDescription` 才是弹窗里那句解释。

Android(两端各一份):

- `res/values/strings.xml` 中文 = 默认(无限定符目录是 Android 的回落目标,
  非英文语言全部取它);`res/values-en/strings.xml` 英文。
- `AndroidManifest` 的 `android:label` 必须是 `@string/app_name`,不得写死中文,
  否则本地化不生效。验证方式:`aapt2 dump resources <apk> | grep -A2 app_name`
  应同时出现 `()` 与 `(en)` 两条。
- **平台事实**:Android 运行时权限弹窗正文由系统渲染、跟随手机系统语言,App 无权干预;
  App 能控的只有弹窗里显示的 App 名与生物识别对话框文案。

生物识别对话框(两端各 `lib/ui/biometric_auth_text.dart`,文案单源):

- `localizedReason` 只是中间那句解释,**标题/提示/取消按钮由 local_auth 插件提供、
  默认英文硬编码串** —— 必须同时传 `authMessages`,否则中英混排。
- 两端 `local_auth` 版本不同(热端 2.3.0 字段多,冷端 3.0.2 已精简),各自适配字段集。
- **冷钱包刻意不设 `IOSAuthMessages.localizedFallbackTitle`**:给值 iOS 会渲染
  「使用密码」回退按钮,直接违反 `biometricOnly` 死规则。已由测试钉死。
- 语言来源留 `debugLocale` 注入接缝:生产直读 `PlatformDispatcher.instance` 时
  `TestPlatformDispatcher.localeTestValue` 覆写的是测试包装器、影响不到真单例,
  不留接缝这段逻辑完全不可测。

App 界面文案(4300+ 条)的全量国际化不在本节范围,见任务卡
`memory/08-tasks/open/20260806-citizenapp-citizenwallet-full-i18n.md`。
