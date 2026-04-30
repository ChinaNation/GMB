# wuminapp offchain 目录收口说明

- **日期**:2026-04-30
- **范围**:`wuminapp/lib/offchain`
- **任务卡**:`memory/08-tasks/open/20260430-wuminapp-offchain-move.md`

## 1. 结论

链下扫码支付是一个独立业务域,统一收口到:

```text
wuminapp/lib/offchain/
  offchain.dart
  models/
  pages/
  rpc/
  services/
  qr/
  widgets/
```

`wuminapp/lib/trade/` 不再承载 `offchain/` 子目录，也不提供"扫码支付"
聚合入口。扫码、清算行端点解析、PaymentIntent 构造、签名与提交均由
`lib/offchain` 负责。

钱包页同理只保留充值 / 提现 / 余额的入口展示,真正的充值、提现、清算行余额
查询与清算行绑定缓存均从 `lib/offchain` 引入。

## 2. 当前文件归属

| 目录 | 职责 |
|---|---|
| `lib/offchain/pages/` | 扫码付款页、清算行设置、绑定 / 切换、充值、提现 |
| `lib/offchain/models/` | `NodePaymentIntent` 等链下支付专属模型 |
| `lib/offchain/rpc/` | 清算行节点 WSS RPC 与清算行链上 extrinsic 构造 |
| `lib/offchain/services/` | 清算行目录、绑定快照、扫码付款入口流程 |
| `lib/offchain/offchain.dart` | offchain 业务域 barrel export |

## 3. 迁移映射

| 旧路径 | 新路径 |
|---|---|
| `lib/trade/offchain/offchain_clearing_pay_page.dart` | `lib/offchain/pages/offchain_pay_page.dart` |
| `lib/trade/offchain/payment_intent.dart` | `lib/offchain/models/payment_intent.dart` |
| `lib/trade/offchain/clearing_bank_settings_page.dart` | `lib/offchain/pages/clearing_bank_settings_page.dart` |
| `lib/trade/offchain/bind_clearing_bank_page.dart` | `lib/offchain/pages/bind_clearing_bank_page.dart` |
| `lib/trade/offchain/deposit_page.dart` | `lib/offchain/pages/deposit_page.dart` |
| `lib/trade/offchain/withdraw_page.dart` | `lib/offchain/pages/withdraw_page.dart` |
| `lib/trade/offchain/clearing_bank_prefs.dart` | `lib/offchain/services/clearing_bank_prefs.dart` |
| `lib/rpc/offchain_clearing.dart` | `lib/offchain/rpc/offchain_clearing_rpc.dart` |
| `lib/rpc/onchain_clearing_bank.dart` | `lib/offchain/rpc/onchain_clearing_bank_rpc.dart` |
| `lib/rpc/clearing_bank_directory.dart` | `lib/offchain/services/clearing_bank_directory.dart` |

## 4. 保留边界

- `lib/qr/` 仍是通用二维码协议底座,继续服务联系人、多签、冷钱包签名等场景。
- `lib/wallet/` 只保留钱包页入口 UI,不得重新放入链下支付业务实现。
- `lib/trade/` 只保留本地交易记录与 pending 对账共用能力，不承载扫码支付、多签或链上支付入口。

## 5. 新入口

扫码支付入口应直接调用:

```text
lib/offchain/services/offchain_scan_flow.dart
```

该流程负责:

1. 打开通用 `QrScanPage`
2. 校验收款码必须包含 `UserTransferBody.bank`
3. 读取链上 `ClearingBankNodes[sfid_id]`
4. 跳转 `lib/offchain/pages/offchain_pay_page.dart`
