# Onchain 纯链上支付模块技术文档（当前实现态）

## 1. 模块定位

`citizenapp/lib/transaction/onchain-transaction/` 只负责普通链上转账 / 纯链上支付：

- 链上支付表单与提交页面
- 支付草稿与错误模型
- 支付提交编排：钱包读取、签名回调注入、调用 `OnchainRpc.transferWithRemark()`

以下能力不属于 `lib/transaction/onchain-transaction/`，不迁入本目录：

- `lib/rpc/`：链上通信、metadata、nonce、extrinsic 编码与提交公共底座
- `lib/transaction/shared/local_tx_store.dart`：本地交易记录共用存储
- `lib/wallet/`：钱包档案、密钥读取、生物识别守卫
- `lib/signer/` 与 `lib/qr/`：热钱包/公民钱包签名协议与扫码会话
- `lib/citizen/institution/`:机构管理(只读);多签转账聚合入口在 `lib/transaction/multisig-transfer/`
- `lib/personal-manage/`：个人多签
- `lib/transaction/offchain-transaction/`：链下扫码支付与清算行能力

## 2. 当前文件结构

```text
citizenapp/lib/transaction/onchain-transaction/
├── onchain_payment_page.dart
├── onchain_payment_models.dart
└── onchain_payment_service.dart
```

相关共用能力：

```text
citizenapp/lib/transaction/shared/
└── local_tx_store.dart
```

`lib/transaction/shared/` 不提供功能聚合入口；它只保留本地交易记录这类共用底座。

## 3. 关键流程

1. `OnchainPaymentPanel` 收集 `toAddress / amount / remark / symbol`；`OnchainPaymentPage` 只是独立链上支付路由包装
2. 页面校验 SS58 前缀、金额、finalized 余额、ED 和预估手续费
   - 左侧 `ContactBookPage` 始终读取“我的钱包”默认用户的通讯录；它与右侧当前付款钱包相互独立
   - 从通讯录返回的联系人 `address` 已经是 SS58，页面只填入收款栏，不做 AccountId hex 转换，也不改变当前付款钱包
3. 页面根据钱包类型注入签名回调：
   - 热钱包：先调用 `WalletManager.authenticateForSigning()`，再用 `signWithWalletNoAuth()` 签名
   - 冷钱包：构造 `sign_request` 二维码，等待 `sign_response` 响应
4. 调用 `OnchainPaymentService.submitTransfer()`
5. 服务调用 `OnchainRpc.transferWithRemark()` 完成 extrinsic 构造、签名和广播
6. 广播成功后写入 `LocalTxEntity(source=local_submit, status=pending, usedNonce=..., remark=...)`
7. 交易池 watch 收到 included 后先把本机记录升级为 `inBlock`；`ChainTxMonitor` 监听 newHeads 并补扫 finalized 之后的未确认区块，把命中本机钱包的收支先写为 `inBlock`，再按 finalized 高度把匹配记录合并并升级为 `finalized`

交易状态仍保留 `pending / inBlock / finalized` 三段；余额、可用金额、余额不足提示和钱包余额回写统一读取 finalized 余额，不能因为 `inBlock` 事件先到就把 best 余额写入展示缓存。

## 4. 链上转账

`OnchainRpc.transferWithRemark()` 仍在 `lib/rpc/onchain.dart`，因为它是链上 extrinsic 公共底座的一部分，不随 `lib/transaction/onchain-transaction/` 迁移。

当前普通转账 call data：

```text
[pallet_index=4] [call_index=0] [beneficiary:AccountId32] [amount:u128_le] [remark:BoundedVec<u8>]
```

对应 `OnchainTransaction::transfer_with_remark`。备注按 UTF-8 字节计数，最大 99 字节；空备注编码为长度 0 的 `BoundedVec<u8>`。

普通单账户链上转账唯一外部入口是 `OnchainTransaction::transfer_with_remark`；`Balances` 只作为 runtime 底层余额账本和内部 `Currency` 能力保留，页面、服务和 RPC 层不得构造 `Balances.transfer_*` 裸调用。

## 5. 交易记录

普通链上支付提交成功后写入 `LocalTxEntity`：

- `recordKey = walletPubkeyHex:pending:txHash`
- `type = transfer`
- `status = pending`
- `source = local_submit`
- `txHash = result.txHash`
- `usedNonce = result.usedNonce`
- `transferAmountFen = 转账本金`
- `remark = 转账备注`，本机 pending 与区块事件合并时保留非空备注
- `feeFen = OnchainRpc.estimateTransferFeeYuan(amount)` 换算后的分值
- `amountDeltaFen = -(transferAmountFen + feeFen)`，方向由正负号推导，不单独保存 `direction`

`LocalTxStore` 留在 `lib/transaction/shared/`，因为它服务于交易记录展示，不属于 onchain 支付目录私有实现。

链上流水由 `lib/rpc/chain_tx_monitor.dart` 解析区块 `System.Events` 写入；newHeads 命中或未确认区块补扫命中时先写 `inBlock`，finalized 命中后升级为 `finalized`。区块事件记录唯一键为 `walletPubkeyHex:blockHash:eventIndex`，pending 记录只用于本机提交后的即时展示和匹配合并；普通转账本机写入统一走 `LocalTxStore.upsertLocalSubmitTransfer()`，区块事件先到时也合并为同一条。

交易页 `OnchainPaymentPanel` 中 `签名交易` 下方的 `已提交 / 已出块 / 已确认 / 失败` 状态行只统计当前交易钱包自己发起的链上转出记录：

- 查询条件按当前钱包 `walletPubkeyHex` 读取本地流水。
- 展示前继续过滤 `type == transfer` 且 `amountDeltaFen < 0`。
- 收入记录不进入交易页状态行；完整收支流水只在 `我的 -> 我的钱包 -> 钱包详情` 及完整交易记录页展示。
- 右上角切换交易钱包后，页面必须先清空旧钱包状态，再按新钱包 `walletPubkeyHex` 重新加载本机转出记录；异步查询返回时还要校验查询发起时的钱包 pubkey，避免旧查询结果覆盖新钱包状态。
- 右上角钱包只选择本次付款钱包；左上角通讯录不读取 `_currentWallet`，始终由通讯录模块按默认用户加载联系人。付款钱包、通讯录所属用户和联系人收款账户是三个独立语义。

## 6. 签名边界

- 签名算法固定 `sr25519`
- `OnchainPaymentService.submitTransfer()` 只接收签名回调，不读取 seed
- 热钱包 seed 只在 `WalletManager` 内短暂存在，签名后清零
- 公民钱包签名通过 `QR_V1` 的 `sign_request / sign_response`
- CitizenApp 只负责生成待签名 payload、校验签名响应、广播交易；离线签名由 CitizenWallet 完成

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

- `OnchainPaymentPanel.extraEntriesBuilder` 只提供 UI 插槽，供 `lib/transaction/transaction_tab_page.dart` 在链状态提示下方、链上支付表单上方插入扫码支付入口；onchain 模块自身不 import `offchain` 或 `multisig`
- `OnchainPaymentPage.initialToAddress` 只用于从通讯录等入口预填收款地址，不得触发付款钱包切换、金额填写、签名或自动提交。
- `lib/transaction/onchain-transaction/` 不放治理提案、投票、多签、链下支付、清算行、钱包密钥管理、二维码协议底座，也不提供“交易/金融”聚合入口
- 新增普通链上支付 UI / model / service 时才进入 `lib/transaction/onchain-transaction/`
- 若新增能力需要 pallet index / call index，必须先确认是否仍属于“普通链上支付”；否则放回对应业务模块
