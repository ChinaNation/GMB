# 任务卡：wuminapp 钱包交易流水三段状态修复

- 任务编号：20260519-wallet-tx-three-status
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-05-19

## 任务需求

修复 wuminapp 钱包交易记录状态链路，将钱包交易流水统一为三段状态：

- `pending` = 已提交
- `inBlock` = 已出块
- `finalized` = 已确认

同时修复普通转账已经到账但本机交易记录仍显示待确认、收款钱包有收入但没有收入交易记录的问题。完成后更新文档、完善中文注释并清理旧二段状态残留。

## 执行范围

- `wuminapp/lib/rpc/`：修复交易池入块状态回调、区块事件订阅重试、new heads/finalized heads 事件处理和钱包流水事件写回边界。
- `wuminapp/lib/transaction/shared/`：调整本地交易流水写入与状态升级逻辑，支持 `pending → inBlock → finalized`。
- `wuminapp/lib/transaction/onchain-transaction/`：普通转账提交后接入 `inBlock` 回调，及时升级本机转出记录。
- `wuminapp/lib/wallet/`：钱包详情和交易记录页面状态文案改为“已提交 / 已出块 / 已确认”。
- `memory/01-architecture/wuminapp/`、`memory/05-modules/wuminapp/`：同步三段状态、监听策略和本地流水边界说明。

## 约束

- 不修改 Isar schema，只调整 `LocalTxEntity.status` 的字符串口径和写入逻辑。
- 不恢复 txHash/nonce 轮询确认路径，避免增加节点负担。
- 不补扫钱包导入前历史，仍保持本机开始记录的边界。
- App 只做本地索引和交互展示，不承担链上信任根。

## 实施记录

- 已将本机交易流水状态统一为 `pending / inBlock / finalized`，页面文案对应“已提交 / 已出块 / 已确认”。
- 已接入普通转账交易池 watch 的 included 回调，广播成功后先写 `pending`，入块后升级本机提交记录为 `inBlock`。
- 已将 `ChainEventSubscription` 改为同时订阅 `chain_subscribeNewHeads` 与 `chain_subscribeFinalizedHeads`，并在 smoldot 未就绪时由 `ChainTxMonitor` 周期重试订阅。
- 已将 `ChainTxMonitor` 改为 newHeads 写入/升级 `inBlock`，finalized 游标补同步后升级 `finalized`；事件解析优先走 metadata 解码，失败时才走旧兜底解析。
- 已调整 `LocalTxStore` 的区块事件写入逻辑，收入在 `inBlock` 阶段即可持久化写入，finalized 阶段升级同一条记录；本机转出记录按同钱包、同发送方、同接收方、同本金合并，避免重复显示。
- 已清理链事件静态新区块常量残留，公民投票列表改为按 `event.type` 判断新区块事件，避免带区块号事件无法命中的隐性问题。
- 已同步钱包、交易、RPC 和总体架构文档，并清理旧 `confirmed` 二段状态说明残留。

## 验证记录

- `dart format wuminapp/lib/rpc/chain_event_subscription.dart wuminapp/lib/rpc/chain_tx_monitor.dart wuminapp/lib/rpc/chain_rpc.dart wuminapp/lib/rpc/onchain.dart wuminapp/lib/rpc/smoldot_client.dart wuminapp/lib/isar/wallet_isar.dart wuminapp/lib/transaction/shared/local_tx_store.dart wuminapp/lib/transaction/onchain-transaction/onchain_payment_service.dart wuminapp/lib/transaction/onchain-transaction/onchain_payment_page.dart wuminapp/lib/wallet/pages/transaction_history_page.dart wuminapp/test/transaction/local_tx_store_status_test.dart`
- `dart analyze lib test`：通过。
- `flutter test test/transaction/local_tx_store_status_test.dart`：通过。
- `git diff --check`：通过。
- 残留扫描：钱包交易流水相关旧二段确认口径已清理；剩余“待确认”命中属于投票 pending 语境或本任务需求原文，不属于钱包交易流水旧状态。
