# wuminapp 钱包交易记录与收款流水修复

## 任务需求

修复 wuminapp 钱包交易记录相关问题：

- 钱包详情页最近交易记录最多显示 5 条。
- 钱包详情页“交易记录”标题和右侧箭头进入完整交易记录页。
- 钱包详情页点击某条交易记录直接进入交易详情页。
- 钱包详情页最近交易记录展示状态标签，与完整交易记录页口径一致。
- 新建或新导入钱包后，收款方交易一旦入块，即使尚未 finalized，也要先显示为“已出块”。
- finalized 后同一条记录升级为“已确认”，不新增重复记录。
- 交易页 `签名交易` 下方 4 个状态只统计当前交易钱包自己发起的链上转出记录；切换交易钱包后必须刷新为新钱包状态，不能残留旧钱包数字。
- 钱包详情页操作卡第三列从“余额”改为“零钱包”，避免和下方“链上余额”卡片语义冲突。

## 不处理范围

- 不迁移旧测试钱包游标。
- 不回扫钱包创建或导入前历史。
- 不兼容旧本地流水格式。
- 旧测试钱包由用户删除后重新创建。

## 预计修改目录

- `wuminapp/lib/wallet/`：修复钱包详情页最近交易记录展示、状态标签、点击路径，并复用交易详情页。
- `wuminapp/lib/rpc/`：给 `ChainTxMonitor` 增加未 finalized 区块补扫，保证入块收入先显示为“已出块”。
- `wuminapp/lib/transaction/shared/`：按需补强本地流水去重和状态升级工具。
- `wuminapp/lib/transaction/onchain-transaction/`：修复交易页切换交易钱包后的转出状态刷新，不改变签名、广播和确认逻辑。
- `wuminapp/lib/wallet/widgets/`：调整钱包详情页操作卡第三列文案，明确零钱包余额不是链上余额。
- `wuminapp/test/`：补交易流水与钱包详情页交互测试。
- `memory/05-modules/wuminapp/`：同步更新钱包流水和交互文档。

## 执行计划

- [x] 读取执行上下文和模块文档。
- [x] 抽出可复用交易详情页。
- [x] 修复钱包详情页最近记录状态展示和点击路径。
- [x] 增加未 finalized 区块补扫。
- [x] 补充测试。
- [x] 更新技术文档。
- [x] 执行验证并清理残留。
- [x] 修复交易页切换交易钱包后状态行残留旧钱包转出统计。
- [x] 将钱包详情页操作卡第三列文案从“余额”改为“零钱包”。

## 验证记录

- `dart format wuminapp/lib/wallet/pages/transaction_history_page.dart wuminapp/lib/wallet/pages/wallet_page.dart wuminapp/lib/rpc/chain_tx_monitor.dart wuminapp/test/wallet/transaction_history_page_test.dart`：通过。
- `flutter analyze lib/rpc/chain_tx_monitor.dart lib/wallet/pages/transaction_history_page.dart lib/wallet/pages/wallet_page.dart test/wallet/transaction_history_page_test.dart`：通过，无问题。
- `flutter test test/transaction/local_tx_store_status_test.dart test/wallet/transaction_history_page_test.dart`：通过。
- `flutter test --concurrency=1`：通过，全量串行测试通过。
- `flutter test`：默认并发跑法曾出现 1 个治理侧本地存储用例波动；单独复跑该用例通过，钱包交易记录相关测试不受影响。
- `git diff --check`：通过，无空白错误。
- 残留关键字扫描：通过，未发现本次修改目录中的临时调试残留。
- `dart format lib/transaction/onchain-transaction/onchain_payment_page.dart test/ui/transaction_tab_page_test.dart`：通过。
- `flutter analyze lib/transaction/onchain-transaction/onchain_payment_page.dart test/ui/transaction_tab_page_test.dart`：通过，无问题。
- `flutter test test/ui/transaction_tab_page_test.dart`：通过，覆盖切换交易钱包后状态行刷新，且 B 钱包只有收入时 4 个状态保持 0。
- `flutter test test/transaction/local_tx_store_status_test.dart test/wallet/transaction_history_page_test.dart test/ui/transaction_tab_page_test.dart`：通过，确认钱包完整流水和交易页转出状态口径互不影响。
- `dart format lib/wallet/widgets/wallet_action_card.dart test/wallet/widgets/wallet_action_card_test.dart`：通过。
- `flutter analyze lib/wallet/widgets/wallet_action_card.dart test/wallet/widgets/wallet_action_card_test.dart`：通过，无问题。
- `flutter test test/wallet/widgets/wallet_action_card_test.dart`：通过，确认第三列显示“零钱包”，且仍不可点击。
