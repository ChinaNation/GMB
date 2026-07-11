# Wallet 模块技术文档（当前实现态）

## 1. 模块目标

`lib/wallet` 是钱包能力唯一收口模块，负责：

- 钱包创建/导入/删除/切换（热钱包 + 冷钱包）
- 本地机密材料读取（热钱包 seed）
- 登录签名编排（签名执行由 `lib/signer` 负责）
- 转账/提案/投票所需钱包上下文输出（地址、公钥、算法、机构角色）
- finalized 余额查询（通过 `lib/rpc/` 直连链上节点）
- 管理员目录、观察账户、证明态等钱包周边能力

约束：钱包相关代码只应从 `wallet/...` 引用。

本地 Android 启动产物约定：`citizenapp/scripts/citizenapp-run.sh` 运行后把 APK 复制到
`citizenapp/target/公民.apk`；`citizenwallet/scripts/citizenwallet-run.sh` 运行后把冷钱包 APK 复制到
`citizenwallet/target/公民钱包.apk`。该本地产物只用于开发启动；正式分发包以手动
`citizenapp-ci.yml` 注入 release keystore 后生成的正式签名 `公民.apk` 为准。

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
    │   ├── wallet_manager.dart         ← 钱包生命周期 + seed 读取守卫
    │   └── wallet_secure_keys.dart
    ├── capabilities/
    │   ├── api_client.dart         ← 管理员目录、机构信息等非链上查询
    │   ├── attestation_service.dart
    │   └── wallet_type_service.dart
    ├── pages/
    │   ├── wallet_page.dart
    │   └── transaction_history_page.dart
    └── widgets/
        ├── wallet_action_card.dart
        ├── wallet_identity_card.dart
        ├── wallet_onchain_balance_card.dart
        └── wallet_qr_dialog.dart
```

`wallet/` 目录只允许一层子目录；不得再出现 `ui/cards/` 这类二级业务目录。

## 3. 分层职责

### 3.1 `core`

- `wallet_manager.dart`
  - 钱包生命周期与地址派生
  - 热钱包：seed 写入 secure storage（不存助记词）
  - 冷钱包：仅存公钥与地址到 Isar（不写 secure storage）
  - seed 读取时强制生物识别/设备密码验证（`_authenticateIfSupported()`）
  - 钱包元数据写入 Isar
- `Isar/wallet_isar.dart`
  - Isar 集合定义与启动迁移
  - 开发阶段直接覆盖，schema 版本 `v2`
  - 提供 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()` 作为全 App 业务读写唯一入口；余额刷新、交易流水同步、多签扫描、钱包导入等并发读写必须排队执行
  - `LocalTxEntity` 保存本机钱包进入 App 后的余额变化流水；`WalletTxSyncCursorEntity` 保存每个钱包的同步起点和最新同步高度
- `wallet_secure_keys.dart`
  - 机密 key 命名规范（`wallet.secret.<id>.seed_hex.v1`）

### 3.2 `capabilities`

- `api_client.dart`
  - 非链上查询的外部服务接口（管理员目录、机构信息等）
- `wallet_type_service.dart`
  - 管理员目录缓存与角色识别
- `attestation_service.dart`
  - 证明 token（secure）+ 元信息（Isar）

电子护照归属 `lib/my/myid/`；钱包模块只提供钱包元数据、热钱包签名和唯一身份钱包标记。电子护照不再复用钱包页作为身份钱包选择器。

### 3.3 `pages`

- `wallet_page.dart`
  - 钱包列表（带热/冷标识）、长按拖拽排序、创建、导入、删除、激活、地址复制
  - 钱包列表只允许把链上唯一 `identity_wallet_account` 对应的钱包标为“身份钱包”，不得按多个钱包分别认证
  - 热钱包创建/导入（`CreateWalletPage` / `ImportWalletPage`）
  - 冷钱包创建/导入（`CreateColdWalletPage` / `ImportColdWalletPage`），导入冷钱包页标题右侧提供扫码图标，复用 `QrScanPage(raw)` 识别钱包二维码并只回填账户地址/公钥输入框
  - 余额显示与刷新（通过 `lib/rpc/ChainRpc.fetchFinalizedBalance()` / `fetchFinalizedBalances()` 直连节点）
  - 钱包详情页（`WalletDetailPage`）：余额卡片（含钱包名称）、二维码（含下载按钮）、地址、交易记录入口+最近记录
- `transaction_history_page.dart`
  - 交易记录列表页（`TransactionHistoryPage`）：按 `walletPubkeyHex` 过滤，显示业务类型、带正负号的余额变化、对方地址、时间、状态
  - 交易记录详情页（`TransactionDetailPage`）：显示余额变化、转账金额、手续费、发送方、接收方、对方地址、区块/事件定位、txHash、来源、状态与失败原因

### 3.4 `widgets`

- `wallet_identity_card.dart`
  - 钱包身份卡：钱包名、短地址、复制与二维码入口
- `wallet_action_card.dart`
  - 钱包操作卡：充值、提现与零钱包余额展示；零钱包余额来自绑定清算行节点，不等同于下方链上余额卡
- `wallet_onchain_balance_card.dart`
  - 链上余额卡：展示链上 finalized total 余额
- `wallet_qr_dialog.dart`
  - 钱包二维码弹窗：生成 `QR_V1 kind=user_contact`

## 4. 关键流程

### 4.1 创建热钱包

1. 生成 `bip39` 助记词
2. 派生 mini-secret：`mnemonic → entropy → PBKDF2(substrate_bip39) → 64 字节 → 前 32 字节`
3. 用 `Keyring.sr25519.fromSeed(miniSecret)` 派生 SS58(2027) 地址与公钥
4. 钱包元信息通过 `WalletIsar.instance.writeTxn()` 写入 Isar（`signMode: 'local'`）
5. seed（32 字节 hex）写入 secure storage
6. 创建流程立即复读 Isar 与 secure storage；校验失败必须回滚钱包记录和机密材料，不能展示助记词后留下空钱包列表
7. 助记词一次性展示给用户

### 4.2 导入热钱包

1. 校验助记词合法性
2. 派生 seed → 地址/公钥
3. 钱包元信息通过 `WalletIsar.instance.writeTxn()` 写入 Isar（`signMode: 'local'`），并在同一事务内分配 `walletIndex` 与更新当前激活钱包
4. seed 写入 secure storage
5. 导入流程立即复读 Isar 与 secure storage；校验失败必须回滚钱包记录和机密材料
6. 设为当前激活钱包

### 4.3 创建冷钱包

1. 生成 `bip39` 助记词
2. 派生地址/公钥（同热钱包）
3. 仅写 Isar（`signMode: 'external'`），不写 secure storage
4. 助记词一次性展示，强警告用户自行保管

### 4.4 导入冷钱包

1. 接受 SS58 地址或 0x/64 hex 公钥
2. 页面可点击顶部扫码图标，调用摄像头识别当前钱包二维码
3. 扫码结果仅提取 `user_contact.body.address`、`user_transfer.body.address`、`gmb://account/<address>`、裸 SS58 地址，或当前输入框已支持的 0x/64 hex 公钥
4. 扫码后只回填输入框，不自动执行导入
5. 导入时解码公钥（SS58 走 `Keyring().decodeAddress()`；hex 走严格 32 字节解析）
6. 仅写 Isar（`signMode: 'external'`），不写 secure storage

### 4.5 余额查询

1. 页面 `initState` 和下拉刷新触发 `_refreshBalancesFromChain()`
2. 页面先通过 `_loadWallets()` 读取一次本地钱包列表，再把同一份列表传给余额刷新，避免首屏加载和余额刷新并发读取 Isar
3. 一次收集所有本地钱包公钥，调用 `ChainRpc.fetchFinalizedBalances(pubkeys)` 批量读取 finalized 块上的 `System.Account`
4. 轻节点先等待同步完成；若轻节点未初始化、同步失败或链路降级，直接向上抛出真实错误
5. 批量解码 SCALE 编码的 `AccountInfo.free` 余额（分），转换为元；钱包详情链上余额卡使用 `fetchFinalizedTotalBalance()` 显示 `free + reserved`
6. 若余额有变化，通过统一写队列更新 Isar 中的 `WalletProfileEntity.balance`
7. 刷新 UI 显示；若轻节点不可用，则页面显示统一提示，而不是把失败误判为 0 余额；若本地库短暂繁忙，只显示“本地钱包数据库繁忙”且保留已有列表
8. `ChainRpc.fetchBalance()` 是 best 视图余额，只能用于交易监听或诊断；钱包页面不得把 best 余额写入展示缓存

### 4.5.1 钱包交易流水同步

1. 钱包新建或导入到本机后，`ChainTxMonitor` 为该钱包建立 `WalletTxSyncCursorEntity`，finalized 补同步起点为当前 finalized 区块；不查询、不补录导入前历史。
2. 钱包页加载本地钱包后按 `walletPubkeyHex` 注册监听；监听 newHeads 时先把当前区块命中的 `OnchainTransaction::TransferWithRemark` 写成 `inBlock`，启动、重连和 finalized 后还会补扫 `finalized+1..best` 的未确认区块，避免错过 newHeads 的收款记录；监听 finalizedHeads 时按游标补同步并升级为 `finalized`。
3. 命中本机钱包的事件写入 `LocalTxEntity`：收入保存正数 `amountDeltaFen`，支出保存负数 `amountDeltaFen`；普通链上转账备注写入 `remark`；不再单独保存 `direction`。
4. 业务类型只写入 `type`，例如 `transfer / fee / reward / interest / issuance / burn / multisig_transfer`；列表方向由金额正负号推导。
5. 区块事件记录唯一键为 `walletPubkeyHex:blockHash:eventIndex`；本机提交后的 pending 记录唯一键为 `walletPubkeyHex:pending:txHash`，写入时按同钱包、同区块、同发送方、同接收方、同转账本金合并本机提交记录和重复区块事件，避免重复显示。
6. 删除钱包时同步删除该 `walletPubkeyHex` 下的 `LocalTxEntity` 和 `WalletTxSyncCursorEntity`；再次导入同一链上账户也从新的本机导入时刻重新记录。
7. 流水同步遇到本地 Isar/MDBX 繁忙时直接让路到下一轮，不和钱包列表、余额刷新、治理页面抢写锁。
8. 交易页 `签名交易` 下方的四个状态只统计当前交易钱包的转出记录；钱包详情页和完整交易记录页才展示该钱包全部收支流水。

### 4.5.2 钱包卡片拖拽排序

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

- **热钱包**：`WalletManager.signWithWallet()` 签名回调注入 `OnchainPaymentService`（seed 不出 WalletManager）；签名前必须重新派生本地公钥，并校验其与当前 `WalletProfile.pubkeyHex` 完全一致，不一致直接拒绝签名
- **冷钱包**：构造 `QR_V1 k=1` 签名请求 → 导航到 `QrSignSessionPage` → 展示请求二维码 → 用户用 CitizenWallet 离线设备扫码签名（离线端按 `a+d` 独立解码 payload）→ 扫描 `k=2` 签名响应二维码 → `QrSigner.parseResponse()` 校验 `request_id + pubkey + signature` → 签名回调注入

`OnchainPaymentService.submitTransfer()` 接受 `sign` 回调参数，由 UI 层根据 `signMode` 提供不同实现。

### 4.8 治理提案/投票签名（由 governance + signer 调用，规划）

1. 治理模块按业务类型组装提案/投票字段。
2. 钱包模块输出当前激活钱包上下文（`address/pubkeyHex/alg/ss58`）。
3. 根据 `signMode` 分流：
   - `local`：`WalletManager.signWithWallet()`（seed 不出类）。
   - `external`：调用 `QrSigner` 发起外部签名会话。
4. 回传签名结果给治理模块提交链上交易。
5. 选择了哪个管理员钱包，就必须由同一钱包完成签名：
   - 热钱包：`walletIndex` 对应的 seed 派生公钥必须等于页面选中的 `pubkeyHex`
   - 冷钱包：签名响应中的 `pubkey` 必须等于页面选中的 `pubkeyHex`
6. 投票引擎人口快照准备流程还要求：
   - 请求人口快照使用的 `account_pubkey`
   - 实际上链发起人的签名账户
   两者必须是同一把钱包，否则链上会把人口快照判为无效。

## 5. 存储设计（当前）

### 5.1 机密层（flutter_secure_storage）

- `wallet.secret.<wallet_id>.seed_hex.v1` — 热钱包 32 字节 seed（hex 编码）
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 5.2 业务层（Isar）

集合定义（`Isar/wallet_isar.dart`）：

- `WalletProfileEntity`
  - `walletIndex, walletName, walletIcon, balance, address, pubkeyHex, alg, ss58, createdAtMillis, source, signMode, sortOrder`
- `WalletSettingsEntity`
  - `activeWalletIndex, updatedAtMillis`
- `LocalTxEntity`
  - `recordKey, walletAddress, walletPubkeyHex, type, amountDeltaFen, transferAmountFen, feeFen, counterpartyAddress, fromAddress, toAddress, remark, status, source, txHash, blockNumber, blockHash, eventIndex, extrinsicIndex, usedNonce, confirmedAtMillis, failureReason, createdAtMillis`
- `WalletTxSyncCursorEntity`
  - `walletAddress, walletPubkeyHex, trackingStartBlock, lastSyncedBlock, createdAtMillis, updatedAtMillis`
- `AdminGroupCacheEntity`
  - `pubkeyHex, adminGroupName, updatedAt`
- `ObservedAccountEntity`
  - `accountId, accountLabel, publicKey, address, balance, source`
- `LoginReplayEntity`
  - `requestId, expiresAt`
- `AppKvEntity`
  - `key, stringValue, intValue, boolValue`

写库约束：

- 钱包模块和其他业务模块不得直接调用 `WalletIsar.instance.db()` 后读写 collection，也不得直接调用 `isar.writeTxn()`；统一使用 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()`，避免 Android 真机上多个异步任务同时读写 MDBX 时出现 `MdbxError (11): Try again`。
- 钱包 settings 行的创建不得在已有写事务中再次开启写事务；事务内只能调用 `_getSettingsInTxn()` 这类明确带 `InTxn` 后缀的方法。
- 钱包创建/导入必须在返回 UI 前完成落库校验；任何一个落库或机密写入步骤失败，都必须回滚同一 `walletIndex` 的 Isar 记录、seed 和助记词。
- `WalletManager.createWallet()` / `importWallet()` / `importColdWallet()` 的钱包元数据写入和当前钱包切换在同一事务内完成，避免钱包索引重复、激活钱包丢失或嵌套事务。
- 钱包页展示错误时，本地 Isar/MDBX 错误统一提示为本地钱包数据库繁忙，不再显示为轻节点或区块链连接失败。

### 5.3 其他 SharedPreferences（尚未迁移）

- 电子护照不再使用 `cid.bind.*` 或 `myid.*` 本地身份缓存；链上身份以 finalized `CitizenIdentity::VotingIdentityByAccount` 为准。

### 5.4 钱包详情页布局 `WalletDetailPage`

页面元素（自上而下）：

1. 余额卡片：左上角钱包名称（可点击编辑），居中余额数字+元+GMB
2. 二维码：`QR_V1 kind=user_contact`，`body.address` 为当前钱包 SS58 地址，下载按钮浮在二维码正中间（半透明圆形背景）
3. 冷钱包离线签名入口由 CitizenWallet 承担；CitizenApp 钱包详情页不承载 `QrOfflineSignPage`
4. 地址+复制：地址居中两行显示，复制图标在右侧
5. 交易记录标题行：左侧"交易记录"，右侧箭头，点击进入完整交易记录列表
6. 最近交易记录：最多显示 5 条，显示与完整列表一致的状态标签，点击单条进入交易详情

### 5.5 交易记录数据来源

钱包详情页和交易记录页面直接复用 `LocalTxStore`（Isar `LocalTxEntity`），按 `walletPubkeyHex` 过滤。

- 本机提交普通转账成功后通过 `LocalTxStore.upsertLocalSubmitTransfer()` 写入 `source=local_submit / status=pending` 记录，用于立即反馈支出；如果区块事件已经先写入，则合并手续费、txHash、nonce 和备注，不新增第二条
- 交易池 included 回调先把本机提交记录升级为 `status=inBlock`；newHeads 命中收入或支出事件时写入 `source=chain_event / status=inBlock`；启动、重连和 finalized 后会补扫 finalized 之后的未确认区块，补齐错过实时订阅的 `inBlock` 流水
- finalized 区块事件监听命中后升级同一条区块事件记录为 `status=finalized`，并把匹配的本机提交记录合并为 finalized；该升级只能来自 finalized 高度，不能来自 best/latest 高度
- 钱包详情页展示最近 5 条，点击"交易记录"标题或右侧箭头进入完整列表，点击单条进入该笔交易详情
- `txHash` 只作为本机 pending 提交标识；单条链上流水的唯一定位以 `recordKey` 为准

## 6. 迁移与清理策略

当前 schema：`wallet.data.schema.version = 3`。

开发阶段直接覆盖，不做增量迁移。v3 会清空旧 `LocalTxEntity` 和 `WalletTxSyncCursorEntity`，丢弃此前错误流水和游标；启动时确保 settings 行存在并更新 schema 版本标记。

## 7. 安全边界

- seed 不写入 Isar/Postgres/日志
- **seed 不出 WalletManager**：所有签名操作通过 `signWithWallet()` / `signUtf8WithWallet()` 完成，seed 仅在方法内短暂存在，签名后立即清零
- 助记词不持久化，仅创建时一次性展示
- 冷钱包不在本机保存任何密钥材料
- 本机签名在本地完成，私钥材料不出端
- 授权分层（2026-07-06 定）：`authenticateForSigning()`（生物识别/设备密码）**只**用于「动钱 / 换身份」——转账、充值、提现、清算行绑定、多签、个人账户、投票、以及**切换默认用户钱包**；聊天登录 mailbox、Chat 设备绑定、发帖（按最低链上费用自动扣 0.1 元入块）一律用 `signWithWalletNoAuth()` **静默签名不弹**。发帖弹窗是多余的，已删；发帖前仍做余额校验（够 ED + 0.1 元才发）。
- `signWithWallet()` / `signUtf8WithWallet()` 内含 `_authenticateIfSupported()`（会弹）；`signWithWalletNoAuth()` 只读 seed 不弹。调用方按上条策略选用。seed 读取后做格式校验，异常立即抛错。
- 设备未启用锁屏时硬拒绝访问，不再跳过验证（`isDeviceSupported()` 返回 false 时抛出异常）
- 热钱包创建/导入入口前置设备锁检查（`_ensureDeviceSecure()`），未启用锁屏的设备无法创建或导入热钱包
- seed 读取后进行格式校验（64 位 hex），异常数据立即抛错
- `wallet.secret.*` 与 `wallet.session.*` 统一命名，避免散落硬编码
- `getLatestWalletSecret()` / `getWalletSecretByIndex()` 已标记 `@Deprecated`，新代码禁止使用
- walletIndex 分配与 profile 写入在同一 Isar 事务中完成（`_appendHotWalletAtomic` / `_appendColdWalletAtomic`），防止并发创建/导入时 index 冲突导致密钥覆盖；secure storage 写入在事务成功后执行

## 8. 主要接口（对外）

- `WalletManager`
  - `createWallet / importWallet / importColdWallet`
  - `deleteWallet / setActiveWallet`
  - `signWithWallet(walletIndex, payload)` — 热钱包 sr25519 签名（seed 不出类）
  - `signUtf8WithWallet(walletIndex, message)` — 热钱包 UTF-8 签名（返回 `WalletSignResult`）
  - ~~`getLatestWalletSecret / getWalletSecretByIndex`~~ — 已弃用
- `ChainRpc`（`lib/rpc/chain_rpc.dart`）
  - `fetchFinalizedBalance` / `fetchFinalizedBalances` / `fetchFinalizedTotalBalance` — 直连节点查询 finalized 链上余额
- `ChainTxMonitor`（`lib/rpc/chain_tx_monitor.dart`）
  - 监听 newHeads/finalizedHeads 区块事件，补扫未 finalized 区块，按本机钱包游标增量写入并升级交易流水

## 9. 测试覆盖（当前）

- `test/wallet/wallet_manager_test.dart`
  - 热钱包创建/导入/删除/seed 存储联动
  - 冷钱包导入/删除/无 seed 存储
  - seed key 移除后不再读取
- `test/wallet/seed_derivation_test.dart`
  - 验证 `fromSeed` 与 `fromMnemonic` 产出一致公钥
- `test/wallet/attestation_service_test.dart`
  - attestation token 落 secure storage
  - attestation 元信息落 Isar
- `test/wallet/sign_service_test.dart`
  - 挑战解析、签名、防重放、钱包匹配
- `test/wallet/wallet_manager_reorder_test.dart`
  - `reorderWallets()` 写入 `sortOrder` 后，`getWallets()` 按新顺序返回
  - 旧钱包首次进入时按原 `walletIndex` 顺序初始化 `sortOrder`
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
| `address` | SS58 地址（当前链 `ss58 = 2027`） |
| `pubkeyHex` | 64 hex（不含 `0x` 前缀） |
| `alg` | 固定 `sr25519` |
| `ss58` | 地址格式版本（当前 2027） |
| `source` | `created/imported` |
| `signMode` | `local/external` |

### 10.3 Seed 派生链

```
mnemonic
  → entropy (bip39_mnemonic Mnemonic.fromSentence)
  → PBKDF2 (substrate_bip39 CryptoScheme.miniSecretFromEntropy)
  → 32 字节 mini-secret
  → Keyring.sr25519.fromSeed(miniSecret)
  → sr25519 keypair
```

说明：使用 Substrate 特定的 BIP39 派生（非标准 BIP32），与 `polkadart_keyring` 的 `fromMnemonic` 内部逻辑一致。

## 11. 治理字段联动要求

- 联合提案人口分母由 runtime 按 `PopulationScope` 从链上公民身份读取。
- 链上投票交易只提交账户签名、提案号和赞反意见。
- 钱包模块负责提供签名账户上下文，不负责生成投票资格或人口凭证。
- 钱包模块必须保证"登录签名"和"转账/治理签名"使用不同签名 payload。

## 12. CID 联调约束

- `ApiClient` 的 `baseUrl` 统一来自 `CidApiConfig.defaultBaseUrl`。
- 生产版固定访问 `https://cid.crcfrcn.com`。
- 本地开发版固定访问 `http://127.0.0.1:8899`，必须由 `adb reverse tcp:8899 tcp:8899` 转发到本电脑运行的 OnChina 后端。
- 不允许钱包模块自行读取或拼接 CID API URL，也不允许从本地开发失败自动回退到生产。

## 13. PQC 抗量子签名升级(设计,待实现)

- **真源 = `memory/04-decisions/ADR-022-unified-pqc-crypto.md`**(取代旧 PQC 迁移方案);任务卡 `memory/08-tasks/open/20260618-pqc-card3-wallet-derivation-signing.md`。

热钱包随全系统从 sr25519 **在位升级**到 ML-DSA-65 签名,"四不变"(不换助记词/账户/地址/余额)。以 ADR-022 为准:

- **派生(sr25519 不套 HKDF)**:`§10.3` 的 32B mini-secret = `AccountSeedV1`;sr25519 地址锚点沿用现有 `sr25519.fromSeed(AccountSeedV1)` **直接派生**(不经 HKDF → 地址比特级不变);ML-DSA-65/ML-KEM-768 用 `HKDF-SHA512(AccountSeedV1, "GMB/account/ml-dsa-65/v1" | ".../ml-kem-768/v1")`。ML-DSA keygen/sign 走 Rust FFI(`gmb-pqc`),非 Dart。
- **签名/交易**:无感 bootstrap——未绑定账户首次交易构造 `bootstrap_pqc_dispatch`(sr25519 bootstrap 签名 + ML-DSA 交易签名,一次确认);后续走 `pqc_dispatch` general-tx(`signed_extrinsic_builder.dart:103/186`,**不扩 MultiSignature**)。
- **QR**:`sig_alg(sr25519|ml-dsa-65)` + `auth_mode(normal|pqc|bootstrap-pqc)` + `key_version` + `chunk_index/chunk_total` 分片(ML-DSA ~3.3KB,最坏体积按 bootstrap 实测)。
- **UI**:只展示一个账户/地址/余额,不暴露多公钥/绑定状态机/换账户。
- **安全**:`AccountSeedV1`/PQC 私钥不出本机;CID 不托管。

> 实现以本节 + ADR-022 为准,旧路线不再适用。
