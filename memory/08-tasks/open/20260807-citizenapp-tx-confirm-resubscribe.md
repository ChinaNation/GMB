# 任务卡：交易「待确认」不自动翻转 —— finalized 订阅断开后永不重连

状态：进行中（2026-08-07）

## 现象

CitizenApp 交易记录停在「待确认」，**必须手动刷新交易页**才变「已确认」。
**iOS 与 Android 两端同时复现**，`16bf8c7e2`(08-02「修复待确认问题」)当时测试通过。

## 根因

**确认逻辑本身没坏**——`_confirmOpenSubmits` 的锚比对 + nonce 兜底仍在原位、仍先于前向循环。
坏的是**驱动它的事件源断了之后再也接不回来**。

自动确认只有一条通路（`chain_tx_monitor.dart:213-218`）：

```
finalized 订阅推 newFinalizedBlock → _syncThrough → _confirmOpenSubmits
```

而这条通路一旦断开就永久失效：

1. `smoldot_client.dart:1073-1081` 的 `subscribe()` 是 `async*`，内层
   `_chain!.subscribe(...)` 随原生 chain 释放而结束 → 外层生成器完成。
2. `chain_event_subscription.dart:97-104` 的 `onDone` **只把 `_newHeadsSub` /
   `_finalizedHeadsSub` 置 null 并打一行日志，不通知任何人**。
3. `chain_tx_monitor.dart:170-173` 的 `_ensureSubscription()` 第一行是
   `if (_subscriptionConnected) return;`，而 `_subscriptionConnected` 只在
   `_connectSubscriptionOnce` 成功时置 true、**只在 `stop()` 里置 false**。

于是断开后：订阅对象已经没了，监控器却仍认为自己连着，`_ensureSubscription()`
每次都从第一行早退，`_subscriptionRetryTimer` 也不会被装上 —— **没有任何重连路径**。

手动刷新走的是另一条路 `_syncToLatest()`，直接查链、不经订阅，所以刷新永远有效。
这正好解释「不刷新不动、一刷新就对」。

之所以是两端同时出现：这是纯 Dart 层的状态机漏洞，与平台无关；
`16bf8c7e2` 当时测试通过，是因为那一轮测试期间订阅没断过。

## 修复

1. `ChainEventSubscription` 增加断开信号 `Stream<void> get dropped`，
   `onDone` 时对外发一次（`disconnect()` 主动取消不发 —— `cancel()` 不触发 `onDone`，
   且用 lifecycle 代际再兜一层，防止停止过程中的迟到事件把监控器唤醒）。
2. `ChainTxMonitor` 订阅该信号：置 `_subscriptionConnected = false` 并**走 5 秒重试定时器**
   重连。**不立即重连** —— 轻节点持续不可用时会变成热循环。
   重连成功后 `_connectSubscriptionOnce` 既有的 `unawaited(_syncToLatest())`
   自动补齐断连期间漏掉的块，不需要额外补扫。
3. `stop()` 一并取消该监听。

## 落地

- `chain_event_subscription.dart`：新增 `Stream<void> get dropped`；`onDone` 发信号，
  并用 lifecycle 代际兜底（`disconnect()` 走 `cancel()`，按 Dart 语义不触发 `onDone`）。
- `chain_tx_monitor.dart`：`_subscription` / `_chainRpc` / 退避时长改构造注入
  （新增 `@visibleForTesting ChainTxMonitor.forTesting`）；`start()` 订阅 `dropped`；
  新增 `_onSubscriptionDropped()` + `_scheduleSubscriptionRetry()`（与首次连接失败共用退避）；
  `stop()` 取消该监听；新增 `subscriptionConnectedForTesting` 供断言。
- 退避时长可注入的原因：`start()` 里有 Isar 真 I/O，`fakeAsync` 拨表会卡在真实事件循环上
  （实测 4 个用例全部 `connectCount=0`），只能走真实定时器把 5 秒缩到毫秒级。

## 验收

- [x] 回归测试：订阅流关闭后监控器**真的重新 connect**（断言 `connectCount`，非标志位）
- [x] 回归测试：两条子订阅先后结束只触发一次重连（不重复排队）
- [x] 回归测试：重连失败后继续退避重试，轻节点恢复后自动连回
- [x] 回归测试：`stop()` 之后订阅流关闭不得触发重连
- [x] **守卫实证**：抽掉 `dropped` 监听后 4 个用例红 3 个，还原后 4/4 绿（不接受"看起来会拦"）
- [x] 连跑 3 次不飘
- [x] `flutter analyze` 干净
- [x] 文档回写 `memory/05-modules/citizenapp/rpc/RPC_TECHNICAL.md`，
      注释写明「为什么走定时器而不是立即重连」
- [ ] 全量测试通过（进行中）
- [ ] 真机验收：发一笔，盯其从待确认**自动**翻已确认（不碰刷新）
