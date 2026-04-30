# Wallet 模块技术文档（当前实现态）

## 1. 模块目标

`lib/wallet` 是钱包能力唯一收口模块，负责：

- 钱包创建/导入/删除/切换（热钱包 + 冷钱包）
- 本地机密材料读取（热钱包 seed）
- 登录签名编排（签名执行由 `lib/signer` 负责）
- 转账/提案/投票所需钱包上下文输出（地址、公钥、算法、机构角色）
- 余额查询（通过 `lib/rpc/` 直连链上节点）
- 管理员目录、观察账户、证明态等钱包周边能力

约束：钱包相关代码只应从 `wallet/...` 引用。

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
    │   ├── api_client.dart         ← SFID 绑定、管理员目录等非链上查询
    │   ├── sfid_binding_service.dart
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
  - 开发阶段直接覆盖，schema 版本 `v1`
- `wallet_secure_keys.dart`
  - 机密 key 命名规范（`wallet.secret.<id>.seed_hex.v1`）

### 3.2 `capabilities`

- `api_client.dart`
  - 非链上查询的外部服务接口（SFID 绑定、管理员目录）
- `wallet_type_service.dart`
  - 管理员目录缓存与角色识别
- `attestation_service.dart`
  - 证明 token（secure）+ 元信息（Isar）
- `sfid_binding_service.dart`
  - SFID 绑定请求状态管理（当前仍用 SharedPreferences）

### 3.3 `pages`

- `wallet_page.dart`
  - 钱包列表（带热/冷标识）、长按拖拽排序、创建、导入、删除、激活、地址复制
  - 热钱包创建/导入（`CreateWalletPage` / `ImportWalletPage`）
  - 冷钱包创建/导入（`CreateColdWalletPage` / `ImportColdWalletPage`）
  - 余额显示与刷新（通过 `lib/rpc/ChainRpc.fetchBalance()` 直连节点）
  - 钱包详情页（`WalletDetailPage`）：余额卡片（含钱包名称）、二维码（含下载按钮）、地址、交易记录入口+最近记录
- `transaction_history_page.dart`
  - 交易记录列表页（`TransactionHistoryPage`）：按 walletIndex 过滤，显示转入/转出、金额、状态
  - 交易记录详情页（`TransactionDetailPage`）：txHash、金额、发送方、接收方、时间、状态、备注

### 3.4 `widgets`

- `wallet_identity_card.dart`
  - 钱包身份卡：钱包名、短地址、复制与二维码入口
- `wallet_action_card.dart`
  - 钱包操作卡：充值、提现与清算行余额展示
- `wallet_onchain_balance_card.dart`
  - 链上余额卡：展示链上 total 余额
- `wallet_qr_dialog.dart`
  - 钱包二维码弹窗：生成 `WUMIN_QR_V1 kind=user_contact`

## 4. 关键流程

### 4.1 创建热钱包

1. 生成 `bip39` 助记词
2. 派生 mini-secret：`mnemonic → entropy → PBKDF2(substrate_bip39) → 64 字节 → 前 32 字节`
3. 用 `Keyring.sr25519.fromSeed(miniSecret)` 派生 SS58(2027) 地址与公钥
4. seed（32 字节 hex）写入 secure storage
5. 钱包元信息写入 Isar（`signMode: 'local'`）
6. 助记词一次性展示给用户，不持久化

### 4.2 导入热钱包

1. 校验助记词合法性
2. 派生 seed → 地址/公钥
3. seed 写入 secure storage
4. 钱包元信息写入 Isar（`signMode: 'local'`）
5. 设为当前激活钱包

### 4.3 创建冷钱包

1. 生成 `bip39` 助记词
2. 派生地址/公钥（同热钱包）
3. 仅写 Isar（`signMode: 'external'`），不写 secure storage
4. 助记词一次性展示，强警告用户自行保管

### 4.4 导入冷钱包

1. 接受 SS58 地址
2. 解码公钥（`Keyring().decodeAddress()`）
3. 仅写 Isar（`signMode: 'external'`），不写 secure storage

### 4.5 余额查询

1. 页面 `initState` 和下拉刷新触发 `_refreshBalancesFromChain()`
2. 一次收集所有本地钱包公钥，调用 `ChainRpc.fetchBalances(pubkeys)` 批量读取 `System.Account`
3. 轻节点先等待同步完成；若轻节点未初始化、同步失败或链路降级，直接向上抛出真实错误
4. 批量解码 SCALE 编码的 `AccountInfo.free` 余额（分），转换为元
5. 若余额有变化，更新 Isar 中的 `WalletProfileEntity.balance`
6. 刷新 UI 显示；若轻节点不可用，则页面显示统一提示，而不是把失败误判为 0 余额

### 4.5.1 钱包卡片拖拽排序

1. `MyWalletPage` 使用 `ReorderableListView` 承载钱包卡片，长按拖拽触发 `_onReorder(oldIndex, newIndex)`。
2. `WalletManager.getWallets()` 返回 fixed-length list，UI 层不能直接对 `_wallets` 执行 `removeAt/insert`。
3. UI 层统一通过 `reorderWalletProfiles()` 先复制成可变列表，再按 Flutter `onReorder` 规则修正目标下标。
4. 页面先 `setState` 展示新顺序，再调用 `WalletManager.reorderWallets()` 把 walletIndex 顺序写入 Isar `sortOrder`。
5. `getWallets()` 查询时按 `sortOrder` 升序返回，相同值再用 `walletIndex` 兜底，保证重启后顺序稳定。

### 4.6 登录签名

- **热钱包**：`LoginService` 解析挑战 → `LoginSystemSignatureVerifier` 验证系统签名 → `WalletManager.signUtf8WithWallet()` 完成 sr25519 签名（seed 不出 `WalletManager`）
- **冷钱包**：
  1. `LoginService.buildExternalSignRequest()` 将登录签名原文包装为 `QrSignRequest`
  2. 在线手机导航到 `QrSignSessionPage` 展示请求二维码
  3. 离线设备进入 `QrOfflineSignPage` 扫描请求，通过 `OfflineSignService` 交叉验证 display 与 payload 后调用本机热钱包签名
  4. 在线手机扫描回执后，`LoginService.buildReceiptFromSignature()` 校验签名（含 `payload_hash`）并生成登录回执

### 4.7 链上支付签名（由 onchain 调用）

- **热钱包**：`WalletManager.signWithWallet()` 签名回调注入 `OnchainPaymentService`（seed 不出 WalletManager）；签名前必须重新派生本地公钥，并校验其与当前 `WalletProfile.pubkeyHex` 完全一致，不一致直接拒绝签名
- **冷钱包**：构造 `QrSignRequest`（含 `display` 字段）→ 导航到 `QrSignSessionPage` → 展示请求二维码 → 用户用离线设备扫码签名（离线端通过 `PayloadDecoder` 独立解码 payload 并与 display 交叉验证）→ 扫描回执二维码 → `QrSigner.parseResponse()` 校验 `request_id + pubkey + payload_hash` → 签名回调注入

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
   - 冷钱包：扫码回执中的 `pubkey` 必须等于页面选中的 `pubkeyHex`
6. 联合提案（如 Runtime 升级）还要求：
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
- `TxRecordEntity`
  - `txHash, fromAddress, toAddress, amount, symbol, createdAtMillis, status, failureReason, usedNonce`
- `AdminRoleCacheEntity`
  - `pubkeyHex, roleName, updatedAt`
- `ObservedAccountEntity`
  - `accountId, orgName, publicKey, address, balance, source`
- `LoginReplayEntity`
  - `requestId, expiresAt`
- `AppKvEntity`
  - `key, stringValue, intValue, boolValue`

### 5.3 其他 SharedPreferences（尚未迁移）

- `sfid.bind.*`（`SfidBindingService`）

### 5.4 钱包详情页布局 `WalletDetailPage`

页面元素（自上而下）：

1. 余额卡片：左上角钱包名称（可点击编辑），居中余额数字+元+GMB
2. 二维码：`gmb://account/{address}`，下载按钮浮在二维码正中间（半透明圆形背景）
3. 热钱包额外显示“离线签名”按钮，进入 `QrOfflineSignPage`
4. 地址+复制：地址居中两行显示，复制图标在右侧
5. 交易记录标题行：左侧"交易记录"，右侧箭头，点击进入完整交易记录列表
6. 最近交易记录：最多显示 5 条，点击进入交易详情

### 5.5 交易记录数据来源

钱包详情页和交易记录页面直接复用 `LocalTxStore`（Isar `LocalTxEntity`），按钱包地址过滤。

- 数据在链上支付页面 `OnchainPaymentService.submitTransfer()` 成功时自动写入 Isar
- 钱包详情页展示最近 5 条，点击"交易记录"进入完整列表
- 状态同步（pending→confirmed）由 `PendingTxReconciler` 通过 nonce 推进完成

## 6. 迁移与清理策略

当前 schema：`wallet.data.schema.version = 1`。

开发阶段直接覆盖，不做增量迁移。启动时仅确保 settings 行存在并更新 schema 版本标记。

## 7. 安全边界

- seed 不写入 Isar/Postgres/日志
- **seed 不出 WalletManager**：所有签名操作通过 `signWithWallet()` / `signUtf8WithWallet()` 完成，seed 仅在方法内短暂存在，签名后立即清零
- 助记词不持久化，仅创建时一次性展示
- 冷钱包不在本机保存任何密钥材料
- 本机签名在本地完成，私钥材料不出端
- seed 读取前强制生物识别/设备密码验证（`_authenticateIfSupported()`），每次签名均需认证
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
- `LoginService`（`lib/qr/login/login_service.dart`，通过 `sign_service.dart` re-export）
  - `parseChallenge / buildReceiptPayload / buildReceiptFromSignature`
- `ChainRpc`（`lib/rpc/chain_rpc.dart`）
  - `fetchBalance` — 直连节点查询链上余额

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

- 联合提案必须包含 `eligible_total/snapshot_nonce/signature` 三元组。
- 公民投票必须包含 `binding_id/nonce/signature` 三元组。
- 钱包模块负责提供签名账户上下文，不负责生成 SFID 凭证签名。
- 钱包模块必须保证"登录签名"和"转账/治理签名"使用不同签名 payload。

## 12. 本地 SFID 联调约束

- `ApiClient` 的 `baseUrl` 优先读取 `WUMINAPP_API_BASE_URL`。
- 手机真机联调时，`WUMINAPP_API_BASE_URL` 必须填写手机可访问的 `sfid` 地址，不能使用 `127.0.0.1`。
- `wuminapp/scripts/app-run.sh` 与 `wuminapp/scripts/app-clean-run.sh` 会优先读取 `sfid/.env.dev.local` 的 `SFID_PUBLIC_BASE_URL`，用于手机访问；只有缺失时才回退到 `SFID_BIND_ADDR`。
