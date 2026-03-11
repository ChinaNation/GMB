# Onchain 交易模块技术文档（当前实现态）

## 1. 模块定位

`lib/trade/onchain/` 负责链上交易全流程：

- 交易表单与提交页面
- 收款码扫码解析
- 交易模型定义
- 本地交易记录持久化
- 后端 `prepare/submit/status` 网关调用
- 交易编排（签名、提交、状态刷新）

## 2. 文件结构

```text
onchain/
├── onchain_trade_page.dart
├── trade_qr_scan_page.dart
├── onchain_trade_models.dart
├── onchain_trade_repository.dart
├── onchain_trade_gateway.dart
├── onchain_trade_service.dart
└── ONCHAIN_TECHNICAL.md
```

## 3. 关键流程

### 3.1 交易提交

1. 页面收集 `toAddress/amount/symbol`
2. 调用 `UserIdentificationService.confirmBeforeSign()` 做签名前验证
3. `OnchainTradeService.submitTransfer()`：
   - 读取当前钱包与助记词
   - 调 `gateway.prepareTransfer()` 获取 signer payload
   - 调用 `LocalSigner` 做本机 `sr25519` 签名
   - 调 `gateway.submitTransfer()` 广播交易
   - 写入 `OnchainTradeRepository`

### 3.2 交易状态刷新

1. 页面定时（6 秒）和下拉刷新触发 `refreshPendingRecords()`
2. 仅对未终态记录轮询 `gateway.queryStatus(txHash)`
3. 更新本地记录并回显统计与列表

### 3.3 收款码扫码

`trade_qr_scan_page.dart` 支持两类输入：

- `gmb://account/<address>`
- 直接 SS58 地址

解析成功后返回给交易页面填入收款地址框。

## 4. 存储与依赖

- 本地存储：Isar `TxRecordEntity`
- 钱包来源：`WalletManager`
- 签名入口：`LocalSigner`
- API 调用：`ApiClient`
- 签名算法：`sr25519`（`polkadart_keyring`）

扫码签名扩展：

- 协议层由 `QrSigner` 提供（`WUMINAPP_QR_SIGN_V1`）
- 当前链上交易 UI 仍以本机签名为主

## 5. 错误处理

统一抛出 `OnchainTradeException`：

- `walletMissing`
- `walletMismatch`
- `invalidDraft`
- `broadcastFailed`

## 6. 转账字段规范（链上交易）

### 6.1 App 草稿字段（页面 -> Service）

| 字段 | 类型 | 规则 |
| --- | --- | --- |
| `toAddress` | `String` | 非空，SS58 地址 |
| `amount` | `double` | `> 0` |
| `symbol` | `String` | 非空，提交前转大写 |

### 6.2 Prepare 请求字段（App -> 网关）

`POST /api/v1/tx/prepare`

| 字段 | 类型 | 规则 |
| --- | --- | --- |
| `from_address` | `String` | 当前激活钱包地址 |
| `pubkey_hex` | `String` | `0x` + 64 hex（sr25519 公钥） |
| `to_address` | `String` | 收款地址 |
| `amount` | `number` | 当前实现为 number；后续建议升级为 decimal string |
| `symbol` | `String` | 大写币种代码（如 `GMB`） |

返回字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `prepared_id` | `String` | 预处理交易 ID |
| `signer_payload_hex` | `String` | 待签名 payload（`0x` hex） |
| `expires_at` | `int` | 预处理过期时间（epoch 秒） |

### 6.3 Submit 请求字段（App -> 网关）

`POST /api/v1/tx/submit`

| 字段 | 类型 | 规则 |
| --- | --- | --- |
| `prepared_id` | `String` | 来自 prepare |
| `pubkey_hex` | `String` | 签名钱包公钥 |
| `signature_hex` | `String` | `0x` + 128 hex（sr25519 64 字节签名） |

返回字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `tx_hash` | `String` | 链上交易哈希 |
| `status` | `String` | `pending/confirmed/failed` |
| `failure_reason` | `String?` | 失败原因（可空） |

### 6.4 状态查询字段（App -> 网关）

`GET /api/v1/tx/status/:tx_hash`

返回字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `tx_hash` | `String` | 交易哈希 |
| `status` | `String` | `pending/confirmed/failed` |
| `updated_at` | `int` | 更新时间（epoch 秒） |
| `failure_reason` | `String?` | 失败原因 |

## 7. 格式与校验标准

- `signer_payload_hex` 必须为偶数字节 hex，解码后非空。
- `signature_hex` 必须与当前钱包公钥匹配（防错签）。
- 交易状态仅允许：`pending`、`confirmed`、`failed`。
- 终态规则：`confirmed/failed` 为终态，终态后停止轮询。
- 签名算法固定 `sr25519`，不支持算法协商。

## 8. 转账流程标准

1. 输入校验：`toAddress/symbol/amount`。
2. 预处理：调用 `tx/prepare` 获取 `prepared_id + signer_payload_hex`。
3. 签名：对 `signer_payload_hex` 原始字节做 `sr25519` 签名。
4. 广播：提交 `prepared_id + signature_hex` 到 `tx/submit`。
5. 落库：写入 `TxRecordEntity`，状态默认按返回值。
6. 轮询：仅轮询未终态记录，收到终态后停止。

## 9. 与治理模块边界

- 本模块只处理“资产转账”。
- 提案/投票字段与流程规范由 `lib/governance/GOVERNANCE_TECHNICAL.md` 定义。
- 治理交易若复用同一签名器，仍必须保持签名域隔离。
