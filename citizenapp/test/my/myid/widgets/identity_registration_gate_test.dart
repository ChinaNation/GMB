import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/widgets/identity_registration_gate.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 可配判据 resolver:注册/未注册/链读失败/无钱包四态,广播重跑用可变字段。
class _FakeResolver extends IdentityAccountResolver {
  _FakeResolver({
    this.registered = false,
    this.throwErr = false,
    this.noWallet = false,
  });
  bool registered;
  bool throwErr;
  bool noWallet;

  @override
  Future<ResolvedIdentity?> resolve() async {
    if (throwErr) throw StateError('链读失败');
    if (noWallet) return null;
    return ResolvedIdentity(
      accountId: '0x${'11' * 32}',
      ss58Address: 'ss58',
      accountIndex: 0,
      snapshot: registered
          ? CitizenIdentityChainSnapshot(
              cidNumber: 'GD-CTZN1-8F3A2B',
              accountId: Uint8List(32),
              votingIdentity: null,
              candidateIdentity: null,
            )
          : null,
    );
  }
}

/// 可配设备子钥绑定器:记录调用次数,可配成失败。
///
/// 生产里这一步会读硬件金库并弹生物识别,测试一律注入假实现;不注入就会真去绑。
class _FakeBinder {
  _FakeBinder({this.fail = false});

  bool fail;
  int calls = 0;

  Future<void> call() async {
    calls += 1;
    if (fail) throw StateError('绑定失败');
  }
}

Widget _wrap(IdentityRegistrationGate gate) => MaterialApp(home: gate);

void main() {
  testWidgets('已注册 CID → 放行真功能', (tester) async {
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '聊天',
      resolver: _FakeResolver(registered: true),
      subkeyBinder: _FakeBinder().call,
      child: const Text('真功能'),
    )));
    await tester.pumpAndSettle();
    expect(find.text('真功能'), findsOneWidget);
  });

  testWidgets('未注册 CID → 挡住并引导去注册', (tester) async {
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '聊天',
      resolver: _FakeResolver(registered: false),
      child: const Text('真功能'),
    )));
    await tester.pumpAndSettle();
    expect(find.text('真功能'), findsNothing);
    expect(find.text('注册'), findsOneWidget);
    expect(find.text('注册后开始聊天'), findsOneWidget);
  });

  testWidgets('链已就绪仍读失败 → 不放行,提示重试(fail-closed)', (tester) async {
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '广场',
      resolver: _FakeResolver(throwErr: true),
      healthListenable:
          ValueNotifier<ChainHealthStatus>(ChainHealthStatus.operational),
      child: const Text('真功能'),
    )));
    await tester.pumpAndSettle();
    expect(find.text('真功能'), findsNothing);
    expect(find.text('身份读取失败'), findsOneWidget);
    expect(find.text('重试'), findsOneWidget);
  });

  testWidgets('冷启动链未就绪读失败 → 停在 loading,不弹读取失败;就绪后自动放行',
      (tester) async {
    final health =
        ValueNotifier<ChainHealthStatus>(ChainHealthStatus.uninitialized);
    // 链未就绪时 resolve 抛;就绪后同一 resolver 返回已注册。
    final resolver = _FakeResolver(throwErr: true);
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '广场',
      resolver: resolver,
      healthListenable: health,
      subkeyBinder: _FakeBinder().call,
      child: const Text('真功能'),
    )));
    // loading 态是无限动画的 spinner,不能 pumpAndSettle(永不 settle);用 pump 冲刷
    // 首次 resolve 的 microtask 后断言。
    await tester.pump();
    await tester.pump();
    // 未就绪:停在 loading spinner,绝不弹「身份读取失败」(不冒充未注册、不放行)。
    expect(find.text('身份读取失败'), findsNothing);
    expect(find.text('真功能'), findsNothing);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    // 链转 operational + resolver 恢复 → 自动重判放行,无需手动重试。
    resolver.throwErr = false;
    resolver.registered = true;
    health.value = ChainHealthStatus.operational;
    await tester.pumpAndSettle();
    expect(find.text('真功能'), findsOneWidget);
  });

  testWidgets('无热钱包 → 挡住并引导', (tester) async {
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '通讯录',
      resolver: _FakeResolver(noWallet: true),
      child: const Text('真功能'),
    )));
    await tester.pumpAndSettle();
    expect(find.text('真功能'), findsNothing);
    expect(find.text('需要热钱包'), findsOneWidget);
  });

  testWidgets('注册成功广播后自动重跑并放行', (tester) async {
    final resolver = _FakeResolver(registered: false);
    final binder = _FakeBinder();
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '聊天',
      resolver: resolver,
      subkeyBinder: binder.call,
      child: const Text('真功能'),
    )));
    await tester.pumpAndSettle();
    expect(find.text('注册'), findsOneWidget);
    // 未注册 CID 时不该去绑子钥:后端不收未绑 CID 账户的子钥。
    expect(binder.calls, 0);

    // 模拟注册成功:判据转已注册 + 广播身份绑定变化 → gate 自动重判放行。
    resolver.registered = true;
    WalletManager.notifyIdentityBindingChanged();
    await tester.pumpAndSettle();
    expect(find.text('真功能'), findsOneWidget);
    expect(find.text('去注册身份'), findsNothing);
  });

  testWidgets('放行前先按需绑定设备子钥(懒绑定触发点)', (tester) async {
    final binder = _FakeBinder();
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '广场',
      resolver: _FakeResolver(registered: true),
      subkeyBinder: binder.call,
      child: const Text('真功能'),
    )));
    await tester.pumpAndSettle();
    // 子钥只服务需 CID 的场景,故绑定发生在进入本门时,而不是建钱包时。
    expect(binder.calls, 1);
    expect(find.text('真功能'), findsOneWidget);
  });

  testWidgets('子钥绑定失败 → 不放行,提示重试(fail-closed)', (tester) async {
    final binder = _FakeBinder(fail: true);
    await tester.pumpWidget(_wrap(IdentityRegistrationGate(
      featureLabel: '聊天',
      resolver: _FakeResolver(registered: true),
      subkeyBinder: binder.call,
      child: const Text('真功能'),
    )));
    await tester.pumpAndSettle();
    expect(find.text('真功能'), findsNothing);
    expect(find.text('设备绑定未完成'), findsOneWidget);
    expect(find.text('重试'), findsOneWidget);

    // 点重试:绑定恢复后放行。
    binder.fail = false;
    await tester.tap(find.text('重试'));
    await tester.pumpAndSettle();
    expect(binder.calls, 2);
    expect(find.text('真功能'), findsOneWidget);
  });
}
