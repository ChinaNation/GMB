# Onchain 交易模块技术文档（当前实现态）

## 1. 模块定位

`lib/trade/onchain/` 负责链上交易全流程：

- 交易表单与提交页面
- 收款码扫码解析
- 交易模型定义
- 本地交易记录持久化
- 交易编排（签名、提交、状态刷新）

链上通信（extrinsic 构造、签名、提交）由 `lib/rpc/onchain.dart` 统一实现，本模块通过 `OnchainRpc` 调用。

## 2. 文件结构

```text
onchain/
├── onchain_trade_page.dart
├── trade_qr_scan_page.dart
├── onchain_trade_models.dart
├── onchain_trade_repository.dart
├── onchain_trade_service.dart
└── ONCHAIN_TECHNICAL.md
```

## 3. 关键流程

### 3.1 交易提交

1. 页面收集 `toAddress/amount/symbol`
2. `OnchainTradeService.submitTransfer()`：
   - 读取当前钱包与助记词（自动触发生物识别/设备密码验证）
   - 调用 `OnchainRpc.transferKeepAlive()` 直连链上节点完成转账
     - 签名通过回调传入：`Keyring.sr25519.fromMnemonic()` → `pair.sign(payload)`
     - 内部自动完成：获取 nonce、构造 extrinsic、提交到节点
   - 返回 `({String txHash, int usedNonce})`，连同 nonce 一起写入 `OnchainTradeRepository`

### 3.2 交易状态刷新

1. 页面定时（6 秒）和下拉刷新触发 `refreshPendingRecords()`
2. 仅对未终态且有 `usedNonce` 的记录调用 `OnchainRpc.isTxConfirmed(address, usedNonce)` 通过 nonce 对比判断确认状态
3. 确认后调用 `repository.upsert()` 将状态更新为 `confirmed`
4. 节点不可达时跳过，下次轮询重试
5. 返回更新后的记录列表回显统计与 UI

### 3.3 收款码扫码

`trade_qr_scan_page.dart` 支持两类输入：

- `gmb://account/<address>`
- 直接 SS58 地址

解析成功后返回给交易页面填入收款地址框。

## 4. 存储与依赖

- 本地存储：Isar `TxRecordEntity`
- 钱包来源：`WalletManager`
- 链上操作：`OnchainRpc`（`lib/rpc/onchain.dart`）
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

### 6.2 链上转账参数

转账通过 `OnchainRpc.transferKeepAlive()` 直连节点完成，参数：

| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `fromAddress` | `String` | 当前激活钱包 SS58 地址 |
| `signerPubkey` | `Uint8List` | sr25519 公钥 32 字节 |
| `toAddress` | `String` | 收款 SS58 地址 |
| `amountYuan` | `double` | 转账金额（元） |
| `sign` | `Function` | 签名回调，接收 payload 返回 64 字节签名 |

返回值：`({String txHash, int usedNonce})` — 交易哈希（`0x` + 64 hex）+ 提交时使用的 nonce

## 7. 格式与校验标准

- 交易状态仅允许：`pending`、`confirmed`、`failed`。
- 终态规则：`confirmed/failed` 为终态，终态后停止轮询。
- 签名算法固定 `sr25519`，不支持算法协商。
- 金额精度：`BigInt.from((amountYuan * 100).round())`，避免浮点误差。

## 8. 转账流程标准

1. 输入校验：`toAddress/symbol/amount`。
2. 直连链上节点：`OnchainRpc.transferKeepAlive()` 一步完成 extrinsic 构造、签名、提交（读取助记词时自动触发生物识别/设备密码验证）。
4. 落库：写入 `TxRecordEntity`（含 `usedNonce`），状态默认 `pending`。
5. 轮询：仅轮询未终态且有 `usedNonce` 的记录，通过 `isTxConfirmed(address, usedNonce)` 对比链上当前 nonce，确认后更新状态并停止轮询。

## 9. 与治理模块边界

- 本模块只处理"资产转账"。
- 提案/投票字段与流程规范由 `lib/governance/GOVERNANCE_TECHNICAL.md` 定义。
- 治理交易若复用同一签名器，仍必须保持签名域隔离。
