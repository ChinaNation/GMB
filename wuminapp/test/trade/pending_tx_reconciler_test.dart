import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/trade/pending_tx_reconciler.dart';

void main() {
  const lostThreshold = Duration(minutes: 10);

  group('decideReconcileOutcome', () {
    test('txHash 在区块里找到 → confirmed 且带区块号', () {
      final d = decideReconcileOutcome(
        foundBlockNumber: 1234,
        chainNonce: null,
        usedNonce: 5,
        age: const Duration(seconds: 10),
        lostThreshold: lostThreshold,
      );
      expect(d.outcome, ReconcileOutcome.confirmed);
      expect(d.confirmedAtBlock, 1234);
    });

    test('区块未找到但 nonce 已推进且未超时 → confirmed（保守策略）', () {
      final d = decideReconcileOutcome(
        foundBlockNumber: null,
        chainNonce: 7,
        usedNonce: 5,
        age: const Duration(minutes: 1),
        lostThreshold: lostThreshold,
      );
      expect(d.outcome, ReconcileOutcome.confirmed);
      expect(d.confirmedAtBlock, isNull);
    });

    test('nonce 已推进且超时 → 仍然 confirmed（保守策略，不再判 lost）', () {
      // 中文注释：c0af679 之后的策略变更——nonce 推进只能证明"某笔交易占用了
      // 这个 nonce 位置"，可能是被同 nonce 的另一笔顶替（本笔其实 lost），
      // 但也可能是本笔执行成功只是超出搜索窗口。后者远比前者常见，
      // 把成功交易误标为 lost 比误标为 confirmed 更容易误导用户。
      // 所以新策略：nonce 推进 → 一律 confirmed，无论是否超时。
      final d = decideReconcileOutcome(
        foundBlockNumber: null,
        chainNonce: 7,
        usedNonce: 5,
        age: const Duration(minutes: 30),
        lostThreshold: lostThreshold,
      );
      expect(d.outcome, ReconcileOutcome.confirmed);
    });

    test('nonce 未推进且未超时 → 继续 pending', () {
      final d = decideReconcileOutcome(
        foundBlockNumber: null,
        chainNonce: 5,
        usedNonce: 5,
        age: const Duration(seconds: 30),
        lostThreshold: lostThreshold,
      );
      expect(d.outcome, ReconcileOutcome.stillPending);
    });

    test('nonce 不可用（pubkey 缓存未命中）→ 继续 pending', () {
      final d = decideReconcileOutcome(
        foundBlockNumber: null,
        chainNonce: null,
        usedNonce: 5,
        age: const Duration(minutes: 1),
        lostThreshold: lostThreshold,
      );
      expect(d.outcome, ReconcileOutcome.stillPending);
    });

    test('usedNonce 缺失 → 不会因 nonce 路径误判，继续 pending', () {
      final d = decideReconcileOutcome(
        foundBlockNumber: null,
        chainNonce: 10,
        usedNonce: null,
        age: const Duration(minutes: 30),
        lostThreshold: lostThreshold,
      );
      expect(d.outcome, ReconcileOutcome.stillPending);
    });

    test('区块找到优先级高于 nonce 超时判定', () {
      final d = decideReconcileOutcome(
        foundBlockNumber: 999,
        chainNonce: 100,
        usedNonce: 5,
        age: const Duration(hours: 1),
        lostThreshold: lostThreshold,
      );
      expect(d.outcome, ReconcileOutcome.confirmed);
      expect(d.confirmedAtBlock, 999);
    });
  });
}
