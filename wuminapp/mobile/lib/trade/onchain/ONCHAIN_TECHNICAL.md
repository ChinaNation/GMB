# Onchain 交易模块技术文档（当前实现态）

## 1. 模块定位

`mobile/lib/trade/onchain/` 负责链上交易全流程：

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
   - 本地 `sr25519` 签名
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
- API 调用：`ApiClient`
- 签名算法：`sr25519`（`polkadart_keyring`）

## 5. 错误处理

统一抛出 `OnchainTradeException`：

- `walletMissing`
- `walletMismatch`
- `invalidDraft`
- `broadcastFailed`
