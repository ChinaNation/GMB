# Wallet 模块技术文档（当前实现态）

## 1. 模块目标

`lib/wallet` 是钱包能力唯一收口模块，负责：

- 钱包创建/导入/删除/切换（热钱包 + 冷钱包）
- 本地机密材料读取（热钱包 seed）
- 热钱包私钥签名的唯一执行入口
- 登录/转账/治理所需钱包上下文输出
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
    ├── core/
    │   ├── wallet_manager.dart         ← 钱包生命周期 + seed 读取守卫
    │   └── wallet_secure_keys.dart
    ├── capabilities/
    │   ├── api_client.dart         ← SFID 绑定、管理员目录等非链上查询
    │   ├── sign_service.dart
    │   ├── sfid_binding_service.dart
    │   ├── attestation_service.dart
    │   └── wallet_type_service.dart
    ├── ui/
    │   ├── wallet_page.dart
    │   └── transaction_history_page.dart
    ├── wallet.dart
    └── WALLET_TECHNICAL.md
```

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

- `sign_service.dart`
  - 兼容层，re-export `qr/login/` 模块（`LoginService`、`LoginChallenge`、`LoginReplayGuard`）
  - 实际登录编排逻辑已迁移到 `lib/qr/login/login_service.dart`
- `api_client.dart`
  - 非链上查询的外部服务接口（SFID 绑定、管理员目录）
- `wallet_type_service.dart`
  - 管理员目录缓存与角色识别
- `attestation_service.dart`
  - 证明 token（secure）+ 元信息（Isar）
- `sfid_binding_service.dart`
  - SFID 绑定请求状态管理（当前仍用 SharedPreferences）

### 3.3 `ui`

- `wallet_page.dart`
  - 钱包列表（带热/冷标识）、创建、导入、删除、激活、地址复制
  - 热钱包创建/导入（`CreateWalletPage` / `ImportWalletPage`）
  - 冷钱包创建/导入（`CreateColdWalletPage` / `ImportColdWalletPage`）
  - 余额显示与刷新（通过 `lib/rpc/ChainRpc.fetchBalance()` 直连节点）
  - 钱包详情页（`WalletDetailPage`）：余额卡片（含钱包名称）、二维码（含下载按钮）、地址、交易记录入口+最近记录
- `transaction_history_page.dart`
  - 交易记录列表页（`TransactionHistoryPage`）：按 walletIndex 过滤，显示转入/转出、金额、状态
  - 交易记录详情页（`TransactionDetailPage`）：txHash、金额、发送方、接收方、时间、状态、备注

## 4. 唯一签名入口原则（冻结）

- **本机私钥签名只能有一处实现**：`WalletManager`
- 所有热钱包签名统一使用：
  - `signWithWallet(walletIndex, payload)`
  - `signUtf8WithWallet(walletIndex, message)`
- 业务模块不得：
  - 直接读取 seed
  - 直接 `Keyring.sr25519.fromSeed(...)`
  - 自己实现第二套热钱包签名逻辑
- 冷钱包不属于“本机私钥签名”，但也必须统一走 `QrSigner` 协议
- 最终目标：业务层只依赖统一签名编排入口，不直接依赖热/冷签名细节

## 5. 关键流程

### 5.1 创建热钱包

1. 生成 `bip39` 助记词
2. 派生 mini-secret：`mnemonic → entropy → PBKDF2(substrate_bip39) → 64 字节 → 前 32 字节`
3. 用 `Keyring.sr25519.fromSeed(miniSecret)` 派生 SS58(2027) 地址与公钥
4. seed（32 字节 hex）写入 secure storage
5. 钱包元信息写入 Isar（`signMode: 'local'`）
6. 助记词一次性展示给用户，不持久化

### 5.2 导入热钱包

1. 校验助记词合法性
2. 派生 seed → 地址/公钥
3. seed 写入 secure storage
4. 钱包元信息写入 Isar（`signMode: 'local'`）
5. 设为当前激活钱包

### 5.3 创建冷钱包

1. 生成 `bip39` 助记词
2. 派生地址/公钥（同热钱包）
3. 仅写 Isar（`signMode: 'external'`），不写 secure storage
4. 助记词一次性展示，强警告用户自行保管

### 5.4 导入冷钱包

1. 接受 SS58 地址
2. 解码公钥（`Keyring().decodeAddress()`）
3. 仅写 Isar（`signMode: 'external'`），不写 secure storage

### 5.5 余额查询

1. 页面 `initState` 和下拉刷新触发 `_refreshBalancesFromChain()`
2. 遍历所有本地钱包，对每个钱包：
   - 调用 `ChainRpc.fetchBalance(wallet.pubkeyHex)` 直连链上节点
   - RPC 方法：`state_getStorage`（`System.Account` storage key）
   - 解码 SCALE 编码的 `AccountInfo`，提取 `free` 余额（分），转换为元
3. 若余额有变化，更新 Isar 中的 `WalletProfileEntity.balance`
4. 刷新 UI 显示

### 5.6 登录签名

当前实现：

- **热钱包**：`LoginService` 解析挑战 → `WalletManager.signUtf8WithWallet()` 完成 sr25519 签名（seed 不出 WalletManager）
- **冷钱包**：`LoginService.buildReceiptFromSignature()` 已提供回执收口能力，但主流程尚未完全统一到冷签路径

目标改造：

1. `LoginService` 只负责：
   - 解析挑战
   - 系统身份验证
   - 生成登录待签名原文
   - 收口登录回执
2. 热/冷签名统一改由 `SigningCoordinator` 编排：
   - `local` → `WalletManager.signUtf8WithWallet()`
   - `external` → `QrSigner` 请求/回执会话
3. 登录模块不再把热钱包路径写死在主流程中

### 5.7 链上交易签名（由 trade/onchain 调用）

当前实现：

- **热钱包**：`WalletManager.signWithWallet()` 签名回调注入 `OnchainTradeService`
- **冷钱包**：UI 页面构造 `QrSignRequest` → 导航到 `QrSignSessionPage` → 扫描回执 → 注入签名回调

目标改造：

- `OnchainTradeService.submitTransfer()` 继续只接受“签名回调/签名结果”
- UI 页面不再自己维护热/冷两套细节
- 统一由 `SigningCoordinator` 按 `signMode` 分流：
  - `local`：委托 `WalletManager`
  - `external`：委托 `QrSigner`

### 5.8 治理提案/投票签名（由 governance + signer 调用，规划）

1. 治理模块按业务类型组装提案/投票字段。
2. 钱包模块输出当前激活钱包上下文（`address/pubkeyHex/alg/ss58`）。
3. 目标改造后统一由 `SigningCoordinator` 根据 `signMode` 分流：
   - `local`：`WalletManager.signWithWallet()`（seed 不出类）。
   - `external`：调用 `QrSigner` 发起外部签名会话。
4. 回传签名结果给治理模块提交链上交易。

## 6. 冷热一体化目标架构

目标新增组件：

- `lib/signer/signing_coordinator.dart`
  - 业务层唯一签名编排入口
- `lib/signer/signature_verifier.dart`
  - 统一公钥验签能力
- `lib/qr/login/login_trust_service.dart`
  - 登录信任链验证（链上 SFID 公钥、CPMS 背书）

目标分工：

- `WalletManager`
  - 唯一热钱包私钥签名实现
- `QrSigner`
  - 唯一冷钱包签名协议实现
- `SigningCoordinator`
  - 统一对业务暴露 `signUtf8/signBytes`
- `LoginService`
  - 不直接关心热签/冷签，只负责编排登录协议

## 7. 存储设计（当前）

### 7.1 机密层（flutter_secure_storage）

- `wallet.secret.<wallet_id>.seed_hex.v1` — 热钱包 32 字节 seed（hex 编码）
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 7.2 业务层（Isar）

集合定义（`Isar/wallet_isar.dart`）：

- `WalletProfileEntity`
  - `walletIndex, walletName, walletIcon, balance, address, pubkeyHex, alg, ss58, createdAtMillis, source, signMode`
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

### 7.3 其他 SharedPreferences（尚未迁移）

- `sfid.bind.*`（`SfidBindingService`）

### 7.4 钱包详情页布局 `WalletDetailPage`

页面元素（自上而下）：

1. 余额卡片：左上角钱包名称（可点击编辑），居中余额数字+元+GMB
2. 二维码：`gmb://account/{address}`，下载按钮浮在二维码正中间（半透明圆形背景）
3. 地址+复制：地址居中两行显示，复制图标在右侧
4. 交易记录标题行：左侧"交易记录"，右侧箭头，点击进入完整交易记录列表
5. 最近交易记录：最多显示 5 条，点击进入交易详情

### 7.5 交易记录数据来源

钱包详情页和交易记录页面直接复用 `OnchainTradeRepository`（Isar `TxRecordEntity`），按钱包地址（fromAddress / toAddress）过滤。

- 数据在交易页面 `OnchainTradeService.submitTransfer()` 成功时自动写入 Isar
- 钱包详情页展示最近 5 条，点击"交易记录"进入完整列表
- 状态同步（pending→confirmed）由交易页面定时轮询 `refreshPendingRecords()` 完成

## 8. 迁移与清理策略

当前 schema：`wallet.data.schema.version = 1`。

开发阶段直接覆盖，不做增量迁移。启动时仅确保 settings 行存在并更新 schema 版本标记。

## 9. 安全边界

- seed 不写入 Isar/Postgres/日志
- **seed 不出 WalletManager**：所有签名操作通过 `signWithWallet()` / `signUtf8WithWallet()` 完成，seed 仅在方法内短暂存在，签名后立即清零
- 助记词不持久化，仅创建时一次性展示
- 冷钱包不在本机保存任何密钥材料
- 本机签名在本地完成，私钥材料不出端
- 业务模块不得直接调用 `LocalSigner` 执行热钱包签名
- 冷钱包扫码签名必须统一走 `QrSigner` 协议，不允许业务页面自行发明第二套协议
- seed 读取前强制生物识别/设备密码验证（`_authenticateIfSupported()`），每次签名均需认证
- 设备无生物识别也无密码时自动跳过验证（`isDeviceSupported()` 返回 false）
- seed 读取后进行格式校验（64 位 hex），异常数据立即抛错
- `wallet.secret.*` 与 `wallet.session.*` 统一命名，避免散落硬编码
- `getLatestWalletSecret()` / `getWalletSecretByIndex()` 已标记 `@Deprecated`，新代码禁止使用

说明：公钥验签（如登录系统验签、CPMS 背书验签）不属于“本机私钥签名”，可以独立于 `WalletManager` 存在，但不得演化出新的热钱包私钥签名入口。

## 10. 主要接口（对外）

- `WalletManager`
  - `createWallet / importWallet / createColdWallet / importColdWallet`
  - `deleteWallet / setActiveWallet`
  - `signWithWallet(walletIndex, payload)` — 热钱包 sr25519 签名（seed 不出类）
  - `signUtf8WithWallet(walletIndex, message)` — 热钱包 UTF-8 签名（返回 `WalletSignResult`）
  - ~~`getLatestWalletSecret / getWalletSecretByIndex`~~ — 已弃用
- `SigningCoordinator`（目标新增）
  - `signUtf8(...)` — 冷热统一字符串签名入口
  - `signBytes(...)` — 冷热统一字节签名入口
- `LoginService`（`lib/qr/login/login_service.dart`，通过 `sign_service.dart` re-export）
  - `parseChallenge / buildReceiptPayload / buildReceiptFromSignature`
- `ChainRpc`（`lib/rpc/chain_rpc.dart`）
  - `fetchBalance` — 直连节点查询链上余额
  - `fetchCurrentSfidVerifyPubkey`（目标新增）— 获取链上当前 SFID 验签公钥

## 11. 测试覆盖（当前）

- `test/wallet/wallet_manager_test.dart`
  - 热钱包创建/导入/删除/seed 存储联动
  - 冷钱包创建/导入/无 seed 存储
  - seed key 移除后不再读取
- `test/wallet/seed_derivation_test.dart`
  - 验证 `fromSeed` 与 `fromMnemonic` 产出一致公钥
- `test/wallet/attestation_service_test.dart`
  - attestation token 落 secure storage
  - attestation 元信息落 Isar
- `test/wallet/sign_service_test.dart`
  - 挑战解析、签名、防重放、钱包匹配

## 12. 钱包模式规范

### 12.1 模式定义

- `signMode: 'local'`（热钱包 — 本机签名）
  - seed 保存在手机 secure storage
  - 转账、登录、提案、投票的热签都只能由 `WalletManager` 执行
- `signMode: 'external'`（冷钱包 — 扫码签名）
  - 手机不保存私钥，仅保存钱包公开信息
  - 转账、登录、提案、投票均通过 `QrSigner` 协议请求外部设备签名

### 12.2 最小钱包上下文字段

| 字段 | 说明 |
| --- | --- |
| `address` | SS58 地址（当前链 `ss58 = 2027`） |
| `pubkeyHex` | 64 hex（不含 `0x` 前缀） |
| `alg` | 固定 `sr25519` |
| `ss58` | 地址格式版本（当前 2027） |
| `source` | `created/imported` |
| `signMode` | `local/external` |

### 12.3 Seed 派生链

```
mnemonic
  → entropy (bip39_mnemonic Mnemonic.fromSentence)
  → PBKDF2 (substrate_bip39 CryptoScheme.miniSecretFromEntropy)
  → 32 字节 mini-secret
  → Keyring.sr25519.fromSeed(miniSecret)
  → sr25519 keypair
```

说明：使用 Substrate 特定的 BIP39 派生（非标准 BIP32），与 `polkadart_keyring` 的 `fromMnemonic` 内部逻辑一致。

## 13. 治理字段联动要求

- 联合提案必须包含 `eligible_total/snapshot_nonce/snapshot_signature` 三元组。
- 公民投票必须包含 `sfid_hash/nonce/signature` 三元组。
- 钱包模块负责提供签名账户上下文，不负责生成 SFID 凭证签名。
- 钱包模块必须保证"登录签名"和"转账/治理签名"使用不同签名 payload。
