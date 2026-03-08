# Wallet 模块技术文档（当前实现态）

## 1. 模块目标

`lib/wallet` 是钱包能力唯一收口模块，负责：

- 钱包创建/导入/删除/切换
- 本地机密材料读取（助记词）
- 登录签名与链上交易签名
- 钱包相关后端 API 调用
- 管理员目录、观察账户、证明态等钱包周边能力

约束：钱包相关代码只应从 `wallet/...` 引用。

## 2. 目录结构

```text
lib/
├── Isar/
│   ├── wallet_isar.dart
│   └── wallet_isar.g.dart
└── wallet/
    ├── core/
    │   ├── wallet_manager.dart
    │   ├── wallet_secure_keys.dart
    │   ├── user_identification.dart
    │   └── user_identification_settings.dart
    ├── capabilities/
    │   ├── api_client.dart
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
  - 助记词写入 secure storage
  - 钱包元数据写入 Isar
- `Isar/wallet_isar.dart`
  - Isar 集合定义与启动迁移
  - schema 升级与历史键清理（当前 `v4`）
- `wallet_secure_keys.dart`
  - 机密 key 命名规范
- `user_identification.dart`
  - 签名前生物识别守卫
- `user_identification_settings.dart`
  - 生物识别开关持久化（Isar）

### 3.2 `capabilities`

- `sign_service.dart`
  - 登录挑战解析与 `sr25519` 签名
  - 白名单校验、防重放校验
- `api_client.dart`
  - 钱包相关后端接口统一入口
- `wallet_type_service.dart`
  - 管理员目录缓存与角色识别
- `attestation_service.dart`
  - 证明 token（secure）+ 元信息（Isar）
- `sfid_binding_service.dart`
  - SFID 绑定请求状态管理（当前仍用 SharedPreferences）

### 3.3 `ui`

- `wallet_page.dart`
  - 钱包列表、创建、导入、删除、激活、地址复制

## 4. 关键流程

### 4.1 创建钱包

1. 生成 `bip39` 助记词
2. 用 `sr25519` 派生 SS58(2027) 地址与公钥
3. 助记词写入 secure storage
4. 钱包元信息写入 Isar

### 4.2 导入钱包

1. 校验助记词合法性
2. 派生地址/公钥
3. 按新钱包写入 Isar + secure storage
4. 设为当前激活钱包

### 4.3 登录签名

1. 登录模块解析挑战并完成白名单/防重放
2. 钱包模块读取当前钱包助记词
3. `sr25519` 签名并回传回执 payload

### 4.4 链上交易签名（由 trade/onchain 调用）

1. 请求后端 `tx/prepare`
2. 用当前钱包本地签 signer payload
3. 请求后端 `tx/submit`
4. 本地 Isar 记录交易状态并轮询刷新

## 5. 存储设计（当前）

### 5.1 机密层（flutter_secure_storage）

- `wallet.secret.<wallet_id>.mnemonic.v1`
- `wallet.session.<scope>.token.v1`
- `wallet.session.<scope>.key.v1`（预留）

### 5.2 业务层（Isar）

集合定义（`Isar/wallet_isar.dart`）：

- `WalletProfileEntity`
  - `walletIndex, walletName, walletIcon, balance, address, pubkeyHex, alg, ss58, createdAtMillis, source`
- `WalletSettingsEntity`
  - `activeWalletIndex, faceAuthEnabled, updatedAtMillis`
- `TxRecordEntity`
  - `txHash, fromAddress, toAddress, amount, symbol, createdAtMillis, status, failureReason`
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

当前 schema：`wallet.data.schema.version = 4`。

启动迁移行为：

1. 首次安装/升级时将历史 `SharedPreferences` 钱包数据迁移至 Isar
2. 将历史助记词键迁移到 `wallet.secret.<wallet_id>.mnemonic.v1`
3. 清理历史兼容键（`wallet.*`、`attest.*` 旧键）
4. 升级 schema 版本并写入 Isar `AppKvEntity`

说明：

- 第 4 阶段后，运行期不再读取旧兼容键
- 旧键仅由升级迁移任务一次性处理

## 7. 安全边界

- 助记词不写入 Isar/Postgres/日志
- 签名在本地完成，私钥材料不出端
- 登录签名与交易签名前都可触发生物识别守卫
- `wallet.secret.*` 与 `wallet.session.*` 统一命名，避免散落硬编码

## 8. 主要接口（对外）

- `WalletManager`
  - `createWallet / importWallet / deleteWallet / setActiveWallet / getLatestWalletSecret`
- `SignService`
  - `parseChallenge / buildReceiptPayloadForChallenge`
- `UserIdentificationService`
  - `confirmBeforeSign`

## 9. 测试覆盖（当前）

- `test/wallet/wallet_manager_test.dart`
  - 创建/导入/删除/机密存储联动
  - 旧助记词兼容键移除后不再读取
- `test/wallet/attestation_service_test.dart`
  - attestation token 落 secure storage
  - attestation 元信息落 Isar
- `test/wallet/sign_service_test.dart`
  - 挑战解析、签名、防重放、钱包匹配
- `test/wallet/user_identification_settings_test.dart`
  - 生物识别开关持久化
- `test/wallet/user_identification_service_test.dart`
  - 生物识别守卫分支
