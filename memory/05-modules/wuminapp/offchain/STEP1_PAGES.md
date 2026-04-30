# wuminapp 扫码支付清算 Step 1 页面与 RPC

- **日期**:2026-04-19
- **历史说明**:本文记录 Step 1 当时的增量路径。2026-04-30 后当前源码目录以
  `OFFCHAIN_DIRECTORY.md` 为准,链下扫码支付业务统一位于 `wuminapp/lib/offchain/`。
- **范围**:wuminapp 端为 Step 1 同清算行内 MVP 新增/改造的 Dart 文件
- **上层 ADR**:`memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md`
- **总任务卡**:`memory/08-tasks/open/20260419-扫码支付-step1-同行MVP.md`

---

## 1. 范围概述

Step 1 在 wuminapp 端**只增量、不破坏**:新增 8 个文件 + 改造 2 个,与旧 `lib/wallet/ui/bind_clearing_page.dart`(43 省储行硬编码 + `bind_clearing_institution`)和 `lib/offchain/pages/offchain_pay_page.dart` 共存,Step 2 切换后删除旧入口。

## 2. 新增文件

### 2.1 RPC 层(3 个)

| 文件 | 职责 |
|---|---|
| `lib/offchain/rpc/onchain_clearing_bank_rpc.dart` | 链上 4 个新 extrinsic(call_index 30/31/32/33):`bindClearingBank` / `deposit` / `withdraw` / `switchBank`。复用 polkadart `SigningPayload` + `ExtrinsicPayload` 模式 + `NonceManager`,与现有 `OnchainRpc` 行为一致 |
| `lib/rpc/sfid_public.dart` | SFID 公开 API 客户端,封装 `GET /api/v1/app/clearing-banks/search`,提供 `searchClearingBanks(province, city, keyword, page, size)` |
| `lib/offchain/rpc/offchain_clearing_rpc.dart` | 清算行节点 RPC 客户端:`offchain_queryBalance` / `queryNextNonce` / `queryPendingCount`(WSS) |

### 2.2 页面层(5 个,历史上位于 `lib/trade/offchain/`)

2026-04-30 目录收口后,仍保留的清算行页面已迁入 `lib/offchain/pages/`。

| 文件 | 职责 |
|---|---|
| `clearing_bank_list_page.dart` | 清算行列表 + 省/市/keyword 过滤,跳转绑定页 |
| `bind_clearing_bank_page.dart` | 绑定确认 + 钱包密码解锁 + 调 `bindClearingBank` |
| `deposit_page.dart` | 输入金额(元)→ 调 `deposit(amountFen)` |
| `withdraw_page.dart` | 显示清算行存款余额(可选)+ 输入金额 + 调 `withdraw` |
| `offchain_home_page.dart` | 4 入口聚合页(简化 wallet 详情页改造量) |

## 3. 改造文件(2 个)

| 文件 | 改动 |
|---|---|
| `lib/wallet/ui/wallet_page.dart` | 顶部接入清算行入口;PopupMenu 加 `clearing_bank_v2`/`扫码支付(清算行,Beta)` 项;`_onMenuAction` 加 case;`SFID_BASE_URL` 仅用于查 SFID 后端,清算节点端点由链上 `ClearingBankNodes` 读取 |
| `lib/wallet/ui/bind_clearing_page.dart` | 顶部加 ⚠️ Deprecated 注释,指向新清算行体系入口 |

## 4. 不动的文件(刻意保留)

| 文件 | 原因 |
|---|---|
| `lib/offchain/pages/offchain_pay_page.dart`(576 行) | 旧扫码支付页,仍走 `bind_clearing_institution` + 省储行清算路径,Step 2 整体重写,本步不破坏 |
| `lib/wallet/ui/receive_qr_page.dart`(315 行) | 已生成 `WUMIN_QR_V1 + user_transfer`,`body.bank` 字段语义已支持清算行 SS58,无需 Step 1 改 |
| `lib/rpc/offchain.dart` | 旧省储行 RPC,与新 `offchain_clearing.dart` 并存 |
| `lib/trade/offchain/clearing_banks.dart` | 43 省储行硬编码,Step 2 删 |

## 5. 关键约束与简化

- **本步仅支持热钱包**:绑定/充值/提现都先用 `WalletManager.authenticateForSigning()` + `signWithWalletNoAuth`。冷钱包扫码签名(QR sign_request/sign_response 协议)留 Step 2,与旧 `bind_clearing_page.dart` 风格一致后再统一抽出 helper。
- **金额单位**:UI 输入"元",内部以 `BigInt fen` 进入 `Compact<u128>` SCALE 编码,与链上 `pub fn deposit(amount: u128)` 严格对齐。超过 2 位小数截断(不四舍五入,避免与链上 `round_div` 冲突)。
- **WSS / SFID URL 配置**:本步用 `String.fromEnvironment` 占位,Step 2 接入完整 `Env` 配置层后改读取统一来源。
- **跨清算行扫码支付**:Step 2 启用,本步入口页有提示"Step 1 仅支持同一清算行内付款"。

## 6. 编译验证

```
$ cd wuminapp && flutter analyze --no-pub
Analyzing wuminapp...                                           
No issues found! (ran in 2.7s)
```

零错误。

## 7. 与 Runtime / Node / SFID 的对应

| 端 | 文件 | 对应链上 |
|---|---|---|
| `OnchainClearingBankRpc.bindClearingBank` | wuminapp | `offchain_transaction::Call::bind_clearing_bank` (call_index 30) |
| `OnchainClearingBankRpc.deposit` | wuminapp | call_index 31 |
| `OnchainClearingBankRpc.withdraw` | wuminapp | call_index 32 |
| `OnchainClearingBankRpc.switchBank` | wuminapp | call_index 33 |
| `SfidPublicApi.searchClearingBanks` | wuminapp | sfid-backend `app_search_clearing_banks` |
| `OffchainClearingNodeRpc.queryBalance` | wuminapp | `citizenchain/node/src/offchain/rpc.rs::query_balance` |
| `OffchainClearingNodeRpc.queryNextNonce` | wuminapp | 同上 `query_next_nonce` |

## 8. 后续 Step 2 / Step 3 计划

**Step 2 wuminapp 工作**:
- 接入冷钱包 QR 签名路径(参照旧 `bind_clearing_page.dart`)
- 完整钱包详情页双余额展示(自持 / 清算行存款)
- 改造 `offchain_pay_page.dart` 为新清算行扫码付款流(每笔 L3 私钥签 PaymentIntent)
- 改造 `receive_qr_page.dart` 在 `body.bank` 填新清算行主账户
- WS 订阅清算行节点的到账推送
- 删除 `lib/wallet/ui/bind_clearing_page.dart` + `lib/trade/offchain/clearing_banks.dart` + `lib/rpc/offchain.dart`(全部旧省储行清算入口)

**Step 3 wuminapp 工作**:
- 凭证库(本地加密保存 `{intent, a_sig, l2_ack_sig}`)
- 申诉页 + 申诉历史

## 9. 变更记录

- 2026-04-19:Step 1 wuminapp 端落地,新增 8 文件 + 改造 2 文件,flutter analyze 零错误。
