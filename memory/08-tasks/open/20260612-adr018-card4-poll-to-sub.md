# 任务卡:卡④三处Timer轮询改finalized订阅

属 ADR-018(memory/04-decisions/ADR-018-wuminapp-unified-query-low-load.md)。

卡④三处Timer轮询改finalized订阅

## 验收
- [ ] flutter analyze 0 + flutter test 全过
- [ ] 旧代码/文档/注释清理无残留

## 完工记录(2026-06-13)
- duoqian_transfer_detail_page + governance/institution_manage_detail_page:`_syncPendingPoll` 的 20s `Timer.periodic` 全部改为 `ChainEventSubscription` 订阅 finalized 头(`newFinalizedBlock` 触发 `_load(showSpinner:false)`),保留"仅待投票确认期"门控;dispose 取消订阅。空闲链零轮询,有新最终块(=有交易上链)才刷新。
- 豁免保留:pin_input_page / qr_sign_session_page 的 1s UI 倒计时、chain_tx_monitor 5s 订阅重连、chain_progress_banner 6s 状态条(状态条需持续显示同步进度,留待卡⑤一并评估)。
- analyze 0 / test 196。待真机验证详情页待投票刷新正常。
