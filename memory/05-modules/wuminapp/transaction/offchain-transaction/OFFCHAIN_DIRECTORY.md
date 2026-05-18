# wuminapp offchain 目录收口说明

- **日期**:2026-04-30
- **范围**:`wuminapp/lib/transaction/offchain-transaction`
- **任务卡**:`memory/08-tasks/open/20260430-wuminapp-offchain-move.md`

## 1. 结论

链下扫码支付是一个独立业务域,统一收口到:

```text
wuminapp/lib/transaction/offchain-transaction/
  offchain.dart
  models/
  pages/
  rpc/
  services/
  qr/
  widgets/
```

顶层交易 Tab 位于 `wuminapp/lib/transaction/transaction_tab_page.dart`。扫码、清算行端点解析、PaymentIntent 构造、签名与提交均由
`lib/transaction/offchain-transaction/` 负责。

钱包页同理只保留充值 / 提现 / 余额的入口展示,真正的充值、提现、清算行余额
查询与清算行绑定缓存均从 `lib/transaction/offchain-transaction/` 引入。

## 2. 当前文件归属

| 目录 | 职责 |
|---|---|
| `lib/transaction/offchain-transaction/pages/` | 扫码付款页、清算行设置、绑定 / 切换、充值、提现 |
| `lib/transaction/offchain-transaction/models/` | `NodePaymentIntent` 等链下支付专属模型 |
| `lib/transaction/offchain-transaction/rpc/` | 清算行节点 WSS RPC 与清算行链上 extrinsic 构造 |
| `lib/transaction/offchain-transaction/services/` | 清算行目录、绑定快照、扫码付款入口流程 |
## 3. 保留边界

- `lib/qr/` 仍是通用二维码协议底座,继续服务联系人、多签、冷钱包签名等场景。
- `lib/wallet/` 只保留钱包页入口 UI,不得重新放入链下支付业务实现。
- `lib/transaction/shared/` 只保留本地交易记录共用能力，不承载扫码支付、多签或链上支付入口。

## 4. 新入口

扫码支付入口应直接调用:

```text
lib/transaction/offchain-transaction/services/offchain_scan_flow.dart
```

该流程负责:

1. 打开通用 `QrScanPage`
2. 校验收款码必须包含 `UserTransferBody.bank`
3. 读取链上 `ClearingBankNodes[sfid_number]`
4. 跳转 `lib/transaction/offchain-transaction/pages/offchain_pay_page.dart`
