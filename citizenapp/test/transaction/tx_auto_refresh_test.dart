import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/transaction/shared/local_tx_store.dart';

import '../support/isar_test_env.dart';

// 响应式刷新的核心机制:`LocalTxStore.watchAccountChanges` 在该账户记录被写库
// 时发出事件。UI 侧 [TxAutoRefreshMixin] 订阅它并去抖重刷。
//
// 说明:mixin→页面「自动翻已确认」的整链路无法在 flutter_test 里稳定断言 ——
// Isar 原生 watcher 的通知走真实事件循环,而 widget 的 initState 订阅在 fake-async
// 区,两个 zone 对不上(store 级测试把 listen/写/断言全放进同一个 runAsync 才通)。
// 该整链路由装机端到端验收(发一笔,盯其从待确认自动翻已确认)。
void main() {
  useIsolatedIsar();

  const fromAccountId =
      '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
  const fromSs58Address = 'from-wallet';
  const toSs58Address = 'to-wallet';

  testWidgets('watchAccountChanges 在记录 pending→finalized 写库时发出事件',
      (tester) async {
    await tester.runAsync(() async {
      await LocalTxStore.upsertLocalSubmitTransfer(
        ss58Address: fromSs58Address,
        accountId: fromAccountId,
        txHash: '0xfeed',
        amountDeltaFen: '-101',
        transferAmountFen: '100',
        feeFen: '1',
        counterpartySs58Address: toSs58Address,
        fromSs58Address: fromSs58Address,
        toSs58Address: toSs58Address,
        usedNonce: 1,
        createdAtMillis: 1,
      );

      final seen = <void>[];
      final sub =
          LocalTxStore.watchAccountChanges(fromAccountId).listen(seen.add);
      await Future<void>.delayed(const Duration(milliseconds: 150)); // 订阅生效

      // 模拟后台 ChainTxMonitor 把记录就地翻 finalized。
      await LocalTxStore.markLocalSubmitFinalized(
        accountId: fromAccountId,
        txHash: '0xfeed',
        blockHash: '0x22',
        blockNumber: 9,
      );
      await Future<void>.delayed(const Duration(milliseconds: 200)); // 通知送达

      expect(seen, isNotEmpty, reason: 'finalized 写库应触发 watch 事件驱动 UI 重刷');
      await sub.cancel();
    });
  }, timeout: const Timeout(Duration(seconds: 40)));
}
