import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/membership/membership_page.dart';

const String _owner = '5GrwvaEF5zXb26Fz9rcQpDWS7u4m6DXb6T6TQvF9j5uQ8g6U';

class _FakeSessionProvider extends SquareSessionProvider {
  _FakeSessionProvider() : super();

  @override
  Future<SquareSession?> ensureSession() async => SquareSession(
        sessionToken: 'tok',
        ownerAccount: _owner,
        expiresAt: DateTime.now().millisecondsSinceEpoch + 600000,
      );
}

class _FakeApiClient extends SquareApiClient {
  _FakeApiClient(this._state) : super(baseUrl: 'https://membership.test');

  final SquareMembershipState _state;

  @override
  Future<SquareMembershipState> fetchMembership(SquareSession session) async =>
      _state;
}

/// 会员与身份彻底解耦（ADR-036）：会员卡只描述订阅，不含任何身份字段。
/// plans 留空 → 页面用三档兜底套餐（自由 / 民主 / 薪火）渲染。
SquareMembershipState _state({
  bool active = false,
  bool subscriptionActive = false,
  bool cancelAtPeriodEnd = false,
  String? membershipLevel,
  int currentPeriodStart = 0,
  String? subscriptionSource,
}) {
  return SquareMembershipState(
    active: active,
    expiresAt: active ? DateTime.now().millisecondsSinceEpoch + 600000 : 0,
    membershipLevel: membershipLevel,
    subscriptionActive: subscriptionActive,
    cancelAtPeriodEnd: cancelAtPeriodEnd,
    currentPeriodStart: currentPeriodStart,
    subscriptionSource: subscriptionSource,
    plans: const [],
  );
}

Future<void> _pump(WidgetTester tester, SquareMembershipState state) async {
  await tester.pumpWidget(
    MaterialApp(
      home: MembershipPage(
        apiClient: _FakeApiClient(state),
        sessionProvider: _FakeSessionProvider(),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

Finder _frontCard(String text) => find.descendant(
      of: find.byKey(const ValueKey('membership-front-card')),
      matching: find.text(text),
    );

void main() {
  testWidgets('renders the three subscription tier cards', (tester) async {
    await _pump(tester, _state());

    expect(find.text('自由会员'), findsOneWidget);
    expect(find.text('民主会员'), findsOneWidget);
    expect(find.text('薪火会员'), findsOneWidget);
  });

  testWidgets('defaults to the freedom card for a fresh account',
      (tester) async {
    await _pump(tester, _state());
    expect(_frontCard('自由会员'), findsOneWidget);
  });

  testWidgets('fronts the current membership tier card + 当前会员 marker',
      (tester) async {
    await _pump(
      tester,
      _state(active: true, subscriptionActive: true, membershipLevel: 'spark'),
    );

    expect(_frontCard('薪火会员'), findsOneWidget);
    expect(find.text('当前会员'), findsOneWidget);
  });

  testWidgets('any identity can subscribe any tier — all cards show 订阅',
      (tester) async {
    await _pump(tester, _state());
    // 三档解耦、无身份门槛：三张卡都可订阅。
    expect(find.text('订阅'), findsNWidgets(3));
    expect(find.text('当前会员'), findsNothing);
  });

  testWidgets('shows 取消订阅 on the active tier when auto-renewing',
      (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
      ),
    );

    expect(find.text('取消订阅'), findsOneWidget);
    // 另外两档仍可订阅。
    expect(find.text('订阅'), findsNWidgets(2));
  });

  testWidgets('shows 续订会员 when cancelled but not yet expired', (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        cancelAtPeriodEnd: true,
        membershipLevel: 'democracy',
      ),
    );

    expect(find.text('续订会员'), findsOneWidget);
    expect(find.text('取消订阅'), findsNothing);
  });

  testWidgets('USDC 预付会员显示订阅起止 + 预付路线横幅', (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
        subscriptionSource: 'usdc_prepaid',
      ),
    );

    expect(find.textContaining('预付 · 到期失效'), findsOneWidget);
    expect(find.textContaining('订阅 '), findsOneWidget);
  });

  testWidgets('卡连续订阅会员显示自动续费横幅', (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        membershipLevel: 'democracy',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
        subscriptionSource: 'stripe',
      ),
    );

    expect(find.textContaining('自动续费'), findsOneWidget);
  });

  testWidgets('卡已发起到期取消 → 横幅标签「到期终止」而非自动续费', (tester) async {
    await _pump(
      tester,
      _state(
        active: true,
        subscriptionActive: true,
        cancelAtPeriodEnd: true,
        membershipLevel: 'democracy',
        currentPeriodStart: DateTime.now().millisecondsSinceEpoch,
        subscriptionSource: 'stripe',
      ),
    );

    expect(find.textContaining('已取消 · 到期终止'), findsOneWidget);
    expect(find.textContaining('自动续费'), findsNothing);
  });

  test('SquareMembershipState 路线 / 订阅窗口 getter', () {
    const prepaid = SquareMembershipState(
      active: true,
      expiresAt: 2000,
      subscriptionActive: true,
      currentPeriodStart: 1000,
      subscriptionSource: 'usdc_prepaid',
    );
    expect(prepaid.isPrepaid, isTrue);
    expect(prepaid.hasSubscriptionWindow, isTrue);

    const noWindow = SquareMembershipState(
      active: true,
      expiresAt: 2000,
      subscriptionActive: true,
      subscriptionSource: 'stripe',
    );
    // 缺 current_period_start（=0）→ 无可展示窗口。
    expect(noWindow.isPrepaid, isFalse);
    expect(noWindow.hasSubscriptionWindow, isFalse);
  });

}
