import 'dart:async';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/rpc/chain_event_subscription.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/chain_tx_monitor.dart';

import '../support/isar_test_env.dart';

/// 自动确认**只有一条**通路：finalized 订阅推事件 → `_syncThrough` →
/// `_confirmOpenSubmits`。订阅一旦断开而不重连，自动确认就永久失效，交易卡片
/// 只能靠手动刷新（走 `_syncToLatest()`，不经订阅）才翻已确认。
///
/// 2026-08-07 iOS/Android 两端同时复现的正是这个：`onDone` 不对外发信号，
/// `_subscriptionConnected` 只在 `stop()` 里才置 false，于是断开后
/// `_ensureSubscription()` 每次从第一行早退，没有任何重连路径。
///
/// 本文件钉死断开→重连这条自愈链路。断言必须落在**真的重新订阅了**
/// （`connectCount`）上：只断言标志位翻转会漏掉「标志翻了但没人去连」的回归。
///
/// 用真实定时器而非 `fakeAsync`：`start()` 里有 Isar 真 I/O，拨假表会卡在真实
/// 事件循环上；退避改为构造注入，缩到毫秒级。
class _FakeSubscription extends ChainEventSubscription {
  _FakeSubscription();

  final StreamController<ChainEvent> _events =
      StreamController<ChainEvent>.broadcast();
  final StreamController<void> _dropped = StreamController<void>.broadcast();

  int connectCount = 0;

  /// 下一次 `connect()` 的返回值；置 false 模拟轻节点尚未就绪。
  bool connectResult = true;

  @override
  Stream<ChainEvent> get events => _events.stream;

  @override
  Stream<void> get dropped => _dropped.stream;

  @override
  Future<bool> connect() async {
    connectCount += 1;
    return connectResult;
  }

  @override
  void disconnect() {}

  /// 模拟底层 smoldot 流结束（原生 chain 被释放）。
  void emitDrop() => _dropped.add(null);

  Future<void> close() async {
    await _events.close();
    await _dropped.close();
  }
}

/// 离线 RPC：`start()` 里的 metadata 预热是 `unawaited` 的，真 [ChainRpc] 会去
/// 拉起 smoldot，单测必须注入立即失败的 fake，否则用例随机 flaky。
class _OfflineChainRpc implements ChainRpc {
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      Future<Never>.error(StateError('offline'));
}

void main() {
  useIsolatedIsar();

  const retryDelay = Duration(milliseconds: 20);
  // 退避到点后还要跑 connect 的微任务，等三倍留足余量。
  const pastRetry = Duration(milliseconds: 60);

  // `start()` 不等订阅连上就返回（连接走 fire-and-forget，是刻意的非阻塞启动），
  // 断言前必须先让那条 future 落定，否则读到的还是初始的未连接态。
  Future<void> settle() => Future<void>.delayed(const Duration(milliseconds: 5));

  late _FakeSubscription subscription;
  late ChainTxMonitor monitor;

  setUp(() {
    subscription = _FakeSubscription();
    monitor = ChainTxMonitor.forTesting(
      subscription: subscription,
      chainRpc: _OfflineChainRpc(),
      subscriptionRetryDelay: retryDelay,
    );
  });

  tearDown(() async {
    monitor.stop();
    await subscription.close();
  });

  test('订阅断开后按退避重连，并真的重新建立订阅', () async {
    await monitor.start();
    await settle();
    expect(subscription.connectCount, 1, reason: '启动时应连接一次');
    expect(monitor.subscriptionConnectedForTesting, isTrue);

    subscription.emitDrop();
    await Future<void>.delayed(Duration.zero);

    expect(monitor.subscriptionConnectedForTesting, isFalse,
        reason: '断开后必须立刻落回未连接，否则 _ensureSubscription 会一直早退');
    expect(subscription.connectCount, 1, reason: '不得立即重连（会退化成热循环）');

    await Future<void>.delayed(pastRetry);
    expect(subscription.connectCount, 2, reason: '退避到点后必须真的重新 connect');
    expect(monitor.subscriptionConnectedForTesting, isTrue);
  });

  test('两条子订阅先后结束只触发一次重连', () async {
    await monitor.start();
    await settle();
    expect(subscription.connectCount, 1);

    subscription.emitDrop();
    subscription.emitDrop();
    await Future<void>.delayed(pastRetry);

    expect(subscription.connectCount, 2, reason: '第二次断开信号必须被忽略');
  });

  test('重连失败后继续退避重试，不放弃', () async {
    await monitor.start();
    await settle();
    expect(subscription.connectCount, 1);

    subscription.connectResult = false;
    subscription.emitDrop();
    await Future<void>.delayed(pastRetry);

    // 退避窗口内可能已重试多次，只断言"至少重连过"，不钉死次数（会随退避抖动飘）。
    final afterFirstRetries = subscription.connectCount;
    expect(afterFirstRetries, greaterThanOrEqualTo(2));
    expect(monitor.subscriptionConnectedForTesting, isFalse);

    await Future<void>.delayed(pastRetry);
    expect(subscription.connectCount, greaterThan(afterFirstRetries),
        reason: '连接失败必须继续退避重试，不能停在第一次失败');

    subscription.connectResult = true;
    await Future<void>.delayed(pastRetry);
    expect(monitor.subscriptionConnectedForTesting, isTrue,
        reason: '轻节点恢复后必须自动连回来');
  });

  test('stop() 之后的断开信号不得唤醒重连', () async {
    await monitor.start();
    await settle();
    expect(subscription.connectCount, 1);

    monitor.stop();
    subscription.emitDrop();
    await Future<void>.delayed(pastRetry);

    expect(subscription.connectCount, 1, reason: '已停止的监控器不得再连接');
  });
}
