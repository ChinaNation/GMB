# Wallet 模块技术文档（当前实现态）

## 1. 模块目标

`lib/wallet` 是钱包能力唯一收口模块，负责：

- 钱包创建/导入/删除/切换
- 本地机密材料读取（助记词）
- 登录签名编排（签名执行由 `lib/signer` 负责）
- 转账/提案/投票所需钱包上下文输出（地址、公钥、算法、机构角色）
- 钱包相关后端 API 调用
- 管理员目录、观察账户、证明态等钱包周边能力

约束：钱包相关代码只应从 `wallet/...` 引用。

## 2. 目录结构

```text
lib/
├── Isar/
│   ├── wallet_isar.dart
│   └── wallet_isar.g.dart
├── signer/
│   ├── local_signer.dart
│   ├── qr_signer.dart
│   └── SIGNER_TECHNICAL.md
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
  - 登录挑战解析与回执编排
  - 复用 `LocalSigner` 执行 `sr25519` 签名
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
3. 调用 `LocalSigner` 完成 `sr25519` 签名并回传回执 payload

### 4.4 链上交易签名（由 trade/onchain + signer 调用）

1. 请求后端 `tx/prepare`
2. 读取当前钱包密钥材料
3. `LocalSigner` 签 signer payload
4. 请求后端 `tx/submit`
5. 本地 Isar 记录交易状态并轮询刷新

### 4.5 治理提案/投票签名（由 governance + signer 调用，规划）

1. 治理模块按业务类型组装提案/投票字段。
2. 钱包模块输出当前激活钱包上下文（`address/pubkeyHex/alg/ss58`）。
3. 根据签名模式分流：
   - 本机模式：读取助记词，调用 `LocalSigner`。
   - 扫码模式：不读取助记词，调用 `QrSigner` 发起外部签名会话。
4. 回传签名结果给治理模块提交链上交易。

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
- 钱包模块不直接实现签名算法，统一走 `lib/signer`
- 本机签名在本地完成，私钥材料不出端
- 登录签名与交易签名前都可触发生物识别守卫
- `wallet.secret.*` 与 `wallet.session.*` 统一命名，避免散落硬编码

## 8. 主要接口（对外）

- `WalletManager`
  - `createWallet / importWallet / deleteWallet / setActiveWallet / getLatestWalletSecret`
- `SignService`
  - `parseChallenge / buildReceiptPayloadForChallenge`
- `LocalSigner`（`lib/signer/local_signer.dart`）
  - `signUtf8 / signBytes`
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

## 10. 钱包模式规范（转账 / 提案 / 投票）

### 10.1 模式定义

- 模式 A：`local`（本机签名）
  - 私钥/助记词保存在手机 secure storage。
  - 转账、提案、投票均可直接在手机签名。
- 模式 B：`external`（扫码签名）
  - 手机不保存私钥，仅保存钱包公开信息。
  - 转账、提案、投票均通过扫码请求外部设备签名。

### 10.2 最小钱包上下文字段

| 字段 | 说明 |
| --- | --- |
| `address` | SS58 地址（当前链 `ss58 = 2027`） |
| `pubkeyHex` | `0x` + 64 hex |
| `alg` | 固定 `sr25519` |
| `ss58` | 地址格式版本（当前 2027） |
| `source` | `created/imported` |

### 10.3 规划中的元数据扩展

为支持“本机签名 + 扫码签名”长期并存，建议在钱包元数据新增：

- `signMode: local | external`
- `externalSignerHint`（可选，外部签名设备标识）
- `signCapabilities`（可选，声明支持 `onchain_tx/gov_proposal/gov_vote`）

说明：当前 schema 为 `v4`，尚未落以上字段；治理模块接入前应先完成 schema 升级。

## 11. 治理字段联动要求

- 联合提案必须包含 `eligible_total/snapshot_nonce/snapshot_signature` 三元组。
- 公民投票必须包含 `sfid_hash/nonce/signature` 三元组。
- 钱包模块负责提供签名账户上下文，不负责生成 SFID 凭证签名。
- 钱包模块必须保证“登录签名”和“转账/治理签名”使用不同签名 payload。
