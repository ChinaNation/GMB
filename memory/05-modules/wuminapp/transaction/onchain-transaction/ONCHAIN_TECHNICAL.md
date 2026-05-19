# Onchain 纯链上支付模块技术文档（当前实现态）

## 1. 模块定位

`wuminapp/lib/transaction/onchain-transaction/` 只负责普通链上转账 / 纯链上支付：

- 链上支付表单与提交页面
- 支付草稿与错误模型
- 支付提交编排：钱包读取、签名回调注入、调用 `OnchainRpc.transferKeepAlive()`

以下能力不属于 `lib/transaction/onchain-transaction/`，不迁入本目录：

- `lib/rpc/`：链上通信、metadata、nonce、extrinsic 编码与提交公共底座
- `lib/transaction/shared/local_tx_store.dart`：本地交易记录共用存储
- `lib/wallet/`：钱包档案、密钥读取、生物识别守卫
- `lib/signer/` 与 `lib/qr/`：热钱包/冷钱包签名协议与扫码会话
- `lib/organization-manage/`：机构多签与多签聚合入口
- `lib/personal-manage/`：个人多签
- `lib/transaction/offchain-transaction/`：链下扫码支付与清算行能力

## 2. 当前文件结构

```text
wuminapp/lib/transaction/onchain-transaction/
├── onchain_payment_page.dart
├── onchain_payment_models.dart
└── onchain_payment_service.dart
```

相关共用能力：

```text
wuminapp/lib/transaction/shared/
└── local_tx_store.dart
```

`lib/transaction/shared/` 不提供功能聚合入口；它只保留本地交易记录这类共用底座。

## 3. 关键流程

1. `OnchainPaymentPanel` 收集 `toAddress / amount / symbol`；`OnchainPaymentPage` 只是独立链上支付路由包装
2. 页面校验 SS58 前缀、金额、余额、ED 和预估手续费
   - 从通讯录进入时，`ContactBookPage` 返回的联系人 `address` 已经是 SS58，页面直接填入收款栏，不做 AccountId hex 转换
3. 页面根据钱包类型注入签名回调：
   - 热钱包：先调用 `WalletManager.authenticateForSigning()`，再用 `signWithWalletNoAuth()` 签名
   - 冷钱包：构造 `sign_request` 二维码，等待 `sign_response` 回执
4. 调用 `OnchainPaymentService.submitTransfer()`
5. 服务调用 `OnchainRpc.transferKeepAlive()` 完成 extrinsic 构造、签名和广播
6. 广播成功后写入 `LocalTxEntity(source=local_submit, status=pending, usedNonce=...)`
7. 交易池 watch 收到 included 后先把本机记录升级为 `inBlock`；`ChainTxMonitor` 监听 finalized 区块事件后把匹配记录合并并升级为 `finalized`

## 4. 链上转账

`OnchainRpc.transferKeepAlive()` 仍在 `lib/rpc/onchain.dart`，因为它是链上 extrinsic 公共底座的一部分，不随 `lib/transaction/onchain-transaction/` 迁移。

当前普通转账 call data：

```text
[pallet_index=2] [call_index=3] [MultiAddress::Id(0x00) + dest_32bytes] [Compact<u128>(fen)]
```

对应 `Balances::transfer_keep_alive`。

## 5. 交易记录

普通链上支付提交成功后写入 `LocalTxEntity`：

- `recordKey = walletPubkeyHex:pending:txHash`
- `type = transfer`
- `status = pending`
- `source = local_submit`
- `txHash = result.txHash`
- `usedNonce = result.usedNonce`
- `transferAmountFen = 转账本金`
- `feeFen = OnchainRpc.estimateTransferFeeYuan(amount)` 换算后的分值
- `amountDeltaFen = -(transferAmountFen + feeFen)`，方向由正负号推导，不单独保存 `direction`

`LocalTxStore` 留在 `lib/transaction/shared/`，因为它服务于交易记录展示，不属于 onchain 支付目录私有实现。

链上流水由 `lib/rpc/chain_tx_monitor.dart` 解析区块 `System.Events` 写入；newHeads 命中时先写 `inBlock`，finalized 命中后升级为 `finalized`。区块事件记录唯一键为 `walletPubkeyHex:blockHash:eventIndex`，pending 记录只用于本机提交后的即时展示和匹配合并。

## 6. 签名边界

- 签名算法固定 `sr25519`
- `OnchainPaymentService.submitTransfer()` 只接收签名回调，不读取 seed
- 热钱包 seed 只在 `WalletManager` 内短暂存在，签名后清零
- 冷钱包签名通过 `WUMIN_QR_V1` 的 `sign_request / sign_response`
- `wuminapp` 只负责生成待签名 payload、校验回执、广播交易；离线签名由 `wumin` 完成

## 7. 手续费

普通链上支付使用客户端静态预估：

```text
fee = max(amount_fen * 0.001, 10 fen)
```

- 费率：0.1%
- 最低手续费：0.10 元
- tip：0
- 金额进入链前统一转换为分：`BigInt.from((amountYuan * 100).round())`

## 8. 边界规则

- `OnchainPaymentPanel.extraEntriesBuilder` 只提供 UI 插槽，供 `lib/transaction/transaction_tab_page.dart` 在链状态提示下方、链上支付表单上方插入扫码支付入口；onchain 模块自身不 import `offchain` 或 `duoqian`
- `lib/transaction/onchain-transaction/` 不放治理提案、投票、多签、链下支付、清算行、钱包密钥管理、二维码协议底座，也不提供“交易/金融”聚合入口
- 新增普通链上支付 UI / model / service 时才进入 `lib/transaction/onchain-transaction/`
- 若新增能力需要 pallet index / call index，必须先确认是否仍属于“普通链上支付”；否则放回对应业务模块
