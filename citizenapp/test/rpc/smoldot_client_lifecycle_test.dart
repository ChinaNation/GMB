import 'dart:async';

import 'package:citizenapp/rpc/chain_event_subscription.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('并发启动复用同一个 Future 且只执行一次初始化', () async {
    final releaseStart = Completer<void>();
    var startCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () {
        startCount += 1;
        return releaseStart.future;
      },
    );

    final first = manager.ensureStarted();
    final second = manager.initialize();

    expect(identical(first, second), isTrue);
    expect(startCount, 1);
    releaseStart.complete();
    await Future.wait([first, second]);

    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('初始化失败会清空在途状态并允许下一次重试', () async {
    var startCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async {
        startCount += 1;
        if (startCount == 1) {
          throw StateError('first start failed');
        }
      },
    );

    await expectLater(manager.ensureStarted(), throwsStateError);
    expect(manager.initializedForTesting, isFalse);

    await manager.ensureStarted();
    expect(startCount, 2);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('初始化成功后 dispose 可以释放并再次启动', () async {
    var startCount = 0;
    var disposeCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async => startCount += 1,
      dispose: () async => disposeCount += 1,
    );

    await manager.ensureStarted();
    await manager.dispose();
    expect(manager.initializedForTesting, isFalse);

    await manager.ensureStarted();
    expect(startCount, 2);
    expect(disposeCount, 1);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('dispose 会让旧的在途初始化失效且不会覆盖新生命周期', () async {
    final firstStart = Completer<void>();
    var startCount = 0;
    var disposeCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () {
        startCount += 1;
        return startCount == 1 ? firstStart.future : Future<void>.value();
      },
      dispose: () async => disposeCount += 1,
    );

    final staleStart = manager.ensureStarted();
    final staleStartExpectation = expectLater(
      staleStart,
      throwsA(isA<Exception>()),
    );
    final disposing = manager.dispose();
    firstStart.complete();

    await staleStartExpectation;
    await disposing;
    expect(manager.initializedForTesting, isFalse);

    await manager.ensureStarted();
    expect(startCount, 2);
    expect(disposeCount, 1);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('dispose 进行中发起的启动会等待释放完成并进入新生命周期', () async {
    final releaseDispose = Completer<void>();
    var startCount = 0;
    var disposeFinished = false;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async {
        expect(disposeFinished || startCount == 0, isTrue);
        startCount += 1;
      },
      dispose: () async {
        await releaseDispose.future;
        disposeFinished = true;
      },
    );

    await manager.ensureStarted();
    final disposing = manager.dispose();
    final restarting = manager.ensureStarted();

    expect(manager.initializedForTesting, isTrue);
    expect(startCount, 1);
    releaseDispose.complete();
    await Future.wait([disposing, restarting]);

    expect(startCount, 2);
    expect(manager.initializedForTesting, isTrue);
    await manager.dispose();
  });

  test('链订阅会等待启动结果且初始化失败时返回 false', () async {
    var startCount = 0;
    final manager = SmoldotClientManager.forTesting(
      initialize: () async {
        startCount += 1;
        throw StateError('start failed');
      },
    );
    final subscription = ChainEventSubscription(
      smoldotClientManager: manager,
    );

    expect(await subscription.connect(), isFalse);
    expect(startCount, 1);

    subscription.disconnect();
    await manager.dispose();
  });
}
