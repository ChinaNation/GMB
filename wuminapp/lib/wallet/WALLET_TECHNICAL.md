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
    │   └── wallet_page.dart
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
2. 遍历所有本地钱包，对每个钱包：
   - 调用 `ChainRpc.fetchBalance(wallet.pubkeyHex)` 直连链上节点
   - RPC 方法：`state_getStorage`（`System.Account` storage key）
   - 解码 SCALE 编码的 `AccountInfo`，提取 `free` 余额（分），转换为元
3. 若余额有变化，更新 Isar 中的 `WalletProfileEntity.balance`
4. 刷新 UI 显示

### 4.6 登录签名

- **热钱包**：`LoginService` 解析挑战 → `WalletManager.signUtf8WithWallet()` 完成 sr25519 签名（seed 不出 WalletManager）
- **冷钱包**：`LoginService.buildReceiptFromSignature()` 接受外部签名结果构建回执

### 4.7 链上交易签名（由 trade/onchain 调用）

- **热钱包**：`WalletManager.signWithWallet()` 签名回调注入 `OnchainTradeService`（seed 不出 WalletManager）
- **冷钱包**：构造 `QrSignRequest` → 导航到 `QrSignSessionPage` → 展示请求二维码 → 用户用离线设备扫码签名 → 扫描回执二维码 → `QrSigner.parseResponse()` → 签名回调注入

`OnchainTradeService.submitTransfer()` 接受 `sign` 回调参数，由 UI 层根据 `signMode` 提供不同实现。

### 4.8 治理提案/投票签名（由 governance + signer 调用，规划）

1. 治理模块按业务类型组装提案/投票字段。
2. 钱包模块输出当前激活钱包上下文（`address/pubkeyHex/alg/ss58`）。
3. 根据 `signMode` 分流：
   - `local`：`WalletManager.signWithWallet()`（seed 不出类）。
   - `external`：调用 `QrSigner` 发起外部签名会话。
4. 回传签名结果给治理模块提交链上交易。

## 5. 存储设计（当前）

### 5.1 机密层（flutter_secure_storage）

- `wallet.secret.<wallet_id>.seed_hex.v1` — 热钱包 32 字节 seed（hex 编码）
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 5.2 业务层（Isar）

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

### 5.3 其他 SharedPreferences（尚未迁移）

- `sfid.bind.*`（`SfidBindingService`）

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
- 设备无生物识别也无密码时自动跳过验证（`isDeviceSupported()` 返回 false）
- seed 读取后进行格式校验（64 位 hex），异常数据立即抛错
- `wallet.secret.*` 与 `wallet.session.*` 统一命名，避免散落硬编码
- `getLatestWalletSecret()` / `getWalletSecretByIndex()` 已标记 `@Deprecated`，新代码禁止使用

## 8. 主要接口（对外）

- `WalletManager`
  - `createWallet / importWallet / createColdWallet / importColdWallet`
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
  - 冷钱包创建/导入/无 seed 存储
  - seed key 移除后不再读取
- `test/wallet/seed_derivation_test.dart`
  - 验证 `fromSeed` 与 `fromMnemonic` 产出一致公钥
- `test/wallet/attestation_service_test.dart`
  - attestation token 落 secure storage
  - attestation 元信息落 Isar
- `test/wallet/sign_service_test.dart`
  - 挑战解析、签名、防重放、钱包匹配

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

- 联合提案必须包含 `eligible_total/snapshot_nonce/snapshot_signature` 三元组。
- 公民投票必须包含 `sfid_hash/nonce/signature` 三元组。
- 钱包模块负责提供签名账户上下文，不负责生成 SFID 凭证签名。
- 钱包模块必须保证"登录签名"和"转账/治理签名"使用不同签名 payload。
