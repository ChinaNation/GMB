# Wallet 模块技术文档

## 1. 目标

`mobile/lib/wallet` 是钱包能力收口模块，统一承载以下能力：

- 钱包核心：创建/导入/删除/切换、地址与公钥派生、密钥材料读取
- 签名能力：登录签名、链上交易签名、签名前身份校验
- 钱包相关业务能力：余额查询、链上交易提交、SFID 绑定、证明态管理
- 钱包 UI：钱包列表、详情、创建/导入流程页

约束：全项目钱包相关代码只从 `wallet/...` 引用，不再走旧路径。

## 2. 目录结构

```text
wallet/
├── core/
│   ├── wallet_manager.dart
│   ├── user_identification.dart
│   └── user_identification_settings.dart
├── capabilities/
│   ├── api_client.dart
│   ├── sign_service.dart
│   ├── onchain_trade_models.dart
│   ├── onchain_trade_repository.dart
│   ├── onchain_trade_gateway.dart
│   ├── onchain_trade_service.dart
│   ├── sfid_binding_service.dart
│   ├── attestation_service.dart
│   └── wallet_type_service.dart
├── ui/
│   └── wallet_page.dart
├── wallet.dart
└── WALLET_TECHNICAL.md
```

## 3. 分层职责

### 3.1 core

- `wallet_manager.dart`
  - 核心钱包管理器，统一钱包生命周期
  - 助记词写入 `flutter_secure_storage`
  - 钱包元信息写入 `SharedPreferences`
  - 兼容旧单钱包数据迁移到多钱包结构
- `user_identification.dart`
  - 所有签名前置校验入口（生物识别）
  - 对外暴露 `confirmBeforeSign()`
- `user_identification_settings.dart`
  - 生物识别开关持久化（`settings.face_auth_enabled`）

### 3.2 capabilities

- `sign_service.dart`
  - 统一登录挑战签名（`WUMINAPP_LOGIN_V1`）
  - 校验协议字段、白名单、防重放
- `onchain_trade_*`
  - 模型定义、仓储、网关、服务四层拆分
  - `onchain_trade_service.dart` 执行交易签名与提交编排
- `api_client.dart`
  - 钱包能力相关 HTTP API 统一出口
  - `prepareTx/submitTx/fetchTxStatus/fetchWalletBalance/requestChainBindByPubkey`
- `sfid_binding_service.dart`
  - SFID 绑定流程状态管理与请求发起
- `attestation_service.dart`
  - 钱包证明态（token/challenge）占位能力
- `wallet_type_service.dart`
  - 动态管理员目录能力（经 backend 从链上 `CurrentAdmins` 拉取后映射）
  - 本地仅缓存公钥到角色的小表，不再硬编码大映射

### 3.3 ui

- `wallet_page.dart`
  - 钱包列表、详情编辑、创建、导入、删除、激活
  - 余额刷新、地址复制、扫码登录入口

## 4. 关键流程

### 4.1 创建钱包

1. `WalletManager.createWallet()`
2. 生成助记词（`bip39`）
3. 使用 `sr25519` 派生地址与公钥
4. 元数据入 `SharedPreferences`，助记词入 `flutter_secure_storage`

### 4.2 导入钱包

1. `WalletManager.importWallet(mnemonic)`
2. 校验助记词合法性
3. 派生地址/公钥并落库
4. 激活为当前钱包

### 4.3 登录签名

1. 扫码得到挑战 JSON
2. `SignService.parseChallenge()` 校验协议与时效
3. `UserIdentificationService.confirmBeforeSign()` 做生物识别守卫
4. `SignService.buildReceiptPayloadForChallenge()` 读取当前钱包并签名

### 4.4 链上交易签名

1. `OnchainTradeService.submitTransfer(draft)`
2. 读取当前钱包密钥材料并校验公钥一致性
3. 调用 `OnchainTradeGateway.prepareTransfer()`
4. 本地签名 signer payload
5. 调用 `OnchainTradeGateway.submitTransfer()` 广播

## 5. 存储与安全边界

- 私钥材料（助记词）：
  - 仅在手机端存储与使用
  - 存储在 `flutter_secure_storage`
- 钱包元数据（地址、公钥、显示信息、激活索引）：
  - 存储在 `SharedPreferences`
- 生物识别策略：
  - 开关配置由 `UserIdentificationSettings` 管理
  - 开启后，所有签名操作前必须通过本地身份校验

## 6. 对外使用规范

- 新代码优先使用 barrel：`package:wuminapp_mobile/wallet/wallet.dart`
- 禁止新增对以下旧路径的引用：
  - `services/wallet_service.dart`
  - `services/app_settings_service.dart`
  - `services/api_client.dart`
  - `services/sfid_binding_service.dart`
  - `trade/onchain/...`
  - `login/services/wuminapp_login_service.dart`
  - `login/services/login_sign_confirm_service.dart`

## 7. 测试矩阵（当前）

- `test/wallet/wallet_manager_test.dart`
  - 创建/导入/删除/密钥存储联动
- `test/wallet/sign_service_test.dart`
  - 挑战解析、签名回执、防重放、钱包选择
- `test/wallet/user_identification_settings_test.dart`
  - 生物识别开关默认值与持久化
- `test/wallet/user_identification_service_test.dart`
  - 生物识别守卫分支（开关关闭直通/无生物识别报错）

## 8. 迁移完成清单

- 已删除旧兼容文件，钱包相关能力收口到 `wallet/`
- 已删除旧 `trade/onchain` 目录下钱包交易实现并迁移到 `wallet/capabilities`
- 已统一核心命名：
  - `WalletManager`
  - `SignService`
  - `UserIdentificationService`

## 9. 本地数据分层与表设计（机密层 + 业务层）

### 9.1 设计目标

- 机密数据与业务数据彻底分层。
- 签名链路只从机密层取密钥材料，不在业务层落任何私钥/助记词明文。
- 业务层支持高频查询、筛选、排序、分页、离线缓存。

### 9.2 分层定义

#### 9.2.1 机密层（Keychain/Keystore，经 flutter_secure_storage）

用途：只存助记词、私钥、会话密钥等高敏感数据。

建议 Key 命名：

| key | value | 说明 |
|---|---|---|
| `wallet.secret.<wallet_id>.mnemonic.v1` | 助记词密文 | 主密钥材料（必须） |
| `wallet.secret.<wallet_id>.seed.v1` | seed 密文（可选） | 仅当业务需要缓存 seed 时启用 |
| `wallet.secret.<wallet_id>.sr25519.v1` | 私钥密文（可选） | 默认不建议落私钥，优先由助记词派生 |
| `wallet.session.<scope>.token.v1` | 会话 token 密文 | 登录会话、链路短期凭据 |
| `wallet.session.<scope>.key.v1` | 会话密钥密文 | 临时签名密钥/握手密钥 |

约束：

- `wallet_id` 由业务层生成并全局唯一（例如 UUID）。
- 删除钱包时，必须同步删除该 `wallet_id` 下所有机密 key。
- 机密层不支持查询；查询全部走业务层索引字段。

#### 9.2.2 业务层（Isar）

用途：存可查询数据（钱包列表、地址、公钥、标签、交易记录、状态、缓存等）。

### 9.3 Isar 集合设计

#### 9.3.1 `wallet_profiles`

用途：钱包主表。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | Isar 主键（自增） |
| `walletId` | `String` | 业务主键，关联机密层 |
| `name` | `String` | 钱包名称 |
| `icon` | `String` | 钱包图标 |
| `address` | `String` | SS58 地址 |
| `pubkeyHex` | `String` | 32 字节公钥 hex（无 0x） |
| `alg` | `String` | `sr25519` |
| `ss58` | `int` | 地址格式，当前 2027 |
| `source` | `String` | `created/imported/observed` |
| `isActive` | `bool` | 是否当前激活钱包 |
| `isBackedUp` | `bool` | 是否完成助记词备份确认 |
| `createdAt` | `DateTime` | 创建时间 |
| `updatedAt` | `DateTime` | 更新时间 |
| `deletedAt` | `DateTime?` | 软删除时间（可选） |

索引建议：

- 唯一：`walletId`、`address`、`pubkeyHex`
- 普通：`isActive`、`updatedAt`

#### 9.3.2 `wallet_labels`

用途：标签定义。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | 主键 |
| `labelCode` | `String` | 唯一编码 |
| `labelName` | `String` | 展示名称 |
| `color` | `String` | 色值 |
| `sort` | `int` | 排序 |
| `createdAt` | `DateTime` | 创建时间 |
| `updatedAt` | `DateTime` | 更新时间 |

索引建议：

- 唯一：`labelCode`
- 普通：`sort`

#### 9.3.3 `wallet_label_links`

用途：钱包与标签多对多关系。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | 主键 |
| `walletId` | `String` | 关联 `wallet_profiles.walletId` |
| `labelCode` | `String` | 关联 `wallet_labels.labelCode` |
| `createdAt` | `DateTime` | 创建时间 |

索引建议：

- 联合唯一：`walletId + labelCode`
- 普通：`labelCode`

#### 9.3.4 `observed_accounts`

用途：观察账户（非本地私钥控制）。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | 主键 |
| `accountId` | `String` | 业务唯一 ID |
| `orgName` | `String` | 机构/别名 |
| `address` | `String` | 地址 |
| `pubkeyHex` | `String` | 公钥 |
| `balance` | `double?` | 余额缓存 |
| `source` | `String` | `manual/imported` |
| `updatedAt` | `DateTime` | 更新时间 |

索引建议：

- 唯一：`accountId`、`address`、`pubkeyHex`
- 普通：`updatedAt`

#### 9.3.5 `tx_records`

用途：交易主记录（转账/绑定/投票等统一模型）。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | 主键 |
| `bizId` | `String` | 业务唯一号（本地生成） |
| `walletId` | `String` | 发起钱包 |
| `txHash` | `String?` | 链上 hash |
| `preparedId` | `String?` | 后端 prepare id |
| `chainId` | `String` | 链标识 |
| `module` | `String` | `Balances` 等 |
| `call` | `String` | `transfer_allow_death` 等 |
| `direction` | `String` | `out/in/self` |
| `counterparty` | `String?` | 对手地址 |
| `amountAtomic` | `String` | 原子单位，避免精度丢失 |
| `symbol` | `String` | 资产符号 |
| `decimals` | `int` | 小数位 |
| `status` | `String` | `draft/pending/confirmed/failed/expired` |
| `failureCode` | `String?` | 失败码 |
| `failureReason` | `String?` | 失败原因 |
| `submittedAt` | `DateTime?` | 提交时间 |
| `finalizedAt` | `DateTime?` | 最终确认时间 |
| `createdAt` | `DateTime` | 创建时间 |
| `updatedAt` | `DateTime` | 更新时间 |

索引建议：

- 唯一：`bizId`
- 唯一（非空）：`txHash`
- 普通：`walletId + createdAt(desc)`、`status + updatedAt(desc)`

#### 9.3.6 `sign_requests`

用途：签名请求队列与审批状态（登录签名、交易签名等）。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | 主键 |
| `requestId` | `String` | 唯一请求号 |
| `walletId` | `String` | 关联钱包 |
| `bizType` | `String` | `login/transfer/bind/vote` |
| `payloadHash` | `String` | 待签名摘要 |
| `status` | `String` | `created/approved/rejected/expired` |
| `challenge` | `String?` | 登录挑战原文 |
| `expireAt` | `DateTime?` | 过期时间 |
| `createdAt` | `DateTime` | 创建时间 |
| `updatedAt` | `DateTime` | 更新时间 |

索引建议：

- 唯一：`requestId`
- 普通：`walletId + createdAt(desc)`、`status + expireAt`

#### 9.3.7 `admin_role_cache`

用途：管理员公钥到机构角色的本地缓存（链上动态拉取结果）。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | 主键 |
| `pubkeyHex` | `String` | 公钥 |
| `roleName` | `String` | 中文角色名 |
| `institutionName` | `String` | 中文机构名 |
| `institutionIdHex` | `String` | 链上机构 ID |
| `org` | `String` | `nrc/prc/prb/unknown` |
| `fetchedAt` | `DateTime` | 拉取时间 |
| `expiresAt` | `DateTime` | 缓存过期时间 |

索引建议：

- 唯一：`pubkeyHex`
- 普通：`expiresAt`

#### 9.3.8 `app_kv_cache`

用途：可查询但结构不固定的小缓存与游标（替代零散 SharedPreferences）。

| 字段 | 类型 | 说明 |
|---|---|---|
| `id` | `Id` | 主键 |
| `k` | `String` | 业务 key |
| `v` | `String` | JSON 字符串 |
| `updatedAt` | `DateTime` | 更新时间 |
| `expireAt` | `DateTime?` | 可选过期时间 |

索引建议：

- 唯一：`k`
- 普通：`expireAt`

### 9.4 查询路径（关键场景）

- 钱包列表：`wallet_profiles` 按 `updatedAt` 排序。
- 当前钱包：`wallet_profiles.where(isActive=true)`。
- 地址/公钥查钱包：`wallet_profiles` 唯一索引直查。
- 钱包交易流水：`tx_records` 按 `walletId + createdAt(desc)`。
- 待处理签名：`sign_requests.where(status in [created]).sort(createdAt desc)`。
- 管理员角色识别：`admin_role_cache` 按 `pubkeyHex` 查，过期时刷新。

### 9.5 迁移方案（从 SharedPreferences 到 Isar）

#### 9.5.1 版本控制

- 新增 `wallet.data.schema.version = 2`（存于 `app_kv_cache` 或 SharedPreferences）。

#### 9.5.2 一次性迁移步骤

1. 打开 Isar。
2. 读取旧 `SharedPreferences`：
   - `wallet.items`
   - `wallet.active_index`
   - `observe.accounts`
   - `wallet.admin_catalog.*`
   - 其他钱包相关 key
3. 写入对应 Isar 集合。
4. 读取机密层旧 key：
   - `wallet.mnemonic`（旧单钱包）
   - 迁移到 `wallet.secret.<wallet_id>.mnemonic.v1`
5. 设置 schema 版本为 2。
6. 保留旧数据一版发布周期，确认稳定后清理旧 key。

#### 9.5.3 删除钱包一致性

事务顺序建议：

1. Isar 内把 `wallet_profiles` 标记删除或移除。
2. 删除 `wallet_label_links`、`tx_records`、`sign_requests` 的该钱包数据。
3. 删除机密层 `wallet.secret.<wallet_id>.*`。
4. 若删除的是当前激活钱包，自动切换下一个钱包并更新 `isActive`。

### 9.6 安全红线

- 禁止在 Isar/日志/埋点写入助记词、私钥、seed 明文。
- 禁止把签名原文中可复用的机密材料持久化到业务层。
- 所有签名前必须经过 `user_identification` 守卫。
