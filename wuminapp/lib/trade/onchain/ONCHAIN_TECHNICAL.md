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
   - 接受外部注入的 `sign` 回调（由 UI 层根据热/冷钱包模式提供）
   - 调用 `OnchainRpc.transferKeepAlive()` 直连链上节点完成转账
     - 热钱包：`Keyring.sr25519.fromSeed(seed)` → `pair.sign(payload)`
     - 冷钱包：构建 `QrSignRequest` → 导航到 `QrSignSessionPage` 展示请求二维码 → 用户用离线设备扫码签名 → 扫描回执二维码 → `QrSigner.parseResponse()` 解析签名 → 返回签名字节
     - 内部自动完成：获取 nonce、构造 extrinsic、提交到节点
   - 计算预估手续费 `OnchainRpc.estimateTransferFeeYuan(amount)`，随交易记录一起写入
   - 返回 `({String txHash, int usedNonce})`，连同 nonce 和 estimatedFee 一起写入 `OnchainTradeRepository`

### 3.2 交易状态刷新

1. 页面定时（6 秒）和下拉刷新触发 `refreshPendingRecords()`
2. 仅对未终态且有 `usedNonce` 的记录调用 `OnchainRpc.isTxConfirmed(address, usedNonce)` 通过 nonce 对比判断确认状态
3. 确认后调用 `repository.upsert()` 将状态更新为 `confirmed`
4. 节点不可达时跳过，下次轮询重试
5. 返回更新后的记录列表回显统计与 UI

### 3.3 收款码扫码

扫码功能已迁移到统一扫码页面 `lib/qr/pages/qr_scan_page.dart`，支持三类输入：

- `WUMINAPP_TRANSFER_V1` JSON 格式 → 完整解析（收款地址 + 金额 + 币种）
- `gmb://account/<address>` → 仅填充收款地址
- 直接 SS58 地址 → 仅填充收款地址

扫码结果通过 `QrScanTransferResult` 返回，交易页面自动预填收款地址、金额和币种。

## 4. 存储与依赖

- 本地存储：Isar `TxRecordEntity`
- 钱包来源：`WalletManager`
- 链上操作：`OnchainRpc`（`lib/rpc/onchain.dart`）
- 签名算法：`sr25519`（`polkadart_keyring`）

扫码签名扩展：

- 协议层由 `QrSigner` 提供（`WUMINAPP_QR_SIGN_V1`）
- 签名会话页面由 `QrSignSessionPage`（`lib/qr/pages/qr_sign_session_page.dart`）提供

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

## 8. 手续费

### 8.1 费率模型

链上使用自定义 `PowOnchainChargeAdapter`，标准 Substrate `payment_queryInfo` 不适用（weight/length 费用均为 0）。

- **费率**：`Perbill::from_parts(1_000_000)` = **0.1%**
- **最低手续费**：`10 fen` = **0.10 元**
- **公式**：`fee = max(amount_fen × 0.001, 10 fen)`
- **tip**：当前硬编码为 0
- **舍入**：half-up 到 fen 精度（与链上 `mul_perbill_round` 一致）

### 8.2 客户端计算

`OnchainRpc.estimateTransferFeeYuan(double amountYuan)` 纯客户端静态方法，无需 RPC 调用。

### 8.3 用户体验

- **提交前**：弹出确认对话框，显示转账金额、预估手续费、合计
- **交易记录列表**：每条记录显示手续费
- **交易详情页**：显示手续费行

### 8.4 存储

`OnchainTxRecord.estimatedFee`（`double?`），提交时写入，旧记录为 `null`。

| 转账金额 | 按费率计算 | 实际手续费 |
|---------|-----------|-----------|
| 1 元 | 0 fen | **0.10 元**（最低） |
| 100 元 | 10 fen | **0.10 元** |
| 500 元 | 50 fen | **0.50 元** |
| 10000 元 | 1000 fen | **10.00 元** |

## 9. 转账流程标准

1. 输入校验：`toAddress/symbol/amount`。
2. 预估手续费并展示确认对话框，用户确认后继续。
3. 直连链上节点：`OnchainRpc.transferKeepAlive()` 一步完成 extrinsic 构造、签名、提交。
   - 当前实现：签名回调由 UI 层根据 `signMode` 注入（热钱包通过 `WalletManager.signWithWallet()`，冷钱包通过 `QrSigner` 协议）
   - 目标改造：UI 层统一通过 `SigningCoordinator` 注入签名能力，不再在页面中分散维护热/冷分支
4. 落库：写入 `TxRecordEntity`（含 `usedNonce` + `estimatedFee`），状态默认 `pending`。
5. 轮询：仅轮询未终态且有 `usedNonce` 的记录，通过 `isTxConfirmed(address, usedNonce)` 对比链上当前 nonce，确认后更新状态并停止轮询。

## 10. 与治理模块边界

- 本模块只处理"资产转账"。
- 提案/投票字段与流程规范由 `lib/governance/GOVERNANCE_TECHNICAL.md` 定义。
- 治理交易若复用同一签名器，仍必须保持签名域隔离。
